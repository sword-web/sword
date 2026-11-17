//! Request body size limiting middleware.
//!
//! The limit itself is enforced by `tower_http` while the body is read; the
//! response mapper rewrites oversized payload responses into Sword's
//! standardized JSON shape.

use std::str::FromStr;

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::{self, Deserialize, Deserializer, Serialize};
use sword_core::{ByteConfig, DisplayConfig};
use tower_http::limit::RequestBodyLimitLayer;

use crate::JsonResponse;

#[derive(Debug, Clone, Serialize)]
#[serde(default)]
pub struct BodyLimitConfig {
    /// The maximum allowed size for request bodies (e.g., "1MB", "500KB").
    /// This is a direct value in TOML (e.g., `body-limit = "5MB"`).
    pub max_size: ByteConfig,
}

impl<'de> Deserialize<'de> for BodyLimitConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize as a simple value (string or number) into ByteConfig.
        // This will reject maps/tables like `{ max-size = "..." }`.
        let max_size = ByteConfig::deserialize(deserializer)?;
        Ok(BodyLimitConfig { max_size })
    }
}

/// Builds the layer that enforces the configured body size limit.
pub fn body_limit_layer(config: &BodyLimitConfig) -> RequestBodyLimitLayer {
    RequestBodyLimitLayer::new(config.max_size.parsed)
}

/// Rewrites `413 Payload Too Large` responses into Sword's JSON shape.
pub async fn body_limit_response_mw(req: Request, next: Next) -> Response {
    let response = next.run(req).await;

    if response.status() != StatusCode::PAYLOAD_TOO_LARGE {
        return response;
    }

    JsonResponse::PayloadTooLarge()
        .message("The request body exceeds the maximum allowed size by the server")
        .into_response()
}

impl DisplayConfig for BodyLimitConfig {
    fn display(&self) {
        tracing::debug!(
            target: "sword.web.body-limit",
            max_body_size = ?self.max_size.raw,
            "HTTP body limit configuration"
        );
    }
}

impl Default for BodyLimitConfig {
    fn default() -> Self {
        let max_size = "10MB".to_string();
        let parsed = byte_unit::Byte::from_str(&max_size)
            .unwrap_or_else(|_| byte_unit::Byte::from_u64(10 * 1024 * 1024))
            .as_u64() as usize;

        BodyLimitConfig {
            max_size: ByteConfig {
                parsed,
                raw: max_size,
            },
        }
    }
}

#[derive(Clone)]
pub struct BodyLimitValue(pub usize);

impl From<BodyLimitConfig> for BodyLimitValue {
    fn from(config: BodyLimitConfig) -> Self {
        BodyLimitValue(config.max_size.parsed)
    }
}

impl Default for BodyLimitValue {
    fn default() -> Self {
        BodyLimitValue(10 * 1024 * 1024) // Default to 10MB
    }
}
