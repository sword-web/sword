#![allow(clippy::new_ret_no_self)]

pub mod layer_stack;

#[cfg(feature = "body-limit")]
pub mod body_limit {
    mod grpc;
    mod web;

    pub use grpc::*;
    pub use web::*;
}

#[cfg(feature = "compression")]
pub mod compression;

#[cfg(feature = "cookies")]
pub mod cookies {
    pub use tower_cookies::*;
}

#[cfg(feature = "cors")]
pub mod cors;

#[cfg(feature = "helmet")]
pub mod helmet;

#[cfg(feature = "not-found")]
pub mod not_found;

pub mod prelude;

#[cfg(feature = "body-limit")]
pub(crate) type ServiceLayer<Inner, Outer> = tower::ServiceBuilder<
    tower_layer::Stack<Inner, tower_layer::Stack<Outer, tower_layer::Identity>>,
>;

#[cfg(any(feature = "body-limit", feature = "req-timeout"))]
pub(crate) type MapResponseLayer = tower::util::MapResponseLayer<ResponseFnMapper>;
#[cfg(any(feature = "body-limit", feature = "req-timeout"))]
pub(crate) type ResponseFnMapper =
    fn(axum::response::Response<axum::body::Body>) -> axum::response::Response<axum::body::Body>;

#[cfg(feature = "request-id")]
pub mod request_id;

#[cfg(feature = "servedir")]
pub mod servedir;

#[cfg(feature = "tracing")]
pub mod tracing;

#[cfg(feature = "req-timeout")]
pub mod timeout;

pub trait DisplayConfig {
    fn display(&self);
}
