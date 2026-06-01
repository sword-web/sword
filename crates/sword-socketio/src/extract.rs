use crate::config::SocketIoParser;
use crate::error::SocketError;

use axum::http::{Extensions as HttpExtensions, HeaderMap, request::Parts};
use bytes::Bytes;
use parking_lot::RwLock;
use serde::{Serialize, de::DeserializeOwned};
use socketioxide::{
    ProtocolVersion, SendError, TransportType,
    ack::AckStream,
    adapter::{Adapter as SocketIoAdapter, LocalAdapter},
    extensions::Extensions,
    extract::{AckSender, Event, SocketRef},
    handler::{FromConnectParts, FromDisconnectParts, FromMessageParts},
    operators::{BroadcastOperators, ConfOperators},
    socket::{DisconnectReason, Socket},
};

use socketioxide::handler::Value;
use socketioxide_core::{
    Sid,
    adapter::{Room, RoomParam},
    parser::ParseError,
};

use std::{convert::Infallible, sync::Arc, time::Duration};

#[cfg(feature = "validation-validator")]
use validator::Validate;

enum SocketKind {
    Connection,
    Message,
    Disconnection,
}

/// A unified extractor that combines multiple socketioxide extractors into a single context.
///
/// Provides access to socket operations, message data, acknowledgments, event names,
/// and disconnect reasons depending on the handler type.
pub struct SocketContext<A: SocketIoAdapter = LocalAdapter> {
    socket: SocketRef<A>,
    data: RwLock<Option<Value>>,
    ack: Option<AckSender<A>>,
    disconnect_reason: Option<DisconnectReason>,
    event: Option<Box<str>>,
    kind: SocketKind,
}

