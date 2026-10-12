#![cfg(test)]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use serial_test::serial;
use sword::events::{EventHandlerResult, EventPublisher, event, handle};
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

// ---- event + handler --------------------------------------------------------

#[event(key = "user.created")]
pub struct UserCreated {
    pub id: u32,
}

#[controller(kind = Controller::EventHandler, source = EventSource::Memory)]
pub struct UserCreatedHandler {
    store: Arc<EventStore>,
}

impl UserCreatedHandler {
    #[handle("user.created")]
    async fn on_created(&self, event: UserCreated) -> EventHandlerResult<()> {
        assert_eq!(event.id, 1, "handler received an unexpected event payload");
        self.store.increment();

        Ok(())
    }
}

// ---- web controller ---------------------------------------------------------

#[controller(kind = Controller::Web, path = "/users")]
pub struct UsersController {
    publisher: Arc<EventPublisher>,
    store: Arc<EventStore>,
}

impl UsersController {
    #[post("/create")]
    async fn create(&self) -> WebResult {
        self.publisher.publish(UserCreated { id: 1 }).await;

        Ok(JsonResponse::Ok().message("published"))
    }

    #[get("/count")]
    async fn count(&self) -> String {
        self.store.count().to_string()
    }
}

// ---- module + harness -------------------------------------------------------

struct TestModule;

impl Module for TestModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<UsersController>();
        controllers.register::<UserCreatedHandler>();
    }
}

/// Builds and runs the full application (web + events) and waits until the web
/// listener accepts connections.
async fn start_app() -> tokio::task::JoinHandle<()> {
    let app = Application::from_config_path("Config.toml")
        .with_provider(EventStore::new())
        .with_module::<TestModule>()
        .build();

    let handle = tokio::spawn(async move { app.run().await });

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

// ---- tests ------------------------------------------------------------------

#[tokio::test]
#[serial]
async fn web_route_publishes_event_that_increments_store() {
    let server = start_app().await;

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
async fn multiple_events_accumulate_in_store() {
    let server = start_app().await;

    let _ = http("POST", "/users/create").await;
    let _ = http("POST", "/users/create").await;

    assert_eq!(
        poll_count(2).await,
        2,
        "store did not accumulate every published event"
    );

    server.abort();
}
