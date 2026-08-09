use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Everything the CLI needs to talk to an Opus instance, resolved once from
/// every source that is allowed to supply it.
#[derive(Debug, Clone)]
pub struct Settings {
    pub api_url: String,
    pub api_key: String,
    pub workspace_id: String,
    pub default_project: String,
    pub config: Option<OpusConfig>,
}

impl Settings {
    /// Resolves the effective settings with the suite precedence:
    /// flag > environment variable > config file > built-in default.
    ///
    /// `dev_env` no longer means "ignore the config file" — the environment is
    /// an override everywhere. It only relaxes the requirement for a config
    /// file and supplies the throwaway localhost defaults.
    pub fn resolve(
        config_path: Option<&str>,
        workspace_override: Option<&str>,
        dev_env: bool,
    ) -> Result<Self, String> {
        let config = OpusConfig::load_from_path(config_path);

        if config.is_none() && !dev_env && env_value("OPUS_API_KEY").is_none() {
            return Err(match config_path {
                Some(path) => format!("config file not found at {}", path),
                None => "no config found — create ~/.opus.yml, or set OPUS_API_KEY".to_string(),
            });
        }

        let api_key = env_value("OPUS_API_KEY")
            .or_else(|| {
                config
                    .as_ref()
                    .and_then(|cfg| cfg.api_key.clone())
                    .filter(|key| !key.trim().is_empty())
            })
            .or_else(|| dev_env.then(|| "demo-token".to_string()))
            .ok_or_else(|| {
                "no API key configured — generate one at Settings > Account > Developer, \
                 or set OPUS_API_KEY"
                    .to_string()
            })?;

        let api_url = env_value("OPUS_API_URL")
            .or_else(|| config.as_ref().map(|cfg| cfg.api_url.clone()))
            .unwrap_or_else(|| "http://localhost:1337".to_string());

        let workspace_id = workspace_override
            .map(str::to_string)
            .or_else(|| env_value("OPUS_WORKSPACE_ID"))
            .or_else(|| config.as_ref().and_then(|cfg| cfg.workspace_id.clone()))
            .unwrap_or_default();

        let default_project = env_value("OPUS_DEFAULT_PROJECT")
            .or_else(|| config.as_ref().and_then(|cfg| cfg.default_project.clone()))
            .unwrap_or_else(|| "Inbox".to_string());

        Ok(Settings {
            api_url,
            api_key,
            workspace_id,
            default_project,
            config,
        })
    }
}

fn env_value(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

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
        }
    }
}

impl OpusConfig {
    #[allow(dead_code)]
    pub fn load() -> Option<Self> {
        Self::load_from_path(None)
    }

    pub fn default_path() -> Option<PathBuf> {
        let mut home = dirs::home_dir()?;
        home.push(".opus.yml");
        Some(home)
    }

    pub fn load_from_path(custom_path: Option<&str>) -> Option<Self> {
        let config_path = match custom_path {
            Some(custom_path) => PathBuf::from(custom_path),
            None => Self::default_path()?,
        };

        let contents = fs::read_to_string(&config_path).ok()?;
        let _ = ensure_secure_permissions(&config_path);
        serde_yaml::from_str(&contents).ok()
    }

    pub fn save(&self) -> Result<(), String> {
        let config_path =
            Self::default_path().ok_or_else(|| "Cannot determine home directory".to_string())?;
        let yaml = serde_yaml::to_string(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        write_private(&config_path, yaml.as_bytes())
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

/// Creates or replaces `path` with mode 0600, so the API key it holds is never
/// world-readable — not even for the instant between `write` and `chmod`.
#[cfg(unix)]
fn write_private(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    file.set_permissions(fs::Permissions::from_mode(0o600))?;
    file.write_all(contents)?;
    file.sync_all()
}

#[cfg(not(unix))]
fn write_private(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    fs::write(path, contents)
}

/// Tightens an existing config to 0600 when it grants group or other access,
/// so a file that leaked before this fix stops leaking on the next read.
///
/// Returns `Ok(true)` only when the mode was actually changed.
#[cfg(unix)]
fn ensure_secure_permissions(path: &Path) -> std::io::Result<bool> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = fs::metadata(path)?;
    if metadata.permissions().mode() & 0o077 == 0 {
        return Ok(false);
    }
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(true)
}

#[cfg(not(unix))]
fn ensure_secure_permissions(_path: &Path) -> std::io::Result<bool> {
    Ok(false)
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

#[cfg(all(test, unix))]
mod permission_tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn scratch(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("opus-cli-{}-{}", std::process::id(), name));
        path
    }

    #[test]
    fn a_saved_config_is_never_readable_by_anyone_else() {
        let path = scratch("write-private.yml");
        let _ = fs::remove_file(&path);
        write_private(&path, b"api_key: secret\n").unwrap();

        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "wrote {:o}", mode);
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn an_already_leaked_config_is_tightened_on_read() {
        let path = scratch("tighten.yml");
        let _ = fs::remove_file(&path);
        fs::write(&path, b"api_key: secret\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();

        assert!(ensure_secure_permissions(&path).unwrap());
        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "left {:o}", mode);
        assert!(!ensure_secure_permissions(&path).unwrap());
        fs::remove_file(&path).unwrap();
    }
}
