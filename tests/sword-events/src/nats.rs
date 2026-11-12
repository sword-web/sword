use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serial_test::serial;

use sword::events::*;
use sword::prelude::*;
use sword::web::*;

use crate::{BASE, ConcurrencyProbe, WEB_ADDR, start_app};

const NATS_USER_PASS: &str = r#"
[events]
concurrency = 2

[events.nats]
url = "tls://localhost:4222"

[events.nats.auth]
user = "app"
password = "secret"

[events.nats.tls]
ca_file = "nats/certs/ca.pem"
"#;

const NATS_TOKEN: &str = r#"
[events]
concurrency = 2

[events.nats]
url = "nats://localhost:4223"

[events.nats.auth]
token = "secret-token"
"#;

const NATS_MTLS: &str = r#"
[events]
concurrency = 2

[events.nats]
url = "tls://localhost:4224"

[events.nats.auth]
user = "app"
password = "secret"

[events.nats.tls]
ca_file = "nats/certs/ca.pem"
client_cert = "nats/certs/client.pem"
client_key = "nats/certs/client-key.pem"
"#;

const NATS_WRONG_PASSWORD: &str = r#"
[events]
concurrency = 2

[events.nats]
url = "tls://localhost:4222"

[events.nats.auth]
user = "app"
password = "wrong"

[events.nats.tls]
ca_file = "nats/certs/ca.pem"
"#;

#[event(key = "nats.created")]
#[derive(Clone, Serialize, Deserialize)]
pub struct NatsCreated {
    pub id: u32,
}

#[controller(kind = Controller::Events, source = EventSource::Nats)]
pub struct NatsHandler {
    probe: Arc<ConcurrencyProbe>,
}

impl NatsHandler {
    #[subscribe(key = "nats.created")]
    async fn on_nats(&self, _event: &NatsCreated) -> EventHandlerResult<()> {
        self.probe.record_hit();

        Ok(())
    }
}

#[controller(kind = Controller::Web, path = "/nats")]
pub struct NatsController {
    publisher: Arc<NatsEventPublisher>,
    probe: Arc<ConcurrencyProbe>,
}

impl NatsController {
    #[post("/start")]
    async fn start(&self) -> WebResult {
        self.publisher
            .publish(NatsCreated { id: 1 })
            .await
            .map_err(|error| JsonResponse::InternalServerError().message(error.to_string()))?;

        Ok(JsonResponse::Ok().message("published"))
    }

    #[get("/stats")]
    async fn stats(&self) -> String {
        self.probe.hits().to_string()
    }
}

/// Polls `GET /nats/stats` until `expected` handlers have run.
async fn poll_nats_hits(expected: usize) -> usize {
    let mut hits = 0;

    for _ in 0..300 {
        let body = reqwest::get(format!("http://{WEB_ADDR}/nats/stats"))
            .await
            .expect("request")
            .error_for_status()
            .expect("success status")
            .text()
            .await
            .expect("body");
        hits = body.trim().parse().unwrap_or(0);

        if hits >= expected {
            break;
        }

        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    hits
}

#[test]
fn nats_auth_rejects_token_and_user_together() {
    let config = Config::builder()
        .add_toml_str(
            "[events.nats.auth]\ntoken = \"token\"\nuser = \"app\"\npassword = \"secret\"\n",
        )
        .build()
        .expect("config");

    assert!(Application::from_config(config).is_err());
}

#[test]
fn nats_auth_rejects_user_without_password() {
    let config = Config::builder()
        .add_toml_str("[events.nats.auth]\nuser = \"app\"\n")
        .build()
        .expect("config");

    assert!(Application::from_config(config).is_err());
}

#[test]
fn nats_tls_rejects_cert_without_key() {
    let config = Config::builder()
        .add_toml_str("[events.nats.tls]\nclient_cert = \"client.pem\"\n")
        .build()
        .expect("config");

    assert!(Application::from_config(config).is_err());
}

#[test]
fn nats_tls_rejects_missing_files() {
    let config = Config::builder()
        .add_toml_str("[events.nats.tls]\nca_file = \"does-not-exist.pem\"\n")
        .build()
        .expect("config");

    assert!(Application::from_config(config).is_err());
}

#[tokio::test]
#[serial]
#[ignore = "requires the NATS test server from tests/sword-events/nats/run.sh"]
async fn nats_user_password_over_tls_delivers_events() {
    let server = start_app(&format!("{BASE}{NATS_USER_PASS}")).await;

    reqwest::Client::new()
        .post(format!("http://{WEB_ADDR}/nats/start"))
        .send()
        .await
        .expect("request")
        .error_for_status()
        .expect("success status");
    let hits = poll_nats_hits(1).await;

    assert_eq!(hits, 1, "the NATS handler did not run");

    server.abort();
}

#[tokio::test]
#[serial]
#[ignore = "requires the NATS test server from tests/sword-events/nats/run.sh"]
async fn nats_token_delivers_events() {
    let server = start_app(&format!("{BASE}{NATS_TOKEN}")).await;

    reqwest::Client::new()
        .post(format!("http://{WEB_ADDR}/nats/start"))
        .send()
        .await
        .expect("request")
        .error_for_status()
        .expect("success status");
    let hits = poll_nats_hits(1).await;

    assert_eq!(hits, 1, "the NATS handler did not run");

    server.abort();
}

#[tokio::test]
#[serial]
#[ignore = "requires the NATS test server from tests/sword-events/nats/run.sh"]
async fn nats_mtls_delivers_events() {
    let server = start_app(&format!("{BASE}{NATS_MTLS}")).await;

    reqwest::Client::new()
        .post(format!("http://{WEB_ADDR}/nats/start"))
        .send()
        .await
        .expect("request")
        .error_for_status()
        .expect("success status");
    let hits = poll_nats_hits(1).await;

    assert_eq!(hits, 1, "the NATS handler did not run");

    server.abort();
}

#[tokio::test]
#[serial]
#[ignore = "requires the NATS test server from tests/sword-events/nats/run.sh"]
async fn nats_wrong_password_fails() {
    let config = Config::builder()
        .add_toml_str(&format!("{BASE}{NATS_WRONG_PASSWORD}"))
        .build()
        .expect("config");

    let app = Application::from_config(config)
        .expect("application builder")
        .build()
        .expect("application");

    let result = tokio::time::timeout(Duration::from_secs(10), app.run()).await;

    assert!(matches!(result, Ok(Err(_))), "expected a startup error");
}
