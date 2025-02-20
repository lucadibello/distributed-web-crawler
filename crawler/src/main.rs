mod agents;
mod clients;
mod messages;
mod requests;
mod validators;

use agents::crawler_agent::CrawlerAgent;

#[tokio::main]
async fn main() {
    // Initialize dotenv
    dotenv::dotenv().ok();

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
        .unwrap_or("4".to_string())
        .parse::<usize>()
        .unwrap();

    // Calculate the approximate number of seeds per agent.
    let chunk_size = seeds.len().div_ceil(n_agents);

    // Create a vector to hold all the agent tasks.
    let mut handles = Vec::new();

    // Client single rabbitmq client
    let rabbit = crate::clients::rabbit_client::build().await;
    if rabbit.is_err() {
        eprintln!("Failed to connect to RabbitMQ: {:?}", rabbit.err());
        return;
    }

    // Now, get the client from the Result
    let client = rabbit.unwrap();

    // For each chunk, spawn a crawler agent.
    for chunk in seeds.chunks(chunk_size) {
        // Convert the chunk of seeds (which are String) into Vec<&str> for the agent.
        let seeds_chunk: Vec<&str> = chunk.to_vec();

        // Spawn the agent task.
        let handle = tokio::task::spawn(async move {
            // Each agent gets its own seeds.
            let mut agent = CrawlerAgent::new_with_seeds(&client, seeds_chunk);
            agent.start().await;
        });

        handles.push(handle);
    }

    // Wait for all agents to complete.
    for handle in handles {
        handle.await.unwrap();
    }
}
