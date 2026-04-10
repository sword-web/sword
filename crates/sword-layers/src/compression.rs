//! Response compression middleware.
//!
//! This module defines compression configuration and conversion into
//! `tower_http::compression::CompressionLayer` using the configured algorithms.

use crate::DisplayConfig;
use serde::{Deserialize, Serialize};
use thisconfig::ConfigItem;
use tower_http::compression::CompressionLayer as TowerCompressionLayer;

pub struct CompressionLayer;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct CompressionConfig {
    /// Whether to display the configuration details.
    pub display: bool,
    /// A list of strings representing the compression algorithms to use
    /// (e.g., "gzip", "deflate", "br", "zstd
    pub algorithms: Vec<String>,
}

impl From<CompressionConfig> for TowerCompressionLayer {
    fn from(config: CompressionConfig) -> Self {
        let mut layer = TowerCompressionLayer::new();

        for algorithm in &config.algorithms {
            match algorithm.to_lowercase().as_str() {
                "gzip" => layer = layer.gzip(true),
                "deflate" => layer = layer.deflate(true),
                "br" | "brotli" => layer = layer.br(true),
                "zstd" => layer = layer.zstd(true),
                _ => {}
            }
        }

        layer
    }
}

impl DisplayConfig for CompressionConfig {
    fn display(&self) {
        if self.display {
            tracing::info!(
                target: "sword.layers.compression",
                algorithms = ?self.algorithms,
            );
        }
    }
}

impl ConfigItem for CompressionConfig {
    fn key() -> &'static str {
        "compression"
    }
}
