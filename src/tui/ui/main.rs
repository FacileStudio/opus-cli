use crate::tui::app::state::App;
use ratatui::prelude::*;
use ratatui::style::{Color, Style, Modifier};
use ratatui::widgets::{Paragraph, Block, Clear};

pub const PRIMARY_COLOR: Color = Color::Rgb(138, 180, 248);
pub const SECONDARY_COLOR: Color = Color::Rgb(187, 134, 252);
pub const SUCCESS_COLOR: Color = Color::Rgb(129, 199, 132);
pub const DIM_COLOR: Color = Color::Rgb(117, 117, 117);
pub const DANGER_COLOR: Color = Color::Rgb(239, 83, 80);
pub const WARNING_COLOR: Color = Color::Rgb(255, 183, 77);

use super::task_list::draw_tasks_table;
use super::task_details::draw_task_details;
use super::modals::{draw_quick_add_modal, draw_edit_modal, draw_confirmation_dialog, draw_quick_actions_modal, draw_add_subtask_modal, draw_subtask_modal};
use super::form_edit::draw_form_edit_modal;
use super::pickers::{draw_project_picker_modal, draw_filter_picker_modal, draw_label_picker_modal};

pub fn hex_to_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&hex[0..2], 16),
            u8::from_str_radix(&hex[2..4], 16),
            u8::from_str_radix(&hex[4..6], 16),
        ) {
            return Color::Rgb(r, g, b);
        }
    }
    Color::White
}

pub fn draw(f: &mut Frame, app: &App) {
    let body_area = f.size();

    let _main_layout = if app.show_debug_pane {
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),
                Constraint::Length(10),
            ])
            .split(body_area);

        let main_horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60),
                Constraint::Percentage(40),
            ])
            .split(vertical_chunks[0]);

        draw_tasks_table(f, app, main_horizontal[0]);
        if app.show_info_pane {
            draw_task_details(f, app, main_horizontal[1]);
        }
        draw_debug_pane(f, app, vertical_chunks[1]);
    } else if app.show_info_pane {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
            .split(body_area);
        draw_tasks_table(f, app, chunks[0]);
        draw_task_details(f, app, chunks[1]);
    } else {
        draw_tasks_table(f, app, body_area);
    };

    if app.show_help_modal {
        crate::tui::ui::modals::draw_help_modal(f, app);
    } else if app.show_advanced_help_modal {
        crate::tui::ui::modals::draw_advanced_help_modal(f, app);
    } else if app.show_advanced_features_modal {
        crate::tui::ui::modals::draw_advanced_features_modal(f, app);
    } else if app.show_sort_modal {
        crate::tui::ui::modals::draw_sort_modal(f, app);
    } else if app.show_form_edit_modal {
        draw_form_edit_modal(f, app);
        if app.show_project_picker {
            draw_project_picker_modal(f, app);
        } else if app.show_label_picker {
            draw_label_picker_modal(f, app);
        }
    } else if app.show_project_picker {
        draw_project_picker_modal(f, app);
    } else if app.show_label_picker {
        draw_label_picker_modal(f, app);
    } else if app.show_quick_add_modal {
        draw_quick_add_modal(f, app);
    } else if app.show_edit_modal {
        draw_edit_modal(f, app);
    } else if app.show_confirmation_dialog {
        draw_confirmation_dialog(f, app);
    } else if app.show_filter_picker {
        draw_filter_picker_modal(f, app);
    } else if app.show_quick_actions_modal {
        draw_quick_actions_modal(f, app);
    } else if app.show_attachment_modal {
        if let Some(ref modal) = app.attachment_modal {
            modal.draw(f, f.size());
        }
    } else if app.show_file_picker_modal {
        if let Some(ref modal) = app.file_picker_modal {
            modal.draw(f, f.size());
        }
    } else if let Some(ref modal) = app.comments_modal {
        modal.draw(f, f.size());
    } else if app.show_subtask_modal {
        draw_subtask_modal(f, app);
    } else if app.show_add_subtask_modal {
        draw_add_subtask_modal(f, app);
    }

    if app.refreshing {
        let refresh_area = Rect {
            x: 0,
            y: f.size().height.saturating_sub(1),
            width: f.size().width,
            height: 1,
        };
        let refresh_msg = Paragraph::new("Refreshing...")
            .style(Style::default().fg(WARNING_COLOR).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        f.render_widget(Clear, refresh_area);
        f.render_widget(refresh_msg, refresh_area);
    }

    if let Some(notification) = app.get_layout_notification() {
        let notification_width = (notification.len() as u16 + 4).min(f.size().width / 2);
        let notification_area = Rect {
            x: f.size().width.saturating_sub(notification_width + 2),
            y: f.size().height.saturating_sub(6),
            width: notification_width,
            height: 3,
        };
        let notification_msg = Paragraph::new(notification.clone())
            .block(Block::default().title("Layout"))
            .style(Style::default().fg(PRIMARY_COLOR).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        f.render_widget(Clear, notification_area);
        f.render_widget(notification_msg, notification_area);
    }

    if let Some(toast) = app.get_toast() {
        let toast_width = (toast.len() as u16 + 4).min(f.size().width / 2);
        let toast_area = Rect {
            x: f.size().width.saturating_sub(toast_width + 2),
            y: f.size().height.saturating_sub(3),
            width: toast_width,
            height: 3,
        };
        let toast_msg = Paragraph::new(toast.clone())
            .block(Block::default().title("Success"))
            .style(Style::default().fg(SUCCESS_COLOR).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        f.render_widget(Clear, toast_area);
        f.render_widget(toast_msg, toast_area);
    }
}

fn draw_debug_pane(f: &mut Frame, app: &App, area: Rect) {
    let debug_block = Block::default()
        .title("Debug Log")
        .style(Style::default().fg(WARNING_COLOR));

    let debug_text: Vec<String> = app.debug_messages
        .iter()
        .rev()
        .take(area.height as usize)
        .map(|(timestamp, message)| {
            let time_str = timestamp.format("%H:%M:%S").to_string();
            format!("[{}] {}", time_str, message)
        })
        .collect();

    let debug_content = debug_text.join("\n");

    let debug_widget = Paragraph::new(debug_content)
        .block(debug_block)
        .style(Style::default().fg(Color::White))
        .scroll((0, 0));

    f.render_widget(debug_widget, area);
}
