use anyhow::Result;
use serde_json::Value;

use super::Client;

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

impl Client {
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

    pub async fn update_merge_request(&self, iid: u64, params: &Value) -> Result<Value> {
        self.put(
            &format!(
                "/projects/{}/merge_requests/{}",
                self.encoded_project(),
                iid
            ),
            params,
        )
        .await
    }

    pub async fn get_merge_request_changes(&self, iid: u64) -> Result<Value> {
        self.get(&format!(
            "/projects/{}/merge_requests/{}/changes",
            self.encoded_project(),
            iid
        ))
        .await
    }

    pub async fn set_automerge(&self, iid: u64, remove_source_branch: bool) -> Result<Value> {
        self.put(
            &format!(
                "/projects/{}/merge_requests/{}/merge",
                self.encoded_project(),
                iid
            ),
            &serde_json::json!({
                "merge_when_pipeline_succeeds": true,
                "should_remove_source_branch": remove_source_branch
            }),
        )
        .await
    }

    pub async fn merge_merge_request(
        &self,
        iid: u64,
        remove_source_branch: bool,
    ) -> Result<Value> {
        self.put(
            &format!(
                "/projects/{}/merge_requests/{}/merge",
                self.encoded_project(),
                iid
            ),
            &serde_json::json!({
                "should_remove_source_branch": remove_source_branch
            }),
        )
        .await
    }

    pub async fn create_merge_request(
        &self,
        title: &str,
        source_branch: &str,
        target_branch: &str,
        description: Option<&str>,
    ) -> Result<Value> {
        let mut body = serde_json::json!({
            "title": title,
            "source_branch": source_branch,
            "target_branch": target_branch
        });

        if let Some(desc) = description {
            body["description"] = serde_json::Value::String(desc.to_string());
        }

        self.post(
            &format!("/projects/{}/merge_requests", self.encoded_project()),
            &body,
        )
        .await
    }

    pub async fn list_mr_pipelines(&self, iid: u64) -> Result<Value> {
        self.get(&format!(
            "/projects/{}/merge_requests/{}/pipelines",
            self.encoded_project(),
            iid
        ))
        .await
    }

    pub async fn list_mr_notes(&self, iid: u64, per_page: u32) -> Result<Value> {
        self.get(&format!(
            "/projects/{}/merge_requests/{}/notes?sort=desc&per_page={}",
            self.encoded_project(),
            iid,
            per_page
        ))
        .await
    }

    pub async fn create_mr_note(&self, iid: u64, body: &str) -> Result<Value> {
        self.post(
            &format!(
                "/projects/{}/merge_requests/{}/notes",
                self.encoded_project(),
                iid
            ),
            &serde_json::json!({ "body": body }),
        )
        .await
    }

    pub async fn approve_merge_request(&self, iid: u64) -> Result<()> {
        self.post_empty(&format!(
            "/projects/{}/merge_requests/{}/approve",
            self.encoded_project(),
            iid
        ))
        .await
    }

    pub async fn list_mr_discussions(&self, iid: u64, per_page: u32) -> Result<Value> {
        self.get(&format!(
            "/projects/{}/merge_requests/{}/discussions?per_page={}",
            self.encoded_project(),
            iid,
            per_page
        ))
        .await
    }

    pub async fn create_mr_discussion(
        &self,
        iid: u64,
        body: &str,
        position: &Value,
    ) -> Result<Value> {
        self.post(
            &format!(
                "/projects/{}/merge_requests/{}/discussions",
                self.encoded_project(),
                iid
            ),
            &serde_json::json!({
                "body": body,
                "position": position
            }),
        )
        .await
    }

    pub async fn reply_to_discussion(
        &self,
        iid: u64,
        discussion_id: &str,
        body: &str,
    ) -> Result<Value> {
        self.post(
            &format!(
                "/projects/{}/merge_requests/{}/discussions/{}/notes",
                self.encoded_project(),
                iid,
                discussion_id
            ),
            &serde_json::json!({ "body": body }),
        )
        .await
    }

    pub async fn resolve_discussion(
        &self,
        iid: u64,
        discussion_id: &str,
        resolved: bool,
    ) -> Result<Value> {
        self.put(
            &format!(
                "/projects/{}/merge_requests/{}/discussions/{}",
                self.encoded_project(),
                iid,
                discussion_id
            ),
            &serde_json::json!({ "resolved": resolved }),
        )
        .await
    }
}
