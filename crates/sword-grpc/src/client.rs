use serde::{Deserialize, Serialize};
use std::time::Duration;
use sword_core::TimeConfig;
use tonic::transport::Channel;

/// Default per-request timeout applied when a client does not declare one.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// How a client channel establishes its connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ClientConnectionKind {
    /// Connect on the first use, retrying if the remote service is unavailable.
    #[default]
    Lazy,
    /// Connect while the application starts, failing early if the remote service is down.
    Eager,
}

/// Configuration for a single gRPC client, declared under `[grpc.clients.<name>]`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct GrpcClientConfig {
    /// Address of the remote service. An empty value is rejected when building the channel.
    pub url: String,
    /// Connection policy for the channel.
    pub connect: ClientConnectionKind,
    /// Per-request timeout. Falls back to [`DEFAULT_TIMEOUT`] when absent.
    pub timeout: Option<TimeConfig>,
}

/// Errors produced while building a gRPC client channel from configuration.
#[derive(Debug, thiserror::Error)]
pub enum GrpcClientError {
    /// The client configuration has no `url`.
    #[error("gRPC client url is empty")]
    MissingUrl,

    /// The configured `url` is not a valid URI.
    #[error("invalid gRPC client url '{url}': {message}")]
    InvalidUrl { url: String, message: String },

    /// The eager connection attempt failed.
    #[error("failed to connect gRPC client at '{url}'")]
    Connect {
        url: String,
        #[source]
        source: tonic::transport::Error,
    },
}

/// Builds a value from a resolved [`GrpcClientConfig`].
///
/// Implemented by Sword for [`Channel`], so a client channel is created from a
/// `[grpc.clients.<name>]` section without repeating the connection setup.
#[allow(async_fn_in_trait)]
pub trait FromConfig: Sized {
    async fn from_config(config: &GrpcClientConfig) -> Result<Self, GrpcClientError>;
}

impl FromConfig for Channel {
    async fn from_config(config: &GrpcClientConfig) -> Result<Self, GrpcClientError> {
        if config.url.is_empty() {
            return Err(GrpcClientError::MissingUrl);
        }

        let endpoint = Channel::from_shared(config.url.clone())
            .map_err(|source| GrpcClientError::InvalidUrl {
                url: config.url.clone(),
                message: source.to_string(),
            })?
            .timeout(
                config
                    .timeout
                    .as_ref()
                    .map(|timeout| timeout.parsed)
                    .unwrap_or(DEFAULT_TIMEOUT),
            );

        match config.connect {
            ClientConnectionKind::Eager => {
                endpoint
                    .connect()
                    .await
                    .map_err(|source| GrpcClientError::Connect {
                        url: config.url.clone(),
                        source,
                    })
            }
            ClientConnectionKind::Lazy => Ok(endpoint.connect_lazy()),
        }
    }
}
