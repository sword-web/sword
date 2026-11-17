//! Request timeout middleware.
//!
//! This module provides the timeout configuration and a `from_fn` middleware
//! that cancels the inner future once the configured duration elapses and
//! answers with Sword's standardized JSON format.

use std::time::Duration;

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use sword_core::{DisplayConfig, TimeConfig};

use crate::JsonResponse;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RequestTimeoutConfig {
    /// Boolean indicating if request timeout is enabled. Defaults to false.
    pub enabled: bool,
    /// The timeout duration as a string (e.g., "30s", "1m"). Defaults to "15s".
    pub timeout: TimeConfig,
    /// Whether to display the configuration details. Defaults to false.
    pub display: bool,
}

/// Cancels the request once the configured duration elapses.
pub async fn request_timeout_mw(
    State(config): State<RequestTimeoutConfig>,
    req: Request,
    next: Next,
) -> Response {
    // `biased` polls the timer first, so a handler that finishes exactly at the
    // configured limit still counts as timed out.
    tokio::select! {
        biased;
        () = tokio::time::sleep(config.timeout.parsed) => {
            JsonResponse::RequestTimeout().into_response()
        }
        response = next.run(req) => response,
    }
}

impl DisplayConfig for RequestTimeoutConfig {
    fn display(&self) {
        if !self.display {
            return;
        }

        tracing::info!(
            target: "sword.web.request-timeout",
            enabled = self.enabled,
            timeout = self.timeout.raw,
            "Request timeout configuration"
        );
    }
}

impl Default for RequestTimeoutConfig {
    fn default() -> Self {
        RequestTimeoutConfig {
            enabled: false,
            timeout: TimeConfig {
                parsed: Duration::from_secs(15),
                raw: "15s".to_string(),
            },
            display: false,
        }
    }
}
