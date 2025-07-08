use tracing::{debug, instrument};
use url::Url;

#[instrument]
pub fn validate_url(s: &str) -> Result<(), String> {
    debug!("Validating URL: {}", s);
    // Try to parse the URL using the `url` crate.
    let url = Url::parse(s).map_err(|e| format!("Invalid URL: {}", e))?;

    // Optionally, ensure that the scheme is either http or https.
    match url.scheme() {
        "http" | "https" => Ok(()),
        scheme => Err(format!("Invalid URL scheme: {}", scheme)),
    }
}
