use std::collections::HashMap;
use url::Url;
use tracing::{debug, info, warn};

pub struct RobotsTxtClient {
    cache: HashMap<String, String>,
}

impl RobotsTxtClient {
    pub fn new() -> Self {
        RobotsTxtClient {
            cache: HashMap::new(),
        }
    }

    pub async fn is_allowed(&mut self, url_str: &str) -> bool {
        let url = match Url::parse(url_str) {
            Ok(url) => url,
            Err(_) => return false,
        };

        let domain = match url.domain() {
            Some(domain) => domain.to_string(),
            None => return false,
        };

        if let Some(robots_txt) = self.cache.get(&domain) {
            debug!("Found robots.txt for {} in cache", domain);
            return self.parse_robots_txt(robots_txt, url_str);
        }

        let robots_url = match url.join("/robots.txt") {
            Ok(url) => url,
            Err(_) => return false,
        };

        info!("Fetching robots.txt from {}", robots_url);
        let response = match reqwest::get(robots_url).await {
            Ok(res) => res,
            Err(e) => {
                warn!("Failed to fetch robots.txt for {}: {}", domain, e);
                return true; // If we can't fetch it, assume we can crawl.
            }
        };

        if response.status().is_success() {
            let body = match response.text().await {
                Ok(text) => text,
                Err(_) => return true, // Assume allowed if we can't read the body
            };
            let is_allowed = self.parse_robots_txt(&body, url_str);
            self.cache.insert(domain, body);
            is_allowed
        } else {
            true // If robots.txt doesn't exist, assume we can crawl.
        }
    }

    fn parse_robots_txt(&self, robots_txt: &str, url_str: &str) -> bool {
        let mut user_agent_found = false;
        let mut disallow_all = false;

        for line in robots_txt.lines() {
            if line.starts_with("User-agent: *") {
                user_agent_found = true;
            }

            if user_agent_found && line.starts_with("Disallow: /") {
                disallow_all = true;
            }

            if user_agent_found && line.starts_with("Disallow:") {
                let path = line.split(":").collect::<Vec<&str>>()[1].trim();
                if path == "/" {
                    disallow_all = true;
                } else if url_str.contains(path) {
                    return false;
                }
            }
        }

        !disallow_all
    }
}