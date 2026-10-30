use clap::ArgMatches;

use super::task_values::{parse_assignee, parse_when};
use crate::opus::models::Priority;
use crate::opus_client::task_patch::TaskPatch;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::task_write::{add_command, update_command};

    fn parse(argv: &[&str]) -> ArgMatches {
        let mut full = vec!["opus", "update", "7"];
        full.extend_from_slice(argv);
        update_command().get_matches_from(&full[1..])
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
