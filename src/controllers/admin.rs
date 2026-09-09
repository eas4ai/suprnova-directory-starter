use suprnova::{InertiaProps, Request, Response, handler, inertia_response};

#[derive(InertiaProps)]
pub struct AdminOverviewProps {}

#[handler]
pub async fn index(req: Request) -> Response {
    inertia_response!(&req, "admin/Overview", AdminOverviewProps {})
}
