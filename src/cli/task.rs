use std::collections::HashMap;

use clap::{Arg, ArgAction, Command};

use crate::opus_client::OpusClient;

use super::output;

pub fn subcommand() -> Command {
    Command::new("task")
        .about("Manage tasks")
        .subcommand_required(true)
        .subcommand(
            Command::new("list")
                .about("List tasks")
                .args(output::output_args())
                .arg(
                    Arg::new("project")
                        .long("project")
                        .help("Filter by project name (case-insensitive substring match)"),
                )
                .arg(
                    Arg::new("label")
                        .long("label")
                        .help("Filter by label name"),
                )
                .arg(
                    Arg::new("priority")
                        .long("priority")
                        .help("Filter by priority (no-priority, low, medium, high, urgent)"),
                )
                .arg(
                    Arg::new("status")
                        .long("status")
                        .help("Filter by status slug"),
                )
                .arg(
                    Arg::new("overdue")
                        .long("overdue")
                        .action(ArgAction::SetTrue)
                        .help("Show only overdue tasks"),
                )
                .arg(
                    Arg::new("done")
                        .long("done")
                        .action(ArgAction::SetTrue)
                        .help("Show only completed tasks"),
                )
                .arg(
                    Arg::new("limit")
                        .long("limit")
                        .value_parser(clap::value_parser!(usize))
                        .default_value("50")
                        .help("Maximum number of tasks to display"),
                ),
        )
        .subcommand(
            Command::new("show")
                .about("Show task details")
                .args(output::output_args())
                .arg(Arg::new("id").required(true).help("Task ID")),
        )
        .subcommand(
            Command::new("add")
                .about("Create a new task")
                .args(output::output_args())
                .arg(
                    Arg::new("text")
                        .required(true)
                        .help("Task text (supports +project *label !priority due:date syntax)"),
                ),
        )
        .subcommand(
            Command::new("done")
                .about("Mark one or more tasks as done")
                .arg(
                    Arg::new("ids")
                        .required(true)
                        .num_args(1..)
                        .help("Task ID(s) to mark as done"),
                ),
        )
        .subcommand(
            Command::new("undone")
                .about("Mark one or more tasks as to-do")
                .arg(
                    Arg::new("ids")
                        .required(true)
                        .num_args(1..)
                        .help("Task ID(s) to mark as to-do"),
                ),
        )
        .subcommand(
            Command::new("delete")
                .about("Delete one or more tasks")
                .arg(
                    Arg::new("ids")
                        .required(true)
                        .num_args(1..)
                        .help("Task ID(s) to delete"),
                ),
        )
}

pub async fn handle(
    client: &OpusClient,
    matches: &clap::ArgMatches,
    default_project: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match matches.subcommand() {
        Some(("list", sub)) => handle_list(client, sub).await,
        Some(("show", sub)) => handle_show(client, sub).await,
        Some(("add", sub)) => handle_add(client, sub, default_project).await,
        Some(("done", sub)) => handle_set_status(client, sub, "done").await,
        Some(("undone", sub)) => handle_set_status(client, sub, "to-do").await,
        Some(("delete", sub)) => handle_delete(client, sub).await,
        _ => unreachable!(),
    }
}

async fn handle_list(
    client: &OpusClient,
    args: &clap::ArgMatches,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mode = output::output_mode_from_args(args);
    let (tasks, project_map, _project_colors) = client.get_tasks_with_projects().await?;

    let filter_project = args.get_one::<String>("project");
    let filter_label = args.get_one::<String>("label");
    let filter_priority = args.get_one::<String>("priority");
    let filter_status = args.get_one::<String>("status");
    let filter_overdue = args.get_flag("overdue");
    let filter_done = args.get_flag("done");
    let limit = *args.get_one::<usize>("limit").unwrap_or(&50);

    let filtered: Vec<_> = tasks
        .into_iter()
        .filter(|task| {
            if let Some(proj) = filter_project {
                let proj_lower = proj.to_lowercase();
                let task_project_name = project_map
                    .get(&task.project_id)
                    .map(|n| n.to_lowercase())
                    .unwrap_or_default();
                if !task_project_name.contains(&proj_lower) {
                    return false;
                }
            }

            if let Some(label) = filter_label {
                let label_lower = label.to_lowercase();
                let has_label = task
                    .labels
                    .as_ref()
                    .map(|labels| {
                        labels
                            .iter()
                            .any(|l| l.name.to_lowercase() == label_lower)
                    })
                    .unwrap_or(false);
                if !has_label {
                    return false;
                }
            }

            if let Some(pri) = filter_priority {
                if task.priority.to_string() != *pri {
                    return false;
                }
            }

            if let Some(status) = filter_status {
                if task.status != *status {
                    return false;
                }
            }

            if filter_overdue && !task.is_overdue() {
                return false;
            }

            if filter_done && !task.done {
                return false;
            }

            true
        })
        .take(limit)
        .collect();

    output::print_tasks(&filtered, &mode, &project_map);
    Ok(())
}

async fn handle_show(
    client: &OpusClient,
    args: &clap::ArgMatches,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mode = output::output_mode_from_args(args);
    let task_id = args.get_one::<String>("id").unwrap();

    let task = client.get_task(task_id).await?;
    let comments = client.get_comments(task_id).await.unwrap_or_default();

    let projects = client.get_all_projects().await?;
    let project_map: HashMap<String, String> = projects
        .into_iter()
        .map(|p| (p.id, p.name))
        .collect();

    output::print_task_detail(&task, &comments, &mode, &project_map);
    Ok(())
}

async fn handle_add(
    client: &OpusClient,
    args: &clap::ArgMatches,
    default_project: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mode = output::output_mode_from_args(args);
    let text = args.get_one::<String>("text").unwrap();

    let default_project_id = client
        .find_or_get_project_id(default_project)
        .await?
        .ok_or_else(|| {
            format!("default project '{}' not found", default_project)
        })?;

    let task = client
        .create_task_with_magic(text, &default_project_id)
        .await?;

    output::print_created_task(&task, &mode);
    Ok(())
}

async fn handle_set_status(
    client: &OpusClient,
    args: &clap::ArgMatches,
    status: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ids: Vec<&String> = args.get_many::<String>("ids").unwrap().collect();

    for id in &ids {
        match client.set_task_status(id, status).await {
            Ok(task) => eprintln!("  {} → {}", task.title, status),
            Err(e) => eprintln!("  {} failed: {}", id, e),
        }
    }
    Ok(())
}

async fn handle_delete(
    client: &OpusClient,
    args: &clap::ArgMatches,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ids: Vec<&String> = args.get_many::<String>("ids").unwrap().collect();

    for id in &ids {
        match client.delete_task(id).await {
            Ok(()) => eprintln!("  {} deleted", id),
            Err(e) => eprintln!("  {} failed: {}", id, e),
        }
    }
    Ok(())
}
