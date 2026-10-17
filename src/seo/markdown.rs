use sea_orm::EntityTrait;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use suprnova::{FrameworkError, HttpResponse, Middleware, Next, Request, Response, async_trait};
use suprnova_markdown::{MarkdownMiddleware, MarkdownSource};

struct PublishedSource {
    failed: Arc<AtomicBool>,
}

async fn content(path: &str) -> Result<Option<String>, FrameworkError> {
    let db = suprnova::DB::connection()?;
    if let Some(slug) = path.strip_prefix("/listings/") {
        use crate::listings::{database_error, entities::revision, queries};
        // Query eligibility here, within the MarkdownSource path, on every request.
        let row = queries::public_listing(slug, chrono::Utc::now().timestamp()).await?;
        let revision = revision::Entity::find_by_id(
            row.approved_revision_id
                .ok_or_else(crate::listings::missing)?,
        )
        .one(db.inner())
        .await
        .map_err(database_error)?
        .ok_or_else(crate::listings::missing)?;
        return Ok(Some(format!(
            "# {}\n\n{}\n\n{}\n\nWebsite: {}\n",
            revision.title, revision.summary, revision.description, revision.url
        )));
    }
    if let Some(slug) = path.strip_prefix("/articles/") {
        use crate::articles::{entities::revision, queries};
        let row = queries::public_article(slug).await?;
        if row.slug != slug {
            return Ok(None);
        }
        let revision = revision::Entity::find_by_id(
            row.published_revision_id
                .ok_or_else(crate::listings::missing)?,
        )
        .one(db.inner())
        .await
        .map_err(crate::listings::database_error)?
        .ok_or_else(crate::listings::missing)?;
        return Ok(Some(format!(
            "# {}\n\n{}\n\n{}\n",
            revision.title, revision.summary, revision.body
        )));
    }
    Ok(None)
}

#[async_trait]
impl MarkdownSource for PublishedSource {
    async fn markdown(&self, path: &str) -> Option<String> {
        match content(path).await {
            Ok(content) => content,
            Err(error) => {
                let response = HttpResponse::from(error);
                if response.status_code() != 404 {
                    self.failed.store(true, Ordering::Relaxed);
                }
                None
            }
        }
    }
}

pub struct PublicMarkdown;
#[async_trait]
impl Middleware for PublicMarkdown {
    async fn handle(&self, request: Request, next: Next) -> Response {
        if !request.path().ends_with(".md") {
            return next(request).await;
        }
        if !matches!(request.method().as_str(), "GET" | "HEAD") {
            return Err(HttpResponse::text("Method not allowed")
                .status(405)
                .header("Allow", "GET, HEAD")
                .header("Cache-Control", "no-store"));
        }
        let head = request.method() == "HEAD";
        let page = request.path().strip_suffix(".md").unwrap_or_default();
        if !super::redirects::safe_path(page)
            || page.split('/').count() != 3
            || !(page.starts_with("/listings/") || page.starts_with("/articles/"))
            || page.ends_with('/')
            || page.ends_with(".md")
        {
            return Err(HttpResponse::text("404 Not Found")
                .status(404)
                .header("Cache-Control", "no-store"));
        }
        let failed = Arc::new(AtomicBool::new(false));
        let middleware = MarkdownMiddleware::new(PublishedSource {
            failed: failed.clone(),
        })
        .max_age(0);
        let fallback: Next =
            Arc::new(|_| Box::pin(async { Err(HttpResponse::text("404 Not Found").status(404)) }));
        let result = middleware.handle(request, fallback).await;
        if failed.load(Ordering::Relaxed) {
            tracing::error!("Public Markdown source failed");
            return Err(
                HttpResponse::text("The public content could not be loaded.")
                    .status(500)
                    .header("Cache-Control", "no-store"),
            );
        }
        let response = match result {
            Ok(response) | Err(response) => response,
        };
        let length = response.body().len();
        let body = if head {
            Vec::new()
        } else {
            response.body().to_vec()
        };
        let mut output = HttpResponse::bytes(
            body.into(),
            response
                .header_value("Content-Type")
                .unwrap_or("text/plain; charset=utf-8"),
        )
        .status(response.status_code());
        for (name, value) in response.headers() {
            if !["content-type", "cache-control", "content-length"]
                .iter()
                .any(|skip| name.eq_ignore_ascii_case(skip))
            {
                output = output.header(name, value);
            }
        }
        output = output
            .header("Cache-Control", "no-store")
            .header("X-Content-Type-Options", "nosniff");
        if head {
            output = output.header("Content-Length", length.to_string());
        }
        Err(output)
    }
}
