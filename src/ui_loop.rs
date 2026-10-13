use std::sync::Arc;
use tokio::sync::Mutex;
use crossterm::event::{KeyEvent, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::prelude::{CrosstermBackend, Terminal};
use std::io::stdout;
use crate::tui::app::state::App;
use crate::tui::events::EventHandler;
use crate::tui::ui::main::draw;
use crate::opus_client::OpusClient;
// dispatch_key and refresh_from_api moved here from main.rs
use crate::tui::modals::{handle_quick_add_modal, handle_edit_modal, handle_form_edit_modal};

/// Run the main UI event loop
pub async fn run_ui(
    app: Arc<Mutex<App>>,
    client_clone: Arc<Mutex<OpusClient>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    let event_handler = EventHandler::new(250);

    loop {

        {
            let app_guard = app.lock().await;
            terminal.draw(|f| draw(f, &app_guard))?;
        }

        match event_handler.next()? {
            // Handle key events only on Press or Repeat, ignore Release
            crate::tui::events::Event::Key(key) if key.kind == KeyEventKind::Press || key.kind == KeyEventKind::Repeat => {
                let mut app_guard = app.lock().await;

                // Modal input handling
                if app_guard.show_quick_add_modal {
                    // Route key to quick add modal handler
                    drop(app_guard);
                    let mut app_guard = app.lock().await;
                    handle_quick_add_modal(&mut *app_guard, &key, &client_clone, &client_clone).await;
                    continue;
                } else if app_guard.show_edit_modal {
                    drop(app_guard);
                    let mut app_guard = app.lock().await;
                    handle_edit_modal(&mut *app_guard, &key, &client_clone, &client_clone).await;
                    continue;
                } else if app_guard.show_form_edit_modal {
                    drop(app_guard);
                    let mut app_guard = app.lock().await;
                    handle_form_edit_modal(&mut *app_guard, &key, &client_clone, &client_clone).await;
                    continue;
                } else if app_guard.show_label_picker {
                    drop(app_guard);
                    let mut app_guard = app.lock().await;
                    crate::tui::pickers::label::handle_label_picker(&mut *app_guard, &key);
                    continue;
                } else if app_guard.show_project_picker {
                    drop(app_guard);
                    let mut app_guard = app.lock().await;
                    crate::tui::pickers::project::handle_project_picker(&mut *app_guard, &key);
                    continue;
                } else if app_guard.show_filter_picker {
                    drop(app_guard);
                    let mut app_guard = app.lock().await;
                    crate::tui::pickers::filter::handle_filter_picker(&mut *app_guard, &key, &client_clone).await;
                    continue;
                } else if app_guard.show_workspace_picker {
                    drop(app_guard);
                    let mut app_guard = app.lock().await;
                    crate::tui::pickers::workspace::handle_workspace_picker(&mut *app_guard, &key, &client_clone).await;
                    continue;
                } else if app_guard.show_comments_modal {
                    drop(app_guard);
                    let mut app_guard = app.lock().await;
                    if let Some(ref mut modal) = app_guard.comments_modal {
                        let action = modal.handle_key(&key);
                        match action {
                            crate::tui::modals::CommentsModalAction::Close => {
                                app_guard.hide_comments_modal();
                            }
                            crate::tui::modals::CommentsModalAction::Submit(comment_text) => {
                                if !comment_text.trim().is_empty() {
                                    let task_id = modal.task_id.clone();
                                    let client_clone = client_clone.clone();
                                    let app_clone = app.clone();
                                    let comment_clone = comment_text.clone();

                                    modal.clear_input();

                                    tokio::spawn(async move {
                                        let submit_result = {
                                            let client = client_clone.lock().await;
                                            client.add_comment_to_task(&task_id, &comment_clone).await
                                        };

                                        let mut app_guard = app_clone.lock().await;
                                        match submit_result {
                                            Ok(_) => {
                                                app_guard.add_debug_message("Comment added successfully".to_string());
                                                app_guard.show_toast("Comment added!".to_string());

                                                let client = client_clone.lock().await;
                                                match client.get_comments(&task_id).await {
                                                    Ok(comments_list) => {
                                                        // Update modal comments directly
                                                        if let Some(ref mut modal) = app_guard.comments_modal {
                                                            modal.comments = comments_list.clone();
                                                        }
                                                        app_guard.add_debug_message("Comments refreshed in modal".to_string());
                                                    }
                                                    Err(e) => {
                                                        app_guard.add_debug_message(format!("Failed to refresh comments: {}", e));
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                app_guard.add_debug_message(format!("Failed to add comment: {}", e));
                                                app_guard.show_toast("Failed to add comment".to_string());
                                            }
                                        }
                                    });
                                }
                            }
                            crate::tui::modals::CommentsModalAction::ToggleMode => {
                                // Mode toggle is handled internally by the modal
                            }
                            crate::tui::modals::CommentsModalAction::None => {}
                        }
                    }
                    continue;
                // Relations modals - DISABLED: Incomplete feature
                // } else if app_guard.show_relations_modal {
                //     if app_guard.show_add_relation_modal {
                //         drop(app_guard);
                //         let mut app_guard = app.lock().await;
                //         crate::tui::modals::handle_add_relation_modal(&mut *app_guard, &key, &client_clone).await;
                //     } else {
                //         drop(app_guard);
                //         let mut app_guard = app.lock().await;
                //         crate::tui::modals::handle_relations_modal(&mut *app_guard, &key, &client_clone).await;
                //     }
                //     continue;
                } else if app_guard.show_quick_actions_modal {
                    // Handle quick actions modal input
                    match key.code {
                        KeyCode::Esc => {
                            app_guard.hide_quick_actions_modal();
                        }
                        KeyCode::Up => {
                            if let Some(ref quick_actions) = app_guard.config.quick_actions {
                                if !quick_actions.is_empty() && app_guard.selected_quick_action_index as usize > 0 {
                                    app_guard.selected_quick_action_index -= 1;
                                }
                            }
                        }
                        KeyCode::Down => {
                            if let Some(ref quick_actions) = app_guard.config.quick_actions {
                                if !quick_actions.is_empty() && (app_guard.selected_quick_action_index as usize + 1) < quick_actions.len() {
                                    app_guard.selected_quick_action_index += 1;
                                }
                            }
                        }
                        KeyCode::Enter => {
                            if let Some(ref quick_actions) = app_guard.config.quick_actions {
                                if (app_guard.selected_quick_action_index as usize) < quick_actions.len() {
                                    let action = quick_actions[app_guard.selected_quick_action_index as usize].clone();
                                    app_guard.hide_quick_actions_modal();
                                    apply_quick_action_and_sync(&mut *app_guard, action, &client_clone).await;
                                }
                            }
                        }
                        KeyCode::Char(c) => {
                            // Direct character-based quick actions
                            if let Some(ref quick_actions) = app_guard.config.quick_actions {
                                if let Some((idx, action)) = quick_actions.iter().enumerate().find(|(_, a)| a.key == c.to_string()) {
                                    let action = action.clone();
                                    app_guard.hide_quick_actions_modal();
                                    app_guard.selected_quick_action_index = idx;
                                    apply_quick_action_and_sync(&mut *app_guard, action, &client_clone).await;
                                }
                            }
                        }
                        _ => {}
                    }
                    continue;
                } else if app_guard.show_subtask_modal {
                    // Handle subtask modal input
                    match key.code {
                        KeyCode::Esc => {
                            app_guard.hide_subtask_modal();
                        }
                        KeyCode::Up => {
                            app_guard.previous_subtask_task();
                        }
                        KeyCode::Down => {
                            app_guard.next_subtask_task();
                        }
                        KeyCode::Enter => {
                            if let Some(ref operation) = app_guard.subtask_operation {
                                match operation {
                                    _ => {
                                        // Handle single selection operations
                                        if let Some((target_task_id, _)) = app_guard.get_selected_subtask_task() {
                                            if let Some(selected_task) = app_guard.get_selected_task() {
                                                let current_task_id = selected_task.id.clone();
                                                let operation = app_guard.subtask_operation.clone();
                                                app_guard.hide_subtask_modal();

                                                // Handle the subtask operation async
                                                let client = client_clone.lock().await;
                                                match operation {
                                                    Some(crate::tui::app::state::SubtaskOperation::MakeSubtask) => {
                                                        match client.create_task_relation(
                                                            &current_task_id,
                                                            &target_task_id,
                                                            crate::opus_client::relations::RelationType::Subtask.as_str()
                                                        ).await {
                                                            Ok(_) => {
                                                                app_guard.show_toast("Task made into subtask successfully!".to_string());
                                                                app_guard.add_debug_message(format!("Task {} is now a subtask of {}", current_task_id, target_task_id));
                                                            }
                                                            Err(e) => {
                                                                app_guard.show_toast(format!("Failed to create subtask relation: {}", e));
                                                                app_guard.add_debug_message(format!("Error creating subtask relation: {}", e));
                                                            }
                                                        }
                                                    }
                                                    Some(crate::tui::app::state::SubtaskOperation::AddSubtask) => {
                                                        match client.create_task_relation(
                                                            &target_task_id,
                                                            &current_task_id,
                                                            crate::opus_client::relations::RelationType::Subtask.as_str()
                                                        ).await {
                                                            Ok(_) => {
                                                                app_guard.show_toast("Subtask added successfully!".to_string());
                                                                app_guard.add_debug_message(format!("Task {} is now a subtask of {}", target_task_id, current_task_id));
                                                            }
                                                            Err(e) => {
                                                                app_guard.show_toast(format!("Failed to create subtask relation: {}", e));
                                                                app_guard.add_debug_message(format!("Error creating subtask relation: {}", e));
                                                            }
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Char(' ') => {
                            // Toggle selection for bulk operations
                            app_guard.toggle_subtask_task_selection();
                        }
                        KeyCode::Char(c) => {
                            app_guard.add_char_to_subtask_input(c);
                        }
                        KeyCode::Backspace => {
                            app_guard.delete_char_from_subtask_input();
                        }
                        _ => {}
                    }
                    continue;
                } else if app_guard.show_add_subtask_modal {
                    // Handle add subtask modal input
                    match key.code {
                        KeyCode::Esc => {
                            app_guard.hide_add_subtask_modal();
                        }
                        KeyCode::Enter => {
                            // Create new subtask
                            let subtask_title = app_guard.get_add_subtask_input().to_string();
                            if !subtask_title.trim().is_empty() {
                                if let Some(parent_task) = app_guard.get_selected_task() {
                                    let parent_task_id = parent_task.id.clone();
                                    let parent_project_id = parent_task.project_id.clone();
                                    app_guard.hide_add_subtask_modal();

                                    let client = client_clone.lock().await;
                                    let subtask = crate::opus_client::tasks::OpusTaskCreate {
                                        title: subtask_title.clone(),
                                        description: None,
                                        priority: None,
                                        due_date: None,
                                        start_date: None,
                                        column_id: None,
                                    };

                                    match client.create_task(&parent_project_id, &subtask).await {
                                        Ok(new_task) => {
                                            app_guard.add_debug_message(format!("Created new task: {}", new_task.title));

                                            let new_task_id = new_task.id.clone();
                                            match client.create_task_relation(
                                                &parent_task_id,
                                                &new_task_id,
                                                crate::opus_client::relations::RelationType::Subtask.as_str()
                                            ).await {
                                                Ok(_) => {
                                                    app_guard.show_toast(format!("Created subtask: {}", subtask_title));
                                                    app_guard.add_debug_message(format!("Task {} is now a subtask of {}", new_task_id, parent_task_id));

                                                    app_guard.refreshing = true;
                                                }
                                                Err(e) => {
                                                    app_guard.show_toast(format!("Created task but failed to make it a subtask: {}", e));
                                                    app_guard.add_debug_message(format!("Error creating subtask relation: {}", e));
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            app_guard.show_toast(format!("Failed to create subtask: {}", e));
                                            app_guard.add_debug_message(format!("Error creating task: {}", e));
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Left => {
                            app_guard.move_add_subtask_cursor_left();
                        }
                        KeyCode::Right => {
                            app_guard.move_add_subtask_cursor_right();
                        }
                        KeyCode::Home => {
                            app_guard.add_subtask_cursor_position = 0;
                        }
                        KeyCode::End => {
                            app_guard.add_subtask_cursor_position = app_guard.add_subtask_input.len();
                        }
                        KeyCode::Char(c) => {
                            app_guard.add_char_to_add_subtask(c);
                        }
                        KeyCode::Backspace => {
                            app_guard.delete_char_from_add_subtask();
                        }
                        _ => {}
                    }
                    continue;
                }

                // Handle Ctrl key combinations first
                if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                    match key.code {
                        KeyCode::Char('z') => {
                            // Ctrl+Z - Undo
                            if let Some(task_id) = app_guard.undo_last_action() {
                                // Update the corresponding task in all_tasks
                                let updated_task = app_guard.tasks.iter().find(|t| t.id == task_id).cloned();
                                if let Some(updated_task) = updated_task {
                                    if let Some(task) = app_guard.all_tasks.iter_mut().find(|t| t.id == task_id) {
                                        *task = updated_task;
                                    }
                                }
                                // Show visual feedback
                                app_guard.add_debug_message("Undo operation completed".to_string());
                            } else {
                                app_guard.add_debug_message("Nothing to undo".to_string());
                            }
                            continue;
                        },
                        KeyCode::Char('y') => {
                            // Ctrl+Y - Redo
                            if let Some(task_id) = app_guard.redo_last_action() {
                                // Update the corresponding task in all_tasks
                                let updated_task = app_guard.tasks.iter().find(|t| t.id == task_id).cloned();
                                if let Some(updated_task) = updated_task {
                                    if let Some(task) = app_guard.all_tasks.iter_mut().find(|t| t.id == task_id) {
                                        *task = updated_task;
                                    }
                                }
                                // Show visual feedback
                                app_guard.add_debug_message("Redo operation completed".to_string());
                            } else {
                                app_guard.add_debug_message("Nothing to redo".to_string());
                            }
                            continue;
                        },
                        _ => {}
                    }
                    continue;
                }

                // Handle confirmation dialog actions async (Enter/y)
                if app_guard.show_confirmation_dialog && (key.code == KeyCode::Enter || matches!(key.code, KeyCode::Char('y') | KeyCode::Char('Y'))) {
                    drop(app_guard);
                    let mut app_guard = app.lock().await;
                    let client = client_clone.lock().await;
                    app_guard.confirm_action_async(&*client).await;
                    continue;
                }

                // handle async star toggle
                if key.code == KeyCode::Char('s') {
                    continue;
                }
                // handle async task completion toggle
                if key.code == KeyCode::Char('d') {
                    drop(app_guard);
                    let mut app_guard = app.lock().await;
                    if let Some(task_id) = app_guard.toggle_task_completion() {
                        // Sync with API
                        let client = client_clone.lock().await;
                        if let Some(task) = app_guard.tasks.iter().find(|t| t.id == task_id) {
                            let columns = client.get_columns_for_project(&task.project_id).await.unwrap_or_default();
                            match client.toggle_task_status(&task.id, &task.status, &columns).await {
                                Ok(_) => {
                                    app_guard.add_debug_message(format!("Task completion synced to API for task {}", task_id));
                                },
                                Err(e) => {
                                    app_guard.add_debug_message(format!("Failed to sync task completion to API: {}", e));
                                    app_guard.show_toast(format!("Sync failed: {}", e));
                                }
                            }
                        }
                    }
                    continue;
                }

                // handle dispatch_key and refresh
                let key_handled = dispatch_key(&mut *app_guard, key, &terminal);

                // After any navigation key, check if we need to fetch detailed task data
                if key_handled && (key.code == KeyCode::Up || key.code == KeyCode::Down ||
                                  key.code == KeyCode::Char('j') || key.code == KeyCode::Char('k') ||
                                  key.code == KeyCode::Char('g') || key.code == KeyCode::Char('G')) {
                    if let Some(task) = app_guard.get_selected_task() {
                        let task_id = task.id.clone();
                        if !app_guard.detailed_task_cache.contains_key(&task_id) {
                            let client_clone = client_clone.clone();
                            let app_clone = app.clone();
                            tokio::spawn(async move {
                                let client = client_clone.lock().await;
                                if let Ok(detailed_task) = client.get_task(&task_id).await {
                                    let mut app_guard = app_clone.lock().await;
                                    app_guard.cache_detailed_task(detailed_task);
                                }
                            });
                        }
                    }
                }

                if key_handled {
                    continue;
                }
                if key.code == KeyCode::Char('r') {
                    drop(app_guard);
                    let mut app_guard = app.lock().await;
                    refresh_from_api(&mut *app_guard, &client_clone).await;
                    continue;
                }

            }
            crate::tui::events::Event::Tick => {
                let app_guard = app.lock().await;
                terminal.draw(|f| draw(f, &app_guard))?;
            }
            // Ignore other events
            _ => {}
        }

        // exit on quit
        if !app.lock().await.running {
            break;
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

/// Handle key events, return true if event was handled
fn dispatch_key(app: &mut App, key: KeyEvent, terminal: &Terminal<CrosstermBackend<std::io::Stdout>>) -> bool {
    use crate::tui::modals::utils::try_show_modal;
    use KeyCode::*;

    // Reset consecutive 'q' counter for any key other than 'q'
    if !matches!(key.code, KeyCode::Char('q')) {
        app.reset_q_counter();
    }

    match key.code {
        KeyCode::Up => {
            app.previous_task();
            true
        }
        KeyCode::Down => {
            app.next_task();
            true
        }
        Char('g') => { app.jump_to_top(); true }
        Char('G') => { app.jump_to_bottom(); true }
        Char('?') => {
            try_show_modal(app, terminal, |app| app.show_help_modal());
            true
        }
        Char('q') => {
            app.handle_q_press();
            true
        }
        Char('Q') => {
            // Capital Q quits immediately without confirmation
            app.quit();
            true
        }
        Char('i') => { app.toggle_info_pane(); true }
        // Navigation: move selection down/up
        Char('j') => {
            app.next_task();
            true
        }
        Char('k') => {
            app.previous_task();
            true
        }
        // Switch layouts backward/forward
        Char('l') => { app.switch_to_next_layout(); true }
        // Cycle filters backward/forward
        Char('H') => { app.cycle_task_filter(); true }
        Char('L') => { app.cycle_task_filter(); true }
        Char('S') => {
            // Create a new subtask under the current task
            if app.get_selected_task().is_some() {
                app.show_add_subtask_modal();
            } else {
                app.show_toast("Select a parent task first".to_string());
            }
            true
        }
        Char('E') => {
            try_show_modal(app, terminal, |app| { app.hide_help_modal(); app.show_form_edit_modal(); });
            true
        }
        Char('e') => {
            try_show_modal(app, terminal, |app| app.show_edit_modal());
            true
        }
        Char('p') => {
            try_show_modal(app, terminal, |app| app.show_project_picker());
            true
        }
        Char('f') => {
            try_show_modal(app, terminal, |app| app.show_filter_picker());
            true
        }
        Char('W') => {
            try_show_modal(app, terminal, |app| app.show_workspace_picker());
            true
        }
        Char(' ') => {
            try_show_modal(app, terminal, |app| app.show_quick_actions_modal());
            true
        }
        Char('a') => {
            try_show_modal(app, terminal, |app| app.show_quick_add_modal());
            true
        }
        Char('c') => {
            try_show_modal(app, terminal, |app| app.show_comments_modal());
            true
        }
        Char('h') => {
            app.switch_to_previous_layout();
            true
        }
        Char('s') => {
            /* async star toggle handled in event loop */
            true
        }
        Enter => true,
        Char('n') | Char('N') => {
            if app.show_confirmation_dialog {
                app.cancel_confirmation();
            }
            true
        }
        Esc => {
            if app.show_confirmation_dialog {
                app.cancel_confirmation();
            } else {
                app.close_all_modals();
            }
            true
        }
        Char('y') | Char('Y') => {
            if app.show_confirmation_dialog {
                // handled async in event loop
            }
            true
        }
        Char('D') => { app.request_delete_task(); true }
        _ => false,
    }
}

/// Refresh tasks from API (stub implementation)
async fn refresh_from_api(
    app: &mut App,
    client: &Arc<Mutex<OpusClient>>,
) {
    app.refreshing = true;

    // Save current filter state before refresh
    let current_filter_id = app.current_filter_id.clone();
    let current_project_id = app.current_project_id.clone();
    let current_task_filter = app.task_filter.clone();
    let active_project_override = app.active_project_override.clone();

    let client = client.lock().await;
    match client.get_tasks_with_projects().await {
        Ok((tasks, project_map, project_colors)) => {
            app.all_tasks = tasks;
            app.project_map = project_map;
            app.project_colors = project_colors;

            // Reapply the current filter state after refresh
            if let Some(filter_id) = current_filter_id {
                // If a saved filter was active, reapply it
                app.current_filter_id = Some(filter_id.clone());
                app.active_project_override = active_project_override;

                // Fetch tasks for the filter
                match client.get_tasks_for_filter(&filter_id).await {
                    Ok(filter_tasks) => {
                        app.apply_filter_tasks(filter_tasks);
                        app.show_toast("Refreshed with filter applied!".to_string());
                    },
                    Err(e) => {
                        app.add_debug_message(format!("Failed to fetch filter tasks after refresh: {}", e));
                        // Fall back to applying task filter to all tasks
                        app.apply_task_filter();
                        app.show_toast("Refreshed! (Filter fetch failed)".to_string());
                    }
                }
            } else if current_project_id.is_some() {
                // If a project was selected, reapply project filter
                app.current_project_id = current_project_id;
                app.task_filter = current_task_filter;
                app.apply_project_filter();
                app.show_toast("Refreshed with project filter applied!".to_string());
            } else {
                // If no special filter was active, just apply the task filter
                app.task_filter = current_task_filter;
                app.apply_task_filter();
                app.show_toast("Refreshed!".to_string());
            }
        }
        Err(e) => {
            app.show_toast(format!("Refresh failed: {}", e));
        }
    }
    app.refreshing = false;
}

/// Apply quick action and sync with API (extracted from old main.rs)
async fn apply_quick_action_and_sync(
    app: &mut App,
    action: crate::config::QuickAction,
    client_clone: &Arc<Mutex<OpusClient>>,
) {
    match app.apply_quick_action(&action) {
        Ok(_) => {
            app.add_debug_message(format!("Quick action applied: {} -> {}", action.key, action.target));

            // Update the task on the server - handle labels differently
            let selected_task = app.get_selected_task().cloned();
            if let Some(task) = selected_task {
                if action.action == "label" {
                    // For label actions, use the specialized label API
                    if let Some(label_id) = app.label_map.iter().find_map(|(id, name)| {
                        if name == &action.target { Some(id.clone()) } else { None }
                    }) {
                        app.add_debug_message(format!("Adding label {} (id={}) to task {}", action.target, label_id, task.id));
                        match client_clone.lock().await.attach_label(&label_id, &task.id).await {
                            Ok(_) => {
                                app.add_debug_message(format!("Label API update successful for task {}", task.id));
                                app.show_toast(format!("Label added: {}", action.target));
                            },
                            Err(e) => {
                                app.add_debug_message(format!("Label API update failed: {}", e));
                                app.show_toast(format!("Label update failed: {}", e));
                            }
                        }
                    } else {
                        app.add_debug_message(format!("Label '{}' not found in label_map", action.target));
                        app.show_toast(format!("Label '{}' not found", action.target));
                    }
                } else {
                    // For non-label actions, use the general task update
                    let update = crate::opus_client::tasks::OpusTaskUpdate {
                        title: Some(task.title.clone()),
                        description: task.description.clone(),
                        status: None,
                        priority: Some(task.priority.to_string()),
                        due_date: task.due_date,
                        start_date: task.start_date,
                        user_id: None,
                    };
                    match client_clone.lock().await.update_task(&task.id, &update).await {
                        Ok(_) => {
                            app.show_toast(format!("Quick action applied: {} -> {}", action.key, action.target));
                        },
                        Err(e) => {
                            app.add_debug_message(format!("API update failed: {}", e));
                            app.show_toast(format!("Update failed: {}", e));
                        }
                    }
                }

            }
        }
        Err(e) => {
            app.add_debug_message(format!("Quick action error: {}", e));
            app.show_toast(format!("Quick action error: {}", e));
        }
    }
}

