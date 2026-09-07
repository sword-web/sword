use serde::{Deserialize, Serialize};
use sword_core::{ConfigItem, ConfigRegistrar, inventory_submit};
use sword_layers::body_limit::GrpcBodyLimitConfig;

use crate::logger::GrpcLoggerConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GrpcApplicationConfig {
    /// The hostname or IP address to bind the gRPC server to. Defaults to "0.0.0.0".
    pub host: String,

    /// The port number to bind the gRPC server to. Defaults to 50051.
    pub port: u16,

    /// Message size limits for gRPC requests/responses.
    #[serde(rename = "body-limit")]
    pub body_limit: GrpcBodyLimitConfig,

    /// gRPC access logger configuration. Disabled when absent.
    #[serde(default)]
    pub logger: Option<GrpcLoggerConfig>,
}

impl Default for GrpcApplicationConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 50051,
            body_limit: GrpcBodyLimitConfig::default(),
            logger: None,
        }
    }
}

impl ConfigItem for GrpcApplicationConfig {
    fn key() -> &'static str {
        "grpc"
    }
}

inventory_submit! {[
    ConfigRegistrar::new(|state, config| {
        state.insert(config.get_or_default::<GrpcApplicationConfig>());
    })
]}
