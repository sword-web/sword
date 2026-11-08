#![allow(async_fn_in_trait)]

mod application;
pub mod prelude;

pub use sword_core::{Diagnostic, Error, Module};
#[cfg(feature = "events")]
pub use sword_events::prelude as events;
#[cfg(feature = "grpc")]
pub use sword_grpc::prelude as grpc;
#[cfg(feature = "jobs")]
pub use sword_jobs::prelude as jobs;
pub use sword_macros::main;
#[cfg(feature = "socketio")]
pub use sword_socketio::prelude as socketio;
#[cfg(feature = "web")]
pub use sword_web::prelude as web;

pub use self::application::*;

#[doc(hidden)]
pub mod internal {
    #[cfg(feature = "hot-reload")]
    pub use dioxus_devtools;
    pub use inventory;
    #[cfg(feature = "hot-reload")]
    pub use subsecond;
    pub use tokio::runtime as tokio_runtime;
    pub use toml;
    pub use tracing;

    pub use sword_core as core;
    #[cfg(feature = "events")]
    pub use sword_events::internal as events;
    #[cfg(feature = "grpc")]
    pub use sword_grpc::internal as grpc;
    #[cfg(feature = "jobs")]
    pub use sword_jobs::internal as jobs;
    #[cfg(feature = "socketio")]
    pub use sword_socketio::internal as socketio;
    #[cfg(feature = "web")]
    pub use sword_web::internal as web;
}
