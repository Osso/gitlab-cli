use anyhow::{anyhow, Result};
use serde_json::Value;

use super::Client;

impl Client {
    pub async fn list_pipelines_for_branch(
        &self,
        branch: Option<&str>,
        per_page: u32,
    ) -> Result<Value> {
        let mut url = format!(
            "/projects/{}/pipelines?per_page={}",
            self.encoded_project(),
            per_page
        );
        if let Some(ref_name) = branch {
            url.push_str(&format!("&ref={}", urlencoding::encode(ref_name)));
        }
        self.get(&url).await
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

    pub async fn retry_job(&self, job_id: u64) -> Result<Value> {
        self.post(
            &format!("/projects/{}/jobs/{}/retry", self.encoded_project(), job_id),
            &serde_json::json!({}),
        )
        .await
    }

    pub async fn retry_pipeline(&self, pipeline_id: u64) -> Result<Value> {
        self.post(
            &format!(
                "/projects/{}/pipelines/{}/retry",
                self.encoded_project(),
                pipeline_id
            ),
            &serde_json::json!({}),
        )
        .await
    }

    pub async fn list_ci_variables(&self) -> Result<Value> {
        self.get(&format!(
            "/projects/{}/variables?per_page=100",
            self.encoded_project()
        ))
        .await
    }

    pub async fn get_ci_variable(&self, key: &str) -> Result<Value> {
        self.get(&format!(
            "/projects/{}/variables/{}",
            self.encoded_project(),
            urlencoding::encode(key)
        ))
        .await
    }
}
