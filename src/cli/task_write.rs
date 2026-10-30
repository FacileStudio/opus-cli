use clap::{Arg, ArgAction, ArgMatches, Command};

use super::output;
use super::task_args::patch_from_args;
use crate::opus_client::OpusClient;

type Res<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn text_arg(name: &'static str, help: &'static str) -> Arg {
    Arg::new(name).long(name).value_name("VALUE").help(help)
}

fn field_args() -> Vec<Arg> {
    vec![
        text_arg("description", "Description (empty string clears it)"),
        text_arg("status", "Status slug (column slug of the project)"),
        text_arg("priority", "no-priority, low, medium, high or urgent"),
        text_arg("project", "Project name or ID"),
        text_arg(
            "start",
            "Start date (YYYY-MM-DD, RFC 3339, natural text, or none)",
        ),
        text_arg(
            "due",
            "Due date (YYYY-MM-DD, RFC 3339, natural text, or none)",
        ),
        text_arg(
            "assignee",
            "Assignee user ID or member name (none to unassign)",
        ),
    ]
}

pub fn add_command() -> Command {
    Command::new("add")
        .about("Create a new task")
        .args(output::output_args())
        .arg(Arg::new("text").required(true).help(
            "Task text (supports +project *label !priority due:date syntax); flags override it",
        ))
        .args(field_args())
        .arg(text_arg("label", "Label name, repeatable").action(ArgAction::Append))
}

pub fn update_command() -> Command {
    Command::new("update")
        .about("Update fields of a task; fields not passed stay untouched")
        .args(output::output_args())
        .arg(Arg::new("id").required(true).help("Task ID"))
        .arg(text_arg("title", "New title"))
        .args(field_args())
        .arg(text_arg("add-label", "Label name to add, repeatable").action(ArgAction::Append))
        .arg(text_arg("remove-label", "Label name to remove, repeatable").action(ArgAction::Append))
}

pub async fn handle_add(client: &OpusClient, args: &ArgMatches, default_project: &str) -> Res<()> {
    let mode = output::output_mode_from_args(args);
    let text = args.get_one::<String>("text").unwrap();
    let patch = patch_from_args(args)?;

    let project_id = match &patch.project {
        Some(wanted) => client.resolve_project_id(wanted).await?,
        None => client
            .find_or_get_project_id(default_project)
            .await?
            .ok_or_else(|| format!("default project '{}' not found", default_project))?,
    };

    let created = client.create_task_with_magic(text, &project_id).await?;
    let task = if patch.is_empty() {
        client.get_task(&created.id).await?
    } else {
        client.patch_task(&created.id, &patch).await?
    };
    output::print_saved_task(&task, &mode, "Created");
    Ok(())
}

pub async fn handle_update(client: &OpusClient, args: &ArgMatches) -> Res<()> {
    let mode = output::output_mode_from_args(args);
    let id = args.get_one::<String>("id").unwrap();
    let patch = patch_from_args(args)?;
    if patch.is_empty() {
        return Err("nothing to update: pass at least one field flag".into());
    }
    let task = client.patch_task(id, &patch).await?;
    output::print_saved_task(&task, &mode, "Updated");
    Ok(())
}
