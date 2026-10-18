use suprnova::{HttpResponse, Middleware, Next, Request, Response, async_trait};
pub struct SeoRoutes;
#[async_trait]
impl Middleware for SeoRoutes {
    async fn handle(&self, request: Request, next: Next) -> Response {
        let path = request.path().to_owned();
        let method = request.method().to_owned();
        if matches!(method.as_str(), "GET" | "HEAD")
            && let Some(destination) = super::redirects::resolve(&path).await?
        {
            return Err(HttpResponse::text("")
                .status(301)
                .header("Location", destination)
                .header("Cache-Control", "no-store"));
        }
        let response = next(request).await;
        let status = match &response {
            Ok(response) | Err(response) => response.status_code(),
        };
        if method == "GET"
            && status == 404
            && super::not_found::reportable(&path)
            && super::not_found::record(&path).await.is_err()
        {
            // Observational reporting must not turn a missing page into an application failure.
            tracing::warn!("Could not record a bounded 404 observation");
        }
        response
    }
}
