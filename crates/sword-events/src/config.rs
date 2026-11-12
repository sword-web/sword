use std::time::Duration;

use serde::Deserialize;
use tokio::sync::Semaphore;

use sword_core::{ConfigCheck, ConfigItem, ConfigProblem, ConfigRegistrar, inventory_submit};

const DEFAULT_CONCURRENCY: usize = 256;
const DEFAULT_HANDLER_TIMEOUT_MS: u64 = 30_000;

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct EventApplicationConfig {
    pub concurrency: usize,
    pub handler_timeout_ms: u64,

    #[cfg(feature = "memory")]
    pub memory: Option<InMemoryEventsConfig>,

    #[cfg(feature = "nats")]
    pub nats: Option<NatsEventsConfig>,
}

impl Default for EventApplicationConfig {
    fn default() -> Self {
        Self {
            concurrency: DEFAULT_CONCURRENCY,
            handler_timeout_ms: DEFAULT_HANDLER_TIMEOUT_MS,
            #[cfg(feature = "memory")]
            memory: None,
            #[cfg(feature = "nats")]
            nats: None,
        }
    }
}

#[cfg(feature = "memory")]
#[derive(Clone, Debug, Deserialize)]
pub struct InMemoryEventsConfig {
    pub buffer_size: usize,
    pub num_of_event_retry: u8,
    pub delay_between_event_retry_ms: u64,
    pub handler_timeout_ms: Option<u64>,
}

#[cfg(feature = "nats")]
#[derive(Clone, Debug, Deserialize)]
pub struct NatsEventsConfig {
    pub url: String,
    pub handler_timeout_ms: Option<u64>,
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

        if self.concurrency == 0 {
            problems.push(
                ConfigProblem::new("concurrency", "`concurrency` must be greater than 0")
                    .with_hint("Remove the key to use 256, or set a positive number"),
            );
        } else if self.concurrency > Semaphore::MAX_PERMITS {
            problems.push(
                ConfigProblem::new("concurrency", "`concurrency` is too large")
                    .with_hint("Use a smaller value"),
            );
        }

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
            handler_timeout_ms: None,
        }
    }
}

#[cfg(feature = "nats")]
impl Default for NatsEventsConfig {
    fn default() -> Self {
        Self {
            url: "nats://127.0.0.1:4222".to_string(),
            handler_timeout_ms: None,
        }
    }
}

inventory_submit! {[
    ConfigRegistrar::new(|state, config| {
        state.insert(config.get_or_default::<EventApplicationConfig>());

        Ok(())
    })
]}

/// Resolves the handler timeout from the transport override and the global value.
///
/// A value of zero disables the timeout.
pub fn resolve_handler_timeout(global_ms: u64, override_ms: Option<u64>) -> Option<Duration> {
    let value = override_ms.unwrap_or(global_ms);

    (value > 0).then(|| Duration::from_millis(value))
}
