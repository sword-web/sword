#![cfg(test)]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use sword::events::{EventHandlerResult, EventPublisher, event, handle};
use sword::prelude::*;
use sword::web::*;

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
    async fn count(&self) -> WebResult<usize> {
        Ok(self.store.count())
    }
}

struct TestModule;

impl Module for TestModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<UsersController>();
        controllers.register::<UserCreatedHandler>();
    }
}

fn test_server() -> axum_test::TestServer {
    let app = Application::from_config_path("Config.toml")
        .with_provider(EventStore::new())
        .with_module::<TestModule>()
        .build();

    axum_test::TestServer::new(app.router())
}

/// Polls `GET /users/count` until it reaches `expected` or times out.
async fn poll_count(server: &axum_test::TestServer, expected: usize) -> usize {
    let mut count = 0;

    for _ in 0..200 {
        let body = server.get("/users/count").await.json::<JsonResponseBody>();

        count = body
            .data
            .as_ref()
            .and_then(|value| value.as_u64())
            .unwrap_or(0) as usize;

        if count >= expected {
            break;
        }

        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    count
}

#[tokio::test]
async fn web_route_publishes_event_that_increments_store() {
    let server = test_server();

    server.post("/users/create").await.assert_status_ok();

    assert_eq!(
        poll_count(&server, 1).await,
        1,
        "event handler did not increment the store"
    );
}

#[tokio::test]
async fn multiple_events_accumulate_in_store() {
    let server = test_server();

    server.post("/users/create").await.assert_status_ok();
    server.post("/users/create").await.assert_status_ok();

    assert_eq!(
        poll_count(&server, 2).await,
        2,
        "store did not accumulate every published event"
    );
}
