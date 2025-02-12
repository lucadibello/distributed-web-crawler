use crate::requests::request::Request;

use super::Parser;

struct ParserFactory;

impl ParserFactory {
    pub fn new() -> ParserFactory {
        ParserFactory
    }

    // pub fn get_parser(&self, parser_type: &str) -> Box<dyn Parser<dyn Request<Output = >> {
    //     match parser_type {
    //         "http" => Box::new(HttpParser::new()),
    //         _ => panic!("Unknown parser type"),
    //     }
    // }
}