impl<A> SocketContext<A>
where
    A: SocketIoAdapter,
{
    fn parser(&self) -> SocketIoParser {
        self.socket
            .req_parts()
            .extensions
            .get::<SocketIoParser>()
            .cloned()
            .unwrap_or_default()
    }

    /// Deserialize the query parameters from the initial HTTP request to the specified type.
    ///
    /// Returns `Ok(None)` if no query string is present or if it is empty.
    ///
    /// # Errors
    /// Returns an error if the query string cannot be deserialized into the requested type.
    pub fn query<T: DeserializeOwned>(&self) -> Result<Option<T>, SocketError> {
        let Some(query_string) = self.socket.req_parts().uri.query() else {
            return Ok(None);
        };

        if query_string.is_empty() {
            return Ok(None);
        }

        let deserializer =
            serde_urlencoded::Deserializer::new(form_urlencoded::parse(query_string.as_bytes()));

        let deserialized =
            T::deserialize(deserializer).map_err(|e| SocketError::Deserialization {
                message: "Failed to deserialize query params to the required type.".into(),
                err: e.into(),
            })?;

        Ok(Some(deserialized))
    }

    /// The `TryData<T>` extractor equivalent method.
    ///
    /// Deserializes message data to the specified type.
    ///
    /// # Errors
    ///
    /// Returns an error if the payload has already been consumed by a previous
    /// call to `try_data`, or if the incoming payload cannot be decoded using
    /// the parser configured for the current Socket.IO server.
    pub fn try_data<T: DeserializeOwned>(&self) -> Result<T, SocketError> {
        let mut data = self.data.write().take().ok_or(ParseError::InvalidData)?;

        let result = match self.kind {
            SocketKind::Connection => self.parser().decode_default(Some(&data)),
            _ => self.parser().decode_value(&mut data, true),
        }
        .inspect_err(|e| tracing::error!(e = ?e))
        .map_err(SocketError::from)?;

        Ok(result)
    }

    #[cfg(feature = "validation-validator")]
    /// # Errors
    ///
    /// Returns an error if payload decoding fails, if the payload was already
    /// consumed, or if the decoded value does not satisfy its `validator`
    /// constraints.
    pub fn try_validated_data<T>(&self) -> Result<T, SocketError>
    where
        T: DeserializeOwned + Validate + std::fmt::Debug,
    {
        let data = self.try_data::<T>()?;

        data.validate()?;

        Ok(data)
    }

    /// Returns the event name for message handlers.
    /// Returns `None` for connect/disconnect handlers (protocol-level events).
    pub fn event(&self) -> Option<&str> {
        self.event.as_deref()
    }

    /// Sends an acknowledgment response to the client.
    ///
    /// # Errors
    ///
    /// Returns an error if the current event does not support acknowledgments
    /// or if the underlying socket cannot send the acknowledgment payload.
    pub fn ack<D>(self, data: &D) -> Result<(), SendError>
    where
        D: Serialize + ?Sized,
    {
        let Some(ack) = self.ack else {
            return Err(SendError::Socket(socketioxide::SocketError::Closed));
        };

        ack.send(data)?;

        Ok(())
    }

    /// Returns the socket's unique identifier.
    ///
    /// See [SocketRef](https://docs.rs/socketioxide/latest/socketioxide/extract/struct.SocketRef.html) for full documentation.
    pub fn id(&self) -> &Sid {
        &self.socket.id
    }

    /// Checks if an acknowledgment sender is available.
    pub fn has_ack(&self) -> bool {
        self.ack.is_some()
    }

    /// Checks if data is still available (not consumed by `try_data()`).
    pub fn has_data(&self) -> bool {
        self.data.read().is_some()
    }

    /// Returns access to the socket's extension storage.
    pub fn extensions(&self) -> &Extensions {
        &self.socket.extensions
    }

    /// Returns the req parts from the incoming http request.
    pub fn req_parts(&self) -> &Parts {
        self.socket.req_parts()
    }

    /// Broadcast to all sockets without any filtering (except the current socket).
    /// If you want to include the current socket use the broadcast operators from the io global context.
    ///
    /// See [SocketRef::broadcast](https://docs.rs/socketioxide/latest/socketioxide/extract/struct.SocketRef.html#method.broadcast) for full documentation.
    pub fn broadcast(&self) -> BroadcastOperators<A> {
        self.socket.broadcast()
    }

    /// Filter out all sockets selected with the previous operators that are in the specified rooms.
    ///
    /// See [SocketRef::except](https://docs.rs/socketioxide/latest/socketioxide/extract/struct.SocketRef.html#method.except) for full documentation.
    pub fn except(&self, rooms: impl RoomParam) -> BroadcastOperators<A> {
        self.socket.except(rooms)
    }

    /// Set a custom timeout when sending a message with an acknowledgement.
    /// Configure `ack_timeout` or see defaults for more details.
    /// See emit_with_ack() for more details on acknowledgements.
    ///
    /// See [SocketRef::timeout](https://docs.rs/socketioxide/latest/socketioxide/extract/struct.SocketRef.html#method.timeout) for full documentation.
    pub fn timeout(&self, timeout: Duration) -> ConfOperators<'_, A> {
        self.socket.timeout(timeout)
    }

    /// Select specific rooms to send to.
    ///
    /// See [SocketRef::to](https://docs.rs/socketioxide/latest/socketioxide/extract/struct.SocketRef.html#method.to) for full documentation.
    pub fn to(&self, rooms: impl RoomParam) -> BroadcastOperators<A> {
        self.socket.to(rooms)
    }

    /// Select specific rooms to send to (alias for `to`).
    ///
    /// See [SocketRef::within](https://docs.rs/socketioxide/latest/socketioxide/extract/struct.SocketRef.html#method.within) for full documentation.
    pub fn within(&self, rooms: impl RoomParam) -> BroadcastOperators<A> {
        self.socket.within(rooms)
    }

    /// Return true if the socket is connected to the namespace.
    ///
    /// See [SocketRef::connected](https://docs.rs/socketioxide/latest/socketioxide/extract/struct.SocketRef.html#method.connected) for full documentation.
    pub fn connected(&self) -> bool {
        self.socket.connected()
    }

    /// Emit a message to one or many clients
    ///
    /// If you provide tuple-like data (tuples, arrays), it will be considered as multiple arguments.
    /// Therefore, if you want to send an array as the first argument of the payload,
    /// you need to wrap it in an array or a tuple. A `Vec` will always be considered a single argument.
    ///
    /// # Emitting binary data
    /// To emit binary data, you must use a data type that implements `Serialize` as binary data.
    /// Currently, if you use `Vec<u8>`, it will be considered a sequence of numbers rather than binary data.
    /// To handle this, you can either use a special type like `Bytes` or the `serde_bytes` crate.
    /// If you want to emit generic data that may contain binary, use `rmpv::Value` instead of
    /// `serde_json::Value`, as binary data will otherwise be serialized as a sequence of numbers.
    ///
    /// # Errors
    /// * When encoding the data, a `SendError::Serialize` may be returned.
    /// * If the underlying engine.io connection is closed, a `SendError::Socket(SocketError::Closed)`
    ///   will be returned, and the data you attempted to send will be included in the error.
    /// * If the packet buffer is full, a `SendError::Socket(SocketError::InternalChannelFull)`
    ///   will be returned, and the data you attempted to send will be included in the error.
    ///   See the `max_buffer_size` key in the `[socketio]` configuration section for more information
    ///   on internal buffer configuration.
    pub fn emit<T: Serialize>(&self, event: impl AsRef<str>, data: &T) -> Result<(), SocketError> {
        self.socket.emit(event, data).map_err(SocketError::from)
    }

    /// Emit an event to the client and wait for an acknowledgement.
    ///
    /// See [SocketRef::emit_with_ack](https://docs.rs/socketioxide/latest/socketioxide/extract/struct.SocketRef.html#method.emit_with_ack) for full documentation.
    pub fn emit_with_ack<T: ?Sized + Serialize, V>(
        &self,
        event: impl AsRef<str>,
        data: &T,
    ) -> Result<AckStream<V>, SocketError> {
        self.socket
            .emit_with_ack(event, data)
            .map_err(SocketError::from)
    }

    /// Broadcast to all sockets only connected to this node.
    /// When using the default in-memory adapter, this operator is a no-op.
    ///
    /// See [SocketRef::local](https://docs.rs/socketioxide/latest/socketioxide/extract/struct.SocketRef.html#method.local) for full documentation.
    pub fn local(&self) -> BroadcastOperators<A> {
        self.socket.local()
    }

    /// Returns access to HTTP request extensions.
    pub fn http_extensions(&self) -> &HttpExtensions {
        &self.socket.req_parts().extensions
    }

    /// Get the SocketIO protocol version used by the client.
    ///
    /// See [SocketRef::protocol](https://docs.rs/socketioxide/latest/socketioxide/extract/struct.SocketRef.html#method.protocol) for full documentation.
    pub fn protocol_version(&self) -> ProtocolVersion {
        self.socket.protocol()
    }

    /// Get the transport type used by the client (e.g. Polling, Websocket).
    ///
    /// See [SocketRef::transport_type](https://docs.rs/socketioxide/latest/socketioxide/extract/struct.SocketRef.html#method.transport_type) for full documentation.
    pub fn transport_type(&self) -> TransportType {
        self.socket.transport_type()
    }

    /// # Errors
    ///
    /// Returns an error if the underlying Socket.IO connection cannot be
    /// closed cleanly.
    pub fn disconnect(self) -> Result<(), SocketError> {
        self.socket.disconnect().map_err(SocketError::from)
    }

    /// Returns the reason for socket disconnection if this context was created from a disconnect event.
    ///
    /// **Returns `None` for:**
    /// - **Connect handlers**: No disconnection has occurred yet
    /// - **Message handlers**: The socket is still connected and processing messages
    ///
    /// **Returns `Some(reason)` for:**
    /// - **Disconnect handlers**: Provides the specific reason why the socket disconnected
    ///   (e.g., client disconnect, server disconnect, transport error, etc.)
    pub fn disconnect_reason(&self) -> Option<&DisconnectReason> {
        self.disconnect_reason.as_ref()
    }

    /// Returns a reference to the socket's request headers.
    /// Shortcut for `&socket.req_parts().headers`.
    pub fn headers(&self) -> &HeaderMap {
        &self.socket.req_parts().headers
    }

    /// Returns a reference to the socket's Authorization http header.
    /// Shortcut for `&socket.req_parts().headers.get('Authorization')`.
    pub fn authorization(&self) -> Option<&str> {
        self.socket
            .req_parts()
            .headers
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
    }

    /// Add the current socket to the specified room(s).
    ///
    /// See [SocketRef::join](https://docs.rs/socketioxide/latest/socketioxide/extract/struct.SocketRef.html#method.join) for full documentation.
    pub fn join(&self, rooms: impl RoomParam) {
        self.socket.join(rooms)
    }

    /// Remove the current socket from the specified room(s).
    ///
    /// See [SocketRef::leave](https://docs.rs/socketioxide/latest/socketioxide/extract/struct.SocketRef.html#method.leave) for full documentation.
    pub fn leave(&self, rooms: impl RoomParam) {
        self.socket.leave(rooms)
    }

    /// Remove the current socket from all its rooms.
    ///
    /// See [SocketRef::leave_all](https://docs.rs/socketioxide/latest/socketioxide/extract/struct.SocketRef.html#method.leave_all) for full documentation.
    pub fn leave_all(&self) {
        self.socket.leave_all()
    }

    /// Get all room names this socket is connected to.
    ///
    /// See [SocketRef::rooms](https://docs.rs/socketioxide/latest/socketioxide/extract/struct.SocketRef.html#method.rooms) for full documentation.
    pub fn rooms(&self) -> Vec<Room> {
        self.socket.rooms()
    }

    /// Get the current namespace path for this socket.
    ///
    /// See [SocketRef::ns](https://docs.rs/socketioxide/latest/socketioxide/extract/struct.SocketRef.html#method.ns) for full documentation.
    pub fn ns(&self) -> &str {
        self.socket.ns()
    }

    #[doc(hidden)]
    pub fn socket_ref(&self) -> &SocketRef<A> {
        &self.socket
    }
}

