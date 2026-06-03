use crate::tui::app::state::App;
use crossterm::event::KeyEvent;
use crate::opus_client::OpusClient;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn handle_workspace_picker(app: &mut App, key: &KeyEvent, api_client: &Arc<Mutex<OpusClient>>) {
    use crossterm::event::KeyCode;
    match key.code {
        KeyCode::Esc => {
            app.hide_workspace_picker();
        },
        KeyCode::Enter => {
            if let Some((workspace_id, workspace_name)) = app.get_selected_workspace() {
                if workspace_id == app.current_workspace_id {
                    app.show_toast(format!("Already in workspace: {}", workspace_name));
                    app.hide_workspace_picker();
                    return;
                }

                app.hide_workspace_picker();
                app.refreshing = true;
                app.current_workspace_id = workspace_id.clone();
                app.current_workspace_name = Some(workspace_name.clone());

                {
                    let mut client = api_client.lock().await;
                    client.set_workspace_id(workspace_id);
                }

                app.all_tasks.clear();
                app.tasks.clear();
                app.project_map.clear();
                app.project_colors.clear();
                app.label_map.clear();
                app.label_colors.clear();
                app.filters.clear();
                app.filter_descriptions.clear();
                app.detailed_task_cache.clear();
                app.current_filter_id = None;
                app.current_project_id = None;
                app.active_project_override = None;
                app.selected_task_index = 0;

                match api_client.lock().await.get_tasks_with_projects().await {
                    Ok((tasks, project_map, project_colors)) => {
                        app.all_tasks = tasks;
                        app.project_map = project_map;
                        app.project_colors = project_colors;
                    }
                    Err(e) => {
                        app.show_toast(format!("Failed to load workspace: {}", e));
                        app.refreshing = false;
                        return;
                    }
                }

                match api_client.lock().await.get_all_labels().await {
                    Ok(labels) => {
                        for label in labels {
                            app.label_map.insert(label.id.clone(), label.name.clone());
                            app.label_colors.insert(label.id.clone(), label.color.clone());
                        }
                    }
                    Err(e) => {
                        app.add_debug_message(format!("Failed to load labels for workspace: {}", e));
                    }
                }

                match api_client.lock().await.get_saved_filters().await {
                    Ok(filters) => {
                        app.set_filters(filters);
                    }
                    Err(e) => {
                        app.add_debug_message(format!("Failed to load filters for workspace: {}", e));
                    }
                }

                app.apply_task_filter();

                app.config.workspace_id = Some(app.current_workspace_id.clone());
                if let Err(e) = app.config.save() {
                    app.add_debug_message(format!("Failed to persist workspace to config: {}", e));
                }

                app.refreshing = false;
                app.show_toast(format!("Switched to workspace: {}", workspace_name));
            }
        },
        KeyCode::Backspace => {
            app.delete_char_from_workspace_picker();
        },
        KeyCode::Up => {
            app.move_workspace_picker_up();
        },
        KeyCode::Down => {
            app.move_workspace_picker_down();
        },
        KeyCode::Char(c) => {
            app.add_char_to_workspace_picker(c);
        },
        _ => {},
    }
}
