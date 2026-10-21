use async_nats::Client;
use std::sync::{Arc, OnceLock};

#[derive(Clone)]
pub(super) struct NatsClient {
    inner: Arc<OnceLock<Client>>,
    url: String,
}

impl NatsClient {
    pub(super) fn new(url: String) -> Self {
        Self {
            inner: Arc::new(OnceLock::new()),
            url,
        }
    }

    pub(super) async fn connect(&self) -> Result<(), async_nats::ConnectError> {
        let client = async_nats::connect(&self.url).await?;
        let _ = self.inner.set(client);

        Ok(())
    }

    pub(super) fn get(&self) -> Option<&Client> {
        self.inner.get()
    }

    pub(super) fn url(&self) -> &str {
        &self.url
    }
}
