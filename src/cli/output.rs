use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::opus::models::{Comment, Priority, Task};

pub fn output_args() -> [clap::Arg; 2] {
    [
        clap::Arg::new("json")
            .long("json")
            .action(clap::ArgAction::SetTrue)
            .help("Output as JSON"),
        clap::Arg::new("quiet")
            .long("quiet")
            .short('q')
            .action(clap::ArgAction::SetTrue)
            .help("Output only IDs"),
    ]
}

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";

#[derive(Debug, Clone, PartialEq)]
pub enum OutputMode {
    Human,
    Json,
    Quiet,
}

pub fn output_mode_from_args(args: &clap::ArgMatches) -> OutputMode {
    if args.get_flag("json") {
        OutputMode::Json
    } else if args.get_flag("quiet") {
        OutputMode::Quiet
    } else {
        OutputMode::Human
    }
}

pub fn print_tasks(tasks: &[Task], mode: &OutputMode, project_map: &HashMap<String, String>) {
    match mode {
        OutputMode::Json => {
            let json = serde_json::to_string_pretty(tasks).unwrap_or_else(|_| "[]".to_string());
            println!("{json}");
        }
        OutputMode::Quiet => {
            for task in tasks {
                println!("{}", task.id);
            }
        }
        OutputMode::Human => {
            if tasks.is_empty() {
                println!("{DIM}No tasks found.{RESET}");
                return;
            }
            print_task_table(tasks, project_map);
        }
    }
}

pub fn print_task_detail(
    task: &Task,
    comments: &[Comment],
    mode: &OutputMode,
    project_map: &HashMap<String, String>,
) {
    match mode {
        OutputMode::Json => {
            let mut value = serde_json::to_value(task).unwrap_or(serde_json::Value::Null);
            if let serde_json::Value::Object(ref mut map) = value {
                let comments_val =
                    serde_json::to_value(comments).unwrap_or(serde_json::Value::Array(vec![]));
                map.insert("comments".to_string(), comments_val);
            }
            let json =
                serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".to_string());
            println!("{json}");
        }
        OutputMode::Quiet => {
            println!("{}", task.id);
        }
        OutputMode::Human => {
            print_task_detail_human(task, comments, project_map);
        }
    }
}

pub fn print_created_task(task: &Task, mode: &OutputMode) {
    match mode {
        OutputMode::Json => {
            let json =
                serde_json::to_string_pretty(task).unwrap_or_else(|_| "{}".to_string());
            println!("{json}");
        }
        OutputMode::Quiet => {
            println!("{}", task.id);
        }
        OutputMode::Human => {
            println!(
                "{GREEN}Created{RESET} #{} {BOLD}{}{RESET}",
                task.number, task.title
            );
        }
    }
}

fn format_date(dt: Option<DateTime<Utc>>) -> String {
    match dt {
        Some(d) => d.format("%Y-%m-%d").to_string(),
        None => "\u{2014}".to_string(),
    }
}

fn priority_display(priority: &Priority) -> String {
    match priority {
        Priority::Urgent => format!("{RED}{BOLD}urgent{RESET}"),
        Priority::High => format!("{YELLOW}high{RESET}"),
        Priority::Medium => format!("{BLUE}medium{RESET}"),
        Priority::Low => format!("{DIM}low{RESET}"),
        Priority::NoPriority => format!("{DIM}\u{2014}{RESET}"),
    }
}

fn status_display(status: &str, done: bool) -> String {
    if done {
        format!("{GREEN}\u{2713} {status}{RESET}")
    } else {
        status.to_string()
    }
}

fn project_name(project_id: &str, project_map: &HashMap<String, String>) -> String {
    project_map
        .get(project_id)
        .cloned()
        .unwrap_or_else(|| "?".to_string())
}

fn visible_len(s: &str) -> usize {
    let mut inside_escape = false;
    let mut count = 0usize;
    for ch in s.chars() {
        if ch == '\x1b' {
            inside_escape = true;
            continue;
        }
        if inside_escape {
            if ch == 'm' {
                inside_escape = false;
            }
            continue;
        }
        count += 1;
    }
    count
}

fn pad_right(s: &str, width: usize) -> String {
    let vis = visible_len(s);
    if vis >= width {
        s.to_string()
    } else {
        format!("{s}{}", " ".repeat(width - vis))
    }
}

fn truncate_visible(s: &str, max: usize) -> String {
    let mut inside_escape = false;
    let mut count = 0usize;
    let mut result = String::new();
    for ch in s.chars() {
        if ch == '\x1b' {
            inside_escape = true;
            result.push(ch);
            continue;
        }
        if inside_escape {
            result.push(ch);
            if ch == 'm' {
                inside_escape = false;
            }
            continue;
        }
        if count >= max {
            break;
        }
        result.push(ch);
        count += 1;
    }
    result.push_str(RESET);
    result
}

