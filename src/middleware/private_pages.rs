use suprnova::{HttpResponse, Middleware, Next, Request, Response, async_trait};

/// Page props and form tokens belong to one visitor, including guest sessions.
pub struct PrivatePages;

fn protect(response: HttpResponse) -> HttpResponse {
    let html = response.header_value("Content-Type").is_some_and(|value| {
        value
            .split(';')
            .next()
            .unwrap_or_default()
            .trim()
            .eq_ignore_ascii_case("text/html")
    });
    if html
        || response.header_value("X-Inertia").is_some()
        || response.header_value("Set-Cookie").is_some()
    {
        response.replace_header("Cache-Control", "private, no-store")
    } else {
        response
    }
}

#[async_trait]
impl Middleware for PrivatePages {
    async fn handle(&self, request: Request, next: Next) -> Response {
        next(request).await.map(protect).map_err(protect)
    }
}
