use reqwest::Client;
use serde::de::DeserializeOwned;
use std::time::Duration;

#[derive(Clone)]
pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub fn new(timeout_secs: u64) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .user_agent("Nolelamphim-Desktop/0.1.0 (Linux x86_64; Slint)")
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { client }
    }

    pub async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T, String> {
        tracing::debug!("GET Request: {}", url);

        let response = self
            .client
            .get(url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| format!("Network error calling {}: {}", url, e))?;

        let status = response.status();
        if !status.is_success() {
            return Err(format!("HTTP error {}: {}", status, url));
        }

        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response body: {}", e))?;

        serde_json::from_str::<T>(&body)
            .map_err(|e| format!("JSON deserialization error from {}: {} (Snippet: {:.150})", url, e, body))
    }

    pub async fn get_json_with_query<T: DeserializeOwned, Q: serde::Serialize + ?Sized>(
        &self,
        url: &str,
        query: &Q,
    ) -> Result<T, String> {
        tracing::debug!("GET Request with query: {}", url);

        let response = self
            .client
            .get(url)
            .header("Accept", "application/json")
            .query(query)
            .send()
            .await
            .map_err(|e| format!("Network error calling {}: {}", url, e))?;

        let status = response.status();
        if !status.is_success() {
            return Err(format!("HTTP error {}: {}", status, url));
        }

        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response body: {}", e))?;

        serde_json::from_str::<T>(&body)
            .map_err(|e| format!("JSON deserialization error from {}: {} (Snippet: {:.150})", url, e, body))
    }
}
