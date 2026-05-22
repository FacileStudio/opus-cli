use crate::debug::debug_log;
use crate::opus::models::User;

impl super::OpusClient {
    pub async fn get_workspace_members(
        &self,
    ) -> Result<Vec<User>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/api/workspace/{}/members",
            self.base_url, self.workspace_id
        );
        debug_log(&format!("Fetching workspace members from: {}", url));

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            let members: Vec<User> = response.json().await?;
            debug_log(&format!("Got {} workspace members", members.len()));
            Ok(members)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            debug_log(&format!(
                "Failed to fetch workspace members: {} - {}",
                status, error_text
            ));
            Err(format!("Failed to fetch workspace members: {} - {}", status, error_text).into())
        }
    }

    pub async fn find_user_by_name(&self, name: &str) -> Option<User> {
        match self.get_workspace_members().await {
            Ok(members) => members
                .into_iter()
                .find(|u| u.name.eq_ignore_ascii_case(name)),
            Err(e) => {
                debug_log(&format!("Failed to search for user '{}': {}", name, e));
                None
            }
        }
    }
}
