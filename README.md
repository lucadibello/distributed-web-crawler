# Distributed Web Crawler 🕸️

A concurrent web crawler built in Rust. It fetches pages, extracts links and metadata, stores visited URLs in Redis, and publishes page data to RabbitMQ for downstream processing.

Each process manages its own crawling tasks locally. Redis ensures URL deduplication, while RabbitMQ handles result distribution. This design supports multiple independent crawlers and consumers running in parallel.

---

## Features (Implemented)

- Concurrency: spawns multiple crawler agents in a single process using Tokio (`N_AGENTS`).
- Seeds: loads seed URLs from `crawler/seeds/*.txt` (one URL per line) or falls back to defaults.
- Fetching: HTTP GET via `reqwest` with timeouts; parses HTML with `scraper` to extract links and meta tags.
- URL validation: only `http`/`https` URLs are accepted.
- robots.txt check: best‑effort allow/deny via a simple client (configurable with `RESPECT_ROBOTS_TXT`).
- Visited tracking: stores visited URLs in Redis to avoid repeats.
- Results queue: enqueues `PageData` (URL, status, headers, meta, links, body) to RabbitMQ.
- Dockerized infra: `docker-compose.yml` spins up Redis and RabbitMQ.

## Architecture

- Crawler (`crawler/`)
  - `src/main.rs`: initializes Redis/RabbitMQ, loads seeds, and launches agents.
  - `src/crawler.rs`: in‑process crawler with a local queue, depth control, robots/visited checks, and publishing of `PageData` to RabbitMQ.
  - `src/clients/http.rs`: lightweight HTTP client wrapper around `reqwest` (timeout, proxy, user‑agent support).
  - `src/clients/robots.rs`: simple robots.txt fetcher and parser (best‑effort).
  - `src/requests/http.rs` + `src/requests/request.rs`: request trait and HTTP request/response structures (extracts links + meta).
  - `src/repositories/*`: seed loading and URL repository over a generic cache driver.
  - `src/controllers.rs` + `src/services.rs`: visited URL orchestration over the repository/driver.
  - `src/validators.rs`: URL validation.

- Drivers (`drivers/`)
  - `redis.rs`: implements a generic `CacheDriver` backed by Redis (JSON serialization via `serde_json`).
  - `rabbit.rs`: RabbitMQ driver using `lapin` with helpers to declare a queue, publish, and consume.
  - `errors.rs`: shared driver error types.

- Consumers (`consumers/`)
  - Minimal example consumer that deserializes `PageData` messages from RabbitMQ and prints them.

Data flow: agents pop URLs from a local queue → check robots/visited → fetch page → extract links/meta → mark URL visited → enqueue discovered links locally (until `MAX_DEPTH`) → publish `PageData` to RabbitMQ.

---

## Quick Start

Prerequisites: Rust (stable) and Docker Compose.

1. Clone

```bash
git clone https://github.com/lucadibello/distributed-web-crawler.git
cd distributed-web-crawler
```

2. Configure environment

Create a `.env` in the repo root (used by Docker services and the binaries):

```env
RABBIT_USER=guest
RABBIT_PASSWORD=guest
RABBIT_HOST=127.0.0.1
RABBIT_PORT=5672
RABBIT_QUEUE=crawler_queue
CRAWLER_TYPE=default

REDIS_HOST=127.0.0.1
REDIS_PORT=6379
REDIS_DB=0

MAX_DEPTH=2
RESPECT_ROBOTS_TXT=true
N_AGENTS=2
```

Notes:

- `CRAWLER_TYPE` is used for RabbitMQ consumer tags and logs.
- `MAX_DEPTH` limits how deep newly discovered links are followed.
- `RESPECT_ROBOTS_TXT` toggles the robots check.
- `N_AGENTS` controls concurrency per process (defaults to number of CPU cores).

3. Start infra

```bash
docker-compose up -d
```

This launches Redis and RabbitMQ (management UI on `http://localhost:15672`).

4. Run the crawler

```bash
cd crawler
cargo run --release
```

Seeds: put one URL per line in any file under `crawler/seeds/` (e.g., `crawler/seeds/general.txt`). Invalid lines are ignored. If the directory is missing/unreadable, a default set of seeds is used.

5. (Optional) Run a consumer

In a separate terminal, print messages as they arrive:

```bash
cd consumers
cargo run --release
```

---

## Configuration Reference

- RabbitMQ
  - `RABBIT_USER`, `RABBIT_PASSWORD`, `RABBIT_HOST`, `RABBIT_PORT`
  - `RABBIT_QUEUE`: queue name used for publishing/consuming `PageData`.
  - `CRAWLER_TYPE`: used in consumer tag naming.

- Redis
  - `REDIS_HOST`, `REDIS_PORT`, `REDIS_DB`

- Crawler
  - `MAX_DEPTH`: maximum crawl depth for newly discovered links.
  - `RESPECT_ROBOTS_TXT`: enable/disable robots.txt checks.
  - `N_AGENTS`: number of concurrent agents within the process.
