use crate::opus::models::Task;
use chrono::{DateTime, Utc};
use reqwest::{Method, Response};
use serde::Deserialize;
use serde_json::{json, Value};
use std::error::Error;

type Res<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TaskPatch {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub project: Option<String>,
    pub start: Option<Option<DateTime<Utc>>>,
    pub due: Option<Option<DateTime<Utc>>>,
    pub assignee: Option<Option<String>>,
    pub add_labels: Vec<String>,
    pub remove_labels: Vec<String>,
}

impl TaskPatch {
    pub fn is_empty(&self) -> bool {
        !self.changes_fields()
            && self.project.is_none()
            && self.add_labels.is_empty()
            && self.remove_labels.is_empty()
    }

    fn changes_fields(&self) -> bool {
        self.title.is_some()
            || self.description.is_some()
            || self.status.is_some()
            || self.priority.is_some()
            || self.start.is_some()
            || self.due.is_some()
            || self.assignee.is_some()
    }
}

#[derive(Deserialize)]
struct MoveResponse {
    task: Task,
}

pub fn merge_update(current: &Task, patch: &TaskPatch) -> Value {
    json!({
        "title": patch.title.clone().unwrap_or_else(|| current.title.clone()),
        "description": patch
            .description
            .clone()
            .or_else(|| current.description.clone())
            .unwrap_or_default(),
        "status": patch.status.clone().unwrap_or_else(|| current.status.clone()),
        "priority": patch
            .priority
            .clone()
            .unwrap_or_else(|| current.priority.to_string()),
        "projectId": current.project_id,
        "position": current.position,
        "startDate": patch.start.unwrap_or(current.start_date),
        "dueDate": patch.due.unwrap_or(current.due_date),
        "userId": patch
            .assignee
            .clone()
            .unwrap_or_else(|| current.user_id.clone()),
    })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opus::models::Priority;
    use chrono::TimeZone;

    fn current() -> Task {
        Task {
            id: "5".into(),
            project_id: "2".into(),
            title: "Old".into(),
            description: Some("Body".into()),
            status: "doing".into(),
            priority: Priority::High,
            position: 3,
            user_id: Some("9".into()),
            start_date: Some(Utc.with_ymd_and_hms(2026, 1, 2, 0, 0, 0).unwrap()),
            due_date: Some(Utc.with_ymd_and_hms(2026, 2, 3, 0, 0, 0).unwrap()),
            ..Task::default()
        }
    }

    #[test]
    fn an_empty_patch_keeps_every_field() {
        let payload = merge_update(&current(), &TaskPatch::default());
        assert_eq!(payload["title"], "Old");
        assert_eq!(payload["description"], "Body");
        assert_eq!(payload["status"], "doing");
        assert_eq!(payload["priority"], "high");
        assert_eq!(payload["projectId"], "2");
        assert_eq!(payload["position"], 3);
        assert_eq!(payload["userId"], "9");
        assert_eq!(payload["startDate"], "2026-01-02T00:00:00Z");
        assert_eq!(payload["dueDate"], "2026-02-03T00:00:00Z");
    }

    #[test]
    fn set_fields_replace_and_the_rest_stay() {
        let patch = TaskPatch {
            title: Some("New".into()),
            priority: Some("low".into()),
            assignee: Some(Some("4".into())),
            ..TaskPatch::default()
        };
        let payload = merge_update(&current(), &patch);
        assert_eq!(payload["title"], "New");
        assert_eq!(payload["priority"], "low");
        assert_eq!(payload["userId"], "4");
        assert_eq!(payload["description"], "Body");
        assert_eq!(payload["dueDate"], "2026-02-03T00:00:00Z");
    }

    #[test]
    fn cleared_fields_become_null_and_empty_description_sticks() {
        let patch = TaskPatch {
            description: Some(String::new()),
            start: Some(None),
            due: Some(None),
            assignee: Some(None),
            ..TaskPatch::default()
        };
        let payload = merge_update(&current(), &patch);
        assert_eq!(payload["description"], "");
        assert!(payload["startDate"].is_null());
        assert!(payload["dueDate"].is_null());
        assert!(payload["userId"].is_null());
    }

    #[test]
    fn emptiness_ignores_nothing_that_writes() {
        assert!(TaskPatch::default().is_empty());
        let labels = TaskPatch {
            remove_labels: vec!["x".into()],
            ..TaskPatch::default()
        };
        assert!(!labels.is_empty());
        let project = TaskPatch {
            project: Some("p".into()),
            ..TaskPatch::default()
        };
        assert!(!project.is_empty());
    }
}
