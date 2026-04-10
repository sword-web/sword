use sword_core::Interceptor;
use tonic::{Request, Status};

/// Result alias used by gRPC request interceptors.
pub type GrpcInterceptorResult = Result<Request<()>, Status>;

/// Async interceptor trait for gRPC controllers.
#[allow(async_fn_in_trait)]
pub trait OnRequest: Interceptor {
    async fn on_request(&self, req: Request<()>) -> GrpcInterceptorResult;
}

/// Async interceptor trait for gRPC controllers with route-specific config.
#[allow(async_fn_in_trait)]
pub trait OnRequestWithConfig<C>: Interceptor {
    async fn on_request(&self, config: C, req: Request<()>) -> GrpcInterceptorResult;
}
