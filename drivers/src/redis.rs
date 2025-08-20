use redis::Commands;
use serde::{Serialize, de::DeserializeOwned};
use std::env;
use tracing::{info, instrument};

use crate::{CacheDriver, errors::DriverError};

pub struct RedisDriver {
    pub conn: redis::Connection,
}

impl RedisDriver {
    #[instrument]
    pub fn build() -> Result<Self, String> {
        // Build the Redis URL from environment variables
        let redis_url = format!(
            "redis://{}:{}/{}",
            env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string()),
            env::var("REDIS_DB").unwrap_or_else(|_| "0".to_string())
        );
        info!("Connecting to Redis at {}", redis_url);

        // Create a Redis client and establish a connection
        let client = redis::Client::open(redis_url.clone())
            .map_err(|e| format!("Failed to create Redis client: {e}"))?;

        let conn = client
            .get_connection()
            .map_err(|e| format!("Failed to connect to Redis at {redis_url}: {e}"))?;

        info!("Redis connection successful");
        Ok(RedisDriver { conn })
    }
}

impl<K, V> CacheDriver<K, V> for RedisDriver
where
    K: AsRef<str> + ?Sized,
    V: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    fn set(&mut self, key: &K, value: &V) -> Result<(), DriverError> {
        // compute payload + store in Redis
        let payload = serde_json::to_vec(value)
            .map_err(|e| DriverError::InternalError(format!("Serialization error: {e}")))?;
        self.conn
            .set::<&str, Vec<u8>, ()>(key.as_ref(), payload)
            .map_err(|e| DriverError::InternalError(format!("Redis set error: {e}")))?;

        Ok(())
    }

    fn get(&mut self, key: &K) -> Result<V, DriverError> {
        // fetch bytes from Redis + deserialize using serde_json
        let bytes = self
            .conn
            .get::<&str, Vec<u8>>(key.as_ref())
            .map_err(|e| DriverError::InternalError(format!("Redis get error: {e}")))?;
        let value: V = serde_json::from_slice(&bytes)
            .map_err(|e| DriverError::InternalError(format!("Deserialization error: {e}")))?;

        Ok(value)
    }

    fn remove(&mut self, key: &K) -> Result<(), DriverError> {
        // remove the key from Redis
        self.conn
            .del::<&str, ()>(key.as_ref())
            .map_err(|e| DriverError::InternalError(format!("Redis remove error: {e}")))?;
        Ok(())
    }

    fn exists(&mut self, key: &K) -> Result<bool, DriverError> {
        self.conn
            .exists(key.as_ref())
            .map_err(|e| DriverError::InternalError(format!("Redis exists check error: {e}")))
    }
}
