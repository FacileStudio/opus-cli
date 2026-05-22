use crate::tui::app::state::App;
use crossterm::event::KeyEvent;
use crate::opus_client::OpusClient;
use std::sync::Arc;
use tokio::sync::Mutex;

#[allow(dead_code)]
pub async fn handle_filter_picker(app: &mut App, key: &KeyEvent, api_client: &Arc<Mutex<OpusClient>>) {
    use crossterm::event::KeyCode;
    match key.code {
        KeyCode::Esc => {
            app.hide_filter_picker();
        },
        KeyCode::Enter => {
            let (id, name) = app.filtered_filters.get(app.selected_filter_picker_index).cloned().unwrap_or(("__clear__".to_string(), "Clear Filter".to_string()));
            app.add_debug_message(format!("Filter picker: Enter pressed, id={}, name={}", id, name));
            if id == "__clear__" {
                app.clear_filter();
                app.add_debug_message("Filter picker: Clear Filter selected, clearing current filter".to_string());
                app.apply_task_filter();
            } else {
                app.apply_filter_with_override(id.clone());
                app.add_debug_message(format!("Filter picker: Fetching tasks for filter id={}", id));
                match api_client.lock().await.get_tasks_for_project(&id).await {
                    Ok(tasks) => {
                        app.add_debug_message(format!("Filter picker: Got {} tasks for filter {}", tasks.len(), id));
                        app.apply_filter_tasks(tasks);
                    },
                    Err(e) => {
                        app.add_debug_message(format!("Filter picker: Failed to fetch tasks for filter {}: {}", id, e));
                    }
                }
            }
            app.hide_filter_picker();
        },
        KeyCode::Backspace => {
            app.delete_char_from_filter_picker();
        },
        KeyCode::Delete => {
            if app.current_filter_id.is_some() {
                app.clear_filter();
                app.add_debug_message("Filter picker: Delete key pressed, clearing current filter".to_string());
                app.apply_task_filter();
                app.hide_filter_picker();
            }
        },
        KeyCode::Up => {
            app.move_filter_picker_up();
        },
        KeyCode::Down => {
            app.move_filter_picker_down();
        },
        KeyCode::Char(c) => {
            app.add_char_to_filter_picker(c);
        },
        _ => {},
    }
}
