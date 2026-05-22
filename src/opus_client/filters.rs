use crate::debug::debug_log;
use crate::opus::models::Column;

impl super::OpusClient {
    pub async fn get_columns_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<Column>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/column/{}", self.base_url, project_id);
        debug_log(&format!(
            "Fetching columns for project {}: {}",
            project_id, url
        ));

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            let columns: Vec<Column> = response.json().await?;
            debug_log(&format!(
                "Got {} columns for project {}",
                columns.len(),
                project_id
            ));
            Ok(columns)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            debug_log(&format!(
                "Failed to fetch columns for project {}: {} - {}",
                project_id, status, error_text
            ));
            Err(format!(
                "Failed to fetch columns: {} - {}",
                status, error_text
            )
            .into())
        }
    }

    pub async fn create_column(
        &self,
        project_id: &str,
        name: &str,
    ) -> Result<Column, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/column/{}", self.base_url, project_id);
        let payload = serde_json::json!({ "name": name });

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
            Err(format!("Failed to create column: {} - {}", status, error_text).into())
        }
    }

    pub async fn update_column(
        &self,
        column_id: &str,
        update: &serde_json::Value,
    ) -> Result<Column, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/column/{}", self.base_url, column_id);

        let response = self
            .client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .json(update)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            Ok(response.json().await?)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to update column: {} - {}", status, error_text).into())
        }
    }

    pub async fn get_saved_filters(
        &self,
    ) -> Result<Vec<(String, String, Option<String>)>, Box<dyn std::error::Error + Send + Sync>>
    {
        debug_log("Fetching columns as saved filters for all projects...");

        let projects = self.get_all_projects().await?;
        let mut filters = Vec::new();

        for project in &projects {
            match self.get_columns_for_project(&project.id).await {
                Ok(columns) => {
                    for col in columns {
                        let description = col.color.clone();
                        filters.push((col.id.clone(), col.name.clone(), description));
                    }
                }
                Err(e) => {
                    debug_log(&format!(
                        "Failed to fetch columns for project '{}': {}",
                        project.name, e
                    ));
                }
            }
        }

        debug_log(&format!("Extracted {} column-filters total", filters.len()));
        Ok(filters)
    }
}
