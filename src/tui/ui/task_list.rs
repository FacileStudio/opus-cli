use crate::config::{TableColumn, TaskColumn};
use crate::opus::models::{Priority, Task};
use crate::tui::app::state::App;
use chrono::{DateTime, Local, Utc};
use ratatui::layout::Constraint;
use ratatui::prelude::*;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Cell, Row, Table};

use super::hex_to_color;

const DUE_SOON_DAYS: i64 = 3;
const BOTTOM_BUFFER: usize = 3;

fn format_date(date: &Option<DateTime<Utc>>) -> String {
    match date {
        Some(value) => value
            .with_timezone(&Local)
            .format("%m/%d/%y")
            .to_string(),
        None => "-".to_string(),
    }
}

fn format_date_relative(date: &Option<DateTime<Utc>>) -> (String, Color) {
    let Some(value) = date else {
        return ("-".to_string(), Color::DarkGray);
    };
    let local = value.with_timezone(&Local);
    let days = local.signed_duration_since(Local::now()).num_days();

    let label = match days {
        0 => "Today".to_string(),
        1 => "Tomorrow".to_string(),
        -1 => "Yesterday".to_string(),
        d if d > 1 => format!("{}d", d),
        d => format!("{}d ago", -d),
    };

    let color = if days < 0 {
        Color::Red
    } else if days == 0 {
        Color::Yellow
    } else if days <= DUE_SOON_DAYS {
        Color::Cyan
    } else {
        Color::White
    };
    (label, color)
}

fn priority_marker(priority: &Priority) -> Option<(&'static str, Color)> {
    match priority {
        Priority::Urgent => Some(("\u{f024} ", Color::Red)),
        Priority::High => Some(("\u{f024} ", Color::Rgb(255, 165, 0))),
        Priority::Medium => Some(("\u{f024} ", Color::Yellow)),
        Priority::Low => Some(("\u{f024} ", Color::Blue)),
        Priority::NoPriority => None,
    }
}

fn truncate(text: &str, width: u16) -> String {
    let width = width as usize;
    if text.len() <= width {
        return text.to_string();
    }
    let mut cut = width.saturating_sub(1);
    while cut > 0 && !text.is_char_boundary(cut) {
        cut -= 1;
    }
    format!("{}…", &text[..cut])
}

fn calculate_column_widths(columns: &[&TableColumn], available_width: u16) -> Vec<u16> {
    let mut widths: Vec<u16> = columns
        .iter()
        .map(|column| column.min_width.unwrap_or(8))
        .collect();
    let total: u16 = widths.iter().sum();

    if available_width > total {
        let flexible: Vec<usize> = columns
            .iter()
            .enumerate()
            .filter(|(_, column)| column.max_width.is_none())
            .map(|(index, _)| index)
            .collect();
        if !flexible.is_empty() {
            let share = (available_width - total) / flexible.len() as u16;
            for index in flexible {
                widths[index] += share;
            }
        }
    }

    for (index, column) in columns.iter().enumerate() {
        if let Some(max_width) = column.max_width {
            widths[index] = widths[index].min(max_width);
        }
    }
    widths
}

fn title_cell<'a>(task: &'a Task, app: &'a App) -> Cell<'a> {
    let mut spans = Vec::new();
    if task.done {
        spans.push(Span::styled("✓ ", Style::default().fg(Color::Green)));
    }
    if let Some((marker, color)) = priority_marker(&task.priority) {
        spans.push(Span::styled(marker, Style::default().fg(color)));
    }

    let (indent, prefix) = app.get_task_hierarchy_info(task);
    if indent > 0 {
        spans.push(Span::raw(" ".repeat(indent)));
    }
    if !prefix.is_empty() {
        spans.push(Span::styled(prefix, Style::default().fg(Color::DarkGray)));
    }
    spans.push(Span::raw(task.title.clone()));

    let mut cell = Cell::from(Line::from(spans));
    if task.done {
        cell = cell.style(
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::CROSSED_OUT),
        );
    }
    cell
}

