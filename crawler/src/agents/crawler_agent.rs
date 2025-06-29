use crate::{
    clients::{rabbit_client::RabbitClient, redis_client::RedisClient},
    requests::{
        http_request::{HttpRequest, HttpResponse},
        request::Request,
    },
};
use std::collections::LinkedList;

pub struct CrawlerAgent {
    name: String,
    queue: LinkedList<HttpRequest>,
    rabbit_conn: std::boxed::Box<RabbitClient>,
    redis_conn: std::boxed::Box<RedisClient>,
}

impl CrawlerAgent {
    fn new(name: String, client_ref: RabbitClient, redis_client: RedisClient) -> Self {
        CrawlerAgent {
            name,
            rabbit_conn: Box::new(client_ref),
            queue: LinkedList::<HttpRequest>::new(),
            redis_conn: Box::new(redis_client),
        }
    }

    // Create crawler agent with a set of initial seeds
    pub async fn new_with_seeds(
        id: u16,
        type_name: String,
        seed: Vec<&str>,
    ) -> Result<Self, String> {
        // generate unique identigier for the agent
        let agent_name = format!("crawler-{}-{}", type_name, id);

        // create rabbitmq and redis clients
        let rabbit = RabbitClient::build()
            .await
            .map_err(|e| format!("Failed to create RabbitMQ client: {}", e))?;
        let redis =
            RedisClient::build().map_err(|e| format!("Failed to create Redis client: {}", e))?;

        // intialize the agent with both clients
        let mut agent = CrawlerAgent::new(agent_name, rabbit, redis);

        // push assigned seed URLs into the queue
        for url in seed {
            agent.push(HttpRequest::new(String::from(url)));
        }

        // return the agent
        Ok(agent)
    }

    // Handle new request by pushing it to the queue.
    pub fn push(&mut self, req: HttpRequest) {
        self.queue.push_back(req);
    }

    // Execute one queued request
    async fn execute(&mut self) -> Result<HttpResponse, String> {
        // Pull new request from the queue. The request is removed from the queue.
        let req = self.queue.pop_front().ok_or("Queue is empty")?;

        // Execute the request asynchronously.
        let res = req.execute().await.map_err(|e| format!("Error: {}", e))?;

        // Enroll discovered links into the queue.
        if let Some(extra) = &res.extra {
            println!("Found {} links", extra.links.len());
            for link in &extra.links {
                if self.redis_conn.is_visited(link.as_str()).unwrap() {
                    continue;
                }
                self.redis_conn.mark_visited(link.as_str()).unwrap();

                // FIXME: enqueue result to RabbitMQ

                // Enqueue the link (assuming HttpRequest::new expects an owned String)
                println!("Enqueueing link: {}", link);
                self.push(HttpRequest::new(link.clone()));
            }
        }

        // Schedule result to be written to a file.

        // Return the response.
        Ok(res)
    }

    // Crawler main loop
    pub async fn start(&mut self) {
        // Continue processing while there are requests in the queue.
        while !self.queue.is_empty() {
            match self.execute().await {
                Ok(response) => {
                    // FIXME: register response somewhere!
                    println!(
                        "Processed response with status code: {}",
                        response.status_code
                    );
                }
                Err(err) => {
                    eprintln!("Error executing request: {}", err);
                }
            }
        }
    }
}
