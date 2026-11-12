use std::{collections::HashMap, sync::Arc};

use async_nats::Subscriber;
use futures_lite::StreamExt;
use tokio::sync::Semaphore;

use crate::dispatch::dispatch;
use crate::registrar::{EventHandlerFn, EventRegistrar};

pub async fn consume(
    mut subscription: Subscriber,
    handlers: Vec<EventHandlerFn>,
    events: Arc<HashMap<&'static str, &'static EventRegistrar>>,
    permits: Arc<Semaphore>,
) {
    while let Some(message) = subscription.next().await {
        let Some(registrar) = events.get(message.subject.as_str()) else {
            tracing::warn!(
                target: "sword.events.nats",
                subject = %message.subject,
                "No event registered for subject"
            );

            continue;
        };

        match (registrar.from_json)(&message.payload) {
            Ok(event) => dispatch(&handlers, Arc::from(event), &permits, 0, 0).await,
            Err(error) => tracing::error!(
                target: "sword.events.nats",
                subject = %message.subject,
                error = %error,
                "Failed to decode event"
            ),
        }
    }
}
