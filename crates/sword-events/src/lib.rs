mod registrar;

#[cfg(feature = "in-memory")]
pub mod in_memory;

pub mod prelude;

pub use registrar::*;

use serde::Deserialize;
use sword_core::ConfigItem;

#[derive(Clone, Debug, Deserialize)]
pub struct EventQueueConfig {
    pub buffer_size: usize,
    pub num_of_event_retry: u8,
    pub delay_between_event_retry_ms: u64,
}

impl Default for EventQueueConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1024,
            num_of_event_retry: 3,
            delay_between_event_retry_ms: 1000,
        }
    }
}

impl ConfigItem for EventQueueConfig {
    fn key() -> &'static str {
        "event-queue"
    }
}

#[doc(hidden)]
pub mod internal {
    pub use crate::EventQueueConfig;
    pub use crate::registrar::*;
}
