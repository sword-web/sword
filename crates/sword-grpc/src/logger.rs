use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use http::{Request as HttpRequest, Response as HttpResponse};
use serde::{Deserialize, Serialize};
use tonic::Code;
use tower::{Layer, Service};
use tracing::Level;

const REQUEST_ID_LENGTH: usize = 8;

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
    enabled: bool,
    skip_paths: Arc<Vec<String>>,
    level: GrpcLoggerLevel,
}

impl GrpcLoggerLayer {
    pub fn new(config: &GrpcLoggerConfig) -> Self {
        Self {
            enabled: config.enabled,
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
            enabled: self.enabled,
            skip_paths: self.skip_paths.clone(),
            level: self.level,
        }
    }
}

/// Wraps a tonic service and logs one access line per completed request.
#[derive(Clone)]
pub struct GrpcLogger<S> {
    inner: S,
    enabled: bool,
    skip_paths: Arc<Vec<String>>,
    level: GrpcLoggerLevel,
}

impl<S, ResBody> Service<HttpRequest<tonic::body::Body>> for GrpcLogger<S>
where
    S: Service<HttpRequest<tonic::body::Body>, Response = HttpResponse<ResBody>>
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
    S::Error: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = HttpResponse<ResBody>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: HttpRequest<tonic::body::Body>) -> Self::Future {
        let path = request.uri().path().to_string();
        let request_id = request
            .headers()
            .get("x-request-id")
            .and_then(|value| value.to_str().ok())
            .map(|id| id.chars().take(REQUEST_ID_LENGTH).collect::<String>())
            .unwrap_or_else(|| "-".to_string());

        let started = Instant::now();

        let enabled = self.enabled;
        let skip_paths = self.skip_paths.clone();
        let level = self.level;
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let response = inner.call(request).await?;

            if enabled && !skip_paths.iter().any(|p| path == *p || path.starts_with(p)) {
                log_grpc_request(level, &path, &request_id, &response, started.elapsed());
            }

            Ok(response)
        })
    }
}

fn log_grpc_request<ResBody>(
    level: GrpcLoggerLevel,
    path: &str,
    request_id: &str,
    response: &HttpResponse<ResBody>,
    latency: Duration,
) {
    let code = response
        .headers()
        .get("grpc-status")
        .and_then(|value| value.to_str().ok())
        .map(|value| Code::from_bytes(value.as_bytes()))
        .unwrap_or(Code::Ok);

    let code_name = code_name(code);

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
            status = %code_name,
            latency = %format_latency(latency),
            request_id = %request_id,
            "gRPC {path}",
        ),
        Level::WARN => tracing::warn!(
            target: "sword.grpc.logger",
            status = %code_name,
            latency = %format_latency(latency),
            request_id = %request_id,
            "gRPC {path}",
        ),
        _ => tracing::info!(
            target: "sword.grpc.logger",
            status = %code_name,
            latency = %format_latency(latency),
            request_id = %request_id,
            "gRPC {path}"
        ),
    }
}

fn code_name(code: Code) -> &'static str {
    match code {
        Code::Ok => "ok",
        Code::Cancelled => "cancelled",
        Code::Unknown => "unknown",
        Code::InvalidArgument => "invalid_argument",
        Code::DeadlineExceeded => "deadline_exceeded",
        Code::NotFound => "not_found",
        Code::AlreadyExists => "already_exists",
        Code::PermissionDenied => "permission_denied",
        Code::ResourceExhausted => "resource_exhausted",
        Code::FailedPrecondition => "failed_precondition",
        Code::Aborted => "aborted",
        Code::OutOfRange => "out_of_range",
        Code::Unimplemented => "unimplemented",
        Code::Internal => "internal",
        Code::Unavailable => "unavailable",
        Code::DataLoss => "data_loss",
        Code::Unauthenticated => "unauthenticated",
    }
}

/// Formats a latency with a human-friendly unit: `ms`, `s`, then `m` for long requests.
fn format_latency(elapsed: Duration) -> String {
    if elapsed >= Duration::from_secs(60) {
        let mins = elapsed.as_secs() / 60;
        let secs = elapsed.as_secs() % 60;
        format!("{mins}m {secs}s")
    } else if elapsed >= Duration::from_secs(1) {
        format!("{:.1}s", elapsed.as_secs_f64())
    } else {
        format!("{}ms", elapsed.as_millis())
    }
}
