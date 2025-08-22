use drivers::rabbit::RabbitDriver;
use models::PageData;

fn process_message(page_data: PageData) -> Result<(), String> {
    // Simulate processing the message...
    println!("Processing message: {}", page_data);

    Ok(())
}

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber
    tracing_subscriber::fmt::init();

    // Initialize dotenv
    dotenv::dotenv().ok();

    // connect to RabbitMQ
    let rabbit = RabbitDriver::new()
        .await
        .expect("Failed to build RabbitMQ client");

    // Start consuming messages
    rabbit
        .consume(process_message)
        .await
        .expect("Failed to start consuming messages");

    println!("All agents have completed their tasks.");
}
