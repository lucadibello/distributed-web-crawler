mod agents;
mod clients;
mod requests;
mod validators;

use agents::crawler_agent::CrawlerAgent;

#[tokio::main]
async fn main() {
    // Initialize dotenv
    dotenv::dotenv().ok();

    // Fetch crawler type from environment variable or default to "default"
    let crawler_type = std::env::var("CRAWLER_TYPE").unwrap_or_else(|_| "default".to_string());

    // Fetch crawler object type from environment variable or assign default prompt
    // FIXME: Implement this when we have a proper prompt system
    // let crawler_objective = std::env::var("CRAWLER_OBJECT_TYPE").unwrap_or_else(|_| "You are a web crawler. You will scrape this website and extract all the relevant inforatino from it. Keep the information in a structured format.".to_string());

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
    log::info!("Number of agents: {}", n_agents);

    // Calculate the approximate number of seeds per agent.
    let chunk_size = seeds.len().div_ceil(n_agents);

    // Create a vector to hold all the agent tasks.
    let mut handles = Vec::new();

    // For each chunk, spawn a crawler agent.
    let mut id_counter: u16 = 1;
    for chunk in seeds.chunks(chunk_size) {
        // Convert the chunk of seeds (which are String) into Vec<&str> for the agent.
        let seeds_chunk: Vec<&str> = chunk.to_vec();

        // Create immutable borrow of the id_counter for the current agent.
        let current_id = id_counter;
        let crawler_type = crawler_type.clone();

        // start the agent in a separate task
        let handle = tokio::task::spawn(async move {
            log::info!("Starting agent with seeds: {:?}", seeds_chunk);
            // Each agent gets its own seeds.
            let mut agent = CrawlerAgent::new_with_seeds(current_id, crawler_type, seeds_chunk)
                .await
                .expect("Failed to create CrawlerAgent");
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
