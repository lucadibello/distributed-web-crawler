mod clients;
mod controllers;
mod crawler;
mod repositories;
mod requests;
mod services;
mod validators;

use std::sync::Arc;

use crawler::Crawler;
use drivers::redis::RedisDriver;
use tokio::sync::Mutex;
use tracing::info;

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber
    tracing_subscriber::fmt::init();

    // Initialize dotenv
    dotenv::dotenv().ok();

    // connect to Redis
    let redis = RedisDriver::build().expect("Failed to build Redis client");

    // Fetch crawler type from environment variable or default to "default"
    let crawler_type = std::env::var("CRAWLER_TYPE").unwrap_or_else(|_| "default".to_string());

    // Fetch max depth from environment variable or default to 2
    let max_depth = std::env::var("MAX_DEPTH")
        .unwrap_or_else(|_| "2".to_string())
        .parse::<u32>()
        .expect("MAX_DEPTH must be a valid u32");

    // Fetch respect_robots_txt from environment variable or default to true
    let respect_robots_txt = std::env::var("RESPECT_ROBOTS_TXT")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .expect("RESPECT_ROBOTS_TXT must be a valid boolean");

    // Define list of seeds where to start scraping
    let seeds = [
        // General-purpose seeds
        "https://en.wikipedia.org/wiki/Main_Page",
        "https://www.bbc.com",
        "https://news.ycombinator.com/",
        // Open data and research
        "https://arxiv.org/",
        "https://scholar.google.com/",
        "https://data.gov/",
        // Technology and development
        "https://github.com/trending",
        "https://stackoverflow.com/",
        "https://www.producthunt.com/",
        // Social and community-driven content
        "https://www.reddit.com/r/technology/",
        "https://medium.com/",
        // E-commerce and marketplaces
        "https://www.amazon.com/",
        "https://www.ebay.com/",
    ];

    // Set the number of agents (threads) you want to run concurrently.
    let n_agents = std::env::var("N_AGENTS")
        .unwrap_or_else(|_| num_cpus::get().to_string())
        .parse::<usize>()
        .unwrap();
    info!("Number of agents: {}", n_agents);

    let chunk_size = seeds.len().div_ceil(n_agents);
    let mut handles = Vec::new();
    let mut id_counter: u16 = 1;

    // create UrlController to mark visited URLs
    // NOTE: we use two Arc here because both UrlController and RedisDriver may be shared
    // independently across multiple agents (e.g. each agent currently has one UrlController, but
    // in the future we may want to have multiple controllers based on the same driver.
    let url_controller = Arc::new(controllers::UrlController::new(Arc::new(Mutex::new(redis))));

    for chunk in seeds.chunks(chunk_size) {
        // Convert the chunk of seeds (which are String) into Vec<&str> for the agent.
        let seeds_chunk: Vec<&str> = chunk.to_vec();

        // Create immutable borrow of the id_counter for the current agent.
        let current_id = id_counter;
        let crawler_type = crawler_type.clone();
        let log_name = format!("crawler-{crawler_type}-{current_id}");
        let agent_url_controller = Arc::clone(&url_controller);

        // start the agent in a separate tas
        let handle = tokio::task::spawn(async move {
            // create new crawler instance
            let mut agent = Crawler::new(
                log_name,
                agent_url_controller,
                respect_robots_txt,
                max_depth,
                seeds_chunk,
            );

            // start agent asynchronously
            agent.start().await;
        });
        handles.push(handle);
        id_counter += 1;
    }

    // Wait for all agents to complete.
    for handle in handles {
        handle.await.unwrap();
    }

    println!("All agents have completed their tasks.");
}
