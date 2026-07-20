use crate::Event;

use std::sync::Arc;
use tokio::sync::mpsc::Sender;

#[derive(Clone)]
pub struct EventPublisher {
    sender: Sender<Arc<dyn Event>>,
}

impl EventPublisher {
    pub fn new(sender: Sender<Arc<dyn Event>>) -> Self {
        Self { sender }
    }

    pub async fn publish(&self, event: impl Event + 'static) {
        if let Err(e) = self.sender.send(Arc::new(event)).await {
            tracing::error!(target: "sword.events", error = %e, "Failed to publish event");
        }
    }

    pub fn try_publish(&self, event: impl Event + 'static) {
        if let Err(e) = self.sender.try_send(Arc::new(event)) {
            tracing::warn!(target: "sword.events", error = %e, "Failed to enqueue event (buffer full)");
        }
    }
}

impl sword_core::Provider for EventPublisher {}
