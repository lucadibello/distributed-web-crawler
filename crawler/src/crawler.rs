use crate::{
    clients::robots::RobotsTxtClient,
    controllers::{urlcontroller::UrlControllerTrait, UrlController},
    requests::{
        http::{HttpRequest, HttpResponse, PageData},
        request::Request,
    },
};
use drivers::rabbit::RabbitDriver;
use std::{collections::LinkedList, sync::Arc};
use tracing::{debug, error, info, instrument, warn};
use url::Url;

pub struct Crawler {
    name: String,
    queue: LinkedList<HttpRequest>,
    url_controller: Arc<UrlController>,
    rabbit: Arc<RabbitDriver>,
    robots_client: RobotsTxtClient,
    max_depth: u32,
    respect_robots_txt: bool,
}

impl Crawler {
    #[instrument(skip(url_controller, rabbit, seed), fields(name = %name))]
    pub fn new(
        name: String,
        url_controller: Arc<UrlController>,
        rabbit: Arc<RabbitDriver>,
        respect_robots_txt: bool,
        max_depth: u32,
        seed: Vec<&str>,
    ) -> Self {
        let mut agent = Crawler {
            name,
            queue: LinkedList::<HttpRequest>::new(),
            url_controller,
            rabbit,
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
            if let Some(extra) = &res.extra {
                debug!("Found {} links", extra.links.len());
                // parse the Url to ensure it's valid
                let target_url =
                    Url::parse(&req.target).map_err(|e| format!("Invalid URL: {e}"))?;

                // check if url is already visited
                if let Ok(visited) = self.url_controller.is_visited(target_url.clone()).await {
                    if visited {
                        info!("URL already visited: {}", target_url);
                        return Ok(res);
                    }
                } else {
                    error!("Error checking if URL is visited: {}", target_url);
                }

                // otherwise, mark it as visited
                if let Err(err) = self.url_controller.mark_visited(target_url).await {
                    error!("Error marking URL as visited: {}", err);
                }

                // now, we need to process the links found during the crawl
                for link in extra.links.iter() {
                    self.push(HttpRequest::new(link.clone(), req.depth + 1));
                }
            }
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

        // enqueue the page data to RabbitMQ for further processing
        self.rabbit
            .enqueue(serde_json::to_string(&page_data).map_err(|e| e.to_string())?)
            .await
            .map_err(|e| format!("RabbitMQ enqueue error: {e}"))?;

        // Return the response (useful for logging)
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
