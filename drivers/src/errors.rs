use std::fmt::{Display, Formatter};

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

impl Display for DriverError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DriverError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            DriverError::NotFound(msg) => write!(f, "Not found: {}", msg),
            DriverError::AlreadyExists(msg) => write!(f, "Already exists: {}", msg),
            DriverError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            DriverError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}
