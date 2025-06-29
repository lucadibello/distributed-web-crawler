use lapin::options::QueueDeclareOptions;
use lapin::{Channel, Connection, ConnectionProperties, Queue};
use std::env;
use std::error::Error;
use tracing::{debug, info, instrument, span, Level};

pub struct RabbitClient {
    conn: Connection,
    channel: Channel,
    queue_name: String,
}

impl RabbitClient {
    /// Creates a new RabbitMQ client instance.
    fn new(conn: Connection, channel: Channel, queue_name: String) -> Self {
        RabbitClient {
            conn,
            channel,
            queue_name,
        }
    }

    /// Establishes a connection to RabbitMQ, creates a channel, and declares a queue.
    #[instrument(name = "RabbitMQ Setup", skip_all)]
    pub async fn build() -> Result<Self, Box<dyn Error>> {
        // Load environment variables
        let user = env::var("RABBIT_USER")?;
        let password = env::var("RABBIT_PASSWORD")?;
        let host = env::var("RABBIT_HOST")?;
        let port = env::var("RABBIT_PORT")?;
        let queue_name = env::var("RABBIT_QUEUE")?;
        let crawler_type = env::var("CRAWLER_TYPE")?;

        // Get the current span and record fields to it individually.
        let span = tracing::Span::current();
        span.record("rabbit.host", &host);
        span.record("rabbit.port", &port);
        span.record("rabbit.queue", &queue_name);

        info!("Starting RabbitMQ client setup");

        let addr = format!("amqp://{}:{}@{}:{}", user, password, host, port);
        info!("Connecting to RabbitMQ");

        let conn = Connection::connect(&addr, ConnectionProperties::default()).await?;
        info!("Connection successful");

        debug!("Creating channel");
        let channel = conn.create_channel().await?;

        let consumer_tag = format!("crawler-{}", crawler_type.trim());

        let queue_span = span!(Level::DEBUG, "Queue Declaration", consumer_tag = %consumer_tag);
        let _enter = queue_span.enter();

        debug!("Declaring queue");
        let queue_options = QueueDeclareOptions {
            durable: true,
            exclusive: false,
            auto_delete: false,
            ..Default::default()
        };

        let _queue: Queue = channel
            .queue_declare(&queue_name, queue_options, Default::default())
            .await?;

        info!("Queue declared successfully");

        Ok(RabbitClient::new(conn, channel, queue_name))
    }

    /// Publishes a message to the declared queue.
    #[instrument(
        name = "Enqueue Message",
        skip(self, payload),
        fields(
            rabbit.queue = %self.queue_name,
            msg.size = payload.len()
        )
    )]
    pub async fn enqueue(&self, payload: String) -> Result<(), Box<dyn Error>> {
        debug!("Publishing message");
        self.channel
            .basic_publish(
                "",
                &self.queue_name,
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
    #[instrument(name = "Close Connection", skip(self))]
    pub async fn close(self) -> Result<(), Box<dyn Error>> {
        info!("Closing channel and connection");
        self.channel.close(200, "Goodbye").await?;
        self.conn.close(200, "Bye").await?;
        info!("Connection closed successfully.");
        Ok(())
    }
}
