use serde::{Deserialize, Serialize};
use sword_core::{ConfigItem, ConfigRegistrar, inventory_submit};
use sword_layers::{body_limit::BodyLimitConfig, timeout::RequestTimeoutConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg(feature = "swagger-ui")]
pub struct OpenApiConfig {
    pub title: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "spec-file-paths")]
    pub spec_file_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WebApplicationConfig {
    /// The hostname or IP address to bind the server to. Defaults to "0.0.0.0"
    pub host: String,

    /// The port number to bind the server to. Defaults to 8000
    pub port: u16,

    /// Optional global prefix for all web controller routes.
    #[serde(rename = "router-prefix")]
    pub router_prefix: Option<String>,

    #[cfg(feature = "swagger-ui")]
    #[serde(default)]
    /// Optional OpenAPI documentation configuration.
    pub openapi: Option<OpenApiConfig>,

    /// Body limit policy for web request extraction.
    #[serde(rename = "body-limit")]
    pub body_limit: BodyLimitConfig,

    /// Request timeout policy applied to web controllers.
    #[serde(rename = "request-timeout")]
    pub request_timeout: RequestTimeoutConfig,
}

impl Default for WebApplicationConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8000,
            router_prefix: None,
            body_limit: BodyLimitConfig::default(),
            request_timeout: RequestTimeoutConfig::default(),
            #[cfg(feature = "swagger-ui")]
            openapi: None,
        }
    }
}

impl ConfigItem for WebApplicationConfig {
    fn key() -> &'static str {
        "web"
    }
}

inventory_submit! {[
    ConfigRegistrar::new(|state, config| {
        state.insert(config.get_or_default::<WebApplicationConfig>());
    })
]}
