mod branches;
mod ci;
mod groups;
mod issues;
mod merge_requests;
mod mirrors;
mod raw;
mod webhooks;

use anyhow::{anyhow, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::Value;

pub use issues::IssueListParams;
pub use merge_requests::MrListParams;
pub use webhooks::{WebhookCreateParams, WebhookUpdateParams};

pub struct Client {
    http: reqwest::Client,
    base_url: String,
    project: String,
}

impl Client {
    pub fn new(host: &str, token: &str, project: &str) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).context("Invalid auth token")?,
        );

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        let base_url = format!("{}/api/v4", host.trim_end_matches('/'));

        Ok(Self {
            http,
            base_url,
            project: project.to_string(),
        })
    }

    pub(crate) fn encoded_project(&self) -> String {
        urlencoding::encode(&self.project).into_owned()
    }

    pub(crate) async fn get(&self, path: &str) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http
            .get(&url)
            .send()
            .await
            .context("Failed to send request")?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(anyhow!("HTTP {}: {}", status, body));
        }

        serde_json::from_str(&body).context("Failed to parse JSON response")
    }

    pub(crate) async fn put(&self, path: &str, body: &Value) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http
            .put(&url)
            .json(body)
            .send()
            .await
            .context("Failed to send request")?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(anyhow!("HTTP {}: {}", status, body));
        }

        serde_json::from_str(&body).context("Failed to parse JSON response")
    }

    pub(crate) async fn post(&self, path: &str, body: &Value) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http
            .post(&url)
            .json(body)
            .send()
            .await
            .context("Failed to send request")?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(anyhow!("HTTP {}: {}", status, body));
        }

        serde_json::from_str(&body).context("Failed to parse JSON response")
    }

    pub(crate) async fn post_empty(&self, path: &str) -> Result<()> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http
            .post(&url)
            .send()
            .await
            .context("Failed to send request")?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await?;
            return Err(anyhow!("HTTP {}: {}", status, body));
        }

        Ok(())
    }

    pub(crate) async fn delete(&self, path: &str) -> Result<()> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http
            .delete(&url)
            .send()
            .await
            .context("Failed to send request")?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await?;
            return Err(anyhow!("HTTP {}: {}", status, body));
        }
        Ok(())
    }

    pub async fn get_project(&self) -> Result<Value> {
        self.get(&format!("/projects/{}", self.encoded_project()))
            .await
    }

    pub async fn get_raw_file(&self, file_path: &str, git_ref: &str) -> Result<String> {
        let encoded_path = urlencoding::encode(file_path);
        let url = format!(
            "{}/projects/{}/repository/files/{}/raw?ref={}",
            self.base_url,
            self.encoded_project(),
            encoded_path,
            urlencoding::encode(git_ref)
        );
        let response = self.http.get(&url).send().await?;
        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(anyhow!("HTTP {}: {}", status, body));
        }

        Ok(body)
    }
}
