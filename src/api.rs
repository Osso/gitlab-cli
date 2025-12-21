use anyhow::{anyhow, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::Value;

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
            HeaderValue::from_str(&format!("Bearer {}", token))
                .context("Invalid auth token")?,
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

    async fn get(&self, path: &str) -> Result<Value> {
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

    fn encoded_project(&self) -> String {
        urlencoding::encode(&self.project).into_owned()
    }

    pub async fn list_merge_requests(&self, params: &MrListParams) -> Result<Value> {
        let mut query_parts = vec![
            format!("per_page={}", params.per_page),
            format!("state={}", params.state),
        ];

        if let Some(author) = &params.author_username {
            query_parts.push(format!("author_username={}", urlencoding::encode(author)));
        }
        if let Some(after) = &params.created_after {
            query_parts.push(format!("created_after={}", urlencoding::encode(after)));
        }
        if let Some(before) = &params.created_before {
            query_parts.push(format!("created_before={}", urlencoding::encode(before)));
        }
        if let Some(after) = &params.updated_after {
            query_parts.push(format!("updated_after={}", urlencoding::encode(after)));
        }
        if let Some(order) = &params.order_by {
            query_parts.push(format!("order_by={}", order));
        }
        if let Some(sort) = &params.sort {
            query_parts.push(format!("sort={}", sort));
        }

        let query = query_parts.join("&");
        self.get(&format!(
            "/projects/{}/merge_requests?{}",
            self.encoded_project(),
            query
        ))
        .await
    }

    pub async fn get_merge_request(&self, iid: u64) -> Result<Value> {
        self.get(&format!(
            "/projects/{}/merge_requests/{}",
            self.encoded_project(),
            iid
        ))
        .await
    }

    pub async fn list_pipelines(&self, per_page: u32) -> Result<Value> {
        self.get(&format!(
            "/projects/{}/pipelines?per_page={}",
            self.encoded_project(),
            per_page
        ))
        .await
    }

    pub async fn get_pipeline(&self, id: u64) -> Result<Value> {
        self.get(&format!(
            "/projects/{}/pipelines/{}",
            self.encoded_project(),
            id
        ))
        .await
    }

    pub async fn list_pipeline_jobs(&self, pipeline_id: u64) -> Result<Value> {
        self.get(&format!(
            "/projects/{}/pipelines/{}/jobs?per_page=100",
            self.encoded_project(),
            pipeline_id
        ))
        .await
    }

    pub async fn get_job_log(&self, job_id: u64) -> Result<String> {
        let url = format!(
            "{}/projects/{}/jobs/{}/trace",
            self.base_url,
            self.encoded_project(),
            job_id
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

#[derive(Default)]
pub struct MrListParams {
    pub per_page: u32,
    pub state: String,
    pub author_username: Option<String>,
    pub created_after: Option<String>,
    pub created_before: Option<String>,
    pub updated_after: Option<String>,
    pub order_by: Option<String>,
    pub sort: Option<String>,
}
