use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serial_test::serial;

use sword::events::{EventHandlerResult, EventPublish, MemoryEventPublisher, event, subscribe};
use sword::prelude::*;
use sword::web::*;

use crate::{BASE, ConcurrencyProbe, EventStore, WEB_ADDR, start_app};

const MEMORY: &str = r#"
[events]
concurrency = 2
"#;

const TIMEOUT: &str = r#"
[events]
concurrency = 1
handler_timeout_ms = 50

[events.memory]
buffer_size = 1024
num_of_event_retry = 0
delay_between_event_retry_ms = 0
"#;

const OVERRIDE: &str = r#"
[events]
concurrency = 1
handler_timeout_ms = 50

[events.memory]
buffer_size = 1024
num_of_event_retry = 0
delay_between_event_retry_ms = 0
handler_timeout_ms = 500
"#;

const NO_TIMEOUT: &str = r#"
[events]
concurrency = 1
handler_timeout_ms = 50

[events.memory]
buffer_size = 1024
num_of_event_retry = 0
delay_between_event_retry_ms = 0
handler_timeout_ms = 0
"#;

#[event(key = "user.created")]
#[derive(Clone, Serialize, Deserialize)]
pub struct UserCreated {
    pub id: u32,
}

#[controller(kind = Controller::Events, source = EventSource::Memory)]
pub struct UserCreatedHandler {
    store: Arc<EventStore>,
}

impl UserCreatedHandler {
    #[subscribe(key = "user.created")]
    async fn on_created(&self, event: &UserCreated) -> EventHandlerResult<()> {
        assert_eq!(event.id, 1, "handler received an unexpected event payload");
        self.store.increment();

        Ok(())
    }
}

#[controller(kind = Controller::Web, path = "/users")]
pub struct UsersController {
    publisher: Arc<MemoryEventPublisher>,
    store: Arc<EventStore>,
}

impl UsersController {
    #[post("/create")]
    async fn create(&self) -> WebResult {
        self.publisher.publish(UserCreated { id: 1 }).await.unwrap();

        Ok(JsonResponse::Ok().message("published"))
    }

    #[get("/count")]
    async fn count(&self) -> String {
        self.store.count().to_string()
    }
}

#[event(key = "burst.created")]
#[derive(Clone, Serialize, Deserialize)]
pub struct BurstCreated {
    pub id: u32,
    pub sleep_ms: u64,
}

#[controller(kind = Controller::Events, source = EventSource::Memory)]
pub struct BurstHandler {
    probe: Arc<ConcurrencyProbe>,
}

impl BurstHandler {
    #[subscribe(key = "burst.created")]
    async fn on_burst(&self, event: &BurstCreated) -> EventHandlerResult<()> {
        self.probe.enter();
        tokio::time::sleep(Duration::from_millis(event.sleep_ms)).await;
        self.probe.record_hit();
        self.probe.leave();

        Ok(())
    }
}

#[controller(kind = Controller::Web, path = "/burst")]
pub struct BurstController {
    publisher: Arc<MemoryEventPublisher>,
    probe: Arc<ConcurrencyProbe>,
}

impl BurstController {
    #[post("/start")]
    async fn start(&self) -> WebResult {
        for id in 0..10 {
            self.publisher
                .publish(BurstCreated { id, sleep_ms: 40 })
                .await
                .unwrap();
        }

        Ok(JsonResponse::Ok().message("published"))
    }

    #[post("/slow")]
    async fn slow(&self) -> WebResult {
        self.publisher
            .publish(BurstCreated {
                id: 0,
                sleep_ms: 200,
            })
            .await
            .unwrap();

        Ok(JsonResponse::Ok().message("published"))
    }

    #[post("/fast")]
    async fn fast(&self) -> WebResult {
        self.publisher
            .publish(BurstCreated {
                id: 0,
                sleep_ms: 10,
            })
            .await
            .unwrap();

        Ok(JsonResponse::Ok().message("published"))
    }

    #[get("/stats")]
    async fn stats(&self) -> String {
        format!("{},{}", self.probe.max(), self.probe.hits())
    }
}

