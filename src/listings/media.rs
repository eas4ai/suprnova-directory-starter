//! Private, pixel-only listing images. Callers authorize access before `read`.
use std::{
    fmt,
    fs::{self, OpenOptions},
    io::{self, Cursor, Read, Write},
    path::{Component, Path, PathBuf},
};

use image::{ImageEncoder, ImageFormat, ImageReader, Limits};
use tokio::sync::Semaphore;

pub const MAX_UPLOAD_BYTES: usize = 5 * 1024 * 1024;
const MAX_DIMENSION: u32 = 4096;
// RGBA8 PNG plus encoding overhead; independent of the compressed upload cap.
const MAX_STORED_BYTES: usize = 70 * 1024 * 1024;
static MEDIA_JOBS: Semaphore = Semaphore::const_new(2);

#[derive(Debug)]
pub enum MediaError {
    InvalidImage,
    TooLarge,
    InvalidKey,
    UnsafeRoot,
    Busy,
    Io(io::Error),
    Worker,
}

impl fmt::Display for MediaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidImage => "Upload a valid JPEG, PNG or WebP image.",
            Self::TooLarge => "Images must be at most 5 MiB and 4096 pixels per dimension.",
            Self::InvalidKey => "Invalid media identifier.",
            Self::UnsafeRoot => "Private media storage is not configured safely.",
            Self::Busy => "Image processing is busy. Please retry.",
            Self::Io(_) | Self::Worker => "Media storage is temporarily unavailable.",
        })
    }
}

impl std::error::Error for MediaError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for MediaError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Debug)]
pub struct StoredMedia {
    pub key: String,
    pub width: u32,
    pub height: u32,
    pub content_type: &'static str,
    pub byte_count: usize,
}

/// Reject oversize input before copying it or reserving a blocking worker.
pub async fn persist(bytes: &[u8]) -> Result<StoredMedia, MediaError> {
    if bytes.len() > MAX_UPLOAD_BYTES {
        return Err(MediaError::TooLarge);
    }
    let permit = MEDIA_JOBS.try_acquire().map_err(|_| MediaError::Busy)?;
    let bytes = bytes.to_vec();
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        let (encoded, width, height) = reencode(&bytes)?;
        let root = private_root()?;
        persist_in(&root, &encoded, width, height)
    })
    .await
    .map_err(|_| MediaError::Worker)?
}

pub async fn read(key: &str) -> Result<Vec<u8>, MediaError> {
    validate_key(key)?;
    let permit = MEDIA_JOBS.try_acquire().map_err(|_| MediaError::Busy)?;
    let key = key.to_owned();
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        let path = private_root()?.join(key);
        if !fs::symlink_metadata(&path)?.file_type().is_file() {
            return Err(MediaError::InvalidKey);
        }
        let mut bytes = Vec::new();
        fs::File::open(path)?
            .take(MAX_STORED_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() > MAX_STORED_BYTES {
            return Err(MediaError::TooLarge);
        }
        Ok(bytes)
    })
    .await
    .map_err(|_| MediaError::Worker)?
}

/// Removing an already absent image succeeds, allowing cleanup retries.
pub async fn remove(key: &str) -> Result<(), MediaError> {
    validate_key(key)?;
    let permit = MEDIA_JOBS.try_acquire().map_err(|_| MediaError::Busy)?;
    let key = key.to_owned();
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        let root = private_root()?;
        match fs::remove_file(root.join(key)) {
            Ok(()) => fs::File::open(root)?.sync_all().map_err(MediaError::from),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    })
    .await
    .map_err(|_| MediaError::Worker)?
}

fn validate_key(key: &str) -> Result<(), MediaError> {
    let stem = key.strip_suffix(".png").ok_or(MediaError::InvalidKey)?;
    let id = uuid::Uuid::parse_str(stem).map_err(|_| MediaError::InvalidKey)?;
    if id.get_version_num() != 4 || id.hyphenated().to_string() != stem {
        return Err(MediaError::InvalidKey);
    }
    Ok(())
}

