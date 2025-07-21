use dotenv::dotenv;
use redis::Commands;
use std::env;
use tracing::{debug, info, instrument};

pub struct RedisClient {
    pub conn: redis::Connection,
}

impl RedisClient {
    #[instrument]
    pub fn build() -> Result<Self, String> {
        // Load environment variables from .env file
        dotenv().ok();

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
        Ok(RedisClient { conn })
    }

    /// Checks if a URL has already been visited by checking the existence of a field in the "visited_urls" hash.
    #[instrument(skip(self))]
    pub fn is_visited(&mut self, url: &str) -> redis::RedisResult<bool> {
        debug!("Checking if URL is visited: {}", url);
        self.conn.hexists("visited_urls", url)
    }

    /// Marks a URL as visited by setting its field in the "visited_urls" hash to "true".
    #[instrument(skip(self))]
    pub fn mark_visited(&mut self, url: &str) -> redis::RedisResult<()> {
        debug!("Marking URL as visited: {}", url);
        self.conn.hset("visited_urls", url, "true")
    }
}
