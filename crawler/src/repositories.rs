pub mod seedrepository;
pub mod urlrepository;

use async_trait::async_trait;
use drivers::errors::DriverError;
use serde::Serialize;

#[async_trait]
#[allow(dead_code)]
pub trait Repository<K, V>
where
    K: AsRef<str> + Send + Sync + 'static,
    V: Serialize + Send + Sync + 'static,
{
    async fn set(&self, key: K, value: V) -> Result<(), DriverError>;
    async fn get(&self, key: K) -> Result<V, DriverError>;
    async fn remove(&self, key: K) -> Result<(), DriverError>;
    async fn exists(&self, key: K) -> Result<bool, DriverError>;
}

// re-export all repositories here
pub use seedrepository::{load_default_seeds, load_seeds_from_dir};
pub use urlrepository::UrlRepository;
