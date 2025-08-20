use crate::{
    clients::robots::RobotsTxtClient,
    repositories::urls::UrlRepository,
    requests::{
        http::{HttpRequest, HttpResponse, PageData},
        request::Request,
    },
};
use std::{collections::LinkedList, sync::Arc};
use tracing::{debug, error, info, instrument, warn};

pub struct Crawler {
    name: String,
    queue: LinkedList<HttpRequest>,
    url_repository: Arc<UrlRepository>,
    robots_client: RobotsTxtClient,
    max_depth: u32,
    respect_robots_txt: bool,
}

impl Crawler {
    #[instrument(skip(url_repository))]
    pub fn new(
        name: String,
        url_repository: Arc<UrlRepository>,
        respect_robots_txt: bool,
        max_depth: u32,
        seed: Vec<&str>,
    ) -> Self {
        let mut agent = Crawler {
            name,
            queue: LinkedList::<HttpRequest>::new(),
            url_repository,
            robots_client: RobotsTxtClient::new(),
            max_depth,
            respect_robots_txt,
        };

        // push seed URLs into the queue if present
        if !seed.is_empty() {
            for url in seed {
                agent.push(HttpRequest::new(String::from(url), 0));
            }
        }

        agent
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
        debug!(
            "Executing request for URL: {} at depth {}",
            req.target, req.depth
        );

        // Ensure the request is allowed by robots.txt if configured.
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
            debug!("Response links will be processed for URL: {}", req.target);
            // if let Some(extra) = &res.extra {
            //     debug!("Found {} links", extra.links.len());
            //     let mut prioritized_links = Vec::new();
            //     let mut other_links = Vec::new();
            //
            //     for link in &extra.links {
            //         if self.redis_conn.is_visited(link.as_str()).unwrap() {
            //             debug!("Link already visited: {}", link);
            //             continue;
            //         }
            //         self.redis_conn.mark_visited(link.as_str()).unwrap();
            //
            //         if self.keywords.iter().any(|keyword| link.contains(keyword)) {
            //             prioritized_links.push(link.clone());
            //         } else {
            //             other_links.push(link.clone());
            //         }
            //     }
            //
            //     for link in prioritized_links {
            //         debug!("Enqueueing prioritized link: {}", link);
            //         self.push(HttpRequest::new(link, req.depth + 1));
            //     }
            //
            //     for link in other_links {
            //         debug!("Enqueueing link: {}", link);
            //         self.push(HttpRequest::new(link, req.depth + 1));
            //     }
            // }
        } else {
            warn!(
                "Max depth reached for {}. Not enqueuing new links.",
                req.target
            );
        }

        // store the page data in the RabbitMQ queue.
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

        // FIXME: send payload to Apache Storm
        debug!("Sending page data to the message queue: {}", payload);

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
