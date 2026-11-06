use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use duration_str::HumanFormat;
use serde::{Deserialize, Serialize};
use std::time::Instant;
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
    if logger_config
        .skip_paths
        .iter()
        .any(|p| req.uri().path() == p || req.uri().path().starts_with(p))
    {
        return next.run(req).await;
    }

    let uri = req.uri().clone();
    let method = req.method().clone();
    let started = Instant::now();

    let response = next.run(req).await;
    let status = response.status();
    let elapsed = started.elapsed();
    let latency = elapsed.human_format();

    let request_id = response
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(|id| id.chars().take(REQUEST_ID_LENGTH).collect::<String>())
        .unwrap_or_else(|| "-".to_string());

    let uri_for_log = if logger_config.log_query {
        uri.path_and_query().map_or("", |pq| pq.as_str())
    } else {
        uri.path()
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
            latency = latency,
            request_id = %request_id,
            "{} {uri_for_log}",
            method,
        ),
        Level::WARN => tracing::warn!(
            target: "sword.web.logger",
            status = status.as_u16(),
            latency = latency,
            request_id = %request_id,
            "{} {uri_for_log}",
            method,
        ),
        _ => tracing::info!(
            target: "sword.web.logger",
            status = status.as_u16(),
            latency = latency,
            request_id = %request_id,
            "{} {uri_for_log}",
            method,
        ),
    }

    response
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
