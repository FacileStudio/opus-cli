use crate::tui::app::state::App;
use crate::tui::utils::contains_ignore_case;

impl App {
    pub fn show_workspace_picker(&mut self) {
        if self.available_workspaces.is_empty() {
            self.show_toast("No workspaces available".to_string());
            return;
        }
        self.close_all_modals();
        self.show_workspace_picker = true;
        self.workspace_picker_input.clear();
        self.selected_workspace_picker_index = 0;
        self.update_filtered_workspaces();
    }

    pub fn hide_workspace_picker(&mut self) {
        self.show_workspace_picker = false;
        self.workspace_picker_input.clear();
    }

    pub fn add_char_to_workspace_picker(&mut self, c: char) {
        self.workspace_picker_input.push(c);
        self.update_filtered_workspaces();
        self.selected_workspace_picker_index = 0;
    }

    pub fn delete_char_from_workspace_picker(&mut self) {
        if !self.workspace_picker_input.is_empty() {
            self.workspace_picker_input.pop();
            self.update_filtered_workspaces();
            self.selected_workspace_picker_index = 0;
        }
    }

    pub fn move_workspace_picker_up(&mut self) {
        if !self.filtered_workspaces.is_empty() {
            self.selected_workspace_picker_index = (self.selected_workspace_picker_index + self.filtered_workspaces.len() - 1) % self.filtered_workspaces.len();
        }
    }

    pub fn move_workspace_picker_down(&mut self) {
        if !self.filtered_workspaces.is_empty() {
            self.selected_workspace_picker_index = (self.selected_workspace_picker_index + 1) % self.filtered_workspaces.len();
        }
    }

    pub fn get_selected_workspace(&self) -> Option<(String, String)> {
        self.filtered_workspaces.get(self.selected_workspace_picker_index).cloned()
    }

    pub fn update_filtered_workspaces(&mut self) {
        let query = &self.workspace_picker_input;
        self.filtered_workspaces = self.available_workspaces.iter()
            .filter(|w| contains_ignore_case(&w.name, query))
            .map(|w| (w.id.clone(), w.name.clone()))
            .collect();
    }

    pub fn get_current_workspace_name(&self) -> String {
        if let Some(ref name) = self.current_workspace_name {
            name.clone()
        } else if !self.current_workspace_id.is_empty() {
            self.current_workspace_id.clone()
        } else {
            "No workspace".to_string()
        }
    }

    pub fn set_available_workspaces(&mut self, workspaces: Vec<crate::opus::models::Workspace>) {
        self.available_workspaces = workspaces;
        self.current_workspace_name = self.available_workspaces.iter()
            .find(|w| w.id == self.current_workspace_id)
            .map(|w| w.name.clone());
    }
}
