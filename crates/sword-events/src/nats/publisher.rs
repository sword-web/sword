use super::client::NatsClient;
use crate::{Event, EventPublish, EventPublishError};

pub struct NatsEventPublisher {
    client: NatsClient,
}

impl NatsEventPublisher {
    pub(super) fn new(client: NatsClient) -> Self {
        Self { client }
    }
}

impl EventPublish for NatsEventPublisher {
    async fn publish(&self, event: impl Event) -> Result<(), EventPublishError> {
        let client = self.client.get().ok_or(EventPublishError::NotConnected)?;

        let bytes = event.to_vec()?;

        client
            .publish(event.key(), bytes.into())
            .await
            .map_err(|e| EventPublishError::Transport(Box::new(e)))
    }
}
