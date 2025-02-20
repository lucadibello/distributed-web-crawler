use crate::{
    clients::{rabbit_client::RabbitClient, redis_client::RedisClient},
    requests::{
        http_request::{HttpRequest, HttpResponse},
        request::Request,
    },
};
use std::collections::LinkedList;

// Idea: make request, collect information, dump to file
pub struct CrawlerAgent {
    queue: LinkedList<HttpRequest>,
    rabbit_client_ref: &'static RabbitClient,
    pub redis_conn: RedisClient,
}

impl CrawlerAgent {
    pub fn new(client_ref: &'static RabbitClient) -> Self {
        CrawlerAgent {
            rabbit_client_ref: client_ref,
            queue: LinkedList::<HttpRequest>::new(),
            redis_conn: RedisClient::new(),
        }
    }

    pub fn new_with_seeds(client_ref: &'static RabbitClient, seed: Vec<&str>) -> Self {
        let mut agent = CrawlerAgent::new(client_ref);

        for url in seed {
            agent.push(HttpRequest::new(String::from(url)));
        }

        agent
    }

    /// Adds a new request to the queue.
    pub fn push(&mut self, req: HttpRequest) {
        self.queue.push_back(req);
    }

    /// Executes one request from the queue:
    /// - It pops a request from the queue,
    /// - Executes the request asynchronously,
    /// - Enrolls discovered links into the queue,
    /// - And returns the HTTP response.
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

    /// Starts the crawler agent.
    ///
    /// This is the core loop of the crawler that:
    /// 1. Logs the start of the agent.
    /// 2. Loops while there are requests in the queue.
    /// 3. Executes each request.
    /// 4. Handles errors and processes responses (e.g., dumping to a file).
    /// 5. Terminates when the queue is empty.
    pub async fn start(&mut self) {
        // Continue processing while there are requests in the queue.
        while !self.queue.is_empty() {
            match self.execute().await {
                Ok(response) => {
                    // Optionally process the response here.
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
