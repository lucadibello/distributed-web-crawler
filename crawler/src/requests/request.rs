use serde::{Deserialize, Serialize};

pub trait Request {
    type Output: std::fmt::Debug + Serialize + Deserialize<'static>;
    fn new(target: &str, depth: u32) -> Self;
    async fn execute(&self) -> Result<Self::Output, String>;
}
