mod publisher;
mod subscriber;

use crate::{Event, EventApplicationConfig, EventKeyHandlerMap, application::HandlerEntry};

pub use publisher::MemoryEventPublisher;
use subscriber::MemoryEventSubscriber;

use std::{collections::HashMap, sync::Arc};
use sword_core::diagnostic::{DiagnosticPhase, Error};
use sword_core::{Shutdown, sword_error};
use tokio::sync::mpsc;

/// Creates the in-memory transport components
/// Since the publisher and subscriber are separate components this struct contains
/// only the subscriber, the publisher is returned separately from the constructor.
pub struct MemoryTransport {
    subscriber: MemoryEventSubscriber,
}

impl MemoryTransport {
    pub fn new(config: &EventApplicationConfig) -> Result<(MemoryEventPublisher, Self), Error> {
        let engine_config = config.memory.clone().unwrap_or_default();

        if engine_config.buffer_size == 0 {
            return Err(sword_error! {
                code: "sword.build.events",
                phase: DiagnosticPhase::Build,
                target: "events.memory",
                title: "Invalid in-memory events buffer size",
                reason: format!(
                    "`buffer_size` must be greater than 0, got {}",
                    engine_config.buffer_size,
                ),
                source: "MemoryTransport::new",
                context: {
                    "buffer_size" => engine_config.buffer_size,
                },
                hints: [
                    "Set `buffer_size` to a positive value in the [events.memory] config section",
                    "The default is 1024",
                ],
            });
        }

        let (tx, rx) = mpsc::channel::<Arc<dyn Event>>(engine_config.buffer_size);

        let publisher = MemoryEventPublisher::new(tx);
        let subscriber = MemoryEventSubscriber {
            receiver: rx,
            handlers: Default::default(),
            config: engine_config,
        };

        Ok((publisher, Self { subscriber }))
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