fn print_task_table(tasks: &[Task], project_map: &HashMap<String, String>) {
    struct Row {
        num: String,
        title: String,
        project: String,
        priority: String,
        status: String,
        due: String,
    }

    let rows: Vec<Row> = tasks
        .iter()
        .map(|t| Row {
            num: format!("#{}", t.number),
            title: t.title.clone(),
            project: project_name(&t.project_id, project_map),
            priority: priority_display(&t.priority),
            status: status_display(&t.status, t.done),
            due: format_date(t.due_date),
        })
        .collect();

    let col_num = rows.iter().map(|r| r.num.len()).max().unwrap_or(3).max(3);
    let col_proj = rows
        .iter()
        .map(|r| r.project.len())
        .max()
        .unwrap_or(7)
        .max(7)
        .min(20);
    let col_pri = 12;
    let col_status = rows
        .iter()
        .map(|r| visible_len(&r.status))
        .max()
        .unwrap_or(6)
        .max(6)
        .min(16);
    let col_due = 10;

    let term_width = terminal_width().unwrap_or(100);
    let fixed = col_num + col_proj + col_pri + col_status + col_due + 10;
    let col_title = if term_width > fixed + 10 {
        term_width - fixed
    } else {
        30
    };

    println!(
        "{DIM}{}  {}  {}  {}  {}  {}{RESET}",
        pad_right("#", col_num),
        pad_right("TITLE", col_title),
        pad_right("PROJECT", col_proj),
        pad_right("PRIORITY", col_pri),
        pad_right("STATUS", col_status),
        pad_right("DUE", col_due),
    );

    for row in &rows {
        let title_truncated = if row.title.len() > col_title {
            format!("{}…", &row.title[..col_title.saturating_sub(1)])
        } else {
            row.title.clone()
        };

        println!(
            "{CYAN}{}{RESET}  {}  {}  {}  {}  {}",
            pad_right(&row.num, col_num),
            pad_right(&title_truncated, col_title),
            pad_right(&row.project, col_proj),
            pad_right(&truncate_visible(&row.priority, col_pri), col_pri),
            pad_right(&row.status, col_status),
            row.due,
        );
    }
}

fn print_task_detail_human(
    task: &Task,
    comments: &[Comment],
    project_map: &HashMap<String, String>,
) {
    println!(
        "{BOLD}#{} {}{RESET}",
        task.number, task.title
    );
    println!();

    let fields: Vec<(&str, String)> = vec![
        ("Project", project_name(&task.project_id, project_map)),
        ("Status", status_display(&task.status, task.done)),
        ("Priority", priority_display(&task.priority)),
        ("Due", format_date(task.due_date)),
        ("Start", format_date(task.start_date)),
        ("Created", task.created_at.format("%Y-%m-%d").to_string()),
        ("Updated", task.updated_at.format("%Y-%m-%d").to_string()),
    ];

    for (label, value) in &fields {
        println!("  {DIM}{:<10}{RESET} {value}", format!("{label}:"));
    }

    if let Some(ref assignees) = task.assignees {
        if !assignees.is_empty() {
            let names: Vec<&str> = assignees.iter().map(|u| u.name.as_str()).collect();
            println!("  {DIM}{:<10}{RESET} {}", "Assignees:", names.join(", "));
        }
    }

    if let Some(ref labels) = task.labels {
        if !labels.is_empty() {
            let names: Vec<&str> = labels.iter().map(|l| l.name.as_str()).collect();
            println!("  {DIM}{:<10}{RESET} {}", "Labels:", names.join(", "));
        }
    }

    if let Some(ref desc) = task.description {
        let trimmed = desc.trim();
        if !trimmed.is_empty() {
            println!();
            println!("{DIM}\u{2500}\u{2500} Description \u{2500}\u{2500}{RESET}");
            for line in trimmed.lines() {
                println!("  {line}");
            }
        }
    }

    if !comments.is_empty() {
        println!();
        println!(
            "{DIM}\u{2500}\u{2500} Comments ({}) \u{2500}\u{2500}{RESET}",
            comments.len()
        );
        for c in comments {
            let author = c
                .user
                .as_ref()
                .map(|u| u.name.as_str())
                .unwrap_or("unknown");
            let date = c.created_at.format("%Y-%m-%d %H:%M");
            println!();
            println!("  {BOLD}{author}{RESET}  {DIM}{date}{RESET}");
            for line in c.content.trim().lines() {
                println!("  {line}");
            }
        }
    }
}

fn terminal_width() -> Option<usize> {
    crossterm::terminal::size().ok().map(|(w, _)| w as usize)
}
