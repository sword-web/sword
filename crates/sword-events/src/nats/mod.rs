mod client;
mod publisher;
mod subscriber;

use crate::{EventApplicationConfig, EventHandlerFn, EventRegistrar, application::HandlerEntry};

use client::NatsClient;
use std::{collections::HashMap, sync::Arc};
use sword_core::diagnostic::{DiagnosticPhase, Error};
use sword_core::{Shutdown, sword_error};
use tokio::task::JoinSet;

pub use publisher::NatsEventPublisher;

fn consumer_stopped() -> Error {
    sword_error! {
        code: "sword.runtime.events.nats",
        phase: DiagnosticPhase::Runtime,
        target: "NATS transport",
        title: "NATS consumer stopped before shutdown",
        reason: "a subscription finished without a shutdown request",
        source: "NatsTransport::run",
        hints: ["Check the NATS connection and server logs"],
    }
}

fn consumer_panicked(error: tokio::task::JoinError) -> Error {
    sword_error! {
        code: "sword.runtime.events.nats",
        phase: DiagnosticPhase::Runtime,
        target: "NATS transport",
        title: "NATS consumer task panicked",
        reason: error,
        source: "NatsTransport::run",
        hints: ["Check the panic message in the logs"],
        cause: error,
    }
}

/// A NATS subscription: `(subject, group)`. `None` means an individual
/// subscription (fan-out); `Some` means a queue group (load balanced).
type NatsEventKey = (&'static str, Option<&'static str>);
type NatsEventHandlerMap = HashMap<NatsEventKey, Vec<EventHandlerFn>>;

pub struct NatsTransport {
    client: NatsClient,
    subscriptions: NatsEventHandlerMap,
    events: Arc<HashMap<&'static str, &'static EventRegistrar>>,
}

impl NatsTransport {
    pub fn new(config: &EventApplicationConfig) -> (NatsEventPublisher, Self) {
        let nats_config = config.nats.clone().unwrap_or_default();

        let client = NatsClient::new(nats_config.url);
        let publisher = NatsEventPublisher::new(client.clone());

        let events = inventory::iter::<EventRegistrar>()
            .map(|registrar| (registrar.key, registrar))
            .collect();

        let transport = Self {
            client,
            subscriptions: HashMap::new(),
            events: Arc::new(events),
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

    pub async fn run(self, shutdown: Shutdown) -> Result<(), Error> {
        self.client.connect().await.map_err(|error| {
            sword_error! {
                code: "sword.runtime.events.nats",
                phase: DiagnosticPhase::Startup,
                target: "NATS transport",
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
                cause: error,
            }
        })?;

        let client = self.client.get().cloned().ok_or_else(|| {
            sword_error! {
                code: "sword.runtime.events.nats",
                phase: DiagnosticPhase::Startup,
                target: "NATS transport",
                title: "Failed to get NATS client",
                reason: "Client is not connected",
                source: "NatsTransport::run",
                hints: ["Ensure the NATS connection was established successfully"],
            }
        })?;

        tracing::info!(
            target: "sword.events.nats",
            url = %self.client.url(),
            "Connected to NATS"
        );

        let mut consumers = Vec::new();

        for ((subject, group), handlers) in self.subscriptions {
            let subscriber = match group {
                Some(group) => client.queue_subscribe(subject, group.into()).await,
                None => client.subscribe(subject).await,
            }
            .map_err(|error| {
                sword_error! {
                    code: "sword.runtime.events.nats",
                    phase: DiagnosticPhase::Startup,
                    target: "NATS transport",
                    title: "Failed to subscribe to NATS subject",
                    reason: error,
                    context: {
                        "subject" => subject,
                        "group" => group.unwrap_or("none"),
                    },
                    hints: [
                        "Check that the subject and queue group are valid",
                        "Check the NATS permissions for this client",
                    ],
                    cause: error,
                }
            })?;

            consumers.push((subscriber, handlers));
        }

        let mut tasks = JoinSet::new();

        for (subscriber, handlers) in consumers {
            tasks.spawn(subscriber::consume(
                subscriber,
                handlers,
                self.events.clone(),
            ));
        }

        if tasks.is_empty() {
            shutdown.cancelled().await;
            return Ok(());
        }

        let result = tokio::select! {
            _ = shutdown.cancelled() => {
                tasks.abort_all();
                return Ok(());
            }
            result = tasks.join_next() => result,
        };

        tasks.abort_all();

        match result {
            Some(Ok(())) => Err(consumer_stopped()),
            Some(Err(join_error)) => Err(consumer_panicked(join_error)),
            None => Ok(()),
        }
    }
}
