use async_nats::Client;
use std::sync::Arc;

use crate::EventPublish;

pub struct NatsEventPublisher {
    client: Arc<Client>,
}

impl NatsEventPublisher {
    pub fn new(client: Arc<Client>) -> Self {
        Self { client }
    }
}

impl EventPublish for NatsEventPublisher {
    async fn publish(
        &self,
        event: impl crate::Event + 'static,
    ) -> Result<(), crate::EventPublishError> {
        Ok(())
    }
}
