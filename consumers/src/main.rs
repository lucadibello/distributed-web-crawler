mod clients;

use tracing::info;

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber
    tracing_subscriber::fmt::init();

    // Initialize dotenv
    dotenv::dotenv().ok();
}
