use serde::{Deserialize, Serialize};
use sword::prelude::*;
use sword::socketio::*;

use crate::interceptors::LoggingInterceptor;
use crate::interceptors::SocketAuditInterceptor;

/// Socket.IO controller example with auto-registration and interceptors.
///
/// The `#[interceptor]` attribute can be used on the controller to apply interceptors
/// to all connection events. The interceptor is applied using `.with()` from socketioxide.
/// The connection handler method must be named `on_connect` for auto-registration.
/// Message handlers can have any name, but each handles a specific event.

#[derive(Debug, Serialize, Deserialize)]
struct Event {
    pub content: String,
}

#[controller(kind = Controller::SocketIo, namespace = "/events")]
#[interceptor(LoggingInterceptor)]
#[interceptor(SocketAuditInterceptor)]
pub struct EventsController;

impl EventsController {
    #[on("connection")]
    async fn on_connect_method(&self, ctx: SocketContext) {
        println!("Client connected: {}", ctx.id());
    }

    #[on("event")]
    async fn handle_message_event(&self, ctx: SocketContext) {
        let payload: Event = ctx.try_data().expect("Failed to parse event data");

        println!("Received 'event' from {}: {payload:?}", ctx.id());
    }

    #[on("eventWithAck")]
    async fn handle_message(&self, ctx: SocketContext) {
        let payload: Event = ctx.try_data().expect("Failed to parse event data");

        println!("Received 'eventWithAck' from {}: {payload:?}", ctx.id());

        if ctx.has_ack() {
            ctx.ack("ok").ok();
        }
    }
}
