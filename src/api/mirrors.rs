use anyhow::Result;
use serde_json::Value;

use super::Client;

impl Client {
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
        let mirror_url = build_https_mirror_url(url, user, password);

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
}

fn build_https_mirror_url(url: &str, user: &str, password: &str) -> String {
    if url.starts_with("https://") {
        let rest = url.strip_prefix("https://").unwrap();
        let encoded_user = urlencoding::encode(user);
        let encoded_password = urlencoding::encode(password);
        format!("https://{}:{}@{}", encoded_user, encoded_password, rest)
    } else {
        url.to_string()
    }
}
