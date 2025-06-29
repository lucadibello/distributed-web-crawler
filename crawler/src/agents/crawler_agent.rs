use crate::{
    clients::{rabbit_client::RabbitClient, redis_client::RedisClient},
    requests::{
        http_request::{HttpRequest, HttpResponse},
        request::Request,
    },
};
use std::{
    collections::LinkedList,
    sync::{Arc, Mutex},
};

type ThreadSafeRabbitClient = Arc<Mutex<RabbitClient>>;

pub struct CrawlerAgent<'client> {
    queue: LinkedList<HttpRequest>,
    rabbit_client: &'client ThreadSafeRabbitClient,
    redis_conn: RedisClient,
}

impl<'client> CrawlerAgent<'client> {
    fn new(client_ref: &'client ThreadSafeRabbitClient) -> Self {
        CrawlerAgent {
            rabbit_client: client_ref,
            queue: LinkedList::<HttpRequest>::new(),
            redis_conn: RedisClient::new(),
        }
    }

    // Create crawler agent with a set of initial seeds
    pub fn new_with_seeds(client_ref: &'client ThreadSafeRabbitClient, seed: Vec<&str>) -> Self {
        let mut agent = CrawlerAgent::new(client_ref);

        for url in seed {
            agent.push(HttpRequest::new(String::from(url)));
        }

        agent
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

                // FIXME: Append message to rabbitmq processing queue

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
