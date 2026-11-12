mod client;
mod publisher;
mod subscriber;

use std::{collections::HashMap, sync::Arc, time::Duration};

use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use client::NatsClient;
use sword_core::{DiagnosticPhase, Error, Shutdown, sword_error};

use crate::application::HandlerEntry;
use crate::config::{NatsEventsConfig, resolve_handler_timeout};
use crate::registrar::{EventHandlerFn, EventRegistrar};

pub use publisher::NatsEventPublisher;

/// A NATS subscription: `(subject, group)`. `None` means an individual
/// subscription (fan-out); `Some` means a queue group (load balanced).
type NatsEventKey = (&'static str, Option<&'static str>);
type NatsEventHandlerMap = HashMap<NatsEventKey, Vec<EventHandlerFn>>;

pub struct NatsTransport {
    client: NatsClient,
    subscriptions: NatsEventHandlerMap,
    events: Arc<HashMap<&'static str, &'static EventRegistrar>>,
    permits: Arc<Semaphore>,
    handler_timeout: Option<Duration>,
}

impl NatsTransport {
    pub fn new(
        nats_config: NatsEventsConfig,
        global_timeout_ms: u64,
        concurrency: usize,
    ) -> (NatsEventPublisher, Self) {
        let handler_timeout =
            resolve_handler_timeout(global_timeout_ms, nats_config.handler_timeout_ms);

        let client = NatsClient::new(nats_config);
        let publisher = NatsEventPublisher::new(client.clone());

        let events = inventory::iter::<EventRegistrar>()
            .map(|registrar| (registrar.key, registrar))
            .collect();

        let transport = Self {
            client,
            subscriptions: HashMap::new(),
            events: Arc::new(events),
            permits: Arc::new(Semaphore::new(concurrency)),
            handler_timeout,
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
                self.permits.clone(),
                self.handler_timeout,
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
            Some(Ok(())) => Err(sword_error! {
                code: "sword.runtime.events.nats",
                phase: DiagnosticPhase::Runtime,
                target: "NATS transport",
                title: "NATS consumer stopped before shutdown",
                reason: "a subscription finished without a shutdown request",
                source: "NatsTransport::run",
                hints: ["Check the NATS connection and server logs"],
            }),
            Some(Err(join_error)) => Err(sword_error! {
                code: "sword.runtime.events.nats",
                phase: DiagnosticPhase::Runtime,
                target: "NATS transport",
                title: "NATS consumer task panicked",
                reason: join_error,
                source: "NatsTransport::run",
                hints: ["Check the panic message in the logs"],
                cause: join_error,
            }),
            None => Ok(()),
        }
    }
}
