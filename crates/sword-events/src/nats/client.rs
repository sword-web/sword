use std::sync::{Arc, OnceLock};
use std::time::Duration;

use async_nats::{Client, ConnectError, ConnectOptions};

use crate::NatsEventsConfig;

#[derive(Clone)]
pub struct NatsClient {
    inner: Arc<OnceLock<Client>>,
    config: NatsEventsConfig,
}

impl NatsClient {
    pub fn new(config: NatsEventsConfig) -> Self {
        Self {
            config,
            inner: Arc::new(OnceLock::new()),
        }
    }

    pub async fn connect(&self) -> Result<(), ConnectError> {
        let mut options = ConnectOptions::new();

        if let Some(timeout_ms) = self.config.connection_timeout_ms {
            options = options.connection_timeout(Duration::from_millis(timeout_ms));
        }

        if self.config.retry_on_initial_connect {
            options = options.retry_on_initial_connect();
        }

        if let Some(max_reconnects) = self.config.max_reconnects {
            options = options.max_reconnects(max_reconnects);
        }

        if let Some(capacity) = self.config.subscription_capacity {
            options = options.subscription_capacity(capacity);
        }

        let client = options.connect(&self.config.url).await?;
        let _ = self.inner.set(client);

        Ok(())
    }

    pub fn get(&self) -> Option<&Client> {
        self.inner.get()
    }

    pub fn url(&self) -> &str {
        &self.config.url
    }
}
