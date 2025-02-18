use lapin::options::{BasicAckOptions, BasicConsumeOptions, QueueDeclareOptions};
use lapin::{types::FieldTable, Connection, ConnectionProperties};

use crate::messages::{JsonMessage, MessageParser, QueueMessage};
use futures_lite::stream::StreamExt;

use log::{debug, error, info};
use std::env;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

pub struct RabbitClient {
    conn: Connection,

    queue: String,
    tag: String,
}

impl RabbitClient {
    pub fn new(conn: Connection, queue: String, tag: String) -> Self {
        RabbitClient { conn, queue, tag }
    }

    // close connection
    pub async fn close(&self) -> Result<(), Box<(dyn Error)>> {
        // try closing the connection
        self.conn.close(200, "Bye").await?;
        // return Ok if everything went well
        Ok(())
    }

    pub async fn consume(&self, sender: &Sender<Arc<JsonMessage>>) -> Result<(), Box<dyn Error>> {
        // create a channel
        debug!("Creating RabbitMQ channel...");
        let channel = self.conn.create_channel().await?;
        debug!("queue name: {}, tag: {}", &self.queue, &self.tag);

        // declare the queue we want to consume from
        let queue_options = QueueDeclareOptions {
            auto_delete: false,
            ..Default::default()
        };

        let result = channel
            .queue_declare(&self.queue, queue_options, Default::default())
            .await?;
        info!("Declared queue: {:?}", result);

        // Start consuming messages
        let mut consumer = channel
            .basic_consume(
                &self.queue,
                &self.tag,
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;

        info!("Declared queue");

        // Start consuming messages
        info!("Consuming messages...");
        while let Some(delivery) = consumer.next().await {
            let delivery = delivery.expect("error in consumer");

            // Ack the message to remove it from the queue
            delivery.ack(BasicAckOptions::default()).await?;

            // We wrap the delivered message into a QueueMessage struct
            let msg = QueueMessage::new(delivery).parse_message();
            if let Err(err) = msg {
                error!("Failed to parse message: {:?}", err);
                continue;
            }
            let msg = Arc::new(msg.unwrap());

            // Pass the message to the sender
            if sender.send(msg).await.is_err() {
                error!("Failed to send message to sender");
            }
        }

        Ok(())
    }
}

pub async fn build() -> Result<RabbitClient, Box<dyn Error>> {
    // Load env variables
    let user = env::var("RABBIT_USER").unwrap();
    let password = env::var("RABBIT_PASSWORD").unwrap();
    let host = env::var("RABBIT_HOST").unwrap();
    let port = env::var("RABBIT_PORT").unwrap();

    // Load queue and exchange
    let queue = env::var("RABBIT_QUEUE").unwrap();
    let tag = env::var("CONSUMER_NAME").unwrap();

    // Build RabbitMQ address
    let addr = format!("amqp://{}:{}@{}:{}", user, password, host, port);

    // Log the connection attempt
    info!("Attempting to connect to RabbitMQ at {}", addr);

    // Connect to RabbitMQ
    let conn = match Connection::connect(&addr, ConnectionProperties::default()).await {
        Ok(conn) => {
            info!("Successfully connected to RabbitMQ");
            Ok(conn)
        }
        Err(e) => {
            error!("Failed to connect to RabbitMQ: {}", e);
            Err(Box::new(e))
        }
    };

    // return struct with connection
    Ok(RabbitClient::new(conn?, queue, tag))
}
