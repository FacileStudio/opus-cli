use crate::tui::app::state::App;
use crate::opus::models::Priority;
use ratatui::prelude::*;
use ratatui::style::{Color, Style, Modifier};
use ratatui::widgets::{Paragraph, Block, Borders, Wrap};
use ratatui::text::{Line, Span};
use chrono::{Datelike, Local};
use super::hex_to_color;


pub fn draw_task_details(f: &mut Frame, app: &App, area: Rect) {
    let selected_task = app.get_selected_task();

    let details = if let Some(basic_task) = selected_task {
        let task = app.get_detailed_task(&basic_task.id).unwrap_or(basic_task);

        let _project_name = app.project_map.get(&task.project_id)
            .map(|s| s.as_str())
            .unwrap_or("Unknown");

        let mut details_lines = vec![
            Line::from(vec![
                Span::styled("Title: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&task.title)
            ]),
            Line::from(""),
        ];

        if let Some(description) = &task.description {
            if !description.is_empty() {
                details_lines.push(Line::from(vec![
                    Span::styled("Description: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(description)
                ]));
                details_lines.push(Line::from(""));
            }
        }

        if !task.project_id.is_empty() {
            if let Some(project_name) = app.project_map.get(&task.project_id) {
                let color = app.project_colors.get(&task.project_id)
                    .map(|hex_str| hex_to_color(hex_str))
                    .unwrap_or(Color::Blue);

                details_lines.push(Line::from(vec![
                    Span::styled("Project: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(&**project_name, Style::default().fg(color))
                ]));
                details_lines.push(Line::from(""));
            }
        }

        if let Some(ref related_tasks) = task.related_tasks {
            if let Some(parent_tasks) = related_tasks.get("parenttask") {
                if !parent_tasks.is_empty() {
                    let parent_task = &parent_tasks[0];
                    details_lines.push(Line::from(vec![
                        Span::styled("Parent Task: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled("🔗", Style::default().fg(Color::Yellow)),
                        Span::raw(format!(" {}", parent_task.title))
                    ]));
                    details_lines.push(Line::from(""));
                }
            }

            if let Some(subtasks) = related_tasks.get("subtask") {
                if !subtasks.is_empty() {
                    details_lines.push(Line::from(vec![
                        Span::styled("Subtasks: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled("📋", Style::default().fg(Color::Green)),
                        Span::raw(format!(" {} subtask(s)", subtasks.len()))
                    ]));

                    for (_i, subtask) in subtasks.iter().take(5).enumerate() {
                        let status_icon = if subtask.done { "✓" } else { "○" };
                        let status_color = if subtask.done { Color::Green } else { Color::Gray };

                        details_lines.push(Line::from(vec![
                            Span::raw("  • "),
                            Span::styled(status_icon, Style::default().fg(status_color)),
                            Span::raw(format!(" {}", subtask.title))
                        ]));
                    }

                    if subtasks.len() > 5 {
                        details_lines.push(Line::from(vec![
                            Span::raw("  "),
                            Span::styled(format!("... and {} more", subtasks.len() - 5),
                                Style::default().fg(Color::DarkGray))
                        ]));
                    }

                    details_lines.push(Line::from(""));
                }
            }
        }

        if let Some(hex_color) = &task.hex_color {
            if !hex_color.is_empty() {
                let color = hex_to_color(hex_color);
                details_lines.push(Line::from(vec![
                    Span::styled("Color: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("●", Style::default().fg(color)),
                    Span::raw(format!(" {}", hex_color))
                ]));
                details_lines.push(Line::from(""));
            }
        }

        details_lines.push(Line::from(vec![
            Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(if task.done { "Completed" } else { "Pending" })
        ]));
        details_lines.push(Line::from(""));

        {
            let (priority_color, priority_label) = match task.priority {
                Priority::NoPriority => (None, None),
                Priority::Low => (Some(Color::Blue), Some("Low")),
                Priority::Medium => (Some(Color::Yellow), Some("Medium")),
                Priority::High => (Some(Color::Rgb(255, 165, 0)), Some("High")),
                Priority::Urgent => (Some(Color::Red), Some("Urgent")),
            };
            if let (Some(pc), Some(pl)) = (priority_color, priority_label) {
                details_lines.push(Line::from(vec![
                    Span::styled("Priority: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("●", Style::default().fg(pc)),
                    Span::raw(format!(" {}", pl))
                ]));
                details_lines.push(Line::from(""));
            }
        }

        if let Some(due_date) = &task.due_date {
            if due_date.year() > 1900 {
                let local_dt = due_date.with_timezone(&Local);
                let now = Local::now();
                let days = local_dt.date_naive().signed_duration_since(now.date_naive()).num_days();
                let rel = if days == 0 {
                    "Today".to_string()
                } else if days > 0 {
                    format!("in {}d", days)
                } else {
                    format!("{}d ago", -days)
                };
                let cal = local_dt.format("%Y-%m-%d %H:%M").to_string();
                details_lines.push(Line::from(vec![
                    Span::styled("Due Date: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(format!("{} ({})", rel, cal)),
                ]));
                details_lines.push(Line::from(""));
            }
        }

        if let Some(start_date) = &task.start_date {
            if start_date.year() > 1900 {
                let local_dt = start_date.with_timezone(&Local);
                let days = local_dt.date_naive().signed_duration_since(Local::now().date_naive()).num_days();
                let rel = if days == 0 {
                    "Today".to_string()
                } else if days > 0 {
                    format!("in {}d", days)
                } else {
                    format!("{}d ago", -days)
                };
                let cal = local_dt.format("%Y-%m-%d %H:%M").to_string();
                details_lines.push(Line::from(vec![
                    Span::styled("Start Date: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(format!("{} ({})", rel, cal)),
                ]));
                details_lines.push(Line::from(""));
            }
        }

        if let Some(labels) = &task.labels {
            if !labels.is_empty() {
                let mut labels_line_spans = vec![Span::styled("Labels: ", Style::default().add_modifier(Modifier::BOLD))];
                for (i, label) in labels.iter().enumerate() {
                    let color = hex_to_color(&label.color);
                    labels_line_spans.push(Span::styled(&*label.name, Style::default().fg(color)));
                    if i < labels.len() - 1 {
                        labels_line_spans.push(Span::raw(", "));
                    }
                }
                details_lines.push(Line::from(labels_line_spans));
                details_lines.push(Line::from(""));
            }
        }

        if let Some(assignees) = &task.assignees {
            if !assignees.is_empty() {
                let mut assignees_line_spans = vec![Span::styled("Assignees: ", Style::default().add_modifier(Modifier::BOLD))];
                for (i, assignee) in assignees.iter().enumerate() {
                    let display_name = if !assignee.name.is_empty() {
                        assignee.name.clone()
                    } else {
                        assignee.email.clone()
                    };
                    assignees_line_spans.push(Span::styled(display_name, Style::default().fg(Color::Cyan)));
                    if i < assignees.len() - 1 {
                        assignees_line_spans.push(Span::raw(", "));
                    }
                }
                details_lines.push(Line::from(assignees_line_spans));
                details_lines.push(Line::from(""));
            }
        }

        if let Some(comments) = &task.comments {
            if !comments.is_empty() {
                details_lines.push(Line::from(vec![
                    Span::styled("Comments: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("💬", Style::default().fg(Color::Green)),
                    Span::raw(format!(" {} comment(s)", comments.len()))
                ]));
                details_lines.push(Line::from(""));

                for (_i, comment) in comments.iter().enumerate() {
                    let author = if let Some(user) = &comment.user {
                        if !user.name.is_empty() {
                            user.name.clone()
                        } else {
                            user.email.clone()
                        }
                    } else {
                        "Unknown user".to_string()
                    };
                    let date_str = comment.created_at.with_timezone(&Local)
                        .format("%Y-%m-%d %H:%M").to_string();
                    let text = &comment.content;

                    details_lines.push(Line::from(vec![
                        Span::raw("  ─ "),
                        Span::styled(author, Style::default().fg(Color::Cyan)),
                        Span::raw("  "),
                        Span::styled(date_str, Style::default().fg(Color::DarkGray)),
                    ]));
                    details_lines.push(Line::from(vec![
                        Span::raw("     "),
                        Span::raw(text),
                    ]));
                    details_lines.push(Line::from(""));
                }
            }
        }

        if let Some(user_id) = &task.user_id {
            if !user_id.is_empty() {
                details_lines.push(Line::from(vec![
                    Span::styled("Created by: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(format!("User {}", user_id), Style::default().fg(Color::Cyan))
                ]));
            }
        }

        if let Some(created) = &task.created {
            if !created.is_empty() {
                if let Ok(parsed_date) = chrono::DateTime::parse_from_rfc3339(created) {
                    let local_dt = parsed_date.with_timezone(&Local);
                    let days = local_dt.date_naive().signed_duration_since(Local::now().date_naive()).num_days();
                    let rel = if days == 0 {
                        "Today".to_string()
                    } else if days > 0 {
                        format!("in {}d", days)
                    } else {
                        format!("{}d ago", -days)
                    };
                    let cal = local_dt.format("%Y-%m-%d %H:%M:%S").to_string();
                    details_lines.push(Line::from(vec![
                        Span::styled("Created: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(format!("{} ({})", rel, cal)),
                    ]));
                } else {
                    details_lines.push(Line::from(vec![
                        Span::styled("Created: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(created)
                    ]));
                }
                details_lines.push(Line::from(""));
            }
        }

        if let Some(updated) = &task.updated {
            if !updated.is_empty() {
                if let Ok(parsed_date) = chrono::DateTime::parse_from_rfc3339(updated) {
                    let local_dt = parsed_date.with_timezone(&Local);
                    let days = local_dt.date_naive().signed_duration_since(Local::now().date_naive()).num_days();
                    let rel = if days == 0 {
                        "Today".to_string()
                    } else if days > 0 {
                        format!("in {}d", days)
                    } else {
                        format!("{}d ago", -days)
                    };
                    let cal = local_dt.format("%Y-%m-%d %H:%M:%S").to_string();
                    details_lines.push(Line::from(vec![
                        Span::styled("Updated: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(format!("{} ({})", rel, cal)),
                    ]));
                } else {
                    details_lines.push(Line::from(vec![
                        Span::styled("Updated: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(updated)
                    ]));
                }
                details_lines.push(Line::from(""));
            }
        }

        if task.number > 0 {
            details_lines.push(Line::from(vec![
                Span::styled("ID: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{} (#{}) ", task.id, task.number))
            ]));
        } else {
            details_lines.push(Line::from(vec![
                Span::styled("ID: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&task.id)
            ]));
        }

        details_lines
    } else {
        vec![Line::from("No task selected")]
    };
    let paragraph = Paragraph::new(details)
        .block(Block::default().borders(Borders::ALL).title("Task Details"))
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}
