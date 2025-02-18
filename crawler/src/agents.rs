pub mod crawler_agent;

use crate::requests::request::Request;

pub trait Agent {
    type Output;

    async fn execute(&self) -> Result<Self::Output, String>;
}

// Define a struct that holds a request
#[allow(dead_code)]
pub struct RequestAgent<R: Request<Output = T>, T> {
    pub target: String,
    pub request: R,
    _marker: std::marker::PhantomData<T>,
}

pub enum ExitStatus {
    Ok,
    Error,
    Cancel,
}

// Implement Agent for RequestAgent
impl<R, T> Agent for RequestAgent<R, T>
where
    R: Request<Output = T>,
{
    type Output = ExitStatus;

    async fn execute(&self) -> Result<ExitStatus, String> {
        match self.request.execute().await {
            Ok(_) => Ok(ExitStatus::Ok),
            Err(_) => Err("Error".to_string()),
        }
    }
}
