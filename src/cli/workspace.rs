use clap::Command;

use crate::opus_client::OpusClient;

use super::output;

pub fn subcommand() -> Command {
    Command::new("workspace")
        .about("Manage workspaces")
        .subcommand_required(true)
        .subcommand(
            Command::new("list")
                .about("List available workspaces")
                .args(output::output_args()),
        )
        .subcommand(
            Command::new("current")
                .about("Show current workspace"),
        )
        .subcommand(
            Command::new("switch")
                .about("Switch to a different workspace")
                .arg(
                    clap::Arg::new("name")
                        .required(true)
                        .help("Workspace name or ID to switch to"),
                ),
        )
}

pub async fn handle(
    client: &OpusClient,
    matches: &clap::ArgMatches,
    current_workspace_id: &str,
    config: &mut Option<crate::config::OpusConfig>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match matches.subcommand() {
        Some(("list", sub)) => {
            let mode = output::output_mode_from_args(sub);
            let workspaces = client.get_workspaces().await?;

            match mode {
                output::OutputMode::Json => {
                    let json = serde_json::to_string_pretty(&workspaces)
                        .unwrap_or_else(|_| "[]".to_string());
                    println!("{json}");
                }
                output::OutputMode::Quiet => {
                    for w in &workspaces {
                        println!("{}", w.id);
                    }
                }
                output::OutputMode::Human => {
                    if workspaces.is_empty() {
                        println!("\x1b[2mNo workspaces found.\x1b[0m");
                        return Ok(());
                    }

                    println!(
                        "\x1b[2m{}  {}  ID\x1b[0m",
                        pad_right("", 3),
                        pad_right("NAME", 30),
                    );
                    for w in &workspaces {
                        let marker = if w.id == current_workspace_id { " *" } else { "  " };
                        let color = if w.id == current_workspace_id { "\x1b[0;32m\x1b[1m" } else { "" };
                        let reset = if w.id == current_workspace_id { "\x1b[0m" } else { "" };
                        println!(
                            "{}{}{} {}  \x1b[2m{}\x1b[0m",
                            color, marker, reset,
                            pad_right(&w.name, 30),
                            w.id,
                        );
                    }
                }
            }
            Ok(())
        }
        Some(("current", _sub)) => {
            let workspaces = client.get_workspaces().await?;
            if let Some(w) = workspaces.iter().find(|w| w.id == current_workspace_id) {
                println!("{} ({})", w.name, w.id);
            } else if !current_workspace_id.is_empty() {
                println!("{}", current_workspace_id);
            } else {
                println!("No workspace configured");
            }
            Ok(())
        }
        Some(("switch", sub)) => {
            let name = sub.get_one::<String>("name").unwrap();
            let workspaces = client.get_workspaces().await?;

            let found = workspaces.iter().find(|w| {
                w.id == *name
                    || w.name.eq_ignore_ascii_case(name)
                    || w.slug.eq_ignore_ascii_case(name)
            });

            match found {
                Some(w) => {
                    if let Some(ref mut cfg) = config {
                        cfg.workspace_id = Some(w.id.clone());
                        if let Err(e) = cfg.save() {
                            eprintln!("Switched to '{}' but failed to save config: {}", w.name, e);
                            return Ok(());
                        }
                    }
                    println!(
                        "\x1b[0;32m\x1b[1m\u{2713}\x1b[0m Switched to workspace: {} ({})",
                        w.name, w.id
                    );
                    Ok(())
                }
                None => {
                    eprintln!("Workspace '{}' not found. Available workspaces:", name);
                    for w in &workspaces {
                        eprintln!("  - {} ({})", w.name, w.id);
                    }
                    std::process::exit(1);
                }
            }
        }
        _ => Ok(()),
    }
}

fn pad_right(s: &str, width: usize) -> String {
    if s.len() >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - s.len()))
    }
}
