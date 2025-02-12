use dotenv::dotenv;
use redis::Commands;
use std::env;

pub struct RedisClient {
    pub conn: redis::Connection,
}

impl RedisClient {
    pub fn new() -> Self {
        // Load environment variables from .env file
        dotenv().ok();

        // Build the Redis URL from environment variables
        let redis_url = format!(
            "redis://{}:{}/{}",
            env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string()),
            env::var("REDIS_DB").unwrap_or_else(|_| "0".to_string())
        );

        // Create a Redis client and establish a connection
        let client = redis::Client::open(redis_url).expect("Failed to connect to Redis");
        let conn = client
            .get_connection()
            .expect("Failed to get Redis connection");

        RedisClient { conn }
    }

    /// Checks if a URL has already been visited by checking the existence of a field in the "visited_urls" hash.
    pub fn is_visited(&mut self, url: &str) -> redis::RedisResult<bool> {
        self.conn.hexists("visited_urls", url)
    }

    /// Marks a URL as visited by setting its field in the "visited_urls" hash to "true".
    pub fn mark_visited(&mut self, url: &str) -> redis::RedisResult<()> {
        self.conn.hset("visited_urls", url, "true")
    }
}
