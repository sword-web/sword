use crate::extract::SocketContext;
pub use socketioxide::{
    ProtocolVersion, SocketIo, TransportType,
    adapter::LocalAdapter,
    extract::{
        AckSender, Data, Event, Extension, HttpExtension, MaybeExtension, MaybeHttpExtension,
        SocketRef, TryData,
    },
    socket::DisconnectReason,
};
use std::any::{Any, TypeId};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use sword_core::{ControllerSpec, State};

pub use sword_macros::on;

/// Trait for providing Socket.IO controller functionality.
///
/// This trait is implemented by types annotated with
/// `#[controller(kind = Controller::SocketIo, namespace = "...")]`
/// and contains the actual handler implementations for Socket.IO events.
pub trait SocketIoController: ControllerSpec {
    fn namespace() -> &'static str;
}

#[derive(Clone, Debug)]
pub enum SocketEventKind {
    Connection,
    Disconnection,
    Message(&'static str),
    Fallback,
}

type SocketCallFn =
    fn(Arc<dyn Any + Send + Sync>, SocketContext) -> Pin<Box<dyn Future<Output = ()> + Send>>;

/// Metadata for a SocketIO event handler registered via the `#[on]` attribute.
///
/// This struct is used internally by Sword's auto-registration system to:
/// 1. Discover handlers at compile-time via the `inventory` crate
/// 2. Register them on the appropriate socket at runtime
#[derive(Clone)]
pub struct HandlerRegistrar {
    /// TypeId of the controller for filtering during registration
    pub controller_type_id: TypeId,

    /// Namespace of this controller (e.g., "/chat")
    pub namespace: &'static str,

    /// Kind of event this handler responds to
    pub event_kind: SocketEventKind,

    /// Name of the handler method
    pub method_name: &'static str,

    /// Registers the handler on the socket (used for Message/Disconnection/Fallback events).
    pub register_fn: fn(Arc<dyn Any + Send + Sync>, SocketRef),

    /// Executes the handler directly (used for Connection events).
    pub call_fn: SocketCallFn,
}

/// Setup function that initializes a SocketIO controller at runtime.
pub struct SocketIoHandlerRegistrar {
    pub handler_type_id: TypeId,
    pub handler_type_name: &'static str,

    pub setup_fn: fn(&State),
}

inventory::collect!(HandlerRegistrar);
inventory::collect!(SocketIoHandlerRegistrar);