impl<A> FromMessageParts<A> for SocketContext<A>
where
    A: SocketIoAdapter,
{
    type Error = Infallible;

    fn from_message_parts(
        s: &Arc<Socket<A>>,
        v: &mut Value,
        ack_id: &Option<i64>,
    ) -> Result<Self, Self::Error> {
        let ack = ack_id.and_then(|id| AckSender::from_message_parts(s, v, &Some(id)).ok());

        let event = Event::from_message_parts(s, v, ack_id)
            .ok()
            .map(|e| e.0.into_boxed_str());

        let data = std::mem::replace(v, Value::Bytes(Bytes::new()));
        let socket_ref = SocketRef::from_message_parts(s, v, ack_id)?;

        Ok(SocketContext {
            socket: socket_ref,
            data: RwLock::new(Some(data)),
            ack,
            disconnect_reason: None,
            event,
            kind: SocketKind::Message,
        })
    }
}

impl<A> FromConnectParts<A> for SocketContext<A>
where
    A: SocketIoAdapter,
{
    type Error = Infallible;

    fn from_connect_parts(s: &Arc<Socket<A>>, auth: &Option<Value>) -> Result<Self, Self::Error> {
        Ok(SocketContext {
            socket: SocketRef::from_connect_parts(s, auth)?,
            data: RwLock::new(auth.clone()),
            ack: None,
            disconnect_reason: None,
            event: None,
            kind: SocketKind::Connection,
        })
    }
}

impl<A> FromDisconnectParts<A> for SocketContext<A>
where
    A: SocketIoAdapter,
{
    type Error = Infallible;

    fn from_disconnect_parts(
        s: &Arc<Socket<A>>,
        reason: DisconnectReason,
    ) -> Result<Self, Self::Error> {
        Ok(SocketContext {
            socket: SocketRef::from_disconnect_parts(s, reason)?,
            data: RwLock::new(None),
            ack: None,
            disconnect_reason: Some(reason),
            event: None,
            kind: SocketKind::Disconnection,
        })
    }
}
