use serde::{Serialize, de::DeserializeOwned};

use crate::errors::DriverError;

pub mod errors;
pub mod rabbit;
pub mod redis;

// General trait definitions
// Here we define the common internface for different kinds of drivers.

// A. CacheDriver trait defines the interface for cache drivers
pub trait CacheDriver<K: ?Sized, V>: Send + Sync
where
    V: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    fn set(&mut self, key: &K, value: &V) -> Result<(), DriverError>;
    fn get(&mut self, key: &K) -> Result<V, DriverError>;
    fn remove(&mut self, key: &K) -> Result<(), DriverError>;
    fn exists(&mut self, key: &K) -> Result<bool, DriverError>;
}
