//! Server-Sent Events (SSE) support.
//!
//! Re-exports axum's [`Sse`], [`Event`], and [`KeepAlive`] types, the
//! `stream!` / `try_stream!` macros from `async_stream`, and defines the
//! [`EventStream`] marker trait for SSE handler return types.
//!
//! Use the `#[sse]` route attribute to expose an SSE stream:
//!
//! ```rust,ignore
//! use sword::web::*;
//! use tokio_stream::{Stream, StreamExt};
//!
//! #[controller(kind = Controller::Web, path = "/sse")]
//! struct SseController;
//!
//! impl SseController {
//!     #[sse("/clock")]
//!     async fn clock(&self) -> Sse<impl EventStream + use<>> {
//!         Sse::new(tokio_stream::iter(0..5).map(|i| {
//!             Ok(Event::default().event("tick").data(i.to_string()))
//!         }))
//!     }
//! }
//! ```
//!
//! > **Note:** SSE connections are long-lived. If `request-timeout` is enabled in the
//! > application config, the global timeout layer will terminate the stream, so leave it
//! > disabled or set a generous timeout for streaming routes.

use std::convert::Infallible;
use tokio_stream::Stream;

pub use async_stream::{stream, try_stream};
pub use axum::response::sse::{Event, KeepAlive, Sse};

/// A stream of Server-Sent Events.
///
/// Marker trait for any `Stream<Item = Result<Event, Infallible>> + Send + 'static`.
/// These are exactly the bounds axum requires for `Sse<S>: IntoResponse`, so a
/// handler returning `Sse<impl EventStream + use<>>` is always a valid SSE response
/// without boxing the stream or naming its concrete type.
pub trait EventStream: Stream<Item = Result<Event, Infallible>> + Send + 'static {}

impl<S> EventStream for S where S: Stream<Item = Result<Event, Infallible>> + Send + 'static {}
