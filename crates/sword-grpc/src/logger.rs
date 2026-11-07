use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Instant,
};

use futures_util::future::Either;
use http::{Request, Response};
use serde::{Deserialize, Serialize};
use tonic::{Code, body::Body};
use tower::{Layer, Service};
use tracing::Level;

/// Configuration for the gRPC access logger, nested under `[grpc.logger]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GrpcLoggerConfig {
    /// Whether the access logger is enabled. Defaults to `true`.
    pub enabled: bool,

    /// RPC method paths to exclude from logging (exact or prefix match).
    #[serde(rename = "skip-paths")]
    pub skip_paths: Vec<String>,

    /// Log level policy. `auto` picks the level from the gRPC status code.
    pub level: GrpcLoggerLevel,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GrpcLoggerLevel {
    #[default]
    Auto,
    Info,
}

impl Default for GrpcLoggerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            skip_paths: Vec::new(),
            level: GrpcLoggerLevel::Auto,
        }
    }
}

impl GrpcLoggerConfig {
    /// A disabled config, used when no `[grpc.logger]` section is present.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Self::default()
        }
    }
}

/// A `tower::Layer` that wraps tonic services with request logging.
#[derive(Clone, Debug)]
pub struct GrpcLoggerLayer {
    skip_paths: Arc<Vec<String>>,
    level: GrpcLoggerLevel,
}

impl GrpcLoggerLayer {
    pub fn new(config: &GrpcLoggerConfig) -> Self {
        Self {
            skip_paths: Arc::new(config.skip_paths.clone()),
            level: config.level,
        }
    }
}

impl<S> Layer<S> for GrpcLoggerLayer {
    type Service = GrpcLogger<S>;

    fn layer(&self, inner: S) -> Self::Service {
        GrpcLogger {
            inner,
            skip_paths: self.skip_paths.clone(),
            level: self.level,
        }
    }
}

type BoxedFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

/// Wraps a tonic service and logs one access line per completed request.
#[derive(Clone)]
pub struct GrpcLogger<S> {
    inner: S,
    skip_paths: Arc<Vec<String>>,
    level: GrpcLoggerLevel,
}

impl<S, Out> Service<Request<Body>> for GrpcLogger<S>
where
    S: Service<Request<Body>, Response = Response<Out>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Send + 'static,
    Out: Send + 'static,
{
    type Response = Response<Out>;
    type Error = S::Error;
    type Future = Either<S::Future, BoxedFuture<Result<Self::Response, Self::Error>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<Body>) -> Self::Future {
        let path = request.uri().path();

        if self
            .skip_paths
            .iter()
            .any(|p| path == p.as_str() || path.starts_with(p.as_str()))
        {
            return Either::Left(self.inner.call(request));
        }

        let uri = request.uri().clone();
        let level = self.level;
        let mut inner = self.inner.clone();

        Either::Right(Box::pin(async move {
            let started = Instant::now();
            let response = inner.call(request).await?;
            let elapsed = started.elapsed();

            let request_id = response
                .headers()
                .get("x-request-id")
                .and_then(|value| value.to_str().ok())
                .unwrap_or("-");

            let code = response
                .headers()
                .get("grpc-status")
                .and_then(|value| value.to_str().ok())
                .map(|value| Code::from_bytes(value.as_bytes()))
                .unwrap_or(Code::Ok);

            let event_level = match level {
                GrpcLoggerLevel::Auto => match code {
                    Code::Ok => Level::INFO,
                    Code::InvalidArgument
                    | Code::NotFound
                    | Code::AlreadyExists
                    | Code::PermissionDenied
                    | Code::FailedPrecondition
                    | Code::OutOfRange
                    | Code::Unauthenticated
                    | Code::Aborted
                    | Code::Cancelled => Level::WARN,
                    _ => Level::ERROR,
                },
                GrpcLoggerLevel::Info => Level::INFO,
            };

            match event_level {
                Level::ERROR => tracing::error!(
                    target: "sword.grpc.logger",
                    status = ?code,
                    latency = ?elapsed,
                    request_id = %request_id,
                    rpc = %uri.path(),
                    "gRPC request completed",
                ),
                Level::WARN => tracing::warn!(
                    target: "sword.grpc.logger",
                    status = ?code,
                    latency = ?elapsed,
                    request_id = %request_id,
                    rpc = %uri.path(),
                    "gRPC request completed",
                ),
                _ => tracing::info!(
                    target: "sword.grpc.logger",
                    status = ?code,
                    latency = ?elapsed,
                    request_id = %request_id,
                    rpc = %uri.path(),
                    "gRPC request completed",
                ),
            }

            Ok(response)
        }))
    }
}
