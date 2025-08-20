use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::{Client, Error, Proxy};
use std::time::Duration;
use tokio::time;
use tracing::{debug, error, instrument, warn};

pub struct HttpClientConfig {
    pub user_agent: Option<String>,
    pub proxy: Option<String>,
    pub timeout: Option<Duration>,
}

// A simple HTTP client wrapper that supports useful features for an HTTP crawler.
#[derive(Debug)]
pub struct HttpClient {
    client: Client,
    // We store the timeout so we can wrap GET requests explicitly.
    timeout: Option<Duration>,
}

impl HttpClient {
    // Creates a new `HttpClient` using the provided configuration.
    #[instrument(skip(config))]
    pub fn new_with_config(config: HttpClientConfig) -> Result<Self, Error> {
        let mut builder = Client::builder();

        // Set timeout if provided. (This sets a default timeout for all requests.)
        if let Some(timeout) = config.timeout {
            debug!("Setting HTTP client timeout to {:?}", timeout);
            builder = builder.timeout(timeout);
        }

        // Set proxy if provided.
        if let Some(proxy_url) = &config.proxy {
            debug!("Setting HTTP client proxy to {}", proxy_url);
            builder = builder.proxy(Proxy::all(proxy_url)?);
        }

        // Set default headers (e.g., custom user agent) if provided.
        if let Some(user_agent) = &config.user_agent {
            debug!("Setting HTTP client user agent to {}", user_agent);
            let mut headers = HeaderMap::new();
            headers.insert(
                USER_AGENT,
                HeaderValue::from_str(user_agent).expect("Invalid user agent header value"),
            );
            builder = builder.default_headers(headers);
        }

        // Build the reqwest client.
        debug!("Building HTTP client");
        let client = builder.build()?;
        Ok(HttpClient {
            client,
            timeout: config.timeout,
        })
    }

    // Sends an asynchronous GET request to the specified URL with an explicit timeout.
    //
    // If a timeout is configured, the request will error if it takes longer than that duration.
    #[instrument(skip(self))]
    pub async fn get(
        &self,
        url: &str,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>> {
        // Use the configured timeout or fall back to 10 seconds if none is provided.
        let timeout_duration = self.timeout.unwrap_or(Duration::from_secs(10));
        let request_future = self.client.get(url).send();

        // Wrap the GET request in a Tokio timeout.
        debug!("Sending GET request to {}", url);
        match time::timeout(timeout_duration, request_future).await {
            Ok(result) => {
                debug!("GET request to {} completed successfully", url);
                result.map_err(|e| e.into())
            }
            Err(_) => {
                warn!("GET request to {} timed out", url);
                Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Request timed out",
                )))
            }
        }
    }
}

/// Returns a default HTTP client (without a custom timeout).
pub fn get_default_http_client() -> HttpClient {
    // Create a default configuration with no custom user agent, proxy, or timeout.
    let config = HttpClientConfig {
        user_agent: None,
        proxy: None,
        timeout: None,
    };

    match HttpClient::new_with_config(config) {
        Ok(client) => client,
        Err(err) => {
            error!("Failed to create default HTTP client: {:?}", err);
            panic!("Cannot continue without a valid HTTP client");
        }
    }
}
