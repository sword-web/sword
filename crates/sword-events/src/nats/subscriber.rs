use crate::{EventHandlerFn, EventRegistrar, dispatch::dispatch};

use async_nats::Subscriber;
use futures_lite::StreamExt;
use std::{collections::HashMap, sync::Arc};

pub async fn consume(
    mut subscription: Subscriber,
    handlers: Vec<EventHandlerFn>,
    events: Arc<HashMap<&'static str, &'static EventRegistrar>>,
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
