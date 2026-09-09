use std::{collections::HashMap, convert::Infallible, sync::Arc, time::Duration};

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{body::Incoming, service::service_fn};
use hyper_util::rt::TokioIo;
use suprnova::{MiddlewareRegistry, Router, handle_request, serde_json::Value};

pub async fn setup() -> suprnova::mail::MailFake {
    use sea_orm_migration::MigratorTrait;
    assert_eq!(std::env::var("APP_ENV").as_deref(), Ok("test"));
    directory::config::register_all();
    suprnova::Crypt::init(
        suprnova::EncryptionKey::from_env().unwrap_or_else(|_| suprnova::EncryptionKey::generate()),
    );
    directory::bootstrap::register().await;
    directory::migrations::Migrator::up(suprnova::DB::connection().unwrap().inner(), None)
        .await
        .unwrap();
    suprnova::rate_limit::bootstrap_default().await;
    directory::bootstrap::register_http_stack();
    suprnova::Mail::fake()
}

#[derive(Clone)]
pub struct Client {
    router: Arc<Router>,
    middleware: Arc<MiddlewareRegistry>,
    cookies: HashMap<String, String>,
}

pub struct Response {
    pub status: u16,
    pub location: Option<String>,
    pub body: String,
}

impl Client {
    pub fn new() -> Self {
        Self {
            router: Arc::new(directory::routes::register()),
            middleware: Arc::new(MiddlewareRegistry::from_global()),
            cookies: HashMap::new(),
        }
    }

    #[allow(dead_code)] // Used by account revocation tests; other suites share this client.
    pub fn forget_session_cookie(&mut self) {
        self.cookies.remove("suprnova_session");
    }

    pub async fn get(&mut self, path: &str) -> Response {
        self.request("GET", path, None, true).await
    }

    pub async fn post(&mut self, path: &str, body: Value) -> Response {
        self.request("POST", path, Some(body), true).await
    }

    pub async fn request(
        &mut self,
        method: &str,
        path: &str,
        body: Option<Value>,
        csrf: bool,
    ) -> Response {
        self.exchange(method, path, body, csrf, false).await
    }

    pub async fn inertia_post(&mut self, path: &str, body: Value) -> Response {
        self.exchange("POST", path, Some(body), true, true).await
    }

    async fn exchange(
        &mut self,
        method: &str,
        path: &str,
        body: Option<Value>,
        csrf: bool,
        inertia: bool,
    ) -> Response {
        // One real Hyper connection per exchange; both tasks are aborted on
        // timeout or panic as well as normal completion.
        let (client, server) = tokio::io::duplex(64 * 1024);
        let router = self.router.clone();
        let middleware = self.middleware.clone();
        let mut tasks = tokio::task::JoinSet::new();
        tasks.spawn(async move {
            let service = service_fn(move |req: hyper::Request<Incoming>| {
                let router = router.clone();
                let middleware = middleware.clone();
                async move { Ok::<_, Infallible>(handle_request(router, middleware, req).await) }
            });
            hyper::server::conn::http1::Builder::new()
                .serve_connection(TokioIo::new(server), service)
                .await
                .expect("serve HTTP request");
        });
        tokio::time::timeout(Duration::from_secs(15), async {
            let (mut sender, connection) =
                hyper::client::conn::http1::handshake::<_, Full<Bytes>>(TokioIo::new(client))
                    .await
                    .expect("HTTP handshake");
            tasks.spawn(async move {
                connection.await.expect("client connection");
            });
            let payload = body.map(|value| value.to_string()).unwrap_or_default();
            let mut request = hyper::Request::builder()
                .method(method)
                .uri(path)
                .header("Host", "directory.test")
                .header("Accept", "text/html");
            if !payload.is_empty() {
                request = request.header("Content-Type", "application/json");
            }
            if inertia {
                request = request
                    .header("X-Inertia", "true")
                    .header("Referer", format!("http://directory.test{path}"));
            }
            if !self.cookies.is_empty() {
                request = request.header(
                    "Cookie",
                    self.cookies
                        .iter()
                        .map(|(k, v)| format!("{k}={v}"))
                        .collect::<Vec<_>>()
                        .join("; "),
                );
            }
            if csrf
                && method != "GET"
                && let Some(token) = self.cookies.get("XSRF-TOKEN")
            {
                request = request.header("X-XSRF-TOKEN", token);
            }
            let response = sender
                .send_request(
                    request
                        .body(Full::new(Bytes::from(payload)))
                        .expect("request"),
                )
                .await
                .expect("HTTP response");
            let (parts, body) = response.into_parts();
            for cookie in parts.headers.get_all("set-cookie") {
                let cookie = cookie.to_str().expect("cookie header");
                let mut fields = cookie.split(';');
                let (name, value) = fields.next().unwrap().split_once('=').expect("cookie pair");
                if value.is_empty()
                    || fields.any(|field| field.trim().eq_ignore_ascii_case("max-age=0"))
                {
                    self.cookies.remove(name);
                } else {
                    self.cookies.insert(name.to_owned(), value.to_owned());
                }
            }
            Response {
                status: parts.status.as_u16(),
                location: parts
                    .headers
                    .get("location")
                    .map(|value| value.to_str().unwrap().to_owned()),
                body: String::from_utf8(
                    body.collect()
                        .await
                        .expect("response body")
                        .to_bytes()
                        .to_vec(),
                )
                .expect("UTF-8 body"),
            }
        })
        .await
        .expect("HTTP exchange exceeded 15 seconds")
    }
}
