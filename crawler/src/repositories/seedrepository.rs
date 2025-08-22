use std::{
    fs::{self, read_to_string},
    path::Path,
};

use tracing::warn;
use url::Url;

pub async fn load_seeds_from_dir(dir_path: &str) -> Result<Vec<Url>, String> {
    let mut seeds = Vec::new();

    let entries = fs::read_dir(Path::new(dir_path))
        .map_err(|e| format!("Failed to read dir '{dir_path}': {e}"))?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                warn!("Skipping unreadable dir entry: {e}");
                continue;
            }
        };

        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let content = match read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                let fname = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");
                warn!("Skipping unreadable file '{}': {}", fname, e);
                continue;
            }
        };

        for (ln, line) in content.lines().enumerate() {
            match Url::parse(line) {
                Ok(u) => seeds.push(u),
                Err(e) => {
                    let fname = path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown");
                    warn!("Invalid URL at {}:{} -> '{}': {}", fname, ln + 1, line, e);
                }
            }
        }
    }

    Ok(seeds)
}

pub fn load_default_seeds() -> Vec<Url> {
    vec![
        // General-purpose seeds
        Url::parse("https://en.wikipedia.org/wiki/Main_Page").unwrap(),
        Url::parse("https://www.bbc.com").unwrap(),
        Url::parse("https://news.ycombinator.com/").unwrap(),
        // Open data and research
        Url::parse("https://arxiv.org/").unwrap(),
        Url::parse("https://scholar.google.com/").unwrap(),
        Url::parse("https://data.gov/").unwrap(),
        // Technology and development
        Url::parse("https://github.com/trending").unwrap(),
        Url::parse("https://stackoverflow.com/").unwrap(),
        Url::parse("https://www.producthunt.com/").unwrap(),
        // Social and community-driven content
        Url::parse("https://www.reddit.com/r/technology/").unwrap(),
        Url::parse("https://medium.com/").unwrap(),
        // E-commerce and marketplaces
        Url::parse("https://www.amazon.com/").unwrap(),
        Url::parse("https://www.ebay.com/").unwrap(),
    ]
}
