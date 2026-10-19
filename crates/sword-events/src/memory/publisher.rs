use crate::{Event, EventPublish, EventPublishError};

use std::sync::Arc;
use tokio::sync::mpsc::Sender;

#[derive(Clone)]
pub struct MemoryEventPublisher {
    sender: Sender<Arc<dyn Event>>,
}

impl MemoryEventPublisher {
    pub(super) fn new(sender: Sender<Arc<dyn Event>>) -> Self {
        Self { sender }
    }
}

impl EventPublish for MemoryEventPublisher {
    async fn publish(&self, event: impl Event + 'static) -> Result<(), EventPublishError> {
        self.sender
            .send(Arc::new(event))
            .await
            .map_err(|_| EventPublishError::Closed)
    }
}

impl sword_core::Provider for MemoryEventPublisher {}
