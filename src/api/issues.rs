use anyhow::Result;
use serde_json::Value;

use super::Client;

#[derive(Default)]
pub struct IssueListParams {
    pub per_page: u32,
    pub state: String,
    pub author_username: Option<String>,
    pub assignee_username: Option<String>,
    pub labels: Option<String>,
    pub search: Option<String>,
    pub created_after: Option<String>,
}

impl Client {
    pub async fn list_issues(&self, params: &IssueListParams) -> Result<Value> {
        let mut query_parts = vec![
            format!("per_page={}", params.per_page),
            format!("state={}", params.state),
        ];

        if let Some(author) = &params.author_username {
            query_parts.push(format!("author_username={}", urlencoding::encode(author)));
        }
        if let Some(assignee) = &params.assignee_username {
            query_parts.push(format!(
                "assignee_username={}",
                urlencoding::encode(assignee)
            ));
        }
        if let Some(labels) = &params.labels {
            query_parts.push(format!("labels={}", urlencoding::encode(labels)));
        }
        if let Some(search) = &params.search {
            query_parts.push(format!("search={}", urlencoding::encode(search)));
        }
        if let Some(after) = &params.created_after {
            query_parts.push(format!("created_after={}", urlencoding::encode(after)));
        }

        let query = query_parts.join("&");
        self.get(&format!(
            "/projects/{}/issues?{}",
            self.encoded_project(),
            query
        ))
        .await
    }

    pub async fn get_issue(&self, iid: u64) -> Result<Value> {
        self.get(&format!(
            "/projects/{}/issues/{}",
            self.encoded_project(),
            iid
        ))
        .await
    }

    pub async fn create_issue(
        &self,
        title: &str,
        description: Option<&str>,
        labels: Option<&str>,
        assignee: Option<&str>,
    ) -> Result<Value> {
        let mut body = serde_json::json!({
            "title": title
        });

        if let Some(desc) = description {
            body["description"] = serde_json::Value::String(desc.to_string());
        }
        if let Some(lbls) = labels {
            body["labels"] = serde_json::Value::String(lbls.to_string());
        }
        if let Some(user) = assignee {
            body["assignee_username"] = serde_json::Value::String(user.to_string());
        }

        self.post(
            &format!("/projects/{}/issues", self.encoded_project()),
            &body,
        )
        .await
    }
}
