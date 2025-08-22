use std::sync::Arc;

use drivers::{errors::DriverError, CacheDriver};
use tokio::sync::Mutex;

use crate::repositories::{Repository, UrlRepository};

pub trait UrlServiceTrait {
    // Define service methods here, e.g., create, read, update, delete URLs
    async fn is_visited(&self, url: url::Url) -> Result<bool, DriverError>;
    async fn mark_visited(&self, url: url::Url) -> Result<(), DriverError>;
}

pub struct UrlService {
    repository: UrlRepository,
}

impl UrlService {
    // constructor method
    pub fn new(client: Arc<Mutex<dyn CacheDriver<str, url::Url>>>) -> Self {
        UrlService {
            repository: UrlRepository::new(client),
        }
    }
}

impl UrlServiceTrait for UrlService {
    async fn is_visited(&self, url: url::Url) -> Result<bool, DriverError> {
        self.repository.get(url).await.map(|_| true)
    }

    async fn mark_visited(&self, url: url::Url) -> Result<(), DriverError> {
        // NOTE: we set the URL as both key and value for simplicity. We just need to track
        // visited.
        self.repository.set(url.clone(), url).await
    }
}
