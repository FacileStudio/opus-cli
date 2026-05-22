use crate::debug::debug_log;
use crate::opus::models::Label;

impl super::OpusClient {
    pub async fn get_all_labels(
        &self,
    ) -> Result<Vec<Label>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/api/label/workspace/{}",
            self.base_url, self.workspace_id
        );
        debug_log(&format!("Fetching workspace labels from: {}", url));

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            let labels: Vec<Label> = response.json().await?;
            debug_log(&format!("Got {} labels", labels.len()));
            Ok(labels)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            debug_log(&format!("Failed to fetch labels: {} - {}", status, error_text));
            Err(format!("Failed to fetch labels: {} - {}", status, error_text).into())
        }
    }

    pub async fn get_task_labels(
        &self,
        task_id: &str,
    ) -> Result<Vec<Label>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/label/task/{}", self.base_url, task_id);
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            Ok(response.json().await?)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to get task labels: {} - {}", status, error_text).into())
        }
    }

    pub async fn create_label(
        &self,
        name: &str,
        color: &str,
    ) -> Result<Label, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/label/", self.base_url);
        let payload = serde_json::json!({
            "name": name,
            "color": color,
            "workspaceId": self.workspace_id,
        });

        debug_log(&format!("Creating label '{}' with color '{}'", name, color));

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .json(&payload)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            let label: Label = response.json().await?;
            debug_log(&format!("Created label: {} (id={})", label.name, label.id));
            Ok(label)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to create label: {} - {}", status, error_text).into())
        }
    }

    pub async fn attach_label(
        &self,
        label_id: &str,
        task_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/label/{}/task", self.base_url, label_id);
        let payload = serde_json::json!({ "taskId": task_id });

        let response = self
            .client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .json(&payload)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to attach label: {} - {}", status, error_text).into())
        }
    }

    pub async fn detach_label(
        &self,
        label_id: &str,
        task_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/label/{}/task", self.base_url, label_id);
        let payload = serde_json::json!({ "taskId": task_id });

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .json(&payload)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to detach label: {} - {}", status, error_text).into())
        }
    }
}
