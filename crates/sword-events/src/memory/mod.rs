mod publisher;
mod subscriber;

use crate::{Event, EventApplicationConfig, EventKeyHandlerMap, application::HandlerEntry};

pub use publisher::MemoryEventPublisher;
use subscriber::MemoryEventSubscriber;

use std::{collections::HashMap, sync::Arc};
use sword_core::Shutdown;
use tokio::sync::mpsc;

/// Creates the in-memory transport components
/// Since the publisher and subscriber are separate components this struct contains
/// only the subscriber, the publisher is returned separately from the constructor.
pub(crate) struct MemoryTransport {
    subscriber: MemoryEventSubscriber,
}

impl MemoryTransport {
    pub(crate) fn new(config: &EventApplicationConfig) -> (MemoryEventPublisher, Self) {
        let engine_config = config.memory.clone().unwrap_or_default();

        let (tx, rx) = mpsc::channel::<Arc<dyn Event>>(engine_config.buffer_size);

        let publisher = MemoryEventPublisher::new(tx);
        let subscriber = MemoryEventSubscriber {
            receiver: rx,
            handlers: Default::default(),
            config: engine_config,
        };

        (publisher, Self { subscriber })
    }

    pub(crate) fn set_handlers(&mut self, entries: Vec<HandlerEntry>) {
        let mut map: EventKeyHandlerMap = HashMap::new();

        for entry in entries {
            map.entry(entry.key).or_default().push(entry.handler);
        }

        self.subscriber.handlers = map;
    }

    pub(crate) async fn run(self, shutdown: Shutdown) {
        self.subscriber.run(shutdown).await;
    }
}
