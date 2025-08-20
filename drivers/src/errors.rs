#[derive(Debug)]
pub enum DriverError {
    ConnectionError(String),
    NotFound(String),
    AlreadyExists(String),
    InvalidInput(String),
    InternalError(String),
}

// DriverError support for Redis driver
impl From<redis::RedisError> for DriverError {
    fn from(err: redis::RedisError) -> Self {
        DriverError::InternalError(err.to_string())
    }
}

impl From<&str> for DriverError {
    fn from(err: &str) -> Self {
        DriverError::InternalError(err.to_string())
    }
}

impl From<std::string::String> for DriverError {
    fn from(err: String) -> Self {
        DriverError::InternalError(err)
    }
}
