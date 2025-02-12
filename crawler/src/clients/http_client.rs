use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::{Client, Error, Proxy};
use std::time::Duration;
use tokio::time;

pub struct HttpClientConfig {
    /// Optional custom user agent.
    pub user_agent: Option<String>,
    /// Optional proxy URL (e.g., "http://127.0.0.1:8080").
    pub proxy: Option<String>,
    /// Optional timeout for requests.
    pub timeout: Option<Duration>,
    // Additional configuration options (e.g., redirect policy, default headers) can be added here.
}

/// A simple HTTP client wrapper that supports useful features for an HTTP crawler.
#[derive(Debug)]
pub struct HttpClient {
    client: Client,
    /// We store the timeout so we can wrap GET requests explicitly.
    timeout: Option<Duration>,
}

impl HttpClient {
    /// Creates a new `HttpClient` using the provided configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if building the underlying reqwest client fails.
    pub fn new_with_config(config: HttpClientConfig) -> Result<Self, Error> {
        let mut builder = Client::builder();

        // Set timeout if provided. (This sets a default timeout for all requests.)
        if let Some(timeout) = config.timeout {
            builder = builder.timeout(timeout);
        }

        // Set proxy if provided.
        if let Some(proxy_url) = config.proxy {
            builder = builder.proxy(Proxy::all(&proxy_url)?);
        }

        // Set default headers (e.g., custom user agent) if provided.
        if let Some(user_agent) = config.user_agent {
            let mut headers = HeaderMap::new();
            headers.insert(
                USER_AGENT,
                HeaderValue::from_str(&user_agent).expect("Invalid user agent header value"),
            );
            builder = builder.default_headers(headers);
        }

        // Build the reqwest client.
        let client = builder.build()?;
        Ok(HttpClient {
            client,
            timeout: config.timeout,
        })
    }

    /// Sends an asynchronous GET request to the specified URL with an explicit timeout.
    ///
    /// If a timeout is configured, the request will error if it takes longer than that duration.
    pub async fn get(
        &self,
        url: &str,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>> {
        // Use the configured timeout or fall back to 10 seconds if none is provided.
        let timeout_duration = self.timeout.unwrap_or(Duration::from_secs(10));
        let request_future = self.client.get(url).send();

        // Wrap the GET request in a Tokio timeout.
        match time::timeout(timeout_duration, request_future).await {
            Ok(result) => result.map_err(|e| e.into()),
            Err(_) => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Request timed out",
            ))),
        }
    }
}

/// Returns a default HTTP client (without a custom timeout).
pub fn get_default_http_client() -> HttpClient {
    HttpClient {
        client: Client::new(),
        timeout: None,
    }
}
