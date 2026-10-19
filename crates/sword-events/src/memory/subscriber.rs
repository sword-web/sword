use crate::{Event, EventKeyHandlerMap, InMemoryEventsConfig};

use std::sync::Arc;
use tokio::sync::mpsc::Receiver;

/// Consumer of the in-memory event channel.
///
/// Holds the channel receiver, the handlers indexed by event key, and the
/// queue configuration used for retries.
///
/// The event handling flow is:
/// consume -> dispatch -> invoke (with retries)
/// (recv)     (per key)    (per handler)
pub(super) struct MemoryEventSubscriber {
    pub(super) receiver: Receiver<Arc<dyn Event>>,
    pub(super) handlers: EventKeyHandlerMap,
    pub(super) config: InMemoryEventsConfig,
}

impl MemoryEventSubscriber {
    /// Consumes events until the channel is closed or `shutdown` is cancelled.
    pub async fn run(self, shutdown: sword_core::Shutdown) {
        let MemoryEventSubscriber {
            mut receiver,
            handlers,
            config,
        } = self;

        loop {
            tokio::select! {
                maybe = receiver.recv() => match maybe {
                    Some(event) => Self::dispatch(&handlers, &config, event),
                    None => break,
                },
                _ = shutdown.cancelled() => break,
            }
        }

        tracing::info!(target: "sword.events", "Event subscriber stopped");
    }

    /// Looks up the handlers registered for the event key and dispatches the
    /// event to them.
    ///
    /// Events without registered handlers are ignored. Each handler runs
    /// concurrently in its own task so a slow handler does not block the others
    /// or the receive loop.
    fn dispatch(
        handlers: &EventKeyHandlerMap,
        config: &InMemoryEventsConfig,
        event: Arc<dyn Event>,
    ) {
        let key = event.key();

        let Some(entries) = handlers.get(key) else {
            tracing::debug!(target: "sword.events", key, "No handlers registered for event");
            return;
        };

        crate::dispatch::dispatch(
            entries,
            event,
            config.num_of_event_retry,
            config.delay_between_event_retry_ms,
        );
    }
}
