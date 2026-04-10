use crate::request::{Request, StreamRequest};
use crate::response::JsonResponse;
use sword_core::Interceptor;

pub use axum::middleware::Next;
use axum::response::Response as AxumResponse;
use std::future::Future;

pub type WebInterceptorResult = Result<AxumResponse, JsonResponse>;

/// Trait for interceptors that handle requests
///
/// This is the standard interceptor trait for HTTP controller request interception.
/// Implement this trait to create interceptors that don't require additional
/// configuration at the route level.
pub trait OnRequest: Interceptor {
    /// Handles an incoming request.
    ///
    /// This method receives the request and the next interceptor in the chain.
    /// It should either call `req.next().await` to continue the chain or return early with
    /// a response to short-circuit the request.
    fn on_request(&self, req: Request) -> impl Future<Output = WebInterceptorResult>;
}

/// Trait for interceptors that handle requests with route-specific configuration.
///
/// This trait allows interceptors to receive configuration parameters when applied
/// to specific routes. The configuration type `C` is provided at compile time
/// through the `#[interceptor]` attribute.
pub trait OnRequestWithConfig<C>: Interceptor {
    /// Handles an incoming request with configuration.
    ///
    /// This method receives configuration, the request, and the next interceptor.
    /// The configuration is passed from the route definition and can be used to
    /// customize the interceptor behavior per route.
    fn on_request(&self, config: C, req: Request) -> impl Future<Output = WebInterceptorResult>;
}

/// Trait for interceptors that handle streaming requests.
///
/// This variant is intended for handlers that use `StreamRequest`, where the
/// request body is not eagerly buffered in memory.
pub trait OnRequestStream: Interceptor {
    fn on_request(&self, req: StreamRequest) -> impl Future<Output = WebInterceptorResult>;
}

/// Trait for interceptors that handle streaming requests with route-specific configuration.
pub trait OnRequestStreamWithConfig<C>: Interceptor {
    fn on_request(
        &self,
        config: C,
        req: StreamRequest,
    ) -> impl Future<Output = WebInterceptorResult>;
}
