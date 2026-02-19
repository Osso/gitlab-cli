use anyhow::{anyhow, Context, Result};
use serde_json::Value;

use super::Client;

impl Client {
    /// Make a raw API request. The endpoint can be with or without the `/api/v4/` prefix.
    pub async fn raw_request(
        &self,
        method: &str,
        endpoint: &str,
        data: Option<&str>,
    ) -> Result<String> {
        let endpoint = endpoint.strip_prefix('/').unwrap_or(endpoint);

        let url = if endpoint.starts_with("api/v4/") {
            format!("{}/{}", self.base_url.trim_end_matches("/api/v4"), endpoint)
        } else {
            format!("{}/{}", self.base_url, endpoint)
        };

        let builder = match method.to_uppercase().as_str() {
            "GET" => self.http.get(&url),
            "POST" => self.http.post(&url),
            "PUT" => self.http.put(&url),
            "DELETE" => self.http.delete(&url),
            "PATCH" => self.http.patch(&url),
            other => return Err(anyhow!("Unsupported HTTP method: {}", other)),
        };

        let builder = if let Some(json_str) = data {
            let body: Value = serde_json::from_str(json_str).context("Invalid JSON in --data")?;
            builder.json(&body)
        } else {
            builder
        };

        let response = builder.send().await.context("Failed to send request")?;
        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(anyhow!("HTTP {}: {}", status, body));
        }

        Ok(body)
    }
}
