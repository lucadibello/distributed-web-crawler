use std::sync::Arc;

use drivers::{errors::DriverError, CacheDriver};
use tokio::sync::Mutex;

use crate::services::urlservice::{UrlService, UrlServiceTrait};

pub trait UrlControllerTrait {
    async fn is_visited(&self, url: url::Url) -> Result<bool, DriverError>;
    async fn mark_visited(&self, url: url::Url) -> Result<(), DriverError>;
}

pub struct UrlController {
    service: UrlService,
}

impl UrlController {
    pub fn new(driver: Arc<Mutex<dyn CacheDriver<str, url::Url>>>) -> Self {
        UrlController {
            service: UrlService::new(driver),
        }
    }
}

impl UrlControllerTrait for UrlController {
    async fn is_visited(&self, url: url::Url) -> Result<bool, DriverError> {
        self.service.is_visited(url).await
    }

    async fn mark_visited(&self, url: url::Url) -> Result<(), DriverError> {
        self.service.mark_visited(url).await
    }
}
