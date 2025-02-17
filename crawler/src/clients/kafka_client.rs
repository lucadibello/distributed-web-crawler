use std::time::Duration;

use kafka::client::{Compression, KafkaClient as KC};

#[derive(Debug)]
pub struct KafkaClient {
    client: KC,
    config: Option<KafkaClientConfig>,
}

#[derive(Debug)]
pub struct KafkaClientConfig {
    brokers: Vec<String>,
    compression: Compression,
    conn_idle_timeout: Duration,
}

impl KafkaClient {
    fn new(config: Option<KafkaClientConfig>) -> Self {
        // If no config is provided, use default values
        if config.is_none() {
            // Create new client + specify list of valid brokers to use
            // client = KC::new(vec!["localhost:9092".to_string()]);

            // Fatal error: cannot infer default URL for the Kafka broker
            panic!("No Kafka broker URL provided");
        }
        // create client here
        let client = KC::new(config.as_ref().unwrap().brokers.clone());

        // Create KafkaClient instance
        KafkaClient { client, config }
    }

    fn publish(&self, message: String, topic: String) {
        // FIXME: Publish a single message to a Kafka topic
    }

    fn publish_bulk(&self, messages: Vec<String>, topic: String) {
        // FIXME: Publish multiple messages on a Kafka topic
    }
}
