#![cfg(test)]

mod memory;
mod nats;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use tokio::net::TcpStream;
use tokio::spawn;
use tokio::time::sleep;

use sword::prelude::*;

pub const WEB_ADDR: &str = "127.0.0.1:8083";

pub const BASE: &str = r#"
[application]

[web]
host = "127.0.0.1"
port = 8083
body-limit = "1MB"
"#;

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

    pub fn enter(&self) {
        let current = self.in_flight.fetch_add(1, Ordering::SeqCst) + 1;
        self.max_in_flight.fetch_max(current, Ordering::SeqCst);
    }

    pub fn leave(&self) {
        self.in_flight.fetch_sub(1, Ordering::SeqCst);
    }

    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::SeqCst);
    }

    pub fn max(&self) -> usize {
        self.max_in_flight.load(Ordering::SeqCst)
    }

    pub fn hits(&self) -> usize {
        self.hits.load(Ordering::SeqCst)
    }
}

impl Default for ConcurrencyProbe {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for ConcurrencyProbe {}

pub struct TestModule;

impl Module for TestModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<crate::memory::UsersController>();
        controllers.register::<crate::memory::UserCreatedHandler>();
        controllers.register::<crate::memory::BurstController>();
        controllers.register::<crate::memory::BurstHandler>();
        controllers.register::<crate::nats::NatsController>();
        controllers.register::<crate::nats::NatsHandler>();
    }
}

/// Builds and runs the full application (web + events) and waits until the web
/// listener accepts connections.
pub async fn start_app(toml: &str) -> tokio::task::JoinHandle<()> {
    let config = Config::builder()
        .add_toml_str(toml)
        .build()
        .expect("config");

    let app = Application::from_config(config)
        .expect("application builder")
        .with_provider(EventStore::new())
        .with_provider(ConcurrencyProbe::new())
        .with_module::<TestModule>()
        .build()
        .expect("application");

    let handle = spawn(async move {
        let _ = app.run().await;
    });

    for _ in 0..100 {
        if TcpStream::connect(WEB_ADDR).await.is_ok() {
            return handle;
        }

        sleep(Duration::from_millis(20)).await;
    }

    panic!("web server did not become ready");
}
