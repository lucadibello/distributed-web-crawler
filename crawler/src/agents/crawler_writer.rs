use std::{fs::OpenOptions, io::Write};

use tokio::sync::mpsc;

use crate::requests::http_request::{ExtraHttpResponseFields, HttpResponse};

pub struct CrawlerWriter {
    channel: mpsc::Receiver<HttpResponse>,
}

impl CrawlerWriter {
    pub fn new(channel: mpsc::Receiver<HttpResponse>) -> Self {
        CrawlerWriter { channel }
    }

    pub async fn listen_and_write(&mut self, file_path: &str) {
        let mut buffer: Vec<String> = Vec::new();
        // Set a threshold for bulk writes.
        let threshold = 10;
        let mut count = 0;

        // Open (or create) the output JSONL file asynchronously.
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(file_path)
            .expect("Unable to open file");

        // Listen for incoming responses from the channel.
        while let Some(mut response) = self.channel.recv().await {
            count += 1;

            if count % 100 == 0 {
                println!("Received {} responses", count);
            }

            // Get additional metadata from the HTTP response.
            let metadata = crate::parsers::http_parser::parse_http(response.clone());

            // Only proceed if the extracted clean text is non-empty.
            if metadata.clean_text.is_empty() {
                println!("[WRITER] Skipping response due to empty clean text");
                continue;
            }

            // Update the response's extra field with the clean text and preserve any links.
            response.extra = Some(ExtraHttpResponseFields {
                links: response.extra.map_or(Vec::new(), |extra| extra.links),
                body: metadata.clean_text,
            });

            // Serialize the response to a JSON string.
            let json_line = serde_json::to_string(&response).expect("Serialization to JSON failed");
            buffer.push(json_line);

            // If the buffer reaches the threshold, write it out.
            if buffer.len() >= threshold {
                for line in buffer.drain(..) {
                    file.write_all(format!("{}\n", line).as_bytes())
                        .expect("Unable to write to file");
                }
                file.flush().expect("Failed to flush file");
            }
        }

        // Write any remaining responses.
        for line in buffer {
            file.write_all(format!("{}\n", line).as_bytes())
                .expect("Unable to write to file");
        }
        file.flush().expect("Failed to flush file");
    }
}
