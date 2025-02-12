pub mod factory;
pub mod http_parser;

// In your parser module (e.g., parser.rs)
use crate::requests::request::Request;

pub trait Parser {
    /// The request type that this parser produces.
    type Req: Request;

    /// Parse some input and return a request.
    fn parse(&self, input: &str) -> Result<Self::Req, String>;
}
