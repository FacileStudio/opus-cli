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
                        .help("Workspace name, slug, or ID to switch to"),
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
                        crate::ui::step("No workspaces");
                        return Ok(());
                    }

                    println!("{}", crate::ui::dim("   NAME                            ID"));
                    for w in &workspaces {
                        let is_current = w.id == current_workspace_id;
                        let marker = if is_current { " *" } else { "  " };
                        if is_current {
                            println!("{} {:<30}  {}", crate::ui::green(marker), w.name, crate::ui::dim(&w.id.to_string()));
                        } else {
                            println!("{marker} {:<30}  {}", w.name, crate::ui::dim(&w.id.to_string()));
                        }
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
                        cfg.save().map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;
                    }
                    crate::ui::success(&format!(
                        "Switched to workspace: {} ({})",
                        w.name, w.id
                    ));
                    Ok(())
                }
                None => {
                    crate::ui::error(&format!("workspace '{}' not found — available workspaces:", name));
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
