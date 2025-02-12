use crate::requests::http_request::HttpResponse;
use regex::Regex;
use rust_stemmers::{Algorithm, Stemmer};

// Additional metadata extracted from the page.
#[derive(Debug)]
pub struct HttpMetadata {
    /// The raw body text after stripping HTML.
    pub clean_text: String,
    /// The tokenized words from the clean text.
    pub tokens: Vec<String>,
    /// The stemmed tokens from the clean text.
    pub stemmed_tokens: Vec<String>,
    // You could also add more fields here like language, keywords, etc.
}

/// Parses the HTTP response to extract additional metadata useful for a RAG pipeline.
/// In this example, we:
/// - Remove script and style blocks as well as HTML comments.
/// - Strip any remaining HTML tags.
/// - Remove extra whitespace and special characters.
/// - Tokenize the resulting text on whitespace.
/// - Stem each token using a stemming algorithm.
pub fn parse_http(res: HttpResponse) -> HttpMetadata {
    // Extract the raw body from the `extra` field if available.
    let raw_body = res
        .extra
        .as_ref()
        .map(|extra| extra.body.clone())
        .unwrap_or_default();

    // Step 1: Remove <script> tags and their content.
    let script_re =
        Regex::new(r"(?is)<script[^>]*>.*?</script>").expect("Invalid regex pattern for scripts");
    let without_scripts = script_re.replace_all(&raw_body, "");

    // Step 2: Remove <style> tags and their content.
    let style_re =
        Regex::new(r"(?is)<style[^>]*>.*?</style>").expect("Invalid regex pattern for styles");
    let without_scripts_and_styles = style_re.replace_all(&without_scripts, "");

    // Step 3: Remove HTML comments.
    let comment_re = Regex::new(r"(?is)<!--.*?-->").expect("Invalid regex pattern for comments");
    let without_comments = comment_re.replace_all(&without_scripts_and_styles, "");

    // Step 4: Remove any remaining HTML tags.
    let tag_re = Regex::new(r"<[^>]*>").expect("Invalid regex pattern for HTML tags");
    let clean_text = tag_re.replace_all(&without_comments, "").to_string();

    // Step 5: Remove extra whitespace and special characters.
    let clean_text = clean_text
        .replace("\n", " ")
        .replace("\r", " ")
        .replace("\t", " ")
        .replace("\u{a0}", " ")
        .replace("\u{200b}", " ")
        .replace("\u{200e}", " ")
        .replace("\u{200f}", " ")
        .replace("\u{2028}", " ")
        .replace("\u{2029}", " ")
        .replace("\u{3000}", " ")
        .replace("\u{feff}", " ")
        .replace("\u{fffd}", " ");

    // Tokenize the clean text on whitespace.
    let tokens: Vec<String> = clean_text
        .split_whitespace()
        .map(|token| token.to_string())
        .collect();

    // Create a stemmer for English.
    let stemmer = Stemmer::create(Algorithm::English);

    // Stem each token.
    let stemmed_tokens: Vec<String> = tokens
        .iter()
        .map(|token| stemmer.stem(token).to_string())
        .collect();

    // Return the extracted metadata.
    HttpMetadata {
        clean_text,
        tokens,
        stemmed_tokens,
    }
}
