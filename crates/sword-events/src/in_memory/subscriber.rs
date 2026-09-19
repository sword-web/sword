use crate::Event;
use crate::EventHandlerFn;
use crate::EventQueueConfig;
use crate::in_memory::EventKeyHandlerMap;

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Receiver;
use tokio::time::sleep;

/// Consumer of the in-memory event channel.
///
/// Holds the channel receiver, the handlers indexed by event key, and the
/// queue configuration used for retries.
///
/// The event  handling flow is:
/// consume  ->  dispatch  ->  invoke (with retries)
/// (recv)       (per key)      (per handler)
pub struct EventSubscriber {
    pub receiver: Receiver<Arc<dyn Event>>,
    pub handlers: EventKeyHandlerMap,
    pub config: EventQueueConfig,
}

impl EventSubscriber {
    /// Spawns the background consumer loop.
    ///
    /// Returns immediately: the actual consumption runs as a detached task and
    /// stops on its own once the channel is closed.
    pub fn run(self) {
        let EventSubscriber {
            receiver,
            handlers,
            config,
        } = self;

        tokio::spawn(Self::consume(receiver, handlers, config));
    }

    /// Receives events until the channel is closed and dispatches each one.
    ///
    /// This is the only place that awaits the channel, so it must stay cheap:
    /// handler execution is offloaded by [`Self::dispatch`].
    async fn consume(
        mut receiver: Receiver<Arc<dyn Event>>,
        handlers: EventKeyHandlerMap,
        config: EventQueueConfig,
    ) {
        while let Some(event) = receiver.recv().await {
            Self::dispatch(&handlers, &config, event);
        }

        tracing::info!(target: "sword.events", "Event subscriber stopped");
    }

    /// Looks up the handlers registered for the event key and spawns one task
    /// per handler.
    ///
    /// Events without registered handlers are ignored. Each handler runs concurrently in its own
    /// task so a slow handler does not block the others or the receive loop.
    fn dispatch(handlers: &EventKeyHandlerMap, config: &EventQueueConfig, event: Arc<dyn Event>) {
        let key = event.key();

        let Some(entries) = handlers.get(key) else {
            tracing::debug!(target: "sword.events", key, "No handlers registered for event");
            return;
        };

        for handle in entries {
            tokio::spawn(Self::invoke(
                handle.clone(),
                event.clone(),
                config.clone(),
                key,
            ));
        }
    }

    /// Runs a single handler, retrying on failure.
    ///
    /// Attempts up to `num_of_event_retry` retries, waiting `delay_between_event_retry_ms` between them.
    /// Failures are logged; once the retries are exhausted the event is dropped.
    async fn invoke(
        handle: EventHandlerFn,
        event: Arc<dyn Event>,
        config: EventQueueConfig,
        key: &'static str,
    ) {
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

                    if remaining == 0 {
                        break;
                    }

                    remaining -= 1;

                    sleep(Duration::from_millis(config.delay_between_event_retry_ms)).await;
                }
            }
        }
    }
}
