use crate::{Event, EventHandlerFn};

use std::{sync::Arc, time::Duration};

/// Dispatches an event to every handler that belongs to a subscription.
///
/// Each handler runs in its own task so a slow handler does not block the
/// others. `retries` and `delay_ms` control the retry policy:
/// - in-memory events use the queue configuration;
/// - core NATS is at-most-once, so it passes `0` retries.
pub(crate) fn dispatch(
    handlers: &[EventHandlerFn],
    event: Arc<dyn Event>,
    retries: u8,
    delay_ms: u64,
) {
    for handler in handlers {
        let handler = handler.clone();
        let event = event.clone();

        tokio::spawn(async move {
            let mut remaining = retries;

            loop {
                match handler(event.clone()).await {
                    Ok(()) => break,
                    Err(error) => {
                        tracing::error!(
                            target: "sword.events",
                            key = event.key(),
                            error = %error,
                            retries_left = remaining,
                            "Event handler failed"
                        );

                        if remaining == 0 {
                            break;
                        }

                        remaining -= 1;

                        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    }
                }
            }
        });
    }
}
