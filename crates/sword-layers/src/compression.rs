//! Response compression middleware.
//!
//! This module defines compression configuration and conversion into
//! `tower_http::compression::CompressionLayer` using the configured algorithms.

use crate::{DisplayConfig, SwordLayerRegistrar};

use serde::{Deserialize, Serialize};
use std::any::Any;
use thisconfig::Config;
use thisconfig::ConfigItem;

pub use tower_http::compression::CompressionLayer;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct CompressionConfig {
    /// Whether to display the configuration details.
    pub display: bool,
    /// A list of strings representing the compression algorithms to use
    /// (e.g., "gzip", "deflate", "br", "zstd
    pub algorithms: Vec<String>,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            display: false,
            algorithms: vec!["gzip".into(), "br".into()],
        }
    }
}

impl From<CompressionConfig> for CompressionLayer {
    fn from(config: CompressionConfig) -> Self {
        let mut layer = CompressionLayer::new();

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
            tracing::debug!(
                target: "sword.layers.compression",
                algorithms = ?self.algorithms,
                "Compression Layer configuration"
            );
        }
    }
}

impl ConfigItem for CompressionConfig {
    fn key() -> &'static str {
        "compression"
    }
}

inventory::submit! {
    SwordLayerRegistrar {
        name: "compression",
        register: |config: &Config| {
            let layer: CompressionLayer = config.get_or_default::<CompressionConfig>().into();

            Box::new(move |any: &mut dyn Any| {
                let stack = any
                    .downcast_mut::<sword_core::LayerStack<sword_core::State>>()
                    .expect("SwordLayerRegistrar: expected LayerStack<State>");

                stack.push(layer);
            })
        },
        display: |config: &Config| {
            config.get_or_default::<CompressionConfig>().display();
        },
    }
}
