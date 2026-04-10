use serde::{Deserialize, Serialize};
use sword_core::{ConfigItem, ConfigRegistrar, inventory_submit};
use sword_layers::body_limit::GrpcBodyLimitConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcApplicationConfig {
    /// The hostname or IP address to bind the gRPC server to. Defaults to "0.0.0.0".
    pub host: String,

    /// The port number to bind the gRPC server to. Defaults to 50051.
    pub port: u16,

    /// Message size limits for gRPC requests/responses.
    #[serde(rename = "body-limit")]
    pub body_limit: GrpcBodyLimitConfig,

    /// Enables tonic reflection service registration.
    #[serde(rename = "enable-tonic-reflection")]
    pub enable_tonic_reflection: bool,
}

impl Default for GrpcApplicationConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 50051,
            body_limit: GrpcBodyLimitConfig::default(),
            enable_tonic_reflection: false,
        }
    }
}

impl ConfigItem for GrpcApplicationConfig {
    fn key() -> &'static str {
        "application"
    }
}

inventory_submit! {[
    ConfigRegistrar::new(|state, config| {
        state.insert(config.get_or_default::<GrpcApplicationConfig>());
    })
]}