fn private_root() -> Result<PathBuf, MediaError> {
    let configured = std::env::var_os("DIRECTORY_MEDIA_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("storage/private/directory"));
    prepare_root(&configured)
}

fn prepare_root(configured: &Path) -> Result<PathBuf, MediaError> {
    // Reject literal public paths before creating anything, and symlink aliases
    // after canonicalization. Operators must keep this directory outside all
    // additional web-server document roots too.
    if configured
        .components()
        .any(|part| matches!(part, Component::Normal(s) if s == "public"))
    {
        return Err(MediaError::UnsafeRoot);
    }
    fs::create_dir_all(configured)?;
    let root = configured.canonicalize()?;
    if root
        .components()
        .any(|part| matches!(part, Component::Normal(s) if s == "public"))
    {
        return Err(MediaError::UnsafeRoot);
    }
    if let Ok(public) = Path::new("public").canonicalize()
        && root.starts_with(public)
    {
        return Err(MediaError::UnsafeRoot);
    }
    Ok(root)
}

fn reencode(bytes: &[u8]) -> Result<(Vec<u8>, u32, u32), MediaError> {
    if bytes.len() > MAX_UPLOAD_BYTES {
        return Err(MediaError::TooLarge);
    }
    let format = image::guess_format(bytes).map_err(|_| MediaError::InvalidImage)?;
    if !matches!(
        format,
        ImageFormat::Jpeg | ImageFormat::Png | ImageFormat::WebP
    ) {
        return Err(MediaError::InvalidImage);
    }
    let pixels = if format == ImageFormat::Jpeg {
        decode_jpeg(bytes)?
    } else {
        decode_raster(bytes, format)?
    };
    let (width, height) = pixels.dimensions();
    // Construct a fresh pixel buffer: metadata from the decoder is never passed
    // to an encoder. Normalize 16-bit inputs to RGBA8 to bound stored size.
    let mut output = CappedOutput(Vec::new());
    image::codecs::png::PngEncoder::new(&mut output)
        .write_image(
            pixels.as_raw(),
            width,
            height,
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|_| MediaError::InvalidImage)?;
    Ok((output.0, width, height))
}

fn decode_raster(bytes: &[u8], format: ImageFormat) -> Result<image::RgbaImage, MediaError> {
    let (width, height) = ImageReader::with_format(Cursor::new(bytes), format)
        .into_dimensions()
        .map_err(|_| MediaError::InvalidImage)?;
    if width == 0 || height == 0 || width > MAX_DIMENSION || height > MAX_DIMENSION {
        return Err(MediaError::TooLarge);
    }
    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_DIMENSION);
    limits.max_image_height = Some(MAX_DIMENSION);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    let decoded = reader.decode().map_err(|_| MediaError::InvalidImage)?;
    Ok(decoded.to_rgba8())
}

fn decode_jpeg(bytes: &[u8]) -> Result<image::RgbaImage, MediaError> {
    use zune_jpeg::zune_core::{
        bytestream::ZCursor, colorspace::ColorSpace, options::DecoderOptions,
    };
    let options = DecoderOptions::default()
        .set_strict_mode(false)
        .set_max_width(u16::MAX as usize)
        .set_max_height(u16::MAX as usize)
        // CMYK-to-RGBA in this decoder release writes three-byte pixels. Use
        // its RGB conversion, then let image add the opaque alpha channel.
        .jpeg_set_out_colorspace(ColorSpace::RGB);
    let mut decoder = zune_jpeg::JpegDecoder::new_with_options(ZCursor::new(bytes), options);
    decoder
        .decode_headers()
        .map_err(|_| MediaError::InvalidImage)?;
    let (width, height) = decoder.dimensions().ok_or(MediaError::InvalidImage)?;
    if width == 0
        || height == 0
        || width > MAX_DIMENSION as usize
        || height > MAX_DIMENSION as usize
    {
        return Err(MediaError::TooLarge);
    }
    decoder.set_options(
        options
            .set_max_width(MAX_DIMENSION as usize)
            .set_max_height(MAX_DIMENSION as usize),
    );
    // RGB plus normalized RGBA stays below the existing 128 MiB pixel budget.
    let mut pixels = vec![0; width * height * 3];
    decoder
        .decode_into(&mut pixels)
        .map_err(|_| MediaError::InvalidImage)?;
    let pixels = image::RgbImage::from_raw(width as u32, height as u32, pixels)
        .ok_or(MediaError::InvalidImage)?;
    Ok(image::DynamicImage::ImageRgb8(pixels).to_rgba8())
}

