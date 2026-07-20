#![allow(async_fn_in_trait)]

mod application;
pub mod prelude;

#[cfg(feature = "events-in-memory")]
pub use sword_events::prelude as events;

#[cfg(feature = "web")]
pub use sword_web::prelude as web;

#[cfg(feature = "socketio")]
pub use sword_socketio::prelude as socketio;

#[cfg(feature = "grpc")]
pub use sword_grpc::prelude as grpc;

pub use application::*;
pub use sword_core::Module;
pub use sword_macros::main;

#[doc(hidden)]
pub mod internal {
    #[cfg(feature = "web")]
    pub use sword_web::internal as web;

    #[cfg(feature = "socketio")]
    pub use sword_socketio::internal as socketio;

    #[cfg(feature = "grpc")]
    pub use sword_grpc::internal as grpc;

    pub mod core {
        pub use sword_core::*;
    }

    #[cfg(feature = "events-in-memory")]
    pub use sword_events::internal as events;

    pub use inventory;
    pub use tokio::runtime as tokio_runtime;
    pub use toml;
    pub use tracing;

    #[cfg(feature = "hot-reload")]
    pub use dioxus_devtools;

    #[cfg(feature = "hot-reload")]
    pub use subsecond;
}
