use std::{env, error::Error};

use lapin::message::Delivery;
use log::info;
use serde::Deserialize;

pub struct QueueMessage {
    msg: Delivery,
    expected_content_type: String,
}

#[derive(Debug, Deserialize)]
pub struct JsonMessage {
    pub lat: f64,
    pub lon: f64,
    pub timestamp: i64,
    pub device_id: String,
    pub bumpiness: i16,
}

impl QueueMessage {
    pub fn new(message: Delivery) -> Self {
        // load expected content type from env
        let expected_content_type = env::var("EXPECTED_CONTENT_TYPE").unwrap_or("*".to_string());

        QueueMessage {
            msg: message,
            expected_content_type,
        }
    }
}

pub trait MessageParser {
    fn parse_message(&self) -> Result<JsonMessage, Box<dyn Error>>;
}

impl MessageParser for QueueMessage {
    fn parse_message(&self) -> Result<JsonMessage, Box<dyn Error>> {
        // extract content type from message
        let content_type = self.msg.properties.content_type();
        print!("{}", self.expected_content_type);

        // ensure that content type is supported
        let status = match content_type {
            Some(content_type) => {
                // If expected content type is "*", skip the content check
                if self.expected_content_type == "*" {
                    true
                } else {
                    // Ensure that content type matches the expected content type
                    self.expected_content_type == content_type.as_str()
                }
            }
            None => {
                // If no content type is provided but expected content type is "*", consider it valid
                if self.expected_content_type == "*" {
                    true
                } else {
                    // Content type is missing and doesn't match expectations
                    false
                }
            }
        };

        // if content type is not supported, return an error
        if !status {
            Err("Unsupported content type".into())
        } else {
            // extract body from message and parse it to JsonMessage
            let body = std::str::from_utf8(&self.msg.data).unwrap();
            println!("{}", body);
            let sanitized = body.replace("\\", "").replace("\n", "");
            println!("{}", sanitized);
            let parsed: JsonMessage = serde_json::from_str(sanitized.as_str())?;
            Ok(parsed)
        }
    }
}
