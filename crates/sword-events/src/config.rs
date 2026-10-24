use serde::Deserialize;
use sword_core::{
    ConfigErrors, ConfigIssue, ConfigItem, ConfigRegistrar, ValidateConfig, inventory_submit,
};

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

impl ValidateConfig for EventApplicationConfig {
    const SECTION: &'static str = "events";
    const CODE: &'static str = "sword.build.events";

    fn validate(&self) -> ConfigErrors {
        let mut errors = ConfigErrors::new();

        #[cfg(feature = "memory")]
        if let Some(memory) = &self.memory
            && memory.buffer_size == 0
        {
            errors.push(
                ConfigIssue::new(
                    "buffer_size",
                    "`buffer_size` must be greater than 0",
                    "must be greater than 0",
                )
                .with_section("events.memory")
                .with_hint("Set `buffer_size` to a positive value; the default is 1024"),
            );
        }

        #[cfg(feature = "nats")]
        if let Some(nats) = &self.nats
            && nats.url.trim().is_empty()
        {
            errors.push(
                ConfigIssue::new("url", "`url` must not be empty", "must not be empty")
                    .with_section("events.nats")
                    .with_hint("Point `url` at a reachable NATS server"),
            );
        }

        errors
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
    })
]}
