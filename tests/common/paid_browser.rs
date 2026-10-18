//! An ignored, disposable test server. No fixture routes or fake gateway are
//! linked into either application binary.
use super::*;
use hyper::{body::Incoming, service::service_fn};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use suprnova::{HttpResponse, MiddlewareRegistry, handle_request};

#[tokio::test(flavor = "current_thread")]
#[ignore = "Started only by the paid browser verifier"]
async fn serve_paid_browser_fixture() {
    use sea_orm_migration::MigratorTrait;
    assert_eq!(std::env::var("APP_ENV").as_deref(), Ok("production"));
    directory::config::register_all();
    suprnova::Crypt::init(suprnova::EncryptionKey::from_env().unwrap());
    directory::bootstrap::register().await;
    directory::migrations::Migrator::up(DB::connection().unwrap().inner(), None)
        .await
        .unwrap();
    suprnova::rate_limit::bootstrap_default().await;
    directory::bootstrap::register_http_stack();
    let _mail = suprnova::Mail::fake();
    let fake = Arc::new(FakeGateway::default());
    App::bind::<dyn Gateway>(fake.clone());
    workflow::seed_categories().await.unwrap();
    let owner = account("browser-owner", true).await;
    let admin = account("browser-admin", true).await;
    change_access(admin.id, AccessAction::Grant).await.unwrap();
    for (key, kind, amount) in [
        ("free", "free", 0),
        ("once", "one_time", 1000),
        ("monthly", "monthly", 1000),
    ] {
        plans::save(
            admin.id,
            None,
            serde_json::from_value(plan_input(key, kind, amount, 0)).unwrap(),
        )
        .await
        .unwrap();
    }
    let mode = checkout::checkout_mode().unwrap();
    settings(
        mode,
        settings_input(billing::load(mode).await.unwrap().revision, mode),
    )
    .await;
    let category = queries::categories()
        .await
        .unwrap()
        .into_iter()
        .find(|c| c.slug == "software")
        .unwrap()
        .id;
    let mut owner_http = login(&owner).await;
    let mut admin_http = login(&admin).await;
    for title in ["Browser Paddle resource", "Browser free resource"] {
        let id = create(&mut owner_http, title, category).await;
        approve(&mut owner_http, &mut admin_http, id).await;
    }
    let port: u16 = std::env::var("PAID_BROWSER_PORT").unwrap().parse().unwrap();
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port))
        .await
        .unwrap();
    let router = Arc::new(directory::routes::register());
    let middleware = Arc::new(MiddlewareRegistry::from_global());
    println!("PAID_BROWSER_READY");
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let (router, middleware, fake) = (router.clone(), middleware.clone(), fake.clone());
        tokio::spawn(async move {
            let service = service_fn(move |req: hyper::Request<Incoming>| {
                let (router, middleware, fake) = (router.clone(), middleware.clone(), fake.clone());
                async move {
                    let path = req.uri().path().to_owned();
                    let response = if path.starts_with("/assets/") {
                        suprnova::StaticFiles::public().handler()(suprnova::Request::new(req))
                            .await
                            .unwrap_or_else(|response| response)
                            .into_hyper()
                    } else if let Some(session) = path.strip_prefix("/__fixture/checkout/") {
                        let p = by_session(session).await;
                        HttpResponse::html(format!("<!doctype html><html lang=\"en\"><title>Provider fixture</title><h1>Provider checkout fixture</h1><p>Synthetic provider response; no charge is made.</p><form method=\"post\" action=\"/__fixture/settle/{}\"><button>Complete fixture payment</button></form></html>", p.id)).into_hyper()
                    } else if let Some(id) = path
                        .strip_prefix("/__fixture/settle/")
                        .filter(|_| req.method() == "POST")
                    {
                        let p = if id.starts_with("txn_") {
                            by_session(id).await
                        } else {
                            purchase_row(id).await
                        };
                        settle(&fake, &p).await;
                        HttpResponse::text("")
                            .status(303)
                            .header("Location", format!("/dashboard/purchases/{}", p.id))
                            .into_hyper()
                    } else if path == "/__fixture/reconcile" && req.method() == "POST" {
                        reconcile::pending(100, fake.as_ref()).await.unwrap();
                        HttpResponse::text("reconciled").into_hyper()
                    } else {
                        handle_request(router, middleware, req).await
                    };
                    Ok::<_, Infallible>(response)
                }
            });
            hyper::server::conn::http1::Builder::new()
                .serve_connection(TokioIo::new(stream), service)
                .await
                .unwrap();
        });
    }
}

async fn by_session(id: &str) -> purchase::Model {
    purchase::Entity::find()
        .filter(purchase::Column::SessionRef.eq(id))
        .one(DB::connection().unwrap().inner())
        .await
        .unwrap()
        .unwrap()
}

async fn settle(fake: &FakeGateway, p: &purchase::Model) {
    let now = chrono::Utc::now().timestamp();
    let end = (p.billing_type != "one_time").then_some(now + 86400 * 30);
    let resource = fake.settle(p, 0, now - 30, end);
    let kind = if p.provider == "stripe" {
        if end.is_some() {
            "invoice.paid"
        } else {
            "checkout.session.completed"
        }
    } else {
        "transaction.completed"
    };
    let id = signed(
        &mut Client::new(),
        p,
        kind,
        &resource,
        &format!("evt_browser_{}", p.id),
    )
    .await;
    processed(&id, fake).await;
}
