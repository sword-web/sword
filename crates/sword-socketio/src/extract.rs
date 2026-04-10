use crate::config::SocketIoParser;
use crate::error::SocketError;

use axum::http::Extensions as HttpExtensions;
use bytes::Bytes;
use parking_lot::RwLock;
use serde::{Serialize, de::DeserializeOwned};
use socketioxide::{
    ProtocolVersion, SendError, TransportType,
    adapter::{Adapter as SocketIoEngineAdapter, LocalAdapter},
    extensions::Extensions,
    extract::{AckSender, Event, SocketRef},
    handler::{FromConnectParts, FromDisconnectParts, FromMessageParts},
    socket::{DisconnectReason, Socket},
};
use socketioxide_core::{
    Sid, Value,
    parser::{Parse, ParseError},
};
use std::{convert::Infallible, sync::Arc};

#[cfg(feature = "validation-validator")]
use validator::Validate;

/// A unified extractor that combines multiple socketioxide extractors into a single context.
///
/// Provides access to socket operations, message data, acknowledgments, event names,
/// and disconnect reasons depending on the handler type.
pub struct SocketContext<A: SocketIoEngineAdapter = LocalAdapter> {
    pub socket: SocketRef<A>,
    data: RwLock<Option<Value>>,
    ack: Option<AckSender<A>>,
    disconnect_reason: Option<DisconnectReason>,
    event: Option<Box<str>>,
}

impl<A> SocketContext<A>
where
    A: SocketIoEngineAdapter,
{
    fn parser(&self) -> SocketIoParser {
        self.socket
            .req_parts()
            .extensions
            .get::<SocketIoParser>()
            .cloned()
            .unwrap_or_default()
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

        let result: T = match self.parser() {
            SocketIoParser::Common(parser) => parser.decode_value(&mut data, true),
            SocketIoParser::MsgPack(parser) => parser.decode_value(&mut data, true),
        }
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
        T: DeserializeOwned + Validate,
    {
        let data: T = self.try_data()?;

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

    /// Returns access to HTTP request extensions.
    pub fn http_extensions(&self) -> &HttpExtensions {
        &self.socket.req_parts().extensions
    }

    pub fn protocol_version(&self) -> ProtocolVersion {
        self.socket.protocol()
    }

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
}

impl<A> FromMessageParts<A> for SocketContext<A>
where
    A: SocketIoEngineAdapter,
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
        })
    }
}

impl<A> FromConnectParts<A> for SocketContext<A>
where
    A: SocketIoEngineAdapter,
{
    type Error = Infallible;

    fn from_connect_parts(s: &Arc<Socket<A>>, auth: &Option<Value>) -> Result<Self, Self::Error> {
        Ok(SocketContext {
            socket: SocketRef::from_connect_parts(s, auth)?,
            data: RwLock::new(auth.clone()),
            ack: None,
            disconnect_reason: None,
            event: None,
        })
    }
}

impl<A> FromDisconnectParts<A> for SocketContext<A>
where
    A: SocketIoEngineAdapter,
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
        })
    }
}
