use anyhow::Result;
use serde_json::Value;

use super::Client;

impl Client {
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
}