struct CappedOutput(Vec<u8>);
impl Write for CappedOutput {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > MAX_STORED_BYTES.saturating_sub(self.0.len()) {
            return Err(io::Error::other("encoded image exceeds storage limit"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn persist_in(
    root: &Path,
    bytes: &[u8],
    width: u32,
    height: u32,
) -> Result<StoredMedia, MediaError> {
    let key = format!("{}.png", uuid::Uuid::new_v4());
    let path = root.join(&key);
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&path)?;
    if let Err(error) = (|| -> io::Result<()> {
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::File::open(root)?.sync_all()
    })() {
        // Cleanup failure is secondary to the failed persistence operation.
        let _ = fs::remove_file(&path);
        return Err(error.into());
    }
    Ok(StoredMedia {
        key,
        width,
        height,
        content_type: "image/png",
        byte_count: bytes.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(format: ImageFormat, width: u32, height: u32) -> Vec<u8> {
        if format == ImageFormat::Jpeg {
            assert_eq!((width, height), (2, 3));
            return include_bytes!("../../tests/fixtures/listing-image.jpg").to_vec();
        }
        let pixels = image::DynamicImage::new_rgb8(width, height);
        let mut output = Cursor::new(Vec::new());
        pixels.write_to(&mut output, format).unwrap();
        output.into_inner()
    }

    #[test]
    fn accepts_only_decodable_raster_images_and_bounds_dimensions() {
        for format in [ImageFormat::Jpeg, ImageFormat::Png, ImageFormat::WebP] {
            let (output, width, height) = reencode(&fixture(format, 2, 3)).unwrap();
            assert_eq!((width, height), (2, 3));
            assert_eq!(image::guess_format(&output).unwrap(), ImageFormat::Png);
            assert_eq!(image::load_from_memory(&output).unwrap().width(), 2);
        }
        for bad in [
            b"<svg xmlns='http://www.w3.org/2000/svg'/>".as_slice(),
            b"\x89PNG\r\n\x1a\n",
            b"\xff\xd8\xff\xe0",
        ] {
            assert!(reencode(bad).is_err());
        }
        assert!(matches!(
            reencode(&fixture(ImageFormat::Png, 4097, 1)),
            Err(MediaError::TooLarge)
        ));
        assert!(reencode(&fixture(ImageFormat::Png, 4096, 1)).is_ok());
        assert!(matches!(
            reencode(&vec![0; MAX_UPLOAD_BYTES + 1]),
            Err(MediaError::TooLarge)
        ));
    }

    #[test]
    fn reencoding_discards_jpeg_comment_metadata() {
        let source = fixture(ImageFormat::Jpeg, 2, 3);
        let comment = b"private location marker";
        let mut tagged = source[..2].to_vec();
        tagged.extend_from_slice(&[0xff, 0xfe]);
        tagged.extend_from_slice(&((comment.len() + 2) as u16).to_be_bytes());
        tagged.extend_from_slice(comment);
        tagged.extend_from_slice(&source[2..]);
        let (encoded, _, _) = reencode(&tagged).unwrap();
        assert!(
            !encoded
                .windows(comment.len())
                .any(|window| window == comment)
        );
    }

    #[test]
    fn jpeg_pixels_and_dimension_limit_are_preserved() {
        // Synthetic RGB fixture: solid (200, 40, 80), 2 by 3 pixels.
        let mut source = fixture(ImageFormat::Jpeg, 2, 3);
        let (encoded, width, height) = reencode(&source).unwrap();
        assert_eq!((width, height), (2, 3));
        let decoded = image::load_from_memory(&encoded).unwrap().to_rgba8();
        for pixel in decoded.pixels() {
            for (actual, expected) in pixel.0.into_iter().zip([200_u8, 40, 80, 255]) {
                assert!(actual.abs_diff(expected) <= 2);
            }
        }
        // Change the baseline JPEG's width in its frame header, without allocating
        // a large raster. Reject it before the decoder allocates pixel storage.
        let frame = source
            .windows(2)
            .position(|bytes| bytes == [0xff, 0xc0])
            .unwrap();
        source[frame + 7..frame + 9].copy_from_slice(&4097_u16.to_be_bytes());
        assert!(matches!(reencode(&source), Err(MediaError::TooLarge)));
    }

    #[test]
    fn jpeg_color_spaces_and_progressive_scans_decode() {
        let cases: &[(&[u8], [u8; 4])] = &[
            (
                include_bytes!("../../tests/fixtures/listing-grayscale.jpg"),
                [80, 80, 80, 255],
            ),
            (
                include_bytes!("../../tests/fixtures/listing-cmyk.jpg"),
                [255, 0, 0, 255],
            ),
            (
                include_bytes!("../../tests/fixtures/listing-progressive.jpg"),
                [200, 40, 80, 255],
            ),
        ];
        for (case, (source, expected)) in cases.iter().enumerate() {
            let (encoded, width, height) = reencode(source).unwrap();
            assert_eq!((width, height), (2, 3));
            let decoded = image::load_from_memory(&encoded).unwrap().to_rgba8();
            for pixel in decoded.pixels() {
                for (actual, expected) in pixel.0.iter().zip(expected) {
                    assert!(
                        actual.abs_diff(*expected) <= 2,
                        "case {case}: {pixel:?} != {expected:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn generated_keys_and_private_root_reject_traversal() {
        assert!(validate_key(&format!("{}.png", uuid::Uuid::new_v4())).is_ok());
        for key in [
            "../image.png",
            "/etc/passwd",
            "a.png",
            "00000000-0000-0000-0000-000000000000.png",
        ] {
            assert!(validate_key(key).is_err());
        }
        assert!(matches!(
            prepare_root(Path::new("public/listings")),
            Err(MediaError::UnsafeRoot)
        ));
    }

    #[test]
    fn persists_complete_bytes_with_generated_name() {
        let root = std::env::temp_dir().join(format!("directory-media-{}", uuid::Uuid::new_v4()));
        let root = prepare_root(&root).unwrap();
        let (encoded, width, height) = reencode(&fixture(ImageFormat::Png, 2, 3)).unwrap();
        let stored = persist_in(&root, &encoded, width, height).unwrap();
        assert!(validate_key(&stored.key).is_ok());
        assert_eq!(stored.byte_count, encoded.len());
        assert_eq!(fs::read(root.join(stored.key)).unwrap(), encoded);
        fs::remove_dir_all(root).unwrap();
    }
}
