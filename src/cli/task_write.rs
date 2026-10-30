use chrono::{DateTime, NaiveDate, Utc};
use clap::{Arg, ArgAction, ArgMatches, Command};

use crate::opus::models::Priority;
use crate::opus_client::task_write::TaskPatch;
use crate::opus_client::OpusClient;
use crate::opus_parser::QuickAddParser;

use super::output;

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

pub fn parse_when(raw: &str) -> Result<Option<DateTime<Utc>>, String> {
    let text = raw.trim();
    if text.eq_ignore_ascii_case("none") || text.eq_ignore_ascii_case("clear") {
        return Ok(None);
    }
    if let Ok(parsed) = DateTime::parse_from_rfc3339(text) {
        return Ok(Some(parsed.with_timezone(&Utc)));
    }
    if let Ok(day) = NaiveDate::parse_from_str(text, "%Y-%m-%d") {
        return Ok(day.and_hms_opt(0, 0, 0).map(|d| d.and_utc()));
    }
    QuickAddParser::new()
        .parse_date(text)
        .map(Some)
        .ok_or_else(|| format!("cannot read date '{}'", raw))
}

pub fn parse_assignee(raw: &str) -> Option<String> {
    let text = raw.trim();
    if text.eq_ignore_ascii_case("none") || text.is_empty() {
        None
    } else {
        Some(text.to_string())
    }
}

fn values(args: &ArgMatches, name: &str) -> Vec<String> {
    args.try_get_many::<String>(name)
        .ok()
        .flatten()
        .map(|v| v.cloned().collect())
        .unwrap_or_default()
}

fn value(args: &ArgMatches, name: &str) -> Option<String> {
    args.try_get_one::<String>(name).ok().flatten().cloned()
}

pub fn patch_from_args(args: &ArgMatches) -> Result<TaskPatch, String> {
    let priority = match value(args, "priority") {
        Some(p) => Some(p.parse::<Priority>()?.to_string()),
        None => None,
    };
    let start = value(args, "start").map(|v| parse_when(&v)).transpose()?;
    let due = value(args, "due").map(|v| parse_when(&v)).transpose()?;
    let mut add_labels = values(args, "label");
    add_labels.extend(values(args, "add-label"));
    Ok(TaskPatch {
        title: value(args, "title"),
        description: value(args, "description"),
        status: value(args, "status"),
        priority,
        project: value(args, "project"),
        start,
        due,
        assignee: value(args, "assignee").map(|v| parse_assignee(&v)),
        add_labels,
        remove_labels: values(args, "remove-label"),
    })
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn parse(argv: &[&str]) -> ArgMatches {
        let mut full = vec!["opus", "update", "7"];
        full.extend_from_slice(argv);
        update_command().get_matches_from(&full[1..])
    }

    #[test]
    fn iso_dates_land_on_utc_midnight() {
        let expected = Utc.with_ymd_and_hms(2026, 11, 5, 0, 0, 0).unwrap();
        assert_eq!(parse_when("2026-11-05"), Ok(Some(expected)));
    }

    #[test]
    fn rfc3339_keeps_the_instant() {
        let expected = Utc.with_ymd_and_hms(2026, 11, 5, 8, 30, 0).unwrap();
        assert_eq!(parse_when("2026-11-05T09:30:00+01:00"), Ok(Some(expected)));
    }

    #[test]
    fn none_and_clear_mean_no_date() {
        assert_eq!(parse_when("none"), Ok(None));
        assert_eq!(parse_when("CLEAR"), Ok(None));
    }

    #[test]
    fn garbage_dates_are_rejected() {
        assert!(parse_when("whenever-ish").is_err());
    }

    #[test]
    fn untouched_flags_stay_unset() {
        let patch = patch_from_args(&parse(&["--title", "T"])).unwrap();
        assert_eq!(patch.title.as_deref(), Some("T"));
        assert!(patch.start.is_none() && patch.due.is_none() && patch.assignee.is_none());
        assert!(patch.description.is_none());
    }

    #[test]
    fn clear_flags_map_to_explicit_nulls() {
        let argv = [
            "--due",
            "none",
            "--start",
            "none",
            "--assignee",
            "none",
            "--description",
            "",
        ];
        let patch = patch_from_args(&parse(&argv)).unwrap();
        assert_eq!(patch.due, Some(None));
        assert_eq!(patch.start, Some(None));
        assert_eq!(patch.assignee, Some(None));
        assert_eq!(patch.description.as_deref(), Some(""));
    }

    #[test]
    fn priority_is_validated_and_labels_collect() {
        assert!(patch_from_args(&parse(&["--priority", "huge"])).is_err());
        let argv = [
            "--priority",
            "urgent",
            "--add-label",
            "a",
            "--add-label",
            "b",
            "--remove-label",
            "c",
        ];
        let patch = patch_from_args(&parse(&argv)).unwrap();
        assert_eq!(patch.priority.as_deref(), Some("urgent"));
        assert_eq!(patch.add_labels, vec!["a", "b"]);
        assert_eq!(patch.remove_labels, vec!["c"]);
    }

    #[test]
    fn add_label_flag_is_collected_for_add() {
        let matches =
            add_command().get_matches_from(["add", "text", "--label", "x", "--label", "y"]);
        let patch = patch_from_args(&matches).unwrap();
        assert_eq!(patch.add_labels, vec!["x", "y"]);
        assert!(!patch.is_empty());
    }
}
