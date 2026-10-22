mod publisher;
mod subscriber;

use crate::{Event, EventApplicationConfig, EventKeyHandlerMap, application::HandlerEntry};

pub use publisher::MemoryEventPublisher;
use subscriber::MemoryEventSubscriber;

use std::{collections::HashMap, sync::Arc};
use sword_core::{Shutdown, sword_error};
use tokio::sync::mpsc;

/// Creates the in-memory transport components
/// Since the publisher and subscriber are separate components this struct contains
/// only the subscriber, the publisher is returned separately from the constructor.
pub struct MemoryTransport {
    subscriber: MemoryEventSubscriber,
}

impl MemoryTransport {
    pub fn new(config: &EventApplicationConfig) -> (MemoryEventPublisher, Self) {
        let engine_config = config.memory.clone().unwrap_or_default();

        if engine_config.buffer_size == 0 {
            sword_error! {
                title: "Invalid in-memory events buffer size",
                reason: format!(
                    "`buffer-size` must be greater than 0, got {}",
                    engine_config.buffer_size,
                ),
                source: "MemoryTransport::new",
                context: {
                    "buffer-size" => engine_config.buffer_size,
                    "config-section" => "events.memory",
                },
                hints: [
                    "Set `buffer-size` to a positive value in the [events.memory] config section",
                    "The default is 1024",
                ],
            }
        }

        let (tx, rx) = mpsc::channel::<Arc<dyn Event>>(engine_config.buffer_size);

        let publisher = MemoryEventPublisher::new(tx);
        let subscriber = MemoryEventSubscriber {
            receiver: rx,
            handlers: Default::default(),
            config: engine_config,
        };

        (publisher, Self { subscriber })
    }

    pub fn set_handlers(&mut self, entries: Vec<HandlerEntry>) {
        let mut map: EventKeyHandlerMap = HashMap::new();

        for entry in entries {
            map.entry(entry.key).or_default().push(entry.handler);
        }

        self.subscriber.handlers = map;
    }

    pub async fn run(self, shutdown: Shutdown) {
        self.subscriber.run(shutdown).await;
    }
}
