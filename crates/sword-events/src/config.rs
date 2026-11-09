use serde::Deserialize;

use sword_core::{ConfigCheck, ConfigItem, ConfigProblem, ConfigRegistrar, inventory_submit};

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(default)]
pub struct EventApplicationConfig {
    #[cfg(feature = "memory")]
    pub memory: Option<InMemoryEventsConfig>,

    #[cfg(feature = "nats")]
    pub nats: Option<NatsEventsConfig>,
}

#[cfg(feature = "memory")]
#[derive(Clone, Debug, Deserialize)]
pub struct InMemoryEventsConfig {
    pub buffer_size: usize,
    pub num_of_event_retry: u8,
    pub delay_between_event_retry_ms: u64,
}

#[cfg(feature = "nats")]
#[derive(Clone, Debug, Deserialize)]
pub struct NatsEventsConfig {
    pub url: String,
}

impl ConfigItem for EventApplicationConfig {
    fn key() -> &'static str {
        "events"
    }
}

impl ConfigCheck for EventApplicationConfig {
    const SECTION: &'static str = "events";
    const CODE: &'static str = "sword.build.events";

    fn check(&self) -> Result<(), Vec<ConfigProblem>> {
        let mut problems = Vec::new();

        #[cfg(feature = "memory")]
        if let Some(memory) = &self.memory
            && memory.buffer_size == 0
        {
            problems.push(
                ConfigProblem::new("buffer_size", "`buffer_size` must be greater than 0")
                    .with_hint("Remove the key to use 1024, or set a positive number"),
            );
        }

        #[cfg(feature = "nats")]
        if let Some(nats) = &self.nats
            && nats.url.trim().is_empty()
        {
            problems.push(
                ConfigProblem::new("url", "`url` must not be empty")
                    .with_hint("Use a URL such as nats://127.0.0.1:4222"),
            );
        }

        if problems.is_empty() {
            Ok(())
        } else {
            Err(problems)
        }
    }
}

#[cfg(feature = "memory")]
impl Default for InMemoryEventsConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1024,
            num_of_event_retry: 3,
            delay_between_event_retry_ms: 1000,
        }
    }
}

#[cfg(feature = "nats")]
impl Default for NatsEventsConfig {
    fn default() -> Self {
        Self {
            url: "nats://127.0.0.1:4222".to_string(),
        }
    }
}

inventory_submit! {[
    ConfigRegistrar::new(|state, config| {
        state.insert(config.get_or_default::<EventApplicationConfig>());

        Ok(())
    })
]}
