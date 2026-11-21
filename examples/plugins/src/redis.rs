use std::{fmt, time::Duration};

use redis::{
    Client,
    aio::{ConnectionManager, ConnectionManagerConfig},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sword::prelude::*;
use tokio::sync::OnceCell;

#[config(key = "redis", required = false)]
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct RedisConfig {
    /// Connection URL, for example `redis://127.0.0.1:6379`.
    pub url: String,

    /// Default time to live for cached values, in milliseconds.
    pub ttl_ms: u64,

    /// Prefix added to every key so several applications can share a server.
    pub prefix: String,

    /// Timeout for establishing the connection, in milliseconds.
    pub connection_timeout_ms: u64,

    /// Timeout for command responses, in milliseconds.
    pub response_timeout_ms: u64,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://127.0.0.1:6379".to_string(),
            ttl_ms: 60_000,
            prefix: "sword".to_string(),
            connection_timeout_ms: 500,
            response_timeout_ms: 500,
        }
    }
}

#[derive(Debug)]
pub enum CacheError {
    Redis(redis::RedisError),
    Json(serde_json::Error),
    Timeout,
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Redis(err) => write!(f, "redis error: {err}"),
            Self::Json(err) => write!(f, "cache serialization error: {err}"),
            Self::Timeout => write!(f, "redis connection timed out"),
        }
    }
}

impl std::error::Error for CacheError {}

impl From<redis::RedisError> for CacheError {
    fn from(err: redis::RedisError) -> Self {
        Self::Redis(err)
    }
}

impl From<serde_json::Error> for CacheError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

/// Redis-backed cache with a lazy connection, so the application starts even if Redis is down.
#[injectable(provider)]
pub struct RedisCache {
    client: Client,
    prefix: String,
    ttl: Duration,
    connection_timeout: Duration,
    response_timeout: Duration,
    connection: OnceCell<ConnectionManager>,
}

impl RedisCache {
    pub fn new(config: RedisConfig) -> Self {
        let client =
            Client::open(config.url.as_str()).expect("redis plugin: invalid connection url");

        Self {
            client,
            prefix: config.prefix,
            ttl: Duration::from_millis(config.ttl_ms),
            connection_timeout: Duration::from_millis(config.connection_timeout_ms),
            response_timeout: Duration::from_millis(config.response_timeout_ms),
            connection: OnceCell::new(),
        }
    }

    async fn connection(&self) -> Result<ConnectionManager, CacheError> {
        let manager = self
            .connection
            .get_or_try_init(|| async {
                let config = ConnectionManagerConfig::new()
                    .set_connection_timeout(Some(self.connection_timeout))
                    .set_response_timeout(Some(self.response_timeout))
                    .set_number_of_retries(0);

                tokio::time::timeout(
                    self.connection_timeout + Duration::from_millis(200),
                    ConnectionManager::new_with_config(self.client.clone(), config),
                )
                .await
                .map_err(|_| CacheError::Timeout)?
                .map_err(CacheError::from)
            })
            .await?;

        Ok(manager.clone())
    }

    fn key(&self, key: &str) -> String {
        format!("{}:{}", self.prefix, key)
    }

    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, CacheError> {
        let mut connection = self.connection().await?;

        let value: Option<String> = redis::cmd("GET")
            .arg(self.key(key))
            .query_async(&mut connection)
            .await?;

        value
            .map(|json| serde_json::from_str(&json))
            .transpose()
            .map_err(Into::into)
    }

    pub async fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<(), CacheError> {
        let mut connection = self.connection().await?;
        let json = serde_json::to_string(value)?;

        let _: () = redis::cmd("SET")
            .arg(self.key(key))
            .arg(json)
            .arg("PX")
            .arg(self.ttl.as_millis() as u64)
            .query_async(&mut connection)
            .await?;

        Ok(())
    }

    pub async fn invalidate(&self, key: &str) -> Result<(), CacheError> {
        let mut connection = self.connection().await?;

        let _: () = redis::cmd("DEL")
            .arg(self.key(key))
            .query_async(&mut connection)
            .await?;

        Ok(())
    }
}

pub struct RedisPlugin;

impl Plugin for RedisPlugin {
    fn name() -> &'static str {
        "redis"
    }

    async fn register_providers(config: &Config, providers: &ProviderRegistry) {
        let config = config.get_or_default::<RedisConfig>();

        providers.register(RedisCache::new(config));
    }
}
