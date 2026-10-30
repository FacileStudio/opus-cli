use super::task_patch::{merge_update, TaskPatch};
use crate::opus::models::Task;
use reqwest::{Method, Response};
use serde::Deserialize;
use serde_json::{json, Value};
use std::error::Error;

type Res<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Deserialize)]
struct MoveResponse {
    task: Task,
}

impl super::OpusClient {
    async fn send_json(&self, method: Method, path: &str, body: Option<Value>) -> Res<Response> {
        let url = format!("{}/api/{}", self.base_url, path);
        let mut request = self
            .client
            .request(method.clone(), &url)
            .header("Authorization", format!("Bearer {}", self.auth_token));
        if let Some(body) = body {
            request = request.json(&body);
        }
        let response = request.send().await?;
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }
        let text = response.text().await.unwrap_or_default();
        Err(format!("{} {} failed: {} - {}", method, path, status, text).into())
    }

    pub async fn resolve_project_id(&self, wanted: &str) -> Res<String> {
        let projects = self.get_all_projects().await?;
        projects
            .iter()
            .find(|p| p.id == wanted || p.name.eq_ignore_ascii_case(wanted))
            .map(|p| p.id.clone())
            .ok_or_else(|| format!("project '{}' not found", wanted).into())
    }

    pub async fn resolve_user_id(&self, wanted: &str) -> Res<String> {
        if !wanted.is_empty() && wanted.chars().all(|c| c.is_ascii_digit()) {
            return Ok(wanted.to_string());
        }
        self.find_user_by_name(wanted)
            .await
            .map(|u| u.id)
            .ok_or_else(|| format!("user '{}' not found", wanted).into())
    }

    pub async fn move_task(
        &self,
        task_id: &str,
        project_id: &str,
        status: Option<&str>,
    ) -> Res<Task> {
        let mut body = json!({ "destinationProjectId": project_id });
        if let (Some(status), Value::Object(map)) = (status, &mut body) {
            map.insert("destinationStatus".to_string(), json!(status));
        }
        let path = format!("task/move/{}", task_id);
        let moved: MoveResponse = self
            .send_json(Method::PUT, &path, Some(body))
            .await?
            .json()
            .await?;
        Ok(moved.task)
    }

    async fn add_task_label(&self, task_id: &str, name: &str) -> Res<()> {
        let workspace = self.get_all_labels().await?;
        let color = workspace
            .iter()
            .find(|l| l.name.eq_ignore_ascii_case(name) && !l.color.is_empty())
            .map(|l| l.color.clone())
            .unwrap_or_else(|| "#808080".to_string());
        let body = json!({
            "name": name,
            "color": color,
            "workspaceId": self.workspace_id,
            "taskId": task_id,
        });
        self.send_json(Method::POST, "label/", Some(body)).await?;
        Ok(())
    }

    async fn remove_task_label(&self, task_id: &str, name: &str) -> Res<()> {
        let labels = self.get_task_labels(task_id).await?;
        let label = labels
            .iter()
            .find(|l| l.name.eq_ignore_ascii_case(name))
            .ok_or_else(|| format!("label '{}' is not on task {}", name, task_id))?;
        let path = format!("label/{}", label.id);
        self.send_json(Method::DELETE, &path, None).await?;
        Ok(())
    }

    pub async fn patch_task(&self, task_id: &str, patch: &TaskPatch) -> Res<Task> {
        let mut patch = patch.clone();
        let project_id = match &patch.project {
            Some(wanted) => Some(self.resolve_project_id(wanted).await?),
            None => None,
        };
        if let Some(Some(wanted)) = &patch.assignee {
            patch.assignee = Some(Some(self.resolve_user_id(wanted).await?));
        }

        let mut current = self.get_task(task_id).await?;
        if let Some(project_id) = project_id.filter(|id| *id != current.project_id) {
            current = self
                .move_task(task_id, &project_id, patch.status.as_deref())
                .await?;
        }
        if patch.changes_fields() {
            let path = format!("task/{}", task_id);
            self.send_json(Method::PUT, &path, Some(merge_update(&current, &patch)))
                .await?;
        }
        for name in super::tasks::unique_names(&patch.add_labels) {
            self.add_task_label(task_id, &name).await?;
        }
        for name in super::tasks::unique_names(&patch.remove_labels) {
            self.remove_task_label(task_id, &name).await?;
        }
        self.get_task(task_id).await
    }
}
