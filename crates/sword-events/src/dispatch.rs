use std::{sync::Arc, time::Duration};

use tokio::spawn;
use tokio::sync::Semaphore;
use tokio::time::{sleep, timeout};

use crate::Event;
use crate::registrar::EventHandlerFn;

/// Dispatches an event to every handler that belongs to a subscription.
///
/// Each handler runs in its own task so a slow handler does not block the
/// others. A permit bounds how many handlers run at once. `limit` is the
/// maximum time a single attempt may take:
/// - in-memory events use the queue configuration;
/// - core NATS is at-most-once, so it passes `0` retries.
pub async fn dispatch(
    handlers: &[EventHandlerFn],
    event: Arc<dyn Event>,
    permits: &Arc<Semaphore>,
    retries: u8,
    delay_ms: u64,
    limit: Option<Duration>,
) {
    for handler in handlers {
        let Ok(permit) = permits.clone().acquire_owned().await else {
            tracing::debug!(target: "sword.events", "Event dispatcher stopped");
            return;
        };

        let handler = handler.clone();
        let event = event.clone();

        spawn(async move {
            let _permit = permit;
            run_handler(&handler, event, retries, delay_ms, limit).await;
        });
    }
}

async fn run_handler(
    handler: &EventHandlerFn,
    event: Arc<dyn Event>,
    retries: u8,
    delay_ms: u64,
    limit: Option<Duration>,
) {
    let mut remaining = retries;

    loop {
        let outcome = match limit {
            Some(limit) => timeout(limit, handler(event.clone())).await,
            None => Ok(handler(event.clone()).await),
        };

        match outcome {
            Ok(Ok(())) => return,
            Ok(Err(error)) => {
                tracing::error!(
                    target: "sword.events",
                    key = event.key(),
                    error = %error,
                    retries_left = remaining,
                    "Event handler failed"
                );
            }
            Err(_) => {
                tracing::error!(
                    target: "sword.events",
                    key = event.key(),
                    timeout_ms = limit.map(|limit| limit.as_millis()),
                    retries_left = remaining,
                    "Event handler timed out"
                );
            }
        }

        if remaining == 0 {
            return;
        }

        remaining -= 1;

        sleep(Duration::from_millis(delay_ms)).await;
    }
}
