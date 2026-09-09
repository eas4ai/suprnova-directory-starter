use suprnova::{Request, Response, handler};

#[handler]
pub async fn index(req: Request) -> Response {
    super::listings::render_index(req, "A place for good discoveries.").await
}
