mod agents;
mod clients;
mod requests;
mod validators;

use std::sync::{Arc, Mutex};

use agents::crawler_agent::CrawlerAgent;
use clients::rabbit_client::RabbitClient;

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

    // Create single connection to RabbitMQ.
    let rabbit = RabbitClient::build().await;
    if rabbit.is_err() {
        eprintln!("Failed to connect to RabbitMQ: {:?}", rabbit.err());
        return;
    }

    // Wrap rabbit client in Arc+Murewx for shared ownership and mutability.
    let rabbit_client = Arc::new(Mutex::new(rabbit.unwrap()));

    // For each chunk, spawn a crawler agent.
    for chunk in seeds.chunks(chunk_size) {
        // Convert the chunk of seeds (which are String) into Vec<&str> for the agent.
        let seeds_chunk: Vec<&str> = chunk.to_vec();

        // clone connection to RabbitMQ client to share with the each agent
        let connection = rabbit_client.clone();

        // Spawn the agent task.
        let handle = tokio::task::spawn(async move {
            // Each agent gets its own seeds.
            let mut agent = CrawlerAgent::new_with_seeds(&connection, seeds_chunk);
            agent.start().await;
        });

        handles.push(handle);
    }

    // Wait for all agents to complete.
    for handle in handles {
        handle.await.unwrap();
    }

    println!("All agents have completed their tasks.");

    // Close all connections to RabbitMQ gracefully when the program ends.
    match Arc::try_unwrap(rabbit_client) {
        // If unwrap successful (when there is only one reference), we can close the connection.
        Ok(mutex) => match mutex.into_inner() {
            // we obtain the RabbitClient from the Mutex.
            Ok(client) => {
                // close the RabbitMQ connection.
                if let Err(e) = client.close().await {
                    eprintln!("Failed to close RabbitMQ connection: {:?}", e);
                } else {
                    println!("RabbitMQ connection closed successfully.");
                }
            }
            Err(e) => {
                eprintln!("Failed to unwrap RabbitClient: {:?}", e);
            }
        },
        Err(_) => {
            eprintln!("Failed to unwrap Arc<Mutex<RabbitClient>>. Multiple references exist.");
            return;
        }
    };
}
