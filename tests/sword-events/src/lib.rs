#![cfg(test)]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serial_test::serial;

use sword::events::{EventHandlerResult, EventPublish, MemoryEventPublisher, event, subscribe};
use sword::prelude::*;
use sword::web::*;

const WEB_ADDR: &str = "127.0.0.1:8083";

// ---- shared in-memory store -------------------------------------------------

/// Pre-instantiated provider shared by the web controller and the event handler.
pub struct EventStore {
    hits: AtomicUsize,
}

impl EventStore {
    pub fn new() -> Self {
        Self {
            hits: AtomicUsize::new(0),
        }
    }

    pub fn increment(&self) {
        self.hits.fetch_add(1, Ordering::SeqCst);
    }

    pub fn count(&self) -> usize {
        self.hits.load(Ordering::SeqCst)
    }
}

impl Default for EventStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for EventStore {}

// ---- concurrency probe ------------------------------------------------------

/// Tracks how many burst handlers run at the same time.
pub struct ConcurrencyProbe {
    in_flight: AtomicUsize,
    max_in_flight: AtomicUsize,
    hits: AtomicUsize,
}

impl ConcurrencyProbe {
    pub fn new() -> Self {
        Self {
            in_flight: AtomicUsize::new(0),
            max_in_flight: AtomicUsize::new(0),
            hits: AtomicUsize::new(0),
        }
    }

    fn enter(&self) {
        let current = self.in_flight.fetch_add(1, Ordering::SeqCst) + 1;
        self.max_in_flight.fetch_max(current, Ordering::SeqCst);
    }

    fn leave(&self) {
        self.in_flight.fetch_sub(1, Ordering::SeqCst);
    }

    fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::SeqCst);
    }

    fn max(&self) -> usize {
        self.max_in_flight.load(Ordering::SeqCst)
    }

    fn hits(&self) -> usize {
        self.hits.load(Ordering::SeqCst)
    }
}

impl Default for ConcurrencyProbe {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for ConcurrencyProbe {}

// ---- event + handler --------------------------------------------------------

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

// ---- web controller ---------------------------------------------------------

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

// ---- burst event ------------------------------------------------------------

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

// ---- module + harness -------------------------------------------------------

struct TestModule;

impl Module for TestModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<UsersController>();
        controllers.register::<UserCreatedHandler>();
        controllers.register::<BurstController>();
        controllers.register::<BurstHandler>();
    }
}

/// Builds and runs the full application (web + events) and waits until the web
/// listener accepts connections.
async fn start_app(config_path: &str) -> tokio::task::JoinHandle<()> {
    let app = Application::from_config_path(config_path)
        .expect("application builder")
        .with_provider(EventStore::new())
        .with_provider(ConcurrencyProbe::new())
        .with_module::<TestModule>()
        .build()
        .expect("application");

    let handle = tokio::spawn(async move {
        let _ = app.run().await;
    });

    for _ in 0..100 {
        if tokio::net::TcpStream::connect(WEB_ADDR).await.is_ok() {
            return handle;
        }

        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    panic!("web server did not become ready");
}

/// Minimal HTTP/1.1 request so the web engine is exercised over a real socket.
async fn http(method: &str, path: &str) -> String {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut stream = tokio::net::TcpStream::connect(WEB_ADDR)
        .await
        .unwrap_or_else(|err| panic!("web server not reachable at {WEB_ADDR}: {err}"));

    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: {WEB_ADDR}\r\nConnection: close\r\nContent-Length: 0\r\n\r\n"
    );

    stream
        .write_all(request.as_bytes())
        .await
        .expect("write request");

    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .await
        .expect("read response");

    let text = String::from_utf8_lossy(&response).to_string();
    text.split("\r\n\r\n")
        .nth(1)
        .unwrap_or_default()
        .to_string()
}

/// Polls `GET /users/count` until it reaches `expected` or times out.
async fn poll_count(expected: usize) -> usize {
    let mut count = 0;

    for _ in 0..200 {
        let body = http("GET", "/users/count").await;
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
    let body = http("GET", "/burst/stats").await;

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

// ---- tests ------------------------------------------------------------------

#[tokio::test]
#[serial]
async fn web_route_publishes_event_that_increments_store() {
    let server = start_app("Config.toml").await;

    let _ = http("POST", "/users/create").await;

    assert_eq!(
        poll_count(1).await,
        1,
        "event handler did not increment the store"
    );

    server.abort();
}

#[tokio::test]
#[serial]
async fn concurrency_limit_bounds_parallel_handlers() {
    let server = start_app("Config.toml").await;

    let _ = http("POST", "/burst/start").await;

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
    let server = start_app("ConfigTimeout.toml").await;

    let _ = http("POST", "/burst/slow").await;
    tokio::time::sleep(Duration::from_millis(150)).await;

    let (_, hits) = burst_stats().await;
    assert_eq!(hits, 0, "the slow handler should time out");

    let _ = http("POST", "/burst/fast").await;
    let (_, hits) = poll_burst_stats(1).await;

    assert_eq!(hits, 1, "the permit should be free after the timeout");

    server.abort();
}

#[tokio::test]
#[serial]
async fn transport_timeout_overrides_the_global_value() {
    let server = start_app("ConfigOverride.toml").await;

    let _ = http("POST", "/burst/slow").await;
    let (_, hits) = poll_burst_stats(1).await;

    assert_eq!(hits, 1, "the 500 ms override should let the handler finish");

    server.abort();
}

#[tokio::test]
#[serial]
async fn zero_timeout_disables_the_limit() {
    let server = start_app("ConfigNoTimeout.toml").await;

    let _ = http("POST", "/burst/slow").await;
    let (_, hits) = poll_burst_stats(1).await;

    assert_eq!(hits, 1, "a timeout of zero should disable the limit");

    server.abort();
}

#[tokio::test]
#[serial]
async fn multiple_events_accumulate_in_store() {
    let server = start_app("Config.toml").await;

    let _ = http("POST", "/users/create").await;
    let _ = http("POST", "/users/create").await;

    assert_eq!(
        poll_count(2).await,
        2,
        "store did not accumulate every published event"
    );

    server.abort();
}
