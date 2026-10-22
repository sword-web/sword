use super::NatsEventKey;
use crate::{EventHandlerFn, EventRegistrar, dispatch::dispatch};

use async_nats::Client;
use futures_lite::StreamExt;
use std::{collections::HashMap, sync::Arc};

/// Consumes a single `(subject, group)` subscription until the connection is drained or closed.
pub async fn consume(
    client: Client,
    (subject, group): NatsEventKey,
    handlers: Vec<EventHandlerFn>,
    events: Arc<HashMap<&'static str, &'static EventRegistrar>>,
) {
    let subscription = match group {
        Some(group) => client.queue_subscribe(subject, group.into()).await,
        None => client.subscribe(subject).await,
    };

    let Ok(mut subscription) = subscription else {
        tracing::error!(
            target: "sword.events.nats",
            subject,
            group = ?group,
            "Failed to subscribe to NATS subject"
        );

        return;
    };

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
            Ok(event) => dispatch(&handlers, Arc::from(event), 0, 0),
            Err(error) => tracing::error!(
                target: "sword.events.nats",
                subject = %message.subject,
                error = %error,
                "Failed to decode event"
            ),
        }
    }
}
