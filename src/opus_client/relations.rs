use serde::{Deserialize, Serialize};
use crate::opus::models::TaskRelation;
use crate::debug::debug_log;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum RelationType {
    Subtask,
    Parenttask,
    Related,
    Blocking,
    Blocked,
    Duplicate,
}

impl RelationType {
    pub fn display_name(&self) -> &'static str {
        match self {
            RelationType::Subtask => "Subtask of",
            RelationType::Parenttask => "Parent of",
            RelationType::Related => "Related to",
            RelationType::Blocking => "Blocking",
            RelationType::Blocked => "Blocked by",
            RelationType::Duplicate => "Duplicate of",
        }
    }

    pub fn is_blocking_relation(&self) -> bool {
        matches!(self, RelationType::Blocked | RelationType::Blocking)
    }

    pub fn reverse(&self) -> RelationType {
        match self {
            RelationType::Subtask => RelationType::Parenttask,
            RelationType::Parenttask => RelationType::Subtask,
            RelationType::Related => RelationType::Related,
            RelationType::Blocking => RelationType::Blocked,
            RelationType::Blocked => RelationType::Blocking,
            RelationType::Duplicate => RelationType::Duplicate,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            RelationType::Subtask => "subtask",
            RelationType::Parenttask => "parenttask",
            RelationType::Related => "related",
            RelationType::Blocking => "blocking",
            RelationType::Blocked => "blocked",
            RelationType::Duplicate => "duplicate",
        }
    }
}

impl std::fmt::Display for RelationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl super::OpusClient {
    pub async fn get_task_relations(
        &self,
        task_id: &str,
    ) -> Result<Vec<TaskRelation>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/task-relation/{}", self.base_url, task_id);
        debug_log(&format!("Fetching relations for task {}", task_id));

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            let relations: Vec<TaskRelation> = response.json().await?;
            debug_log(&format!(
                "Got {} relations for task {}",
                relations.len(),
                task_id
            ));
            Ok(relations)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!(
                "Failed to get task relations: {} - {}",
                status, error_text
            )
            .into())
        }
    }

    pub async fn create_task_relation(
        &self,
        source_task_id: &str,
        target_task_id: &str,
        relation_type: &str,
    ) -> Result<TaskRelation, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/task-relation/", self.base_url);
        let payload = serde_json::json!({
            "sourceTaskId": source_task_id,
            "targetTaskId": target_task_id,
            "relationType": relation_type,
        });

        debug_log(&format!(
            "Creating relation: {} -> {} ({})",
            source_task_id, target_task_id, relation_type
        ));

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .json(&payload)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            let relation: TaskRelation = response.json().await?;
            debug_log(&format!("Created relation: {}", relation.id));
            Ok(relation)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to create relation: {} - {}", status, error_text).into())
        }
    }

    pub async fn delete_task_relation(
        &self,
        relation_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/task-relation/{}", self.base_url, relation_id);

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            debug_log(&format!("Deleted relation: {}", relation_id));
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to delete relation: {} - {}", status, error_text).into())
        }
    }
}