fn cell_for_column<'a>(task: &'a Task, column: &TableColumn, app: &'a App, width: u16) -> Cell<'a> {
    match column.column_type {
        TaskColumn::Title => title_cell(task, app),
        TaskColumn::Project => {
            let name = app
                .project_map
                .get(&task.project_id)
                .cloned()
                .unwrap_or_else(|| "Unknown".to_string());
            let color = app
                .project_colors
                .get(&task.project_id)
                .map(|hex| hex_to_color(hex))
                .unwrap_or(Color::White);
            Cell::from(truncate(&name, width)).style(Style::default().fg(color))
        }
        TaskColumn::Labels => {
            let Some(labels) = &task.labels else {
                return Cell::from("");
            };
            let mut spans = Vec::new();
            for (index, label) in labels.iter().enumerate() {
                let color = app
                    .label_colors
                    .get(&label.id)
                    .map(|hex| hex_to_color(hex))
                    .unwrap_or(Color::Gray);
                spans.push(Span::styled(label.name.clone(), Style::default().fg(color)));
                if index < labels.len() - 1 {
                    spans.push(Span::raw(", "));
                }
            }
            Cell::from(Line::from(spans))
        }
        TaskColumn::DueDate => {
            let (label, color) = format_date_relative(&task.due_date);
            Cell::from(label).style(Style::default().fg(color))
        }
        TaskColumn::StartDate => {
            let (label, color) = format_date_relative(&task.start_date);
            Cell::from(label).style(Style::default().fg(color))
        }
        TaskColumn::Priority => {
            let Some((marker, color)) = priority_marker(&task.priority) else {
                return Cell::from("-");
            };
            let label = match task.priority {
                Priority::Urgent => "urgent",
                Priority::High => "high",
                Priority::Medium => "medium",
                Priority::Low => "low",
                Priority::NoPriority => "-",
            };
            Cell::from(format!("{}{}", marker, label)).style(Style::default().fg(color))
        }
        TaskColumn::Status => Cell::from(if task.done { "Done" } else { "Open" }),
        TaskColumn::Assignees => {
            let assignees = task
                .assignees
                .as_ref()
                .map(|users| {
                    users
                        .iter()
                        .map(|user| user.name.as_str())
                        .collect::<Vec<&str>>()
                        .join(", ")
                })
                .unwrap_or_default();
            Cell::from(truncate(&assignees, width))
        }
        TaskColumn::Created => Cell::from(format_date(&task.created_at.into())),
        TaskColumn::Updated => Cell::from(format_date(&task.updated_at.into())),
    }
}

fn window_bounds(total: usize, selected: usize, visible: usize) -> (usize, usize) {
    if total <= visible {
        return (0, total);
    }
    if selected >= total.saturating_sub(BOTTOM_BUFFER) {
        return (total - visible, total);
    }
    if selected < visible / 2 {
        return (0, visible);
    }
    let start = selected + 1 - visible / 2;
    (start.min(total - visible), start.min(total - visible) + visible)
}

pub fn draw_tasks_table(f: &mut Frame, app: &App, area: Rect) {
    let columns = app.get_current_layout_columns();
    let enabled: Vec<&TableColumn> = columns.iter().filter(|column| column.enabled).collect();

    let available_width = area.width.saturating_sub(enabled.len() as u16 + 1);
    let widths = calculate_column_widths(&enabled, available_width);

    let header = Row::new(
        enabled
            .iter()
            .map(|column| Cell::from(column.name.as_str()).style(Style::default().fg(Color::Cyan))),
    )
    .height(1)
    .bottom_margin(1);

    let constraints: Vec<Constraint> = widths.iter().map(|width| Constraint::Length(*width)).collect();

    let height = area.height as usize;
    let visible_rows = if height > 4 { height - 4 } else { 1 }.max(1);
    let (start, end) = window_bounds(app.tasks.len(), app.selected_task_index, visible_rows);

    let rows = app
        .tasks
        .iter()
        .enumerate()
        .skip(start)
        .take(end.saturating_sub(start))
        .map(|(index, task)| {
            let cells: Vec<Cell> = enabled
                .iter()
                .zip(widths.iter())
                .map(|(column, width)| cell_for_column(task, column, app, *width))
                .collect();
            let row = Row::new(cells);
            if index == app.selected_task_index {
                row.style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                row
            }
        });

    let table = Table::new(rows, constraints)
        .header(header)
        .block(Block::default().title(task_list_title(app)));

    f.render_widget(table, area);
}

fn task_list_title(app: &App) -> String {
    let filter = app.get_filter_display_name();
    if app.current_project_id.is_some() {
        format!("Tasks ({}) - Project: {}", filter, app.get_current_project_name())
    } else if app.active_project_override.is_some() {
        format!(
            "Tasks ({}) - Default Project: {}",
            filter,
            app.get_active_default_project()
        )
    } else {
        format!("Tasks ({})", filter)
    }
}
