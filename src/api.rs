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

    async fn put(&self, path: &str, body: &Value) -> Result<Value> {
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
        self.list_pipelines_for_branch(None, per_page).await
    }

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

    pub async fn get_project(&self) -> Result<Value> {
        self.get(&format!("/projects/{}", self.encoded_project()))
            .await
    }

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

    pub async fn get_issue(&self, iid: u64) -> Result<Value> {
        self.get(&format!(
            "/projects/{}/issues/{}",
            self.encoded_project(),
            iid
        ))
        .await
    }

    async fn post(&self, path: &str, body: &Value) -> Result<Value> {
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

    async fn post_empty(&self, path: &str) -> Result<()> {
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

    // Group operations (don't require project)
    pub async fn list_group_members(
        &self,
        group: &str,
        per_page: u32,
        show_email: bool,
    ) -> Result<Value> {
        let encoded_group = urlencoding::encode(group);
        if show_email {
            // Use billable_members endpoint which includes emails for group owners
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

    // Project operations
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

    // Push mirror operations
    pub async fn create_push_mirror(
        &self,
        project: &str,
        url: &str,
        enabled: bool,
        only_protected: bool,
    ) -> Result<Value> {
        let encoded_project = urlencoding::encode(project);

        // Ensure SSH URLs have password placeholder for proper auth_method detection
        // ssh://git@host -> ssh://git:x@host
        let mirror_url = if url.starts_with("ssh://git@") {
            url.replacen("ssh://git@", "ssh://git:x@", 1)
        } else {
            url.to_string()
        };

        self.post(
            &format!("/projects/{}/remote_mirrors", encoded_project),
            &serde_json::json!({
                "url": mirror_url,
                "enabled": enabled,
                "only_protected_branches": only_protected,
                "auth_method": "ssh_public_key"
            }),
        )
        .await
    }

    pub async fn create_push_mirror_https(
        &self,
        project: &str,
        url: &str,
        user: &str,
        password: &str,
        only_protected: bool,
    ) -> Result<Value> {
        let encoded_project = urlencoding::encode(project);

        // Build URL with credentials: https://user:password@host/path
        let mirror_url = if url.starts_with("https://") {
            let rest = url.strip_prefix("https://").unwrap();
            let encoded_user = urlencoding::encode(user);
            let encoded_password = urlencoding::encode(password);
            format!("https://{}:{}@{}", encoded_user, encoded_password, rest)
        } else {
            url.to_string()
        };

        self.post(
            &format!("/projects/{}/remote_mirrors", encoded_project),
            &serde_json::json!({
                "url": mirror_url,
                "enabled": true,
                "only_protected_branches": only_protected,
                "auth_method": "password"
            }),
        )
        .await
    }

    pub async fn get_push_mirror(&self, project: &str, mirror_id: u64) -> Result<Value> {
        let encoded_project = urlencoding::encode(project);
        self.get(&format!(
            "/projects/{}/remote_mirrors/{}",
            encoded_project, mirror_id
        ))
        .await
    }

    pub async fn get_push_mirror_public_key(
        &self,
        project: &str,
        mirror_id: u64,
    ) -> Result<String> {
        let encoded_project = urlencoding::encode(project);
        let result = self
            .get(&format!(
                "/projects/{}/remote_mirrors/{}/public_key",
                encoded_project, mirror_id
            ))
            .await?;
        Ok(result["public_key"].as_str().unwrap_or("").to_string())
    }

    pub async fn list_push_mirrors(&self, project: &str) -> Result<Value> {
        let encoded_project = urlencoding::encode(project);
        self.get(&format!("/projects/{}/remote_mirrors", encoded_project))
            .await
    }

    pub async fn delete_push_mirror(&self, project: &str, mirror_id: u64) -> Result<()> {
        let encoded_project = urlencoding::encode(project);
        self.delete(&format!(
            "/projects/{}/remote_mirrors/{}",
            encoded_project, mirror_id
        ))
        .await
    }

    pub async fn sync_push_mirror(&self, project: &str, mirror_id: u64) -> Result<()> {
        let encoded_project = urlencoding::encode(project);
        self.post_empty(&format!(
            "/projects/{}/remote_mirrors/{}/sync",
            encoded_project, mirror_id
        ))
        .await
    }

    async fn delete(&self, path: &str) -> Result<()> {
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

    // Protected branch operations
    pub async fn list_protected_branches(&self) -> Result<Value> {
        self.get(&format!(
            "/projects/{}/protected_branches",
            self.encoded_project()
        ))
        .await
    }

    pub async fn protect_branch(&self, branch: &str, allow_force_push: bool) -> Result<Value> {
        self.post(
            &format!("/projects/{}/protected_branches", self.encoded_project()),
            &serde_json::json!({
                "name": branch,
                "allow_force_push": allow_force_push
            }),
        )
        .await
    }

    pub async fn unprotect_branch(&self, branch: &str) -> Result<()> {
        let encoded_branch = urlencoding::encode(branch);
        self.delete(&format!(
            "/projects/{}/protected_branches/{}",
            self.encoded_project(),
            encoded_branch
        ))
        .await
    }

    // Webhook operations
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
