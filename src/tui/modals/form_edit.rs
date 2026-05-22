use crate::tui::app::state::App;
use crate::tui::app::form_edit_state::FormEditState;
use crossterm::event::KeyEvent;
use crate::opus_client::OpusClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::debug::debug_log;
use chrono::Local;

pub async fn handle_form_edit_modal(
    app: &mut App,
    key: &KeyEvent,
    api_client: &Arc<Mutex<OpusClient>>,
    client_clone: &Arc<Mutex<OpusClient>>,
) {
    use crossterm::event::KeyCode;

    if let Some(form) = app.form_edit_state.as_mut() {
        match key.code {
            KeyCode::Tab => {
                let current_text = form.get_current_field_text();
                form.set_current_field_text(current_text);
                form.field_index = (form.field_index + 1) % FormEditState::get_field_count();
                update_cursor_position(form);
            }
            KeyCode::BackTab => {
                let current_text = form.get_current_field_text();
                form.set_current_field_text(current_text);
                if form.field_index == 0 {
                    form.field_index = FormEditState::get_field_count() - 1;
                } else {
                    form.field_index -= 1;
                }
                update_cursor_position(form);
            }
            KeyCode::Up => {
                let current_text = form.get_current_field_text();
                form.set_current_field_text(current_text);
                if form.field_index == 0 {
                    form.field_index = FormEditState::get_field_count() - 1;
                } else {
                    form.field_index -= 1;
                }
                update_cursor_position(form);
            }
            KeyCode::Down => {
                let current_text = form.get_current_field_text();
                form.set_current_field_text(current_text);
                form.field_index = (form.field_index + 1) % FormEditState::get_field_count();
                update_cursor_position(form);
            }
            KeyCode::Esc => {
                app.hide_form_edit_modal();
            }
            KeyCode::Enter => {
                let current_text = form.get_current_field_text();
                form.set_current_field_text(current_text);
                if let Some(form) = app.form_edit_state.as_ref() {
                    let mut errors: Vec<String> = Vec::new();
                    if form.title.trim().is_empty() {
                        errors.push("Title is required.".to_string());
                    }
                    if let Some(due) = &form.due_date {
                        if !due.trim().is_empty() && chrono::NaiveDate::parse_from_str(due.trim(), "%Y-%m-%d").is_err() {
                            errors.push("Due date must be in YYYY-MM-DD format.".to_string());
                        }
                    }
                    if let Some(start) = &form.start_date {
                        if !start.trim().is_empty() && chrono::NaiveDate::parse_from_str(start.trim(), "%Y-%m-%d").is_err() {
                            errors.push("Start date must be in YYYY-MM-DD format.".to_string());
                        }
                    }
                    if !form.project_id.is_empty() && !app.project_map.contains_key(&form.project_id) {
                        errors.push("Selected project does not exist.".to_string());
                    }
                    for label_id in &form.label_ids {
                        if !app.label_map.contains_key(label_id) {
                            errors.push(format!("Label ID {} does not exist.", label_id));
                        }
                    }
                    if !errors.is_empty() {
                        let msg = errors.join("\n");
                        debug_log(&format!("FORM VALIDATION ERROR: {}", msg));
                        app.toast_notification = Some(msg);
                        app.toast_notification_start = Some(Local::now());
                        return;
                    }
                }
                if let Err(e) = save_form_task(app, api_client, client_clone).await {
                    debug_log(&format!("Failed to save task from form: {}", e));
                    app.toast_notification = Some(format!("Failed to save: {}", e));
                    app.toast_notification_start = Some(Local::now());
                } else {
                    app.hide_form_edit_modal();
                }
            }
            KeyCode::Char(' ') => {
                match form.field_index {
                    5 => {
                        app.open_project_picker_from_form();
                    }
                    6 => {
                        app.open_label_picker_from_form();
                    }
                    _ => {
                        add_char_to_current_field(form, ' ');
                    }
                }
            }
            KeyCode::Char(c) => {
                add_char_to_current_field(form, c);
            }
            KeyCode::Backspace => {
                delete_char_from_current_field(form);
            }
            KeyCode::Left => {
                if form.cursor_position > 0 {
                    form.cursor_position -= 1;
                }
            }
            KeyCode::Right => {
                let current_text = form.get_current_field_text();
                if form.cursor_position < current_text.len() {
                    form.cursor_position += 1;
                }
            }
            _ => {}
        }
    }
}

fn update_cursor_position(form: &mut FormEditState) {
    let current_text = form.get_current_field_text();
    form.cursor_position = current_text.len();
}

