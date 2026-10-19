mod publisher;
mod subscriber;

use crate::EventApplicationConfig;
use async_nats::Client;
use sword_core::{Shutdown, sword_error};

pub struct NatsTransport {}

impl NatsTransport {
    pub async fn new(config: &EventApplicationConfig) -> Self {
        let nats_config = config.nats.clone().unwrap_or_default();

        let client = async_nats::connect(&nats_config.url)
            .await
            .unwrap_or_else(|error| {
                sword_error! {
                    title: "Failed to connect to NATS",
                    reason: error,
                    source: "NatsTransport::connect",
                    context: {
                        "url" => nats_config.url,
                    },
                    hints: [
                        "Ensure the NATS server is running and reachable at the configured url",
                        "Check the `[events.nats] url` value in your configuration",
                    ],
                }
            });

        todo!()

        // Self { client }
    }

    pub fn run(self, shutdown: Shutdown) {}
}
