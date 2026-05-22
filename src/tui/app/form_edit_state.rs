#[cfg(test)]
mod tests {
    use super::*;
    use crate::opus::models::{Task, Priority};

    fn mock_task() -> Task {
        Task {
            id: "1".to_string(),
            title: "Test Task".to_string(),
            description: Some("Description".to_string()),
            due_date: None,
            start_date: None,
            priority: Priority::Medium,
            project_id: "42".to_string(),
            labels: None,
            assignees: None,
            ..Default::default()
        }
    }

    #[test]
    fn test_new_form_edit_state() {
        let task = mock_task();
        let form = FormEditState::new(&task);
        assert_eq!(form.title, "Test Task");
        assert_eq!(form.description, "Description");
        assert_eq!(form.project_id, "42");
        assert_eq!(form.priority, Some("medium".to_string()));
    }

    #[test]
    fn test_get_current_field_text() {
        let mut form = FormEditState::new(&mock_task());
        form.field_index = 0;
        assert_eq!(form.get_current_field_text(), "Test Task");
        form.field_index = 1;
        assert_eq!(form.get_current_field_text(), "Description");
    }

    #[test]
    fn test_set_current_field_text_title() {
        let mut form = FormEditState::new(&mock_task());
        form.field_index = 0;
        form.set_current_field_text("New Title".to_string());
        assert_eq!(form.title, "New Title");
    }

    #[test]
    fn test_quick_add_modal_integration() {
        let input = "Buy groceries *shopping @john +personal tomorrow !2";
        use crate::opus_parser::QuickAddParser;
        let parser = QuickAddParser::new();
        let parsed = parser.parse(input);

        assert_eq!(parsed.title, "Buy groceries");
        assert_eq!(parsed.labels, vec!["shopping"]);
        assert_eq!(parsed.assignees, vec!["john"]);
        assert_eq!(parsed.project, Some("personal".to_string()));
        assert_eq!(parsed.priority, Some(2));
        assert!(parsed.due_date.is_some());
    }

    #[test]
    fn test_form_edit_state_field_navigation_and_editing() {
        let task = mock_task();
        let mut form = FormEditState::new(&task);
        let test_values = [
            "New Title", "New Description", "2025-12-31", "2025-11-01", "high", "", "", "", "", "A comment"
        ];
        for i in 0..FormEditState::get_field_count() {
            form.field_index = i;
            if !test_values[i].is_empty() {
                form.set_current_field_text(test_values[i].to_string());
                let value = form.get_current_field_text();
                assert_eq!(value, test_values[i]);
            }
        }
        assert_eq!(form.title, "New Title");
        assert_eq!(form.description, "New Description");
        assert_eq!(form.due_date, Some("2025-12-31".to_string()));
        assert_eq!(form.start_date, Some("2025-11-01".to_string()));
        assert_eq!(form.priority, Some("high".to_string()));
        assert_eq!(form.comment, "A comment");
    }

    #[test]
    fn test_form_edit_state_priority_parsing() {
        let mut form = FormEditState::new(&mock_task());
        form.field_index = 4;
        form.set_current_field_text("urgent".to_string());
        assert_eq!(form.priority, Some("urgent".to_string()));
        form.set_current_field_text("".to_string());
        assert_eq!(form.priority, None);
    }

    #[test]
    fn test_form_edit_state_due_and_start_date_empty() {
        let mut form = FormEditState::new(&mock_task());
        form.field_index = 2;
        form.set_current_field_text("".to_string());
        assert_eq!(form.due_date, None);
        form.field_index = 3;
        form.set_current_field_text("".to_string());
        assert_eq!(form.start_date, None);
    }
}
use crate::opus::models::{Task, Priority};

#[derive(Clone, Debug)]
pub struct FormEditState {
    pub field_index: usize,
    pub title: String,
    pub description: String,
    pub due_date: Option<String>,
    pub start_date: Option<String>,
    pub priority: Option<String>,
    pub project_id: String,
    pub label_ids: Vec<String>,
    pub assignee_ids: Vec<String>,
    pub task_id: String,
    pub comment: String,
    pub cursor_position: usize,
}

impl FormEditState {
    pub fn set_project_id(&mut self, project_id: String) {
        self.project_id = project_id;
    }

    pub fn set_label_ids(&mut self, label_ids: Vec<String>) {
        self.label_ids = label_ids;
    }
    pub fn new(task: &Task) -> Self {
        let priority_str = match &task.priority {
            Priority::NoPriority => None,
            p => Some(p.to_string()),
        };
        Self {
            field_index: 0,
            title: task.title.clone(),
            description: task.description.clone().unwrap_or_default(),
            due_date: task.due_date.map(|d| d.format("%Y-%m-%d").to_string()),
            start_date: task.start_date.map(|d| d.format("%Y-%m-%d").to_string()),
            priority: priority_str,
            project_id: task.project_id.clone(),
            label_ids: task.labels.as_ref().map(|labels| labels.iter().map(|l| l.id.clone()).collect()).unwrap_or_default(),
            assignee_ids: task.assignees.as_ref().map(|assignees| assignees.iter().map(|a| a.id.clone()).collect()).unwrap_or_default(),
            task_id: task.id.clone(),
            comment: String::new(),
            cursor_position: 0,
        }
    }
    pub fn get_field_count() -> usize {
        10
    }
    pub fn get_current_field_text(&self) -> String {
        match self.field_index {
            0 => self.title.clone(),
            1 => self.description.clone(),
            2 => self.due_date.clone().unwrap_or_default(),
            3 => self.start_date.clone().unwrap_or_default(),
            4 => self.priority.clone().unwrap_or_default(),
            9 => self.comment.clone(),
            _ => String::new(),
        }
    }
    pub fn set_current_field_text(&mut self, text: String) {
        match self.field_index {
            0 => {
                self.title = text;
            }
            1 => {
                self.description = text;
            }
            2 => {
                self.due_date = if text.is_empty() { None } else { Some(text) };
            }
            3 => {
                self.start_date = if text.is_empty() { None } else { Some(text) };
            }
            4 => {
                self.priority = if text.is_empty() { None } else { Some(text) };
            }
            9 => {
                self.comment = text;
            }
            _ => {}
        }
    }
}
