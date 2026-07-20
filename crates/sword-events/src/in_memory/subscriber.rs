use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc::Receiver;
use tokio::sync::watch;

use crate::Event;
use crate::{EventHandlerFn, EventQueueConfig};

#[derive(Clone)]
struct HandlerEntry {
    handle: EventHandlerFn,
}

pub struct EventSubscriber {
    receiver: Option<Receiver<Arc<dyn Event>>>,
    handlers: HashMap<&'static str, Vec<HandlerEntry>>,
    config: EventQueueConfig,
}

impl EventSubscriber {
    pub fn new(
        receiver: Receiver<Arc<dyn Event>>,
        handlers: HashMap<&'static str, Vec<EventHandlerFn>>,
        config: EventQueueConfig,
    ) -> Self {
        let handlers = handlers
            .into_iter()
            .map(|(key, fns)| {
                let entries = fns
                    .into_iter()
                    .map(|handle| HandlerEntry { handle })
                    .collect();
                (key, entries)
            })
            .collect();

        Self {
            receiver: Some(receiver),
            handlers,
            config,
        }
    }

    pub fn run(mut self, mut shutdown: watch::Receiver<bool>) {
        tokio::spawn(async move {
            let mut receiver = self.receiver.take().unwrap();
            let config = self.config;

            loop {
                let event = tokio::select! {
                    event = receiver.recv() => {
                        match event {
                            Some(event) => event,
                            None => break,
                        }
                    }
                    _ = shutdown.changed() => {
                        if *shutdown.borrow() {
                            tracing::info!(target: "sword.events", "Shutdown signal received, stopping subscriber");
                            break;
                        }
                        continue;
                    }
                };

                let key = event.key();
                let Some(entries) = self.handlers.get(key) else {
                    tracing::debug!(target: "sword.events", key, "No handlers registered for event");
                    continue;
                };

                for entry in entries {
                    let handle = entry.handle.clone();
                    let event = event.clone();
                    let config = config.clone();

                    tokio::spawn(async move {
                        let mut remaining = config.num_of_event_retry;

                        loop {
                            match handle(event.clone()).await {
                                Ok(()) => break,
                                Err(e) => {
                                    tracing::error!(
                                        target: "sword.events",
                                        key,
                                        error = %e,
                                        retries_left = remaining,
                                        "Event handler failed"
                                    );

                                    if remaining > 0 {
                                        remaining -= 1;
                                        tokio::time::sleep(Duration::from_millis(
                                            config.delay_between_event_retry_ms,
                                        ))
                                        .await;
                                    } else {
                                        break;
                                    }
                                }
                            }
                        }
                    });
                }
            }

            tracing::info!(target: "sword.events", "Event subscriber stopped");
        });
    }
}
