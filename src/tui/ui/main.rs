use crate::tui::app::state::App;
use ratatui::prelude::*;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Clear, Paragraph};

pub const PRIMARY_COLOR: Color = Color::Cyan;
pub const SUCCESS_COLOR: Color = Color::Green;
pub const DIM_COLOR: Color = Color::DarkGray;
pub const WARNING_COLOR: Color = Color::Yellow;

use super::form_edit::draw_form_edit_modal;
use super::modals::{
    draw_add_subtask_modal, draw_confirmation_dialog, draw_edit_modal, draw_quick_actions_modal,
    draw_quick_add_modal, draw_subtask_modal,
};
use super::pickers::{
    draw_filter_picker_modal, draw_label_picker_modal, draw_project_picker_modal,
    draw_workspace_picker_modal,
};
use super::task_details::draw_task_details;
use super::task_list::draw_tasks_table;

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

    if app.show_info_pane {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(body_area);
        draw_tasks_table(f, app, chunks[0]);
        draw_task_details(f, app, chunks[1]);
    } else {
        draw_tasks_table(f, app, body_area);
    }

    if app.show_help_modal {
        super::modals::draw_help_modal(f, app);
    } else if app.show_sort_modal {
        super::modals::draw_sort_modal(f, app);
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
    } else if app.show_workspace_picker {
        draw_workspace_picker_modal(f, app);
    } else if app.show_quick_actions_modal {
        draw_quick_actions_modal(f, app);
    } else if app.show_comments_modal {
        if let Some(ref modal) = app.comments_modal {
            modal.draw(f, f.size());
        }
    } else if app.show_subtask_modal {
        draw_subtask_modal(f, app);
    } else if app.show_add_subtask_modal {
        draw_add_subtask_modal(f, app);
    }

    draw_status_bar(f, app);

    if let Some(toast) = app.get_toast() {
        draw_toast(f, toast, f.size());
    }
}

fn draw_status_bar(f: &mut Frame, app: &App) {
    let area = Rect {
        x: 0,
        y: f.size().height.saturating_sub(1),
        width: f.size().width,
        height: 1,
    };
    f.render_widget(Clear, area);

    if app.refreshing {
        f.render_widget(
            Paragraph::new("Refreshing...")
                .style(Style::default().fg(WARNING_COLOR))
                .alignment(Alignment::Center),
            area,
        );
        return;
    }

    let line = Line::from(vec![
        Span::styled(
            app.get_current_workspace_name(),
            Style::default().fg(PRIMARY_COLOR),
        ),
        Span::styled(
            format!("  {} tasks", app.tasks.len()),
            Style::default().fg(DIM_COLOR),
        ),
        Span::styled(
            "   ? help  a add  e edit  q quit",
            Style::default().fg(DIM_COLOR),
        ),
    ]);
    f.render_widget(Paragraph::new(line), area);
}

fn draw_toast(f: &mut Frame, toast: &str, screen: Rect) {
    let width = (toast.len() as u16 + 4).min(screen.width / 2);
    let area = Rect {
        x: screen.width.saturating_sub(width + 2),
        y: screen.height.saturating_sub(3),
        width,
        height: 3,
    };
    let widget = Paragraph::new(toast.to_string())
        .block(Block::default())
        .style(Style::default().fg(SUCCESS_COLOR))
        .alignment(Alignment::Center);
    f.render_widget(Clear, area);
    f.render_widget(widget, area);
}
