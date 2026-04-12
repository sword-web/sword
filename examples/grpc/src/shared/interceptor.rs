use sword::grpc::*;
use sword::prelude::*;

#[derive(Interceptor)]
pub struct AuthInterceptor;

impl OnRequest for AuthInterceptor {
    async fn on_request(&self, req: Request<()>) -> GrpcInterceptorResult {
        if req.metadata().get("authorization").is_none() {
            return Err(Status::unauthenticated("missing authorization metadata"));
        }

        tracing::info!("Authorization metadata found, proceeding with request");

        Ok(req)
    }
}

#[derive(Interceptor)]
pub struct LoggingInterceptor;

impl OnRequestWithConfig<&'static str> for LoggingInterceptor {
    async fn on_request(&self, config: &'static str, req: Request<()>) -> GrpcInterceptorResult {
        tracing::info!(
            "[gRPC] - Incoming request: ID: {} - Controller: {}",
            req.metadata()
                .get("request-id")
                .map(|v| v.to_str().unwrap_or("invalid UTF-8"))
                .unwrap_or("unknown"),
            config
        );

        Ok(req)
    }
}
