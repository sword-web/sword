//! Server-Sent Events (SSE) support.
//!
//! Re-exports axum's [`Sse`], [`Event`], and [`KeepAlive`] types, the
//! `stream!` / `try_stream!` macros from `async_stream`, and provides the
//! [`SseResult`] type alias for ergonomic SSE handler return types.
//!
//! Use the `#[sse]` route attribute to expose an SSE stream:
//!
//! ```rust,ignore
//! use std::convert::Infallible;
//! use sword::web::*;
//! use tokio_stream::{Stream, StreamExt};
//!
//! #[controller(kind = Controller::Web, path = "/sse")]
//! struct SseController;
//!
//! impl SseController {
//!     #[sse("/clock")]
//!     async fn clock(&self) -> Sse<impl Stream<Item = Result<Event, Infallible>> + use<>> {
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

use std::{convert::Infallible, pin::Pin};
use tokio_stream::Stream;

pub use async_stream::{stream, try_stream};
pub use axum::response::sse::{Event, KeepAlive, Sse};

pub type SseResult<T = Event> =
    axum::response::Sse<Pin<Box<dyn Stream<Item = Result<T, Infallible>> + Send>>>;
