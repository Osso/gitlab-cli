use anyhow::Result;
use serde_json::Value;

use super::Client;

pub struct WebhookCreateParams {
    pub url: String,
    pub token: Option<String>,
    pub push_events: bool,
    pub merge_requests_events: bool,
    pub issues_events: bool,
    pub pipeline_events: bool,
    pub tag_push_events: bool,
    pub note_events: bool,
    pub job_events: bool,
    pub releases_events: bool,
    pub enable_ssl_verification: bool,
}

pub struct WebhookUpdateParams {
    pub url: Option<String>,
    pub token: Option<String>,
    pub push_events: Option<bool>,
    pub merge_requests_events: Option<bool>,
    pub issues_events: Option<bool>,
    pub pipeline_events: Option<bool>,
    pub tag_push_events: Option<bool>,
    pub note_events: Option<bool>,
    pub job_events: Option<bool>,
    pub releases_events: Option<bool>,
    pub enable_ssl_verification: Option<bool>,
}

impl Client {
    pub async fn list_webhooks(&self) -> Result<Value> {
        self.get(&format!("/projects/{}/hooks", self.encoded_project()))
            .await
    }

    pub async fn get_webhook(&self, hook_id: u64) -> Result<Value> {
        self.get(&format!(
            "/projects/{}/hooks/{}",
            self.encoded_project(),
            hook_id
        ))
        .await
    }

    pub async fn create_webhook(&self, params: &WebhookCreateParams) -> Result<Value> {
        let mut body = serde_json::json!({
            "url": params.url,
            "push_events": params.push_events,
            "merge_requests_events": params.merge_requests_events,
            "issues_events": params.issues_events,
            "pipeline_events": params.pipeline_events,
            "tag_push_events": params.tag_push_events,
            "note_events": params.note_events,
            "job_events": params.job_events,
            "releases_events": params.releases_events,
            "enable_ssl_verification": params.enable_ssl_verification
        });

        if let Some(token) = &params.token {
            body["token"] = serde_json::Value::String(token.clone());
        }

        self.post(
            &format!("/projects/{}/hooks", self.encoded_project()),
            &body,
        )
        .await
    }

    pub async fn update_webhook(
        &self,
        hook_id: u64,
        params: &WebhookUpdateParams,
    ) -> Result<Value> {
        let body = build_webhook_update_body(params);
        self.put(
            &format!("/projects/{}/hooks/{}", self.encoded_project(), hook_id),
            &body,
        )
        .await
    }

    pub async fn delete_webhook(&self, hook_id: u64) -> Result<()> {
        self.delete(&format!(
            "/projects/{}/hooks/{}",
            self.encoded_project(),
            hook_id
        ))
        .await
    }

    pub async fn test_webhook(&self, hook_id: u64, trigger: &str) -> Result<Value> {
        self.post(
            &format!(
                "/projects/{}/hooks/{}/test/{}",
                self.encoded_project(),
                hook_id,
                trigger
            ),
            &serde_json::json!({}),
        )
        .await
    }
}

fn build_webhook_update_body(params: &WebhookUpdateParams) -> Value {
    let mut body = serde_json::json!({});

    if let Some(url) = &params.url {
        body["url"] = serde_json::Value::String(url.clone());
    }
    if let Some(token) = &params.token {
        body["token"] = serde_json::Value::String(token.clone());
    }
    if let Some(v) = params.push_events {
        body["push_events"] = serde_json::Value::Bool(v);
    }
    if let Some(v) = params.merge_requests_events {
        body["merge_requests_events"] = serde_json::Value::Bool(v);
    }
    if let Some(v) = params.issues_events {
        body["issues_events"] = serde_json::Value::Bool(v);
    }
    if let Some(v) = params.pipeline_events {
        body["pipeline_events"] = serde_json::Value::Bool(v);
    }
    if let Some(v) = params.tag_push_events {
        body["tag_push_events"] = serde_json::Value::Bool(v);
    }
    if let Some(v) = params.note_events {
        body["note_events"] = serde_json::Value::Bool(v);
    }
    if let Some(v) = params.job_events {
        body["job_events"] = serde_json::Value::Bool(v);
    }
    if let Some(v) = params.releases_events {
        body["releases_events"] = serde_json::Value::Bool(v);
    }
    if let Some(v) = params.enable_ssl_verification {
        body["enable_ssl_verification"] = serde_json::Value::Bool(v);
    }

    body
}
