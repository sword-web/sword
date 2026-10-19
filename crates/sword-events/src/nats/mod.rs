mod client;
mod publisher;
mod subscriber;

use std::collections::HashMap;

use crate::{EventApplicationConfig, EventHandlerFn, application::HandlerEntry};

use client::NatsClient;
pub use publisher::NatsEventPublisher;
use sword_core::{Shutdown, sword_error};
use tokio::task::JoinSet;

/// A NATS subscription: `(subject, group)`. `None` means an individual
/// subscription (fan-out); `Some` means a queue group (load balanced).
type NatsEventKey = (&'static str, Option<&'static str>);
type NatsEventHandlerMap = HashMap<NatsEventKey, Vec<EventHandlerFn>>;

pub struct NatsTransport {
    client: NatsClient,
    subscriptions: NatsEventHandlerMap,
}

impl NatsTransport {
    pub(crate) fn new(config: &EventApplicationConfig) -> (NatsEventPublisher, Self) {
        let nats_config = config.nats.clone().unwrap_or_default();

        let client = NatsClient::new(nats_config.url);
        let publisher = NatsEventPublisher::new(client.clone());

        let transport = Self {
            client,
            subscriptions: HashMap::new(),
        };

        (publisher, transport)
    }

    pub fn set_handlers(&mut self, entries: Vec<HandlerEntry>) {
        let mut by_subscription: NatsEventHandlerMap = HashMap::new();

        for entry in entries {
            by_subscription
                .entry((entry.key, entry.group))
                .or_default()
                .push(entry.handler);
        }

        self.subscriptions = by_subscription;
    }

    pub async fn run(self, shutdown: Shutdown) {
        self.client.connect().await.unwrap_or_else(|error| {
            sword_error! {
                title: "Failed to connect to NATS",
                reason: error,
                source: "NatsTransport::run",
                context: {
                    "url" => self.client.url(),
                },
                hints: [
                    "Ensure the NATS server is running and reachable",
                    "Check the `[events.nats] url` value in your configuration",
                ],
            }
        });

        let client = self.client.get().cloned().unwrap_or_else(|| {
            sword_error! {
                title: "Failed to get NATS client",
                reason: "Client is not connected",
                source: "NatsTransport::run",
            }
        });

        tracing::info!(
            target: "sword.events.nats",
            url = %self.client.url(),
            "Connected to NATS"
        );

        let mut tasks = JoinSet::new();

        for (key, handlers) in self.subscriptions {
            let client = client.clone();
            tasks.spawn(subscriber::consume(client, key, handlers));
        }

        shutdown.cancelled().await;

        let _ = client.drain().await;

        while tasks.join_next().await.is_some() {}
    }
}
