use std::sync::Arc;

use crate::repositories::Repository;
use async_trait::async_trait;
use drivers::errors::DriverError;
use drivers::CacheDriver;
use tokio::sync::Mutex;
use url::Url;

pub struct UrlRepository {
    driver: Arc<Mutex<dyn CacheDriver<str, url::Url>>>,
}

impl UrlRepository {
    pub fn new(driver: Arc<Mutex<dyn CacheDriver<str, url::Url>>>) -> Self {
        UrlRepository { driver }
    }
}

#[async_trait]
impl<K> Repository<K, Url> for UrlRepository
where
    K: AsRef<str> + Send + Sync + 'static,
{
    async fn set(&self, key: K, value: Url) -> Result<(), DriverError> {
        self.driver.lock().await.set(key.as_ref(), &value)
    }

    async fn get(&self, key: K) -> Result<Url, DriverError> {
        self.driver.lock().await.get(key.as_ref())
    }

    async fn remove(&self, key: K) -> Result<(), DriverError> {
        self.driver.lock().await.remove(key.as_ref())
    }

    async fn exists(&self, key: K) -> Result<bool, DriverError> {
        self.driver.lock().await.exists(key.as_ref())
    }
}
