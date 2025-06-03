use anyhow::Result;
use reqwest::{Client, ClientBuilder, Response};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, warn};

#[derive(Debug, Clone)]
pub struct HttpClient {
    client: Client,
    base_url: String,
    rate_limiter: Option<RateLimiter>,
}

#[derive(Debug, Clone)]
pub struct RateLimiter {
    requests_per_second: u32,
    last_request: std::sync::Arc<tokio::sync::Mutex<std::time::Instant>>,
}

impl HttpClient {
    pub fn new(base_url: String) -> Result<Self> {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .user_agent(format!("SniperBot/{}", env!("CARGO_PKG_VERSION")))
            .build()?;

        Ok(Self {
            client,
            base_url,
            rate_limiter: None,
        })
    }

    pub fn with_rate_limit(mut self, requests_per_second: u32) -> Self {
        self.rate_limiter = Some(RateLimiter {
            requests_per_second,
            last_request: std::sync::Arc::new(tokio::sync::Mutex::new(std::time::Instant::now())),
        });
        self
    }

    pub async fn get<T>(&self, endpoint: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.rate_limit().await;
        
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), endpoint.trim_start_matches('/'));
        debug!("GET request to: {}", url);

        let response = self.client.get(&url).send().await?;
        self.handle_response(response).await
    }

    pub async fn post<T, B>(&self, endpoint: &str, body: &B) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
        B: Serialize,
    {
        self.rate_limit().await;
        
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), endpoint.trim_start_matches('/'));
        debug!("POST request to: {}", url);

        let response = self.client
            .post(&url)
            .json(body)
            .send()
            .await?;
        
        self.handle_response(response).await
    }

    pub async fn get_raw(&self, endpoint: &str) -> Result<Response> {
        self.rate_limit().await;
        
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), endpoint.trim_start_matches('/'));
        debug!("GET raw request to: {}", url);

        let response = self.client.get(&url).send().await?;
        Ok(response)
    }

    async fn handle_response<T>(&self, response: Response) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let status = response.status();
        
        if status.is_success() {
            let text = response.text().await?;
            debug!("Response body: {}", text);
            
            let result: T = serde_json::from_str(&text)?;
            Ok(result)
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            warn!("HTTP error {}: {}", status, error_text);
            anyhow::bail!("HTTP error {}: {}", status, error_text);
        }
    }

    async fn rate_limit(&self) {
        if let Some(limiter) = &self.rate_limiter {
            let mut last_request = limiter.last_request.lock().await;
            let now = std::time::Instant::now();
            let min_interval = Duration::from_millis(1000 / limiter.requests_per_second as u64);
            
            if now.duration_since(*last_request) < min_interval {
                let sleep_duration = min_interval - now.duration_since(*last_request);
                debug!("Rate limiting: sleeping for {:?}", sleep_duration);
                tokio::time::sleep(sleep_duration).await;
            }
            
            *last_request = std::time::Instant::now();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_client_creation() {
        let client = HttpClient::new("http://example.com".to_string());
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let client = HttpClient::new("http://example.com".to_string())
            .unwrap()
            .with_rate_limit(2); // 2 requests per second

        let start = std::time::Instant::now();

        // This should trigger rate limiting
        client.rate_limit().await;
        client.rate_limit().await;
        client.rate_limit().await;

        let elapsed = start.elapsed();

        // Should take at least 1 second due to rate limiting
        assert!(elapsed >= Duration::from_millis(500));
    }
}
