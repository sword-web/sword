use crate::extract::SocketContext;
use socketioxide::adapter::{Adapter as SocketIoSocketAdapter, LocalAdapter};
use std::fmt::Display;
use sword_core::Interceptor;

/// An interceptor that is called when a new Socket.IO connection is established.
///
/// ## Example
/// ```rust,ignore
/// use sword::prelude::*;
///
/// #[derive(Interceptor)]
/// pub struct MyInterceptor;
///
/// impl OnConnect for MyInterceptor {
///     type Error = String;
///     async fn on_connect(&self, socket: SocketContext) -> Result<(), Self::Error> {
///         println!("New connection established: {}", socket.id());
///     }
/// }
/// ```
///
/// ## Applying the Interceptor
/// For Socket.IO controllers, apply this interceptor at controller level using
/// `#[interceptor(...)]` on the struct annotated with `#[controller(...)]`.
/// ```rust,ignore
/// use sword::prelude::*;
///
/// #[derive(Interceptor)]
/// pub struct MyInterceptor;
///
/// #[controller(kind = Controller::SocketIo, namespace = "/my_namespace")]
/// #[interceptor(MyInterceptor)]
/// pub struct MyController;
///
/// impl MyController {
///     #[on("connection")]
///     async fn handle_connection(&self, socket: SocketContext) {
///        // handle the initial connection (logging, authentication)
///     }
/// }
/// ```
#[allow(async_fn_in_trait)]
pub trait OnConnect<A = LocalAdapter>: Interceptor
where
    A: SocketIoSocketAdapter,
{
    type Error: Display;

    async fn on_connect(&self, socket: SocketContext<A>) -> Result<(), Self::Error>;
}
