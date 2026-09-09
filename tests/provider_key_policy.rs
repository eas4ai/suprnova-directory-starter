#[allow(dead_code)] // Reuse application setup; this subprocess does not exercise HTTP.
mod common;

use directory::billing::{self, Mode, SaveSettings};
use suprnova::{DB, serde_json::json};

#[tokio::test(flavor = "current_thread")]
async fn deployment_key_policy() {
    let _mail = common::setup().await;
    let before: String = DB::scalar(
        "SELECT payload FROM billing_settings WHERE mode = 'test'",
        vec![],
    )
    .await
    .unwrap();
    let revision: i64 = DB::scalar(
        "SELECT revision FROM billing_settings WHERE mode = 'test'",
        vec![],
    )
    .await
    .unwrap();
    match std::env::var("BILLING_KEY_EXPECT").as_deref() {
        Ok("accept") => {
            let settings = billing::load(Mode::Test)
                .await
                .expect("the original key must decrypt after restart");
            assert_eq!(settings.revision, revision);
            assert!(settings.paddle.has_secrets);
        }
        Ok("reject") => {
            assert!(billing::load(Mode::Test).await.is_err());
            let blank = json!({"enabled":false,"public_key":"","api_key":"","webhook_key":"","clear_secrets":true});
            let replacement: SaveSettings = suprnova::serde_json::from_value(
                json!({"revision":revision,"default_provider":null,
                "stripe":blank,"paddle":blank,"mappings":[]}),
            )
            .unwrap();
            assert!(
                billing::save(Mode::Test, replacement).await.is_err(),
                "invalid key material must not let a clear overwrite stored secrets"
            );
        }
        _ => panic!(
            "Run this check through verify-provider-administration.mjs with its explicit key expectation"
        ),
    }
    let after: String = DB::scalar(
        "SELECT payload FROM billing_settings WHERE mode = 'test'",
        vec![],
    )
    .await
    .unwrap();
    assert_eq!(after, before);
}
