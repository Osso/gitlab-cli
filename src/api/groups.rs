use anyhow::Result;
use serde_json::Value;

use super::Client;

impl Client {
    pub async fn list_group_members(
        &self,
        group: &str,
        per_page: u32,
        show_email: bool,
    ) -> Result<Value> {
        let encoded_group = urlencoding::encode(group);
        if show_email {
            self.get(&format!(
                "/groups/{}/billable_members?per_page={}",
                encoded_group, per_page
            ))
            .await
        } else {
            self.get(&format!(
                "/groups/{}/members?per_page={}",
                encoded_group, per_page
            ))
            .await
        }
    }

    pub async fn list_group_subgroups(&self, group: &str, per_page: u32) -> Result<Value> {
        let encoded_group = urlencoding::encode(group);
        self.get(&format!(
            "/groups/{}/subgroups?per_page={}",
            encoded_group, per_page
        ))
        .await
    }

    pub async fn get_group(&self, group: &str) -> Result<Value> {
        let encoded_group = urlencoding::encode(group);
        self.get(&format!("/groups/{}", encoded_group)).await
    }

    pub async fn archive_project(&self, project: &str) -> Result<Value> {
        let encoded_project = urlencoding::encode(project);
        self.post(
            &format!("/projects/{}/archive", encoded_project),
            &serde_json::json!({}),
        )
        .await
    }

    pub async fn unarchive_project(&self, project: &str) -> Result<Value> {
        let encoded_project = urlencoding::encode(project);
        self.post(
            &format!("/projects/{}/unarchive", encoded_project),
            &serde_json::json!({}),
        )
        .await
    }

    pub async fn update_project(&self, project: &str, body: &Value) -> Result<Value> {
        let encoded_project = urlencoding::encode(project);
        self.put(&format!("/projects/{}", encoded_project), body)
            .await
    }

    pub async fn list_group_projects(
        &self,
        group: &str,
        per_page: u32,
        include_archived: bool,
    ) -> Result<Value> {
        let encoded_group = urlencoding::encode(group);
        let archived_param = if include_archived {
            "&archived=true"
        } else {
            ""
        };
        self.get(&format!(
            "/groups/{}/projects?per_page={}{}",
            encoded_group, per_page, archived_param
        ))
        .await
    }
}
