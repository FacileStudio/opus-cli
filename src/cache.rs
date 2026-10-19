use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::opus::models::{Task, Workspace};
use crate::tui::app::state::App;

const CACHE_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
pub struct Snapshot {
    version: u32,
    pub workspaces: Vec<Workspace>,
    pub tasks: Vec<Task>,
    pub project_map: HashMap<String, String>,
    pub project_colors: HashMap<String, String>,
    pub label_map: HashMap<String, String>,
    pub label_colors: HashMap<String, String>,
    pub filters: Vec<(String, String, Option<String>)>,
}

impl Snapshot {
    pub fn from_app(app: &App) -> Self {
        Self {
            version: CACHE_VERSION,
            workspaces: app.available_workspaces.clone(),
            tasks: app.all_tasks.clone(),
            project_map: app.project_map.clone(),
            project_colors: app.project_colors.clone(),
            label_map: app.label_map.clone(),
            label_colors: app.label_colors.clone(),
            filters: app
                .filters
                .iter()
                .map(|(id, title)| {
                    (
                        id.clone(),
                        title.clone(),
                        app.filter_descriptions.get(id).cloned(),
                    )
                })
                .collect(),
        }
    }

    pub fn apply(self, app: &mut App) {
        app.set_available_workspaces(self.workspaces);
        app.update_all_tasks(self.tasks);
        app.project_map = self.project_map;
        app.project_colors = self.project_colors;
        app.label_map = self.label_map;
        app.label_colors = self.label_colors;
        app.set_filters(self.filters);
    }
}

fn cache_path(api_url: &str, workspace_id: &str) -> Option<PathBuf> {
    let mut dir = crate::config::xdg_dir("XDG_CACHE_HOME", ".cache")?;
    dir.push("opus");
    dir.push(format!(
        "{}-{}.json",
        slug(api_url),
        slug(if workspace_id.is_empty() {
            "default"
        } else {
            workspace_id
        })
    ));
    Some(dir)
}

fn slug(value: &str) -> String {
    let mut slug = String::new();
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }
    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        "default".to_string()
    } else {
        slug
    }
}

pub fn load(api_url: &str, workspace_id: &str) -> Option<Snapshot> {
    let path = cache_path(api_url, workspace_id)?;
    let contents = fs::read_to_string(path).ok()?;
    let snapshot: Snapshot = serde_json::from_str(&contents).ok()?;
    if snapshot.version != CACHE_VERSION {
        return None;
    }
    Some(snapshot)
}

pub fn save(api_url: &str, workspace_id: &str, snapshot: &Snapshot) -> Result<(), String> {
    let path = cache_path(api_url, workspace_id)
        .ok_or_else(|| "Cannot determine the cache directory".to_string())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create {}: {}", parent.display(), e))?;
    }
    let json = serde_json::to_vec(snapshot)
        .map_err(|e| format!("Failed to serialize the cache: {}", e))?;
    crate::config::write_private(&path, &json)
        .map_err(|e| format!("Failed to write the cache to {}: {}", path.display(), e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cache_file_name_is_stable_for_a_url_and_workspace() {
        assert_eq!(slug("http://localhost:1337"), "http-localhost-1337");
        assert_eq!(
            slug("https://opus.facile.studio/"),
            "https-opus-facile-studio"
        );
        assert_eq!(slug(""), "default");
    }
}
