//! Static file serving middleware.
//!
//! This module defines configuration and conversion into Tower's `ServeDir`
//! service for serving files from a configured directory.

use crate::DisplayConfig;
use serde::{Deserialize, Serialize};
use thisconfig::{ByteConfig, ConfigItem};
use tower_http::services::{ServeDir, ServeFile};

pub type ServeDirLayer = ServeDir<ServeFile>;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct ServeDirConfig {
    /// The directory path containing static files to serve. Defaults to "public".
    #[serde(rename = "static-dir")]
    pub static_dir: String,

    /// The route path where static files will be accessible. Defaults to "/static".
    #[serde(rename = "router-path")]
    pub router_path: String,

    /// The compression algorithm to use for serving files. (e.g., "gzip", "br"). Defaults to None.
    #[serde(rename = "compression-algorithm")]
    pub compression_algorithm: Option<String>,

    /// The chunk size for streaming files (e.g., "64KB"). Defaults to None.
    #[serde(rename = "chunk-size")]
    pub chunk_size: Option<ByteConfig>,

    /// The custom 404 file path to serve when a file is not found. Defaults to None.
    #[serde(rename = "not-found-file")]
    pub not_found_file: Option<String>,

    /// Whether to display the configuration details. Defaults to false.
    pub display: bool,
}

impl From<ServeDirConfig> for ServeDirLayer {
    fn from(config: ServeDirConfig) -> ServeDirLayer {
        let mut fallback = ServeFile::new(format!("{}/404.html", config.static_dir));

        if let Some(not_found_file) = &config.not_found_file {
            fallback = ServeFile::new(format!("{}/{not_found_file}", config.static_dir));
        }

        let mut layer = ServeDir::new(&config.static_dir).fallback(fallback);

        if let Some(algorithm) = &config.compression_algorithm {
            match algorithm.as_str() {
                "br" => {
                    layer = layer.precompressed_br();
                }
                "gzip" => {
                    layer = layer.precompressed_gzip();
                }
                "deflate" => {
                    layer = layer.precompressed_deflate();
                }
                "zstd" => {
                    layer = layer.precompressed_zstd();
                }
                _ => {}
            }
        }

        if let Some(chunk_size) = &config.chunk_size {
            layer = layer.with_buf_chunk_size(chunk_size.parsed);
        }

        layer
    }
}

impl Default for ServeDirConfig {
    fn default() -> Self {
        ServeDirConfig {
            static_dir: "public".to_string(),
            router_path: "/static".to_string(),
            compression_algorithm: None,
            chunk_size: None,
            not_found_file: None,
            display: false,
        }
    }
}

impl DisplayConfig for ServeDirConfig {
    fn display(&self) {
        if !self.display {
            return;
        }

        tracing::info!(
            target: "sword.layers.serve_dir",
            static_dir = %self.static_dir,
            router_path = %self.router_path,
            compression_algorithm = ?self.compression_algorithm,
            chunk_size = ?self.chunk_size.as_ref().map(|value| &value.raw),
            not_found_file = ?self.not_found_file,
        );
    }
}

impl ConfigItem for ServeDirConfig {
    fn key() -> &'static str {
        "serve-dir"
    }
}
