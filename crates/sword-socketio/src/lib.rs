pub mod config;
pub mod controller;
pub mod error;
pub mod extract;
mod integration;
pub mod interceptor;

pub(crate) use socketioxide::layer::SocketIoLayer;

pub mod prelude {
    pub use crate::config::{SocketIoParser, SocketIoServerConfig, SocketIoServerLayer};
    pub use crate::controller::{
        AckSender, Data, DisconnectReason, Event, Extension, HandlerRegistrar, HttpExtension,
        LocalAdapter, MaybeExtension, MaybeHttpExtension, ProtocolVersion, SocketEventKind,
        SocketIo, SocketIoController, SocketIoHandlerRegistrar, SocketRef, TransportType, TryData,
        on,
    };
    pub use crate::error::SocketError;
    pub use crate::extract::SocketContext;
    pub use crate::interceptor::OnConnect;
}

#[doc(hidden)]
pub mod internal {
    pub use crate::controller::{
        HandlerRegistrar, SocketEventKind, SocketIoController, SocketIoHandlerRegistrar,
    };
    pub use socketioxide::SocketError;
    pub use socketioxide::handler::ConnectHandler;
    pub use socketioxide::handler::connect::FromConnectParts;
}
