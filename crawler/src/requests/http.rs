use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, instrument, warn};

use crate::{
    clients::http::{get_default_http_client, HttpClient},
    requests::request::Request,
    validators,
};

#[derive(Debug)]
pub struct HttpRequest {
    pub target: String,
    pub client: Option<HttpClient>,
    pub depth: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtraHttpResponseFields {
    pub links: Vec<String>,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub title: String,
    pub status_code: u16,
    pub headers: Vec<String>,
    pub meta: Vec<String>,
    pub extra: Option<ExtraHttpResponseFields>,
}

#[derive(Debug, Serialize)]
pub struct PageData {
    pub url: String,
    pub title: String,
    pub status_code: u16,
    pub headers: Vec<String>,
    pub meta: Vec<String>,
    pub links: Vec<String>,
    pub body: String,
}

impl Request for HttpRequest {
    type Output = HttpResponse;

    #[instrument]
    fn new(target: String, depth: u32) -> Self {
        info!(
            "Creating new HTTP request for target: {} at depth {}",
            target, depth
        );
        HttpRequest {
            target,
            client: Some(get_default_http_client()),
            depth,
        }
    }

    #[instrument(skip(self), fields(url = %self.target))]
    async fn execute(&self) -> Result<HttpResponse, String> {
        // ensure url is valid
        debug!("Validating URL");
        match validators::validate_url(&self.target) {
            Ok(_) => (),
            Err(e) => {
                error!("URL validation failed: {}", e);
                return Err(e);
            }
        }

        // Perform HTTP GET request.
        info!("Performing HTTP GET request");
        let response = self
            .client
            .as_ref()
            .unwrap()
            .get(&self.target)
            .await
            .map_err(|e| format!("HTTP request error: {e}"))?;

        // Get status code.
        let status_code = response.status().as_u16();
        debug!("Response status code: {}", status_code);

        // Get the title of the HTML page.
        let title = response
            .headers()
            .get("title")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("No title")
            .to_string();
        debug!("Response title: {}", title);

        // Collect headers as "Key: Value" strings.
        let headers: Vec<String> = response
            .headers()
            .iter()
            .map(|(name, value)| format!("{}: {}", name, value.to_str().unwrap_or("")))
            .collect();

        // Read the response body as text.
        debug!("Reading response body");
        let body = response
            .text()
            .await
            .map_err(|e| format!("Error reading body: {e}"))?;

        // Parse the HTML body using the scraper crate.
        debug!("Parsing HTML body");
        let document = scraper::Html::parse_document(&body);

        // Extract all links from anchor tags (<a href="...">).
        let link_selector = scraper::Selector::parse("a[href]")
            .map_err(|e| format!("Selector parse error: {e}"))?;

        let mut links: Vec<String> = document
            .select(&link_selector)
            .filter_map(|element| element.value().attr("href").map(|s| s.to_string()))
            .collect();
        debug!("Found {} links", links.len());

        // If links start with a slash, prepend the domain.
        let url = url::Url::parse(&self.target).map_err(|_| "Error parsing target URL")?;

        for link in links.iter_mut() {
            if link.starts_with('/') {
                *link = format!("{}{}", url.origin().ascii_serialization(), link);
            }
        }

        // Now, only keep links that are valid URLs.
        links.retain(|link| {
            let is_ok = validators::validate_url(link).is_ok();
            if !is_ok {
                warn!("Invalid link found and removed: {}", link);
            }
            is_ok
        });

        // Extract meta tags with a name attribute.
        let meta_selector = scraper::Selector::parse("meta[name]")
            .map_err(|e| format!("Selector parse error: {e}"))?;
        let mut meta: Vec<String> = document
            .select(&meta_selector)
            .filter_map(|element| {
                let name = element.value().attr("name")?;
                let content = element.value().attr("content")?;
                Some(format!("{name}: {content}"))
            })
            .collect();

        // Also extract meta tags with a charset attribute.
        let meta_charset_selector = scraper::Selector::parse("meta[charset]")
            .map_err(|e| format!("Selector parse error: {e}"))?;
        meta.extend(
            document
                .select(&meta_charset_selector)
                .filter_map(|element| {
                    element
                        .value()
                        .attr("charset")
                        .map(|charset| format!("charset: {charset}"))
                }),
        );
        debug!("Found {} meta tags", meta.len());

        Ok(HttpResponse {
            title,
            status_code,
            headers,
            meta,
            extra: Some(ExtraHttpResponseFields { links, body }),
        })
    }
}
