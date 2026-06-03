use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickAction {
    pub key: String,
    pub action: String,
    pub target: String,
}

impl QuickAction {
    pub fn get_description(&self) -> String {
        match self.action.as_str() {
            "project" => format!("Move to project: {}", self.target),
            "priority" => format!("Set priority to: {}", self.target),
            "label" => format!("Add label: {}", self.target),
            "status" => format!("Set status to: {}", self.target),
            "workspace" => format!("Switch workspace: {}", self.target),
            _ => format!("Unknown action: {} -> {}", self.action, self.target),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceEntry {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpusConfig {
    pub api_url: String,
    pub api_key: Option<String>,
    pub workspace_id: Option<String>,
    pub default_project: Option<String>,
    pub default_filter: Option<String>,
    pub quick_actions: Option<Vec<QuickAction>>,
    pub table_columns: Option<Vec<TableColumn>>,
    pub column_layouts: Option<Vec<ColumnLayout>>,
    pub active_layout: Option<String>,
    pub refresh_interval_seconds: Option<u64>,
    pub auto_refresh: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspaces: Option<Vec<WorkspaceEntry>>,
}

impl Default for OpusConfig {
    fn default() -> Self {
        OpusConfig {
            api_url: "http://localhost:1337".to_string(),
            api_key: None,
            workspace_id: None,
            default_project: None,
            default_filter: None,
            quick_actions: None,
            table_columns: None,
            column_layouts: None,
            active_layout: None,
            refresh_interval_seconds: Some(300),
            auto_refresh: Some(true),
            workspaces: None,
        }
    }
}

impl OpusConfig {
    pub fn get_workspaces(&self) -> Vec<WorkspaceEntry> {
        self.workspaces.clone().unwrap_or_default()
    }

    pub fn add_workspace(&mut self, id: String, name: String) {
        let workspaces = self.workspaces.get_or_insert_with(Vec::new);
        if !workspaces.iter().any(|w| w.id == id) {
            workspaces.push(WorkspaceEntry { id, name });
        }
    }

    pub fn remove_workspace(&mut self, id_or_name: &str) -> bool {
        if let Some(ref mut workspaces) = self.workspaces {
            let before = workspaces.len();
            workspaces.retain(|w| w.id != id_or_name && !w.name.eq_ignore_ascii_case(id_or_name));
            before != workspaces.len()
        } else {
            false
        }
    }

    pub fn find_workspace(&self, id_or_name: &str) -> Option<&WorkspaceEntry> {
        self.workspaces.as_ref().and_then(|ws| {
            ws.iter().find(|w| w.id == id_or_name || w.name.eq_ignore_ascii_case(id_or_name))
        })
    }

    pub fn ensure_current_workspace_in_list(&mut self) {
        if let Some(ref ws_id) = self.workspace_id {
            if !ws_id.is_empty() {
                let already = self.workspaces.as_ref()
                    .map(|ws| ws.iter().any(|w| w.id == *ws_id))
                    .unwrap_or(false);
                if !already {
                    self.add_workspace(ws_id.clone(), ws_id.clone());
                }
            }
        }
    }
}

impl OpusConfig {
    #[allow(dead_code)]
    pub fn load() -> Option<Self> {
        Self::load_from_path(None)
    }

    pub fn load_from_path(custom_path: Option<&str>) -> Option<Self> {
        let config_path = if let Some(custom_path) = custom_path {
            PathBuf::from(custom_path)
        } else {
            let mut home = dirs::home_dir()?;
            home.push(".opus.yml");
            home
        };

        let contents = fs::read_to_string(&config_path).ok()?;
        serde_yaml::from_str(&contents).ok()
    }

    pub fn save(&self) -> Result<(), String> {
        let config_path = dirs::home_dir()
            .map(|mut h| { h.push(".opus.yml"); h })
            .ok_or_else(|| "Cannot determine home directory".to_string())?;
        let yaml = serde_yaml::to_string(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        std::fs::write(&config_path, yaml)
            .map_err(|e| format!("Failed to write config to {}: {}", config_path.display(), e))
    }

    pub fn has_api_key_config(&self) -> bool {
        self.api_key
            .as_ref()
            .map(|k| !k.trim().is_empty())
            .unwrap_or(false)
    }

    pub fn get_api_key(&self) -> Result<String, String> {
        if let Some(ref key) = self.api_key {
            if !key.trim().is_empty() {
                return Ok(key.clone());
            }
        }
        Err("No API key found. Set 'api_key' in config.yaml or run device auth.".to_string())
    }

    #[allow(dead_code)]
    pub fn get_quick_actions_map(&self) -> HashMap<String, QuickAction> {
        self.quick_actions
            .as_ref()
            .map(|actions| {
                actions
                    .iter()
                    .map(|action| (action.key.clone(), action.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    #[allow(dead_code)]
    pub fn has_quick_action(&self, key: &str) -> bool {
        self.quick_actions
            .as_ref()
            .map(|actions| actions.iter().any(|action| action.key == key))
            .unwrap_or(false)
    }

    #[allow(dead_code)]
    pub fn get_quick_action(&self, key: &str) -> Option<&QuickAction> {
        self.quick_actions
            .as_ref()
            .and_then(|actions| actions.iter().find(|action| action.key == key))
    }

    #[allow(dead_code)]
    pub fn get_refresh_interval_seconds(&self) -> u64 {
        self.refresh_interval_seconds.unwrap_or(300)
    }

    #[allow(dead_code)]
    pub fn is_auto_refresh_enabled(&self) -> bool {
        self.auto_refresh.unwrap_or(true)
    }

    pub fn get_columns(&self) -> Vec<TableColumn> {
        self.table_columns
            .clone()
            .unwrap_or_else(|| TaskColumn::default_columns())
    }

    #[allow(dead_code)]
    pub fn get_table_columns(&self) -> Vec<TableColumn> {
        if let Some(layouts) = &self.column_layouts {
            let active_layout_name = self.active_layout.as_deref().unwrap_or("default");
            if let Some(layout) = layouts.iter().find(|l| l.name == active_layout_name) {
                return layout.columns.clone();
            }
            if let Some(first_layout) = layouts.first() {
                return first_layout.columns.clone();
            }
        }
        self.table_columns
            .clone()
            .unwrap_or_else(|| TaskColumn::default_columns())
    }

    pub fn get_column_layouts(&self) -> Vec<ColumnLayout> {
        self.column_layouts
            .clone()
            .unwrap_or_else(|| ColumnLayout::default_layouts())
    }

    pub fn get_active_layout_name(&self) -> String {
        self.active_layout
            .clone()
            .unwrap_or_else(|| "default".to_string())
    }

    pub fn next_layout(&self, current_layout: &str) -> String {
        let layouts = self.get_column_layouts();
        if let Some(current_index) = layouts.iter().position(|l| l.name == current_layout) {
            let next_index = (current_index + 1) % layouts.len();
            layouts[next_index].name.clone()
        } else {
            layouts
                .first()
                .map(|l| l.name.clone())
                .unwrap_or_else(|| "default".to_string())
        }
    }

    pub fn previous_layout(&self, current_layout: &str) -> String {
        let layouts = self.get_column_layouts();
        if let Some(current_index) = layouts.iter().position(|l| l.name == current_layout) {
            let prev_index = if current_index == 0 {
                layouts.len() - 1
            } else {
                current_index - 1
            };
            layouts[prev_index].name.clone()
        } else {
            layouts
                .first()
                .map(|l| l.name.clone())
                .unwrap_or_else(|| "default".to_string())
        }
    }

    pub fn get_layout(&self, name: &str) -> Option<ColumnLayout> {
        self.get_column_layouts()
            .into_iter()
            .find(|l| l.name == name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumn {
    pub name: String,
    pub column_type: TaskColumn,
    #[serde(default)]
    pub width_percentage: Option<u16>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub min_width: Option<u16>,
    #[serde(default)]
    pub max_width: Option<u16>,
    #[serde(default)]
    pub wrap_text: Option<bool>,
    #[serde(default)]
    pub sort: Option<ColumnSort>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskColumn {
    Title,
    Project,
    Labels,
    DueDate,
    StartDate,
    Priority,
    Status,
    Assignees,
    Created,
    Updated,
}

impl TaskColumn {
    pub fn default_columns() -> Vec<TableColumn> {
        vec![
            TableColumn {
                name: "Title".to_string(),
                column_type: TaskColumn::Title,
                width_percentage: None,
                enabled: true,
                min_width: Some(20),
                max_width: None,
                wrap_text: Some(true),
                sort: None,
            },
            TableColumn {
                name: "Project".to_string(),
                column_type: TaskColumn::Project,
                width_percentage: None,
                enabled: true,
                min_width: Some(10),
                max_width: Some(20),
                wrap_text: Some(false),
                sort: None,
            },
            TableColumn {
                name: "Due Date".to_string(),
                column_type: TaskColumn::DueDate,
                width_percentage: None,
                enabled: true,
                min_width: Some(10),
                max_width: Some(12),
                wrap_text: Some(false),
                sort: None,
            },
            TableColumn {
                name: "Start Date".to_string(),
                column_type: TaskColumn::StartDate,
                width_percentage: None,
                enabled: true,
                min_width: Some(10),
                max_width: Some(12),
                wrap_text: Some(false),
                sort: None,
            },
            TableColumn {
                name: "Labels".to_string(),
                column_type: TaskColumn::Labels,
                width_percentage: None,
                enabled: true,
                min_width: Some(8),
                max_width: Some(25),
                wrap_text: Some(true),
                sort: None,
            },
        ]
    }

    #[allow(dead_code)]
    pub fn get_display_name(&self) -> &'static str {
        match self {
            TaskColumn::Title => "Title",
            TaskColumn::Project => "Project",
            TaskColumn::Labels => "Labels",
            TaskColumn::DueDate => "Due Date",
            TaskColumn::StartDate => "Start Date",
            TaskColumn::Priority => "Priority",
            TaskColumn::Status => "Status",
            TaskColumn::Assignees => "Assignees",
            TaskColumn::Created => "Created",
            TaskColumn::Updated => "Updated",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnLayout {
    pub name: String,
    pub description: Option<String>,
    pub columns: Vec<TableColumn>,
}

impl ColumnLayout {
    pub fn default_layouts() -> Vec<ColumnLayout> {
        vec![
            ColumnLayout {
                name: "default".to_string(),
                description: Some("Standard task view with all essential columns".to_string()),
                columns: TaskColumn::default_columns(),
            },
            ColumnLayout {
                name: "minimal".to_string(),
                description: Some("Clean, minimal view with just task and due date".to_string()),
                columns: vec![
                    TableColumn {
                        name: "Task".to_string(),
                        column_type: TaskColumn::Title,
                        width_percentage: None,
                        enabled: true,
                        min_width: Some(30),
                        max_width: None,
                        wrap_text: Some(true),
                        sort: None,
                    },
                    TableColumn {
                        name: "Due".to_string(),
                        column_type: TaskColumn::DueDate,
                        width_percentage: None,
                        enabled: true,
                        min_width: Some(10),
                        max_width: Some(12),
                        wrap_text: Some(false),
                        sort: None,
                    },
                    TableColumn {
                        name: "Project".to_string(),
                        column_type: TaskColumn::Project,
                        width_percentage: None,
                        enabled: true,
                        min_width: Some(10),
                        max_width: Some(15),
                        wrap_text: Some(false),
                        sort: None,
                    },
                ],
            },
            ColumnLayout {
                name: "project-focused".to_string(),
                description: Some("Project-centric view for team collaboration".to_string()),
                columns: vec![
                    TableColumn {
                        name: "Project".to_string(),
                        column_type: TaskColumn::Project,
                        width_percentage: None,
                        enabled: true,
                        min_width: Some(12),
                        max_width: Some(20),
                        wrap_text: Some(false),
                        sort: None,
                    },
                    TableColumn {
                        name: "Task".to_string(),
                        column_type: TaskColumn::Title,
                        width_percentage: None,
                        enabled: true,
                        min_width: Some(25),
                        max_width: None,
                        wrap_text: Some(true),
                        sort: None,
                    },
                    TableColumn {
                        name: "Priority".to_string(),
                        column_type: TaskColumn::Priority,
                        width_percentage: None,
                        enabled: true,
                        min_width: Some(8),
                        max_width: Some(10),
                        wrap_text: Some(false),
                        sort: None,
                    },
                    TableColumn {
                        name: "Due".to_string(),
                        column_type: TaskColumn::DueDate,
                        width_percentage: None,
                        enabled: true,
                        min_width: Some(10),
                        max_width: Some(12),
                        wrap_text: Some(false),
                        sort: None,
                    },
                    TableColumn {
                        name: "Labels".to_string(),
                        column_type: TaskColumn::Labels,
                        width_percentage: None,
                        enabled: true,
                        min_width: Some(8),
                        max_width: Some(20),
                        wrap_text: Some(true),
                        sort: None,
                    },
                ],
            },
            ColumnLayout {
                name: "time-management".to_string(),
                description: Some("Time-focused view for scheduling and deadlines".to_string()),
                columns: vec![
                    TableColumn {
                        name: "Task".to_string(),
                        column_type: TaskColumn::Title,
                        width_percentage: None,
                        enabled: true,
                        min_width: Some(25),
                        max_width: None,
                        wrap_text: Some(true),
                        sort: None,
                    },
                    TableColumn {
                        name: "Start".to_string(),
                        column_type: TaskColumn::StartDate,
                        width_percentage: None,
                        enabled: true,
                        min_width: Some(10),
                        max_width: Some(12),
                        wrap_text: Some(false),
                        sort: None,
                    },
                    TableColumn {
                        name: "Due".to_string(),
                        column_type: TaskColumn::DueDate,
                        width_percentage: None,
                        enabled: true,
                        min_width: Some(10),
                        max_width: Some(12),
                        wrap_text: Some(false),
                        sort: None,
                    },
                    TableColumn {
                        name: "Created".to_string(),
                        column_type: TaskColumn::Created,
                        width_percentage: None,
                        enabled: true,
                        min_width: Some(10),
                        max_width: Some(12),
                        wrap_text: Some(false),
                        sort: None,
                    },
                    TableColumn {
                        name: "Project".to_string(),
                        column_type: TaskColumn::Project,
                        width_percentage: None,
                        enabled: true,
                        min_width: Some(10),
                        max_width: Some(15),
                        wrap_text: Some(false),
                        sort: None,
                    },
                ],
            },
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnSort {
    pub order: u16,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    Asc,
    Desc,
}
