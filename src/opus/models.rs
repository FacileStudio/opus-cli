use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Datelike, Utc};
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

fn deserialize_optional_datetime<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(s) if s.is_empty() => Ok(None),
        Some(s) => match DateTime::parse_from_rfc3339(&s) {
            Ok(dt) => {
                if dt.year() <= 1900 {
                    Ok(None)
                } else {
                    Ok(Some(dt.with_timezone(&Utc)))
                }
            }
            Err(_) => Ok(None),
        },
        None => Ok(None),
    }
}

fn deserialize_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::String(s) => Ok(s),
        serde_json::Value::Number(n) => Ok(n.to_string()),
        other => Err(D::Error::custom(format!(
            "expected an id as string or number, got {other}"
        ))),
    }
}

fn deserialize_optional_id<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::String(s) if s.is_empty() => Ok(None),
        serde_json::Value::String(s) => Ok(Some(s)),
        serde_json::Value::Number(n) => Ok(Some(n.to_string())),
        other => Err(D::Error::custom(format!(
            "expected an id as string or number, got {other}"
        ))),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Priority {
    #[serde(rename = "no-priority")]
    NoPriority,
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "high")]
    High,
    #[serde(rename = "urgent")]
    Urgent,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::NoPriority
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Priority::NoPriority => "no-priority",
            Priority::Low => "low",
            Priority::Medium => "medium",
            Priority::High => "high",
            Priority::Urgent => "urgent",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for Priority {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "no-priority" => Ok(Priority::NoPriority),
            "low" => Ok(Priority::Low),
            "medium" => Ok(Priority::Medium),
            "high" => Ok(Priority::High),
            "urgent" => Ok(Priority::Urgent),
            other => Err(format!("unknown priority: {}", other)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub name: String,
    pub slug: String,
    pub logo: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub workspace_id: String,
    pub name: String,
    pub slug: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub is_public: bool,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_datetime",
        skip_serializing_if = "Option::is_none"
    )]
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl Project {
    pub fn is_archived(&self) -> bool {
        self.archived_at.is_some()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Column {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub project_id: String,
    pub name: String,
    pub slug: String,
    pub position: i32,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub is_final: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Label {
    #[serde(default, deserialize_with = "deserialize_id")]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub color: String,
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub task_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub workspace_id: Option<String>,
    #[serde(default = "default_now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "default_now")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(default, deserialize_with = "deserialize_id")]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub email: String,
    pub image: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub task_id: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub user_id: String,
    pub content: String,
    pub user: Option<User>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub task_id: String,
    #[serde(rename = "type")]
    pub activity_type: String,
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub user_id: Option<String>,
    pub content: Option<String>,
    pub event_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TaskRelation {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub source_task_id: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub target_task_id: String,
    pub relation_type: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub project_id: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub column_id: Option<String>,
    #[serde(default)]
    pub priority: Priority,
    #[serde(default)]
    pub position: i32,
    #[serde(default)]
    pub number: i32,
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub user_id: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_datetime",
        skip_serializing_if = "Option::is_none"
    )]
    pub start_date: Option<DateTime<Utc>>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_datetime",
        skip_serializing_if = "Option::is_none"
    )]
    pub due_date: Option<DateTime<Utc>>,
    #[serde(
        default = "default_now",
        deserialize_with = "deserialize_datetime_or_default"
    )]
    pub created_at: DateTime<Utc>,
    #[serde(
        default = "default_now",
        deserialize_with = "deserialize_datetime_or_default"
    )]
    pub updated_at: DateTime<Utc>,

    #[serde(default)]
    pub labels: Option<Vec<Label>>,
    #[serde(default)]
    pub done: bool,
    #[serde(default)]
    pub assignees: Option<Vec<User>>,
    #[serde(default)]
    pub created: Option<String>,
    #[serde(default)]
    pub updated: Option<String>,
    #[serde(default)]
    pub hex_color: Option<String>,
    #[serde(default)]
    pub comments: Option<Vec<Comment>>,
    #[serde(default)]
    pub related_tasks: Option<std::collections::HashMap<String, Vec<Task>>>,
}

fn default_status() -> String {
    "to-do".to_string()
}

fn default_now() -> DateTime<Utc> {
    Utc::now()
}

fn deserialize_datetime_or_default<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(s) if !s.is_empty() => match DateTime::parse_from_rfc3339(&s) {
            Ok(dt) => Ok(dt.with_timezone(&Utc)),
            Err(_) => Ok(Utc::now()),
        },
        _ => Ok(Utc::now()),
    }
}

impl Task {
    pub fn done(&self, columns: &[Column]) -> bool {
        columns.iter().any(|c| c.slug == self.status && c.is_final)
    }

    pub fn is_assigned(&self) -> bool {
        self.user_id.is_some()
    }

    pub fn is_overdue(&self) -> bool {
        self.due_date.map(|d| d < Utc::now()).unwrap_or(false)
    }
}

impl Default for Task {
    fn default() -> Self {
        let now = Utc::now();
        Task {
            id: String::new(),
            project_id: String::new(),
            title: String::new(),
            description: None,
            status: String::from("to-do"),
            column_id: None,
            priority: Priority::default(),
            position: 0,
            number: 0,
            user_id: None,
            start_date: None,
            due_date: None,
            created_at: now,
            updated_at: now,
            labels: None,
            done: false,
            assignees: None,
            created: None,
            updated: None,
            hex_color: None,
            comments: None,
            related_tasks: None,
        }
    }
}

/// One personal API token, which the backend models as a labelled porte
/// session. `token` is set only on the create answer: the server stores a
/// hash and can never show the plaintext again.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ApiToken {
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub last_used_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CreateTokenRequest {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ListTokensResponse {
    #[serde(default)]
    pub tokens: Vec<ApiToken>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_numeric_ids_as_strings() {
        let task: Task = serde_json::from_str(
            r#"{"id":7,"projectId":3,"title":"Ship it","status":"to-do","priority":"high","position":2,"number":5,"createdAt":"2026-10-13T00:00:00Z","updatedAt":"2026-10-13T00:00:00Z"}"#,
        )
        .unwrap();
        assert_eq!(task.id, "7");
        assert_eq!(task.project_id, "3");
        assert_eq!(task.priority, Priority::High);
    }

    #[test]
    fn deserializes_optional_ids_from_number_string_and_null() {
        let task: Task = serde_json::from_str(
            r#"{"id":1,"projectId":2,"title":"t","columnId":null,"userId":"9","createdAt":"2026-10-13T00:00:00Z","updatedAt":"2026-10-13T00:00:00Z"}"#,
        )
        .unwrap();
        assert_eq!(task.column_id, None);
        assert_eq!(task.user_id.as_deref(), Some("9"));

        let workspace: Workspace = serde_json::from_str(
            r#"{"id":42,"name":"Team","slug":"team","logo":null,"description":null,"createdAt":"2026-10-13T00:00:00Z"}"#,
        )
        .unwrap();
        assert_eq!(workspace.id, "42");
    }

    #[test]
    fn comment_author_without_id_or_email_still_parses() {
        let comment: Comment = serde_json::from_str(
            r#"{"id":4,"taskId":9,"userId":2,"content":"hi","createdAt":"2026-10-13T00:00:00Z","updatedAt":"2026-10-13T00:00:00Z","user":{"name":"Ann","image":""}}"#,
        )
        .unwrap();
        assert_eq!(comment.id, "4");
        assert_eq!(comment.user.map(|u| u.name), Some("Ann".to_string()));
    }
}
