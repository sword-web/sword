mod config;
mod registrar;

#[cfg(feature = "memory")]
pub mod application;

#[cfg(feature = "memory")]
pub mod memory;

pub mod prelude;

pub use config::EventApplicationConfig;
pub use registrar::*;

#[cfg(feature = "memory")]
pub use config::InMemoryEventsConfig;

#[cfg(feature = "nats")]
pub use config::NatsEventsConfig;

#[doc(hidden)]
pub mod internal {
    pub use crate::EventApplicationConfig;
    pub use crate::registrar::*;
}
