use crate::tui::app::state::App;

impl App {
    pub fn set_filters(&mut self, filters: Vec<(String, String, Option<String>)>) {
        self.filters = filters.iter().map(|(id, title, _)| (id.clone(), title.clone())).collect();
        self.filter_descriptions = filters.into_iter()
            .filter_map(|(id, _, desc)| desc.map(|d| (id, d)))
            .collect();
    }
    pub fn apply_filter_tasks(&mut self, tasks: Vec<crate::opus::models::Task>) {
        self.tasks = tasks;
        self.apply_hierarchical_sort();
    }
    pub fn apply_task_filter(&mut self) {
        self.tasks = self.all_tasks.iter().filter(|task| match self.task_filter {
            crate::tui::app::task_filter::TaskFilter::ActiveOnly => !task.done,
            crate::tui::app::task_filter::TaskFilter::All => true,
            crate::tui::app::task_filter::TaskFilter::CompletedOnly => task.done,
        }).cloned().collect();

        self.apply_hierarchical_sort();

        if self.current_sort.is_none() {
            self.apply_layout_sort();
        }
    }
    pub fn get_filter_display_name(&self) -> String {
        if let Some(filter_id) = &self.current_filter_id {
            if let Some(filter) = self.filters.iter().find(|f| f.0 == *filter_id) {
                return filter.1.clone();
            }
            format!("Filter {}", filter_id)
        } else {
            match self.task_filter {
                crate::tui::app::task_filter::TaskFilter::ActiveOnly => "Active Tasks Only".to_string(),
                crate::tui::app::task_filter::TaskFilter::All => "All Tasks".to_string(),
                crate::tui::app::task_filter::TaskFilter::CompletedOnly => "Completed Tasks Only".to_string(),
            }
        }
    }
    pub fn cycle_task_filter(&mut self) {
        self.task_filter = match self.task_filter {
            crate::tui::app::task_filter::TaskFilter::ActiveOnly => crate::tui::app::task_filter::TaskFilter::All,
            crate::tui::app::task_filter::TaskFilter::All => crate::tui::app::task_filter::TaskFilter::CompletedOnly,
            crate::tui::app::task_filter::TaskFilter::CompletedOnly => crate::tui::app::task_filter::TaskFilter::ActiveOnly,
        };

        if self.current_project_id.is_some() {
            self.apply_project_filter();
        } else {
            self.apply_task_filter();
        }
    }
    pub fn update_all_tasks(&mut self, tasks: Vec<crate::opus::models::Task>) {
        self.all_tasks = tasks.clone();
        self.reapply_current_filters();
    }

    pub fn reapply_current_filters(&mut self) {
        if let Some(_filter_id) = &self.current_filter_id {
            self.apply_task_filter();
        } else if self.current_project_id.is_some() {
            self.apply_project_filter();
        } else {
            self.apply_task_filter();
        }
    }
    pub fn extract_project_override(&self, filter_id: &str) -> Option<String> {
        crate::debug::debug_log(&format!("extract_project_override: Checking filter_id={}", filter_id));

        if let Some(description) = self.filter_descriptions.get(filter_id) {
            crate::debug::debug_log(&format!("extract_project_override: Found description: '{}'", description));

            if let Some(start) = description.find("opus_project:") {
                let after_colon = &description[start + "opus_project:".len()..];
                let mut project_name = after_colon.trim()
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim();

                if let Some(tag_start) = project_name.find('<') {
                    project_name = &project_name[..tag_start];
                }

                project_name = project_name.trim();

                if !project_name.is_empty() {
                    crate::debug::debug_log(&format!("extract_project_override: Extracted project name: '{}'", project_name));
                    return Some(project_name.to_string());
                }
            }
            crate::debug::debug_log("extract_project_override: No 'opus_project:' pattern found in description");
        } else {
            crate::debug::debug_log(&format!("extract_project_override: No description found for filter_id={}", filter_id));
        }
        None
    }

    pub fn apply_filter_with_override(&mut self, filter_id: String) {
        crate::debug::debug_log(&format!("apply_filter_with_override: Processing filter_id={}", filter_id));

        self.current_filter_id = Some(filter_id.clone());

        if let Some(project_name) = self.extract_project_override(&filter_id) {
            crate::debug::debug_log(&format!("apply_filter_with_override: Project override detected: '{}'", project_name));
            self.active_project_override = Some(project_name.clone());
            self.show_toast(format!("Default project overridden to: {}", project_name));
            crate::debug::debug_log(&format!("apply_filter_with_override: Toast shown for project override: '{}'", project_name));
        } else {
            crate::debug::debug_log("apply_filter_with_override: No project override found in filter description");
            self.active_project_override = None;
        }

        crate::debug::debug_log(&format!("apply_filter_with_override: Final state - filter_id={:?}, override={:?}",
                                        self.current_filter_id, self.active_project_override));
    }

    pub fn get_active_default_project(&self) -> String {
        if let Some(ref override_project) = self.active_project_override {
            override_project.clone()
        } else {
            self.default_project_name.clone()
        }
    }

    pub fn find_filter_by_name(&self, name: &str) -> Option<String> {
        self.filters.iter()
            .find(|(_, title)| title.eq_ignore_ascii_case(name))
            .map(|(id, _)| id.clone())
    }

    pub async fn apply_default_filter_from_config(&mut self, config: &crate::config::OpusConfig, api_client: &std::sync::Arc<tokio::sync::Mutex<crate::opus_client::OpusClient>>) {
        if let Some(ref default_filter_name) = config.default_filter {
            crate::debug::debug_log(&format!("Attempting to apply default filter: '{}'", default_filter_name));

            if let Some(filter_id) = self.find_filter_by_name(default_filter_name) {
                crate::debug::debug_log(&format!("Found default filter '{}' with ID: {}", default_filter_name, filter_id));

                self.apply_filter_with_override(filter_id.clone());

                match api_client.lock().await.get_tasks_for_project(&filter_id).await {
                    Ok(tasks) => {
                        crate::debug::debug_log(&format!("Default filter: Got {} tasks for filter '{}'", tasks.len(), default_filter_name));
                        self.apply_filter_tasks(tasks);
                        self.show_toast(format!("Applied default filter: {}", default_filter_name));
                    },
                    Err(e) => {
                        crate::debug::debug_log(&format!("Default filter: Failed to fetch tasks for filter '{}': {}", default_filter_name, e));
                        self.show_toast(format!("Failed to load default filter: {}", default_filter_name));
                    }
                }
            } else {
                crate::debug::debug_log(&format!("Default filter '{}' not found in available filters", default_filter_name));
                self.show_toast(format!("Default filter '{}' not found", default_filter_name));
            }
        }
    }
}
