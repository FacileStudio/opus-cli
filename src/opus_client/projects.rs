use crate::debug::debug_log;
use crate::opus::models::Project;
use crate::tui::utils::{normalize_string, equals_ignore_case};

impl super::OpusClient {
    pub async fn get_all_projects(
        &self,
    ) -> Result<Vec<Project>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/project/", self.base_url);
        debug_log(&format!("Fetching all projects from: {}", url));

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            let projects: Vec<Project> = response.json().await?;
            let workspace_projects: Vec<Project> = projects
                .into_iter()
                .filter(|p| p.workspace_id == self.workspace_id)
                .collect();
            debug_log(&format!(
                "Got {} projects for workspace {}",
                workspace_projects.len(),
                self.workspace_id
            ));
            Ok(workspace_projects)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to fetch projects: {} - {}", status, error_text).into())
        }
    }

    pub async fn get_project(
        &self,
        project_id: &str,
    ) -> Result<Project, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/project/{}", self.base_url, project_id);
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
            Err(format!("Failed to get project {}: {} - {}", project_id, status, error_text).into())
        }
    }

    pub async fn create_project(
        &self,
        name: &str,
    ) -> Result<Project, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/project/", self.base_url);
        let payload = serde_json::json!({
            "name": name,
            "workspaceId": self.workspace_id,
        });
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .json(&payload)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            Ok(response.json().await?)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to create project: {} - {}", status, error_text).into())
        }
    }

    pub async fn find_project_by_name(
        &self,
        name: &str,
    ) -> Result<Option<Project>, Box<dyn std::error::Error + Send + Sync>> {
        let projects = self.get_all_projects().await?;
        Ok(projects
            .into_iter()
            .find(|p| equals_ignore_case(&p.name, name)))
    }

    pub async fn find_or_get_project_id(
        &self,
        project_name: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let normalized_input = normalize_string(project_name);
        debug_log(&format!(
            "Looking for project: '{}' (normalized: '{}')",
            project_name, normalized_input
        ));

        let projects = self.get_all_projects().await?;
        debug_log(&format!(
            "Available projects: {:?}",
            projects
                .iter()
                .map(|p| format!("{} (id={})", p.name, p.id))
                .collect::<Vec<_>>()
        ));

        Ok(projects
            .iter()
            .find(|p| equals_ignore_case(&p.name, project_name))
            .map(|p| p.id.clone()))
    }
}
