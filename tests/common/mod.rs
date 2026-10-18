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
    #[allow(dead_code)]
    pub body_bytes: Vec<u8>,
    #[allow(dead_code)]
    pub headers: http::HeaderMap,
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

    pub async fn inertia_get(&mut self, path: &str) -> Response {
        self.exchange("GET", path, None, true, true).await
    }

    async fn exchange(
        &mut self,
        method: &str,
        path: &str,
        body: Option<Value>,
        csrf: bool,
        inertia: bool,
    ) -> Response {
        self.exchange_bytes(
            method,
            path,
            body.map(|value| value.to_string().into_bytes())
                .unwrap_or_default(),
            "application/json",
            csrf,
            inertia,
            &[],
        )
        .await
    }

    #[allow(dead_code)]
    pub async fn raw_request(
        &mut self,
        method: &str,
        path: &str,
        body: Vec<u8>,
        content_type: &str,
        csrf: bool,
    ) -> Response {
        self.exchange_bytes(method, path, body, content_type, csrf, false, &[])
            .await
    }

    pub async fn signed_request(
        &mut self,
        path: &str,
        bytes: Vec<u8>,
        header: &str,
        signature: &str,
    ) -> Response {
        self.exchange_bytes(
            "POST",
            path,
            bytes,
            "application/json",
            false,
            false,
            &[(header, signature)],
        )
        .await
    }

    async fn exchange_bytes(
        &mut self,
        method: &str,
        path: &str,
        payload: Vec<u8>,
        content_type: &str,
        csrf: bool,
        inertia: bool,
        headers: &[(&str, &str)],
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
        });
        let response = tokio::time::timeout(Duration::from_secs(15), async {
            let (mut sender, connection) =
                hyper::client::conn::http1::handshake::<_, Full<Bytes>>(TokioIo::new(client))
                    .await
                    .expect("HTTP handshake");
            tasks.spawn(connection);
            let mut request = hyper::Request::builder()
                .method(method)
                .uri(path)
                .header("Host", "directory.test")
                .header("Accept", "text/html");
            for (name, value) in headers {
                request = request.header(*name, *value);
            }
            if !payload.is_empty() {
                request = request.header("Content-Type", content_type);
            }
            if inertia {
                request = request
                    .header("X-Inertia", "true")
                    .header(
                        "X-Inertia-Version",
                        suprnova::InertiaConfig::new().version.resolve(),
                    )
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
            let body_bytes = body
                .collect()
                .await
                .expect("response body")
                .to_bytes()
                .to_vec();
            Response {
                status: parts.status.as_u16(),
                location: parts
                    .headers
                    .get("location")
                    .map(|value| value.to_str().unwrap().to_owned()),
                body: String::from_utf8_lossy(&body_bytes).into_owned(),
                body_bytes,
                headers: parts.headers,
            }
        })
        .await
        .expect("HTTP exchange exceeded 15 seconds");
        tasks.abort_all();
        while let Some(result) = tasks.join_next().await {
            match result {
                Ok(Ok(())) => {}
                Err(error) if error.is_cancelled() => {}
                Ok(Err(error)) => {
                    // The server may reject an oversized body before the client
                    // finishes writing it. Only that response permits BrokenPipe.
                    let mut source: Option<&(dyn std::error::Error + 'static)> = Some(&error);
                    let mut broken_pipe = false;
                    while let Some(error) = source {
                        broken_pipe |= error
                            .downcast_ref::<std::io::Error>()
                            .is_some_and(|error| error.kind() == std::io::ErrorKind::BrokenPipe);
                        source = error.source();
                    }
                    assert!(
                        response.status == 413 && broken_pipe,
                        "HTTP connection failed: {error}"
                    );
                }
                Err(error) => panic!("HTTP task failed: {error}"),
            }
        }
        response
    }
}
