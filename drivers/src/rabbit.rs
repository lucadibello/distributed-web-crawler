use futures_lite::StreamExt;
use lapin::options::{
    BasicAckOptions, BasicConsumeOptions, BasicNackOptions, BasicPublishOptions,
    QueueDeclareOptions,
};
use lapin::types::FieldTable;
use lapin::{BasicProperties, Channel, Connection, ConnectionProperties};
use serde::Serialize;
use std::env;
use tracing::{Level, debug, error, info, instrument, span, trace, warn};

#[allow(dead_code)]
pub struct RabbitDriver {
    conn: Connection,
    channel: Channel,
    queue_name: String,
    consumer_tag: String,
}

impl RabbitDriver {
    /// Build from environment. Defaults: guest/guest@127.0.0.1:5672, queue=default_queue, crawler=generic
    #[instrument(
        name = "RabbitMQ Setup",
        level = "info",
        skip_all,
        fields(rabbit.host, rabbit.port, rabbit.queue, rabbit.addr, rabbit.consumer_tag)
    )]
    pub async fn new() -> Result<Self, String> {
        // env with defaults
        let user = env::var("RABBIT_USER").unwrap_or_else(|_| "guest".to_string());
        let password = env::var("RABBIT_PASSWORD").unwrap_or_else(|_| "guest".to_string());
        let host = env::var("RABBIT_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("RABBIT_PORT").unwrap_or_else(|_| "5672".to_string());
        let queue_name = env::var("RABBIT_QUEUE").unwrap_or_else(|_| "default_queue".to_string());
        let crawler_type = env::var("CRAWLER_TYPE").unwrap_or_else(|_| "generic".to_string());

        // never log credentials
        let addr = format!("amqp://{}:{}@{}:{}", user, password, host, port);
        let conn_addr = format!("amqp://{}:{}", host, port); // safe to log
        let consumer_tag = format!("crawler-{}", crawler_type.trim());

        // enrich span
        let span = tracing::Span::current();
        span.record("rabbit.host", &host);
        span.record("rabbit.port", &port);
        span.record("rabbit.queue", &queue_name);
        span.record("rabbit.addr", &conn_addr);
        span.record("rabbit.consumer_tag", &consumer_tag);

        info!("Connecting to RabbitMQ at {}", conn_addr);
        let conn = Connection::connect(&addr, ConnectionProperties::default())
            .await
            .map_err(|e| {
                error!("Connection failed: {}", e);
                format!("Failed to connect to RabbitMQ at {conn_addr}: {e}")
            })?;
        info!("Connection established");

        debug!("Creating channel");
        let channel = conn.create_channel().await.map_err(|e| {
            error!("Channel creation failed: {}", e);
            format!("Failed to create channel: {e}")
        })?;

        let queue_span = span!(Level::DEBUG, "Queue Declaration", %consumer_tag, %queue_name);
        let _enter = queue_span.enter();

        debug!("Declaring durable queue");
        let queue_options = QueueDeclareOptions {
            durable: true,
            exclusive: false,
            auto_delete: false,
            ..Default::default()
        };

        channel
            .queue_declare(&queue_name, queue_options, FieldTable::default())
            .await
            .map_err(|e| {
                error!("Queue declare failed for '{}': {}", queue_name, e);
                format!("Queue declare failed for '{}': {e}", queue_name)
            })?;
        info!("Queue declared: {}", queue_name);

        Ok(RabbitDriver {
            conn,
            channel,
            queue_name,
            consumer_tag,
        })
    }

    #[instrument(
        name = "Enqueue Message",
        level = "info",
        skip(self, payload),
        fields(rabbit.queue = %self.queue_name, msg.size)
    )]

    pub async fn enqueue<T: Serialize + Sized>(&self, payload: T) -> Result<(), String> {
        // convert to string
        let data = serde_json::to_string(&payload).map_err(|e| {
            error!("Serialization failed: {}", e);
            format!("Failed to serialize payload: {e}")
        })?;

        self.channel
            .basic_publish(
                "", // empty exchange for default
                &self.queue_name,
                BasicPublishOptions::default(),
                data.as_bytes(),
                BasicProperties::default(),
            )
            .await
            .map_err(|e| {
                error!("Publish send failed: {}", e);
                format!("Publish send failed: {e}")
            })?
            .await
            .map_err(|e| {
                error!("Publish confirm failed: {}", e);
                format!("Publish confirm failed: {e}")
            })?;

        debug!("Message published to {}", self.queue_name);
        Ok(())
    }

    #[instrument(name = "Close Connection", level = "info", skip(self))]
    pub async fn close(self) -> Result<(), String> {
        info!("Closing channel and connection");
        self.channel.close(200, "Goodbye").await.map_err(|e| {
            error!("Channel close failed: {}", e);
            format!("Channel close failed: {e}")
        })?;
        self.conn.close(200, "Bye").await.map_err(|e| {
            error!("Connection close failed: {}", e);
            format!("Connection close failed: {e}")
        })?;
        info!("Closed");
        Ok(())
    }

    #[instrument(
        name = "Consume Messages",
        level = "info",
        skip(self, on_message),
        fields(rabbit.queue = %self.queue_name, rabbit.consumer_tag = %self.consumer_tag)
    )]
    pub async fn consume<F, V>(&self, on_message: F) -> Result<(), String>
    where
        // thread-safe function that receives the message payload, and returns Ok(()) on success or
        // Err(String) on failure
        F: Fn(V) -> Result<(), String> + Send + Sync + 'static,
        V: serde::de::DeserializeOwned + 'static,
    {
        info!("Starting consumer");
        let mut consumer = self
            .channel
            .basic_consume(
                &self.queue_name,
                &self.consumer_tag,
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| {
                error!("Failed to start consumer: {}", e);
                format!("Failed to start consumer: {e}")
            })?;

        info!("Consumer started, waiting for messages...");
        while let Some(delivery) = consumer.next().await {
            let delivery = match delivery {
                Ok(d) => d,
                Err(e) => {
                    error!("Consumer yielded error: {}", e);
                    return Err(format!("Consumer yielded error: {e}"));
                }
            };

            let tag = delivery.delivery_tag;
            let corr = delivery
                .properties
                .correlation_id()
                .as_ref()
                .map(|c| String::from_utf8_lossy(c.as_str().as_bytes()).to_string())
                .unwrap_or_default();

            let msg_span = span!(Level::DEBUG, "Handle Delivery", delivery.tag = %tag, correlation_id = %corr, size = delivery.data.len());
            let _enter = msg_span.enter();

            debug!("Received message");
            trace!("Payload size: {} bytes", delivery.data.len());

            // deserialize message to expected type
            let actual_data: V = match serde_json::from_slice::<V>(&delivery.data) {
                Ok(v) => v,
                Err(e) => {
                    error!("Deserialization failed for tag {}: {}", tag, e);
                    if let Err(e2) = delivery
                        .nack(BasicNackOptions {
                            requeue: false,
                            ..Default::default()
                        })
                        .await
                    {
                        error!("Nack failed after deserialization error '{}': {}", e, e2);
                    }
                    return Err(format!("Failed to deserialize message for tag {tag}: {e}"));
                }
            };

            // check result of handler
            match on_message(actual_data) {
                Ok(_) => {
                    delivery
                        .ack(BasicAckOptions::default())
                        .await
                        .map_err(|e| {
                            error!("Ack failed for tag {}: {}", tag, e);
                            format!("Ack failed: {e}")
                        })?;
                    debug!("Acked tag {}", tag);
                }
                Err(handler_err) => {
                    warn!("Handler error for tag {}: {}", tag, handler_err);
                    let opts = BasicNackOptions::default();
                    delivery.nack(opts).await.map_err(|e2| {
                        error!("Nack failed after handler error '{}': {}", handler_err, e2);
                        format!("Nack failed after handler error '{handler_err}': {e2}")
                    })?;
                    debug!("Nacked tag {} (requeue=false)", tag);
                }
            }
        }

        info!("Consumer stopped");
        Ok(())
    }
}
