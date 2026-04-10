//! Request timeout middleware.
//!
//! This module provides timeout configuration, conversion into
//! `tower_http::timeout::TimeoutLayer`, and a response mapper that rewrites
//! timeout responses into Sword's standardized JSON format.

use crate::{MapResponseLayer, ResponseFnMapper};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use axum_responses::JsonResponse;
use tower::ServiceBuilder;
pub use tower_http::timeout::TimeoutLayer;
use tower_layer::{Identity, Stack};

use crate::DisplayConfig;

use serde::{Deserialize, Serialize};
use std::time::Duration;
use thisconfig::TimeConfig;

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

impl From<RequestTimeoutConfig> for TimeoutLayer {
    fn from(config: RequestTimeoutConfig) -> Self {
        TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, config.timeout.parsed)
    }
}

pub struct RequestTimeoutResponseLayer;

impl RequestTimeoutResponseLayer {
    pub fn new() -> ServiceBuilder<Stack<MapResponseLayer, Identity>> {
        fn timeout_mapper(response: Response) -> Response {
            if response.status().as_u16() == 408 {
                return JsonResponse::RequestTimeout().into_response();
            }

            response
        }

        ServiceBuilder::new().map_response(timeout_mapper as ResponseFnMapper)
    }
}

impl DisplayConfig for RequestTimeoutConfig {
    fn display(&self) {
        if !self.display {
            return;
        }

        tracing::info!(
            target: "sword.layers.request-timeout",
            enabled = self.enabled,
            timeout = self.timeout.raw,
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