fn add_char_to_current_field(form: &mut FormEditState, c: char) {
    match form.field_index {
        0 => {
            form.title.insert(form.cursor_position, c);
            form.cursor_position += 1;
        }
        1 => {
            form.description.insert(form.cursor_position, c);
            form.cursor_position += 1;
        }
        2 => {
            let date_str = form.due_date.get_or_insert_with(String::new);
            date_str.insert(form.cursor_position, c);
            form.cursor_position += 1;
        }
        3 => {
            let date_str = form.start_date.get_or_insert_with(String::new);
            date_str.insert(form.cursor_position, c);
            form.cursor_position += 1;
        }
        4 => {
            let priority_str = form.priority.get_or_insert_with(String::new);
            priority_str.insert(form.cursor_position, c);
            form.cursor_position += 1;
        }
        9 => {
            form.comment.insert(form.cursor_position, c);
            form.cursor_position += 1;
        }
        _ => {}
    }
}

fn delete_char_from_current_field(form: &mut FormEditState) {
    match form.field_index {
        0 => {
            if form.cursor_position > 0 && form.cursor_position <= form.title.len() {
                form.cursor_position -= 1;
                form.title.remove(form.cursor_position);
            }
        }
        1 => {
            if form.cursor_position > 0 && form.cursor_position <= form.description.len() {
                form.cursor_position -= 1;
                form.description.remove(form.cursor_position);
            }
        }
        2 => {
            if let Some(ref mut date_str) = form.due_date {
                if form.cursor_position > 0 && form.cursor_position <= date_str.len() {
                    form.cursor_position -= 1;
                    date_str.remove(form.cursor_position);
                    if date_str.is_empty() {
                        form.due_date = None;
                    }
                }
            }
        }
        3 => {
            if let Some(ref mut date_str) = form.start_date {
                if form.cursor_position > 0 && form.cursor_position <= date_str.len() {
                    form.cursor_position -= 1;
                    date_str.remove(form.cursor_position);
                    if date_str.is_empty() {
                        form.start_date = None;
                    }
                }
            }
        }
        4 => {
            if let Some(ref mut priority_str) = form.priority {
                if form.cursor_position > 0 && form.cursor_position <= priority_str.len() {
                    form.cursor_position -= 1;
                    priority_str.remove(form.cursor_position);
                    if priority_str.is_empty() {
                        form.priority = None;
                    }
                }
            } else {
                form.cursor_position = 0;
            }
        }
        9 => {
            if form.cursor_position > 0 && form.cursor_position <= form.comment.len() {
                form.cursor_position -= 1;
                form.comment.remove(form.cursor_position);
            }
        }
        _ => {}
    }
}

async fn save_form_task(
    app: &mut App,
    api_client: &Arc<Mutex<OpusClient>>,
    client_clone: &Arc<Mutex<OpusClient>>,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(form) = &app.form_edit_state {
        debug_log(&format!("Saving task from form: ID {}", form.task_id));

        let api_client_guard = api_client.lock().await;

        let update = crate::opus_client::tasks::OpusTaskUpdate {
            title: Some(form.title.clone()),
            description: Some(form.description.clone()),
            priority: form.priority.clone(),
            due_date: form.due_date.as_ref().and_then(|d| {
                chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                    .ok()
                    .and_then(|nd| nd.and_hms_opt(0, 0, 0))
                    .map(|ndt| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(ndt, chrono::Utc))
            }),
            start_date: form.start_date.as_ref().and_then(|d| {
                chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                    .ok()
                    .and_then(|nd| nd.and_hms_opt(0, 0, 0))
                    .map(|ndt| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(ndt, chrono::Utc))
            }),
            column_id: None,
        };

        let result = api_client_guard.update_task(&form.task_id, &update).await;

        drop(api_client_guard);

        match result {
            Ok(task) => {
                debug_log(&format!("SUCCESS: Task updated from form! ID: {}, Title: '{}' Description: {:?}", task.id, task.title, task.description));

                let (mut tasks, project_map, project_colors) = client_clone.lock().await.get_tasks_with_projects().await.unwrap_or_default();
                for t in &mut tasks {
                    if t.id == task.id {
                        *t = task.clone();
                        break;
                    }
                }
                app.all_tasks = tasks;
                app.project_map = project_map;
                app.project_colors = project_colors;
                app.apply_task_filter();

                app.flash_task_id = Some(task.id.clone());
                app.flash_start = Some(Local::now());
                app.flash_cycle_count = 0;
                app.flash_cycle_max = 6;

                debug_log(&format!("Tasks refreshed. Total tasks: {}", app.tasks.len()));
                Ok(())
            }
            Err(e) => {
                debug_log(&format!("ERROR: Failed to update task from form: {}", e));
                Err(format!("{}", e).into())
            }
        }
    } else {
        Err("No form state available".into())
    }
}
