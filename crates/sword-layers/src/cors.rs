//! Cross-origin resource sharing middleware.
//!
//! This module defines CORS configuration and conversion into
//! `tower_http::cors::CorsLayer` for controlling cross-origin policies.

use crate::DisplayConfig;

use axum::http::{HeaderName, HeaderValue, Method};
use serde::{Deserialize, Serialize};
use thisconfig::{ConfigItem, TimeConfig};
use tower_http::cors::Any;

pub use tower_http::cors::CorsLayer;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct CorsConfig {
    /// A list of allowed origins for cross-origin requests.
    #[serde(rename = "allow-origins")]
    pub allow_origins: Option<Vec<String>>,

    /// A list of allowed HTTP methods (e.g., "GET", "POST").
    #[serde(rename = "allow-methods")]
    pub allow_methods: Option<Vec<String>>,

    /// A list of allowed HTTP headers.
    #[serde(rename = "allow-headers")]
    pub allow_headers: Option<Vec<String>>,

    /// A boolean indicating if credentials are allowed in cross-origin requests.
    #[serde(rename = "allow-credentials")]
    pub allow_credentials: Option<bool>,

    /// The maximum age in seconds for CORS preflight responses.
    #[serde(rename = "max-age")]
    pub max_age: Option<TimeConfig>,

    /// Whether to display the configuration details.
    pub display: bool,
}

impl DisplayConfig for CorsConfig {
    fn display(&self) {
        if !self.display {
            return;
        }
        tracing::info!(
            target: "sword.layers.cors",
            allow_origins = ?self.allow_origins,
            allow_methods = ?self.allow_methods,
            allow_headers = ?self.allow_headers,
            allow_credentials = self.allow_credentials,
            max_age = ?self.max_age.as_ref().map(|value| &value.raw),
        );
    }
}

impl From<CorsConfig> for CorsLayer {
    fn from(config: CorsConfig) -> CorsLayer {
        let mut layer = CorsLayer::new();

        if let Some(allow_credentials) = config.allow_credentials {
            layer = layer.allow_credentials(allow_credentials);
        }

        if let Some(origin) = &config.allow_origins {
            if origin.iter().any(|o| o == "*") {
                layer = layer.allow_origin(Any);
            } else {
                let parsed_origin: Vec<HeaderValue> = origin
                    .iter()
                    .filter_map(|o| HeaderValue::from_str(o).ok())
                    .collect();

                layer = layer.allow_origin(parsed_origin);
            }
        }

        if let Some(methods) = &config.allow_methods {
            if methods.iter().any(|m| m == "*") {
                layer = layer.allow_methods(Any);
            } else {
                let parsed_methods: Vec<Method> =
                    methods.iter().filter_map(|m| m.parse().ok()).collect();

                layer = layer.allow_methods(parsed_methods);
            }
        }

        if let Some(headers) = &config.allow_headers {
            if headers.iter().any(|h| h == "*") {
                layer = layer.allow_headers(Any);
            } else {
                let parsed_headers: Vec<HeaderName> =
                    headers.iter().filter_map(|h| h.parse().ok()).collect();

                layer = layer.allow_headers(parsed_headers);
            }
        }

        if let Some(max_age) = &config.max_age {
            layer = layer.max_age(max_age.parsed);
        }

        layer
    }
}

impl ConfigItem for CorsConfig {
    fn key() -> &'static str {
        "cors"
    }
}
