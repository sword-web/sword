use crate::{client::GrpcClientConfig, logger::GrpcLoggerConfig};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sword_core::{
    ConfigErrors, ConfigIssue, ConfigItem, ConfigRegistrar, ValidateConfig, inventory_submit,
};
use sword_layers::body_limit::GrpcBodyLimitConfig;

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

    /// Dynamic configurable client options
    pub clients: HashMap<String, GrpcClientConfig>,
}

impl GrpcApplicationConfig {
    pub fn client(&self, name: &str) -> Option<&GrpcClientConfig> {
        self.clients.get(name)
    }

    pub fn expect_client(&self, name: &str) -> &GrpcClientConfig {
        self.client(name)
            .unwrap_or_else(|| panic!("Expected gRPC '{name}' client config"))
    }
}

impl Default for GrpcApplicationConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 50051,
            body_limit: GrpcBodyLimitConfig::default(),
            logger: None,
            clients: HashMap::new(),
        }
    }
}

impl ConfigItem for GrpcApplicationConfig {
    fn key() -> &'static str {
        "grpc"
    }
}

impl ValidateConfig for GrpcApplicationConfig {
    const SECTION: &'static str = "grpc";
    const CODE: &'static str = "sword.build.grpc";

    fn validate(&self) -> ConfigErrors {
        let mut errors = ConfigErrors::new();

        if self.host.trim().is_empty() {
            errors.push(
                ConfigIssue::new("host", "host must not be empty")
                    .with_hint("Use 0.0.0.0, 127.0.0.1, or a specific interface"),
            );
        }

        errors
    }
}

inventory_submit! {[
    ConfigRegistrar::new(|state, config| {
        state.insert(config.try_get::<GrpcApplicationConfig>().ok().flatten().unwrap_or_default());

        Ok(())
    })
]}
