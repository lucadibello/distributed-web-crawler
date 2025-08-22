use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PageData {
    pub url: String,
    pub title: String,
    pub status_code: u16,
    pub headers: Vec<String>,
    pub meta: Vec<String>,
    pub links: Vec<String>,
    pub body: String,
}

impl Display for PageData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PageData {{ url: {}, title: {}, status_code: {}, headers: {:?}, meta: {:?}, links: {:?}, body_length: {} }}",
            self.url,
            self.title,
            self.status_code,
            self.headers,
            self.meta,
            self.links,
            self.body.len()
        )
    }
}
