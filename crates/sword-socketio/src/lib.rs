#![allow(rustdoc::broken_intra_doc_links)]

mod config;
mod controller;
mod error;
mod extract;
mod integration;
mod interceptor;

pub use self::config::{SocketIoParser, SocketIoServerConfig};
pub use self::controller::{
    AckSender, AckStream, BroadcastOperators, ConfOperators, Data, DisconnectReason, Event,
    Extension, HttpExtension, LocalAdapter, MaybeExtension, MaybeHttpExtension, ProtocolVersion,
    Room, RoomParam, SendError, SocketIo, SocketRef, TransportType, TryData, on,
};
pub use self::error::SocketError;
pub use self::extract::SocketContext;
pub use self::interceptor::OnConnect;

pub mod prelude {
    pub use crate::config::{SocketIoParser, SocketIoServerConfig};
    pub use crate::controller::{
        AckSender, Data, DisconnectReason, Event, Extension, HttpExtension, LocalAdapter,
        MaybeExtension, MaybeHttpExtension, ProtocolVersion, SocketIo, SocketRef, TransportType,
        TryData, on,
    };
    pub use crate::error::SocketError;
    pub use crate::extract::SocketContext;
    pub use crate::interceptor::OnConnect;
}

/// Items required by generated code.
///
/// This module is public so procedural macros can name these items from the
/// user's crate, but it is not part of the stable API.
#[doc(hidden)]
pub mod internal {
    pub use socketioxide::handler::ConnectHandler;

    pub use crate::controller::{
        HandlerRegistrar, SocketEventKind, SocketIoController, SocketIoHandlerRegistrar,
    };
}