/// Polls `GET /users/count` until it reaches `expected` or times out.
async fn poll_count(expected: usize) -> usize {
    let mut count = 0;

    for _ in 0..200 {
        let body = reqwest::get(format!("http://{WEB_ADDR}/users/count"))
            .await
            .expect("request")
            .error_for_status()
            .expect("success status")
            .text()
            .await
            .expect("body");
        count = body.trim().parse::<usize>().unwrap_or(0);

        if count >= expected {
            break;
        }

        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    count
}

/// Reads `GET /burst/stats` once.
async fn burst_stats() -> (usize, usize) {
    let body = reqwest::get(format!("http://{WEB_ADDR}/burst/stats"))
        .await
        .expect("request")
        .error_for_status()
        .expect("success status")
        .text()
        .await
        .expect("body");

    match body.trim().split_once(',') {
        Some((max, hits)) => (max.parse().unwrap_or(0), hits.parse().unwrap_or(0)),
        None => (0, 0),
    }
}

/// Polls `GET /burst/stats` until `expected` handlers have run.
async fn poll_burst_stats(expected: usize) -> (usize, usize) {
    let mut stats = (0, 0);

    for _ in 0..300 {
        stats = burst_stats().await;

        if stats.1 >= expected {
            break;
        }

        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    stats
}

#[tokio::test]
#[serial]
async fn web_route_publishes_event_that_increments_store() {
    let server = start_app(&format!("{BASE}{MEMORY}")).await;

    reqwest::Client::new()
        .post(format!("http://{WEB_ADDR}/users/create"))
        .send()
        .await
        .expect("request")
        .error_for_status()
        .expect("success status");

    assert_eq!(
        poll_count(1).await,
        1,
        "event handler did not increment the store"
    );

    server.abort();
}

#[tokio::test]
#[serial]
async fn multiple_events_accumulate_in_store() {
    let server = start_app(&format!("{BASE}{MEMORY}")).await;

    reqwest::Client::new()
        .post(format!("http://{WEB_ADDR}/users/create"))
        .send()
        .await
        .expect("request")
        .error_for_status()
        .expect("success status");
    reqwest::Client::new()
        .post(format!("http://{WEB_ADDR}/users/create"))
        .send()
        .await
        .expect("request")
        .error_for_status()
        .expect("success status");

    assert_eq!(
        poll_count(2).await,
        2,
        "store did not accumulate every published event"
    );

    server.abort();
}

#[tokio::test]
#[serial]
async fn concurrency_limit_bounds_parallel_handlers() {
    let server = start_app(&format!("{BASE}{MEMORY}")).await;

    reqwest::Client::new()
        .post(format!("http://{WEB_ADDR}/burst/start"))
        .send()
        .await
        .expect("request")
        .error_for_status()
        .expect("success status");

    let (max, hits) = poll_burst_stats(10).await;

    assert_eq!(hits, 10, "every burst event should be handled");
    assert!(max <= 2, "handlers exceeded the concurrency limit: {max}");
    assert!(max >= 2, "the limit was never reached, got {max}");

    server.abort();
}

#[test]
fn concurrency_zero_fails_at_startup() {
    let config = Config::builder()
        .add_toml_str("[events]\nconcurrency = 0\n")
        .build()
        .expect("config");

    assert!(Application::from_config(config).is_err());
}

#[tokio::test]
#[serial]
async fn handler_timeout_releases_the_permit() {
    let server = start_app(&format!("{BASE}{TIMEOUT}")).await;

    reqwest::Client::new()
        .post(format!("http://{WEB_ADDR}/burst/slow"))
        .send()
        .await
        .expect("request")
        .error_for_status()
        .expect("success status");
    tokio::time::sleep(Duration::from_millis(150)).await;

    let (_, hits) = burst_stats().await;
    assert_eq!(hits, 0, "the slow handler should time out");

    reqwest::Client::new()
        .post(format!("http://{WEB_ADDR}/burst/fast"))
        .send()
        .await
        .expect("request")
        .error_for_status()
        .expect("success status");
    let (_, hits) = poll_burst_stats(1).await;

    assert_eq!(hits, 1, "the permit should be free after the timeout");

    server.abort();
}

#[tokio::test]
#[serial]
async fn transport_timeout_overrides_the_global_value() {
    let server = start_app(&format!("{BASE}{OVERRIDE}")).await;

    reqwest::Client::new()
        .post(format!("http://{WEB_ADDR}/burst/slow"))
        .send()
        .await
        .expect("request")
        .error_for_status()
        .expect("success status");
    let (_, hits) = poll_burst_stats(1).await;

    assert_eq!(hits, 1, "the 500 ms override should let the handler finish");

    server.abort();
}

#[tokio::test]
#[serial]
async fn zero_timeout_disables_the_limit() {
    let server = start_app(&format!("{BASE}{NO_TIMEOUT}")).await;

    reqwest::Client::new()
        .post(format!("http://{WEB_ADDR}/burst/slow"))
        .send()
        .await
        .expect("request")
        .error_for_status()
        .expect("success status");
    let (_, hits) = poll_burst_stats(1).await;

    assert_eq!(hits, 1, "a timeout of zero should disable the limit");

    server.abort();
}
