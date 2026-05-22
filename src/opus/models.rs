use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Datelike, Utc};
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
    pub id: String,
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
    pub id: String,
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
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub color: String,
    #[serde(default)]
    pub task_id: Option<String>,
    #[serde(default)]
    pub workspace_id: Option<String>,
    #[serde(default = "default_now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "default_now")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub image: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub id: String,
    pub task_id: String,
    pub user_id: String,
    pub content: String,
    pub user: Option<User>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    pub id: String,
    pub task_id: String,
    #[serde(rename = "type")]
    pub activity_type: String,
    pub user_id: Option<String>,
    pub content: Option<String>,
    pub event_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TaskRelation {
    pub id: String,
    pub source_task_id: String,
    pub target_task_id: String,
    pub relation_type: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default)]
    pub column_id: Option<String>,
    #[serde(default)]
    pub priority: Priority,
    #[serde(default)]
    pub position: i32,
    #[serde(default)]
    pub number: i32,
    #[serde(default)]
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
        columns
            .iter()
            .any(|c| c.slug == self.status && c.is_final)
    }

    pub fn is_assigned(&self) -> bool {
        self.user_id.is_some()
    }

    pub fn is_overdue(&self) -> bool {
        self.due_date
            .map(|d| d < Utc::now())
            .unwrap_or(false)
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id: String,
    pub task_id: Option<String>,
    pub filename: String,
    pub mime_type: Option<String>,
    pub size: Option<i64>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileAttachment {
    pub id: String,
    pub name: Option<String>,
    pub mime: Option<String>,
    pub size: Option<i64>,
}
