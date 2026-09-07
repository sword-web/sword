use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tracing::Level;

const REQUEST_ID_LENGTH: usize = 8;

/// Configuration for the web access logger, nested under `[web.logger]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WebLoggerConfig {
    /// Whether the access logger is enabled. Defaults to `true`.
    pub enabled: bool,

    /// Request paths to exclude from logging (exact or prefix match).
    #[serde(rename = "skip-paths")]
    pub skip_paths: Vec<String>,

    /// Log level policy. `auto` picks the level from the response status.
    pub level: WebLoggerLevel,

    /// Whether to include the query string in the logged path. Defaults to `false`.
    #[serde(rename = "log-query")]
    pub log_query: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WebLoggerLevel {
    #[default]
    Auto,
    Info,
}

pub(crate) async fn web_logger_mw(
    State(logger_config): State<WebLoggerConfig>,
    req: Request,
    next: Next,
) -> Response {
    let path = req.uri().path().to_owned();
    let method = req.method().to_owned();
    let query = req.uri().query().map(|q| q.to_string());
    let started = Instant::now();

    let response = next.run(req).await;

    if logger_config
        .skip_paths
        .iter()
        .any(|p| path == *p || path.starts_with(p))
    {
        return response;
    }

    let elapsed = started.elapsed();
    let status = response.status();

    let request_id = response
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(|id| id.chars().take(REQUEST_ID_LENGTH).collect::<String>())
        .unwrap_or_else(|| "-".to_string());

    let uri = if logger_config.log_query {
        match &query {
            Some(q) => format!("{path}?{q}"),
            None => path,
        }
    } else {
        path
    };

    let event_level = match logger_config.level {
        WebLoggerLevel::Auto if status.is_success() || status.is_redirection() => Level::INFO,
        WebLoggerLevel::Auto if status.is_client_error() => Level::WARN,
        WebLoggerLevel::Auto => Level::ERROR,
        WebLoggerLevel::Info => Level::INFO,
    };

    match event_level {
        Level::ERROR => tracing::error!(
            target: "sword.web.logger",
            status = status.as_u16(),
            latency = %format_latency(elapsed),
            request_id = %request_id,
            "{} {uri}",
            method,
        ),
        Level::WARN => tracing::warn!(
            target: "sword.web.logger",
            status = status.as_u16(),
            latency = %format_latency(elapsed),
            request_id = %request_id,
            "{} {uri}",
            method,
        ),
        _ => tracing::info!(
            target: "sword.web.logger",
            status = status.as_u16(),
            latency = %format_latency(elapsed),
            request_id = %request_id,
            "{} {uri}",
            method,
        ),
    }

    response
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

impl Default for WebLoggerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            skip_paths: Vec::new(),
            level: WebLoggerLevel::Auto,
            log_query: false,
        }
    }
}
