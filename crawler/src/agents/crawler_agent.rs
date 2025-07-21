use crate::{
    clients::{
        rabbit_client::RabbitClient, redis_client::RedisClient, robots_client::RobotsTxtClient,
    },
    requests::{
        http_request::{HttpRequest, HttpResponse, PageData},
        request::Request,
    },
};
use std::collections::LinkedList;
use tracing::{debug, error, info, instrument, warn};

pub struct CrawlerAgent {
    name: String,
    queue: LinkedList<HttpRequest>,
    rabbit_conn: std::boxed::Box<RabbitClient>,
    redis_conn: std::boxed::Box<RedisClient>,
    robots_client: RobotsTxtClient,
    keywords: Vec<String>,
    max_depth: u32,
    respect_robots_txt: bool,
}

impl CrawlerAgent {
    #[instrument(skip(client_ref, redis_client))]
    fn new(
        name: String,
        client_ref: RabbitClient,
        redis_client: RedisClient,
        respect_robots_txt: bool,
    ) -> Self {
        CrawlerAgent {
            name,
            rabbit_conn: Box::new(client_ref),
            queue: LinkedList::<HttpRequest>::new(),
            redis_conn: Box::new(redis_client),
            robots_client: RobotsTxtClient::new(),
            keywords: vec![
                "news".to_string(),
                "article".to_string(),
                "blog".to_string(),
            ],
            max_depth: 2, // Default max depth
            respect_robots_txt,
        }
    }

    // Create crawler agent with a set of hard-coded URL seeds
    #[instrument(skip(seed))]
    pub async fn new_with_seeds(
        id: u16,
        type_name: String,
        seed: Vec<&str>,
        max_depth: u32,
        respect_robots_txt: bool,
    ) -> Result<Self, String> {
        // generate unique identigier for the agent
        let agent_name = format!("crawler-{type_name}-{id}");
        info!("Creating new crawler agent with name: {}", agent_name);

        // create rabbitmq and redis clients
        let rabbit = RabbitClient::build()
            .await
            .map_err(|e| format!("Failed to create RabbitMQ client: {e}"))?;
        let redis =
            RedisClient::build().map_err(|e| format!("Failed to create Redis client: {e}"))?;

        // intialize the agent with both clients
        let mut agent = CrawlerAgent::new(agent_name, rabbit, redis, respect_robots_txt);
        agent.max_depth = max_depth;

        // push assigned seed URLs into the queue
        for url in seed {
            agent.push(HttpRequest::new(String::from(url), 0));
        }

        // return the agent
        Ok(agent)
    }

    // Handle new request by pushing it to the queue.
    #[instrument(skip(self, req), fields(url = %req.target))]
    pub fn push(&mut self, req: HttpRequest) {
        debug!("Pushing new request to the queue");
        self.queue.push_back(req);
    }

    // Execute one queued request
    #[instrument(skip(self))]
    async fn execute(&mut self) -> Result<HttpResponse, String> {
        // Pull new request from the queue. The request is removed from the queue.
        let req = self.queue.pop_front().ok_or("Queue is empty")?;
        info!(
            "Executing request for URL: {} at depth {}",
            req.target, req.depth
        );

        if self.respect_robots_txt && !self.robots_client.is_allowed(&req.target).await {
            warn!("URL is not allowed by robots.txt: {}", req.target);
            return Err(format!("URL is not allowed by robots.txt: {}", req.target));
        }

        // Execute the request asynchronously.
        let res = req
            .execute()
            .await
            .map_err(|e| format!("Request error: {e}",))?;
        info!("Request executed successfully");

        // Enroll discovered links into the queue.
        if req.depth < self.max_depth {
            if let Some(extra) = &res.extra {
                debug!("Found {} links", extra.links.len());
                let mut prioritized_links = Vec::new();
                let mut other_links = Vec::new();

                for link in &extra.links {
                    if self.redis_conn.is_visited(link.as_str()).unwrap() {
                        debug!("Link already visited: {}", link);
                        continue;
                    }
                    self.redis_conn.mark_visited(link.as_str()).unwrap();

                    if self.keywords.iter().any(|keyword| link.contains(keyword)) {
                        prioritized_links.push(link.clone());
                    } else {
                        other_links.push(link.clone());
                    }
                }

                for link in prioritized_links {
                    debug!("Enqueueing prioritized link: {}", link);
                    self.push(HttpRequest::new(link, req.depth + 1));
                }

                for link in other_links {
                    debug!("Enqueueing link: {}", link);
                    self.push(HttpRequest::new(link, req.depth + 1));
                }
            }
        } else {
            info!(
                "Max depth reached for {}. Not enqueuing new links.",
                req.target
            );
        }

        let page_data = PageData {
            url: req.target.clone(),
            title: res.title.clone(),
            status_code: res.status_code,
            headers: res.headers.clone(),
            meta: res.meta.clone(),
            links: res.extra.as_ref().unwrap().links.clone(),
            body: res.extra.as_ref().unwrap().body.clone(),
        };

        let payload = serde_json::to_string(&page_data).unwrap();
        self.rabbit_conn.enqueue(payload).await.unwrap();

        // Return the response.
        Ok(res)
    }

    // Crawler main loop
    #[instrument(skip(self))]
    pub async fn start(&mut self) {
        info!("Starting crawler agent {}", self.name);
        // Continue processing while there are requests in the queue.
        while !self.queue.is_empty() {
            match self.execute().await {
                Ok(response) => {
                    info!(
                        "Processed response with status code: {}",
                        response.status_code
                    );
                }
                Err(err) => {
                    error!("Error executing request: {}", err);
                }
            }
        }
        info!("Crawler agent finished");
    }
}
