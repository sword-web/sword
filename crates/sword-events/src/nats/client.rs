use std::sync::{Arc, OnceLock};

use async_nats::{Client, ConnectError};

#[derive(Clone)]
pub struct NatsClient {
    inner: Arc<OnceLock<Client>>,
    url: String,
}

impl NatsClient {
    pub fn new(url: String) -> Self {
        Self {
            inner: Arc::new(OnceLock::new()),
            url,
        }
    }

    pub async fn connect(&self) -> Result<(), ConnectError> {
        let client = async_nats::connect(&self.url).await?;
        let _ = self.inner.set(client);

        Ok(())
    }

    pub fn get(&self) -> Option<&Client> {
        self.inner.get()
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}
