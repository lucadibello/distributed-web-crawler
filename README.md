# Distributed Web Crawler 🕸️

This project implements a **distributed web crawler** designed to efficiently scrape web pages, extract valuable information, and process it in a scalable manner. Leveraging Rust for its core crawling logic and a distributed architecture, it aims to provide a robust but simple solution for data collection from the web.

-----

## 🚀 Features

  * **Distributed Architecture**: Utilizes RabbitMQ for message queuing, enabling multiple crawler agents to operate concurrently and process tasks.
  * **Web Scraping**: Capable of sending HTTP requests, retrieving HTML content, and extracting links, titles, headers, and meta-information from web pages.
  * **Link Prioritization**: Prioritizes links containing specific keywords (e.g., "news", "article", "blog") to focus crawling on relevant content.
  * **Robots.txt Adherence**: Respects `robots.txt` rules to ensure ethical crawling practices.
  * **URL Validation**: Includes robust URL validation to ensure only valid HTTP/HTTPS links are processed and enqueued.
  * **Visited URL Tracking**: Employs Redis to keep track of visited URLs, preventing redundant processing and infinite loops.
  * **Configurable Depth**: Allows setting a maximum crawling depth to control the extent of the crawl.
  * **Concurrency**: Supports running multiple crawler agents concurrently to speed up the scraping process.
  * **Dockerized Deployment**: Includes a `docker-compose.yml` file for easy setup of the necessary services like Redis and RabbitMQ.

-----

## ⚙️ Architecture

The project is structured around a distributed model with the following key components:

  * **Crawler Agents**: Written in Rust, these agents are responsible for fetching web pages, parsing their content, extracting links, and managing their crawling queue. They interact with Redis to check visited URLs and RabbitMQ to enqueue discovered pages.
  * **RabbitMQ**: Acts as a message broker, facilitating communication between different parts of the system, particularly for enqueuing web pages to be processed.
  * **Redis**: Used as a data store for keeping track of URLs that have already been visited, ensuring efficiency and preventing redundant work.

The `docker-compose.yml` file orchestrates these services, setting up Redis and RabbitMQ containers, making it easy to deploy the environment.

-----

## 🛠️ Installation and Setup

To get this project up and running, follow these steps:

1.  **Clone the Repository**:

    ```bash
    git clone https://github.com/lucadibello/distributed-web-crawler.git
    cd distributed-web-crawler
    ```

2.  **Environment Variables**:
    Create a `.env` file in the root directory of the project and define the following environment variables. These are used by the RabbitMQ and Redis clients:

    ```env
    RABBIT_USER=guest
    RABBIT_PASSWORD=guest
    RABBIT_HOST=rabbit
    RABBIT_PORT=5672
    RABBIT_QUEUE=crawler_queue
    CRAWLER_TYPE=default
    REDIS_HOST=redis
    REDIS_PORT=6379
    REDIS_DB=0
    MAX_DEPTH=2
    RESPECT_ROBOTS_TXT=true
    N_AGENTS=2
    ```

      * `RABBIT_USER`, `RABBIT_PASSWORD`: Credentials for RabbitMQ.
      * `RABBIT_HOST`, `RABBIT_PORT`: Host and port for RabbitMQ.
      * `RABBIT_QUEUE`: The name of the RabbitMQ queue for messages.
      * `CRAWLER_TYPE`: A type identifier for the crawler.
      * `REDIS_HOST`, `REDIS_PORT`, `REDIS_DB`: Host, port, and database for Redis.
      * `MAX_DEPTH`: The maximum depth the crawler will traverse (default: 2).
      * `RESPECT_ROBOTS_TXT`: Boolean to enable/disable `robots.txt` adherence (default: `true`).
      * `N_AGENTS`: Number of concurrent crawler agents to run (default: number of CPU cores).

3.  **Docker Compose**:
    Ensure Docker and Docker Compose are installed on your system. Then, start the necessary services:

    ```bash
    docker-compose up -d
    ```

    This will bring up the Redis and RabbitMQ containers.

4.  **Build and Run the Crawler**:
    Navigate into the `crawler` directory and build the Rust project. Make sure you have Rust and Cargo installed.

    ```bash
    cd crawler
    cargo build --release
    ```

    Then, run the crawler:

    ```bash
    cargo run --release
    ```

    The crawler will start processing the defined seed URLs and enqueueing discovered pages.

-----

## 📄 Project Structure and Key Modules

The project is divided into `crawler` and `consumers` components, with the main logic residing in `crawler`.

  * **`crawler/`**: Contains the core logic for web scraping.

      * **`src/main.rs`**: The entry point for the crawler application, responsible for initializing agents and managing the crawling process.
      * **`src/agents/crawler_agent.rs`**: Defines the `CrawlerAgent` responsible for managing the crawling process, including queueing requests, executing HTTP requests, and processing responses. It interacts with `RedisClient` and `RabbitClient`.
      * **`src/clients/`**:
          * **`http_client.rs`**: Provides a wrapper around `reqwest` for making HTTP GET requests, with support for timeouts and user agents.
          * **`rabbit_client.rs`**: Handles connections to RabbitMQ, including declaring queues, enqueuing messages, and consuming messages.
          * **`redis_client.rs`**: Manages connections to Redis for tracking visited URLs.
          * **`robots_client.rs`**: Implements logic for fetching and parsing `robots.txt` files to determine if a URL is allowed to be crawled.
      * **`src/requests/`**:
          * **`http_request.rs`**: Defines the `HttpRequest` and `HttpResponse` structures and implements the `execute` method for making and processing HTTP requests, including link and meta tag extraction.
          * **`request.rs`**: A trait defining the `Request` interface, allowing for different types of requests to be executed.
      * **`src/validators.rs`**: Contains utility functions for validating URLs.

  * **`consumers/`**: (Currently a placeholder) Intended to process messages from RabbitMQ.

      * **`src/main.rs`**: Entry point for consumers.
      * **`src/clients/rabbit_client.rs`**: Similar to the crawler's RabbitMQ client, for consuming messages.

-----

## 💡 How it Works

The crawler operates by initializing a configurable number of **crawler agents**. Each agent is given a set of **seed URLs** to begin its crawl. When an agent processes a URL:

1.  It first checks with `RobotsTxtClient` to ensure it's allowed to crawl the URL.
2.  It then verifies if the URL has already been visited using `RedisClient`.
3.  If allowed and not visited, an `HttpRequest` is executed to fetch the page content.
4.  From the `HttpResponse`, relevant data like title, headers, and meta tags are extracted.
5.  All discoverable links on the page are extracted and validated.
6.  Discovered links are then prioritized (e.g., if they contain keywords) and marked as visited in Redis.
7.  Finally, the extracted page data (including URL, title, status, links, and body) is serialized into a JSON payload and enqueued into RabbitMQ for further processing by consumers.

The agents continue this process recursively until the maximum defined depth is reached or no new links are found.
