#![allow(async_fn_in_trait)]

mod application;
mod module;
pub mod prelude;

#[cfg(feature = "web-controllers")]
pub use sword_web::prelude as web;

#[cfg(feature = "socketio-controllers")]
pub use sword_socketio::prelude as socketio;

#[cfg(feature = "grpc-controllers")]
pub use sword_grpc::prelude as grpc;

pub use application::*;
pub use sword_macros::main;

#[doc(hidden)]
pub mod internal {
    #[cfg(feature = "web-controllers")]
    pub use sword_web::internal as web;

    #[cfg(feature = "socketio-controllers")]
    pub use sword_socketio::internal as socketio;

    #[cfg(feature = "grpc-controllers")]
    pub use sword_grpc::internal as grpc;

    pub mod core {
        pub use sword_core::*;
    }

    pub use inventory;
    pub use tokio::runtime as tokio_runtime;
    pub use tracing;

    #[cfg(feature = "hot-reload")]
    pub use dioxus_devtools;

    #[cfg(feature = "hot-reload")]
    pub use subsecond;
}
