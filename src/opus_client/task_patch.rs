use crate::opus::models::Task;
use chrono::{DateTime, Utc};
use serde_json::{json, Value};

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

    pub fn changes_fields(&self) -> bool {
        self.title.is_some()
            || self.description.is_some()
            || self.status.is_some()
            || self.priority.is_some()
            || self.start.is_some()
            || self.due.is_some()
            || self.assignee.is_some()
    }
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
