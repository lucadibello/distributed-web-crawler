use lapin::options::QueueDeclareOptions;
use lapin::{Channel, Connection, ConnectionProperties, Queue};

use log::{debug, info};
use std::env;
use std::error::Error;

pub struct RabbitClient {
    conn: Connection,
    channel: Channel,
}

impl RabbitClient {
    /// Creates a new RabbitMQ client instance.
    fn new(conn: Connection, channel: Channel) -> Self {
        RabbitClient { conn, channel }
    }

    /// Establishes a connection to RabbitMQ, creates a channel, and declares a queue.
    pub async fn build() -> Result<Self, Box<dyn Error>> {
        // Load environment variables, propagating errors instead of panicking
        let user = env::var("RABBIT_USER")?;
        let password = env::var("RABBIT_PASSWORD")?;
        let host = env::var("RABBIT_HOST")?;
        let port = env::var("RABBIT_PORT")?;
        let queue_name = env::var("RABBIT_QUEUE")?;
        let crawler_type = env::var("CRAWLER_TYPE")?;

        let addr = format!("amqp://{}:{}@{}:{}", user, password, host, port);
        info!("Attempting to connect to RabbitMQ at {}", host);

        // Connect to RabbitMQ, using `?` for concise error handling
        let conn = Connection::connect(&addr, ConnectionProperties::default()).await?;
        info!("Successfully connected to RabbitMQ");

        // Create a channel from the connection
        debug!("Creating RabbitMQ channel...");
        let channel = conn.create_channel().await?;
        debug!("Channel created successfully");

        // Get current Thread Id for consumer tag
        let consumer_tag = format!("crawler-{}-{}", crawler_type.trim(), -1);
        debug!(
            "Declaring queue '{}' with consumer tag '{}'",
            &queue_name, &consumer_tag
        );

        // Declare queue options. Making it durable is often a good default.
        let queue_options = QueueDeclareOptions {
            durable: true,
            exclusive: false,
            auto_delete: false,
            ..Default::default()
        };

        // Declare the queue and wait for the confirmation from the server
        let _queue: Queue = channel
            .queue_declare(&queue_name, queue_options, Default::default())
            .await?;

        info!("Queue '{}' declared successfully", &queue_name);

        // Return the constructed client
        Ok(RabbitClient::new(conn, channel))
    }

    pub async fn enqueue(&self, payload: String) -> Result<(), Box<dyn Error>> {
        debug!("Publishing message to RabbitMQ queue");
        // Publish the message to the queue
        self.channel
            .basic_publish(
                "",
                &env::var("RABBIT_QUEUE")?,
                lapin::options::BasicPublishOptions::default(),
                payload.as_bytes(),
                lapin::BasicProperties::default(),
            )
            .await?
            .await?;
        debug!("Message published successfully");
        Ok(())
    }

    /// Gracefully closes the channel and the connection.
    pub async fn close(self) -> Result<(), Box<dyn Error>> {
        info!("Closing RabbitMQ channel and connection...");
        // Close the channel first
        self.channel.close(200, "Goodbye").await?;
        // Then close the connection
        self.conn.close(200, "Bye").await?;
        info!("Connection and channel closed successfully.");
        Ok(())
    }
}
