use std::{fs, fs::read_to_string, path::Path};
use tracing::{debug, info, instrument, trace, warn};
use url::Url;

#[instrument(name = "Load seeds from directory", level = "info", skip_all, fields(dir = %dir_path))]
pub async fn load_seeds_from_dir(dir_path: &str) -> Result<Vec<Url>, String> {
    info!("Scanning directory");
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
            trace!("Skipping non-file: {:?}", path);
            continue;
        }

        let fname = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        debug!("Reading file '{}'", fname);

        let content = match read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                warn!("Skipping unreadable file '{}': {}", fname, e);
                continue;
            }
        };

        for (ln, line) in content.lines().enumerate() {
            match Url::parse(line) {
                Ok(u) => {
                    trace!("Accepted URL at {}:{} -> '{}'", fname, ln + 1, line);
                    seeds.push(u)
                }
                Err(e) => warn!("Invalid URL at {}:{} -> '{}': {}", fname, ln + 1, line, e),
            }
        }
    }

    info!(count = seeds.len(), "Loaded seeds");
    Ok(seeds)
}

#[instrument(name = "Load default seeds", level = "debug")]
pub fn load_default_seeds() -> Vec<Url> {
    let seeds = vec![
        Url::parse("https://en.wikipedia.org/wiki/Main_Page").unwrap(),
        Url::parse("https://www.bbc.com").unwrap(),
        Url::parse("https://news.ycombinator.com/").unwrap(),
        Url::parse("https://arxiv.org/").unwrap(),
        Url::parse("https://scholar.google.com/").unwrap(),
        Url::parse("https://data.gov/").unwrap(),
        Url::parse("https://github.com/trending").unwrap(),
        Url::parse("https://stackoverflow.com/").unwrap(),
        Url::parse("https://www.producthunt.com/").unwrap(),
        Url::parse("https://www.reddit.com/r/technology/").unwrap(),
        Url::parse("https://medium.com/").unwrap(),
        Url::parse("https://www.amazon.com/").unwrap(),
        Url::parse("https://www.ebay.com/").unwrap(),
    ];
    debug!(count = seeds.len(), "Default seeds prepared");
    seeds
}
