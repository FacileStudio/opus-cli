use clap::{Arg, Command};

use crate::opus_client::OpusClient;

pub fn subcommand() -> Command {
    Command::new("workspace")
        .about("Manage workspaces")
        .subcommand_required(true)
        .subcommand(
            Command::new("list")
                .about("List configured workspaces"),
        )
        .subcommand(
            Command::new("current")
                .about("Show current workspace"),
        )
        .subcommand(
            Command::new("switch")
                .about("Switch to a different workspace")
                .arg(
                    Arg::new("name")
                        .required(true)
                        .help("Workspace name or ID to switch to"),
                ),
        )
        .subcommand(
            Command::new("add")
                .about("Add a workspace to config")
                .arg(
                    Arg::new("id")
                        .required(true)
                        .help("Workspace ID"),
                )
                .arg(
                    Arg::new("name")
                        .long("name")
                        .short('n')
                        .required(true)
                        .help("Display name for this workspace"),
                ),
        )
        .subcommand(
            Command::new("remove")
                .about("Remove a workspace from config")
                .arg(
                    Arg::new("name")
                        .required(true)
                        .help("Workspace name or ID to remove"),
                ),
        )
}

const GREEN: &str = "\x1b[0;32m\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

pub async fn handle(
    _client: &OpusClient,
    matches: &clap::ArgMatches,
    current_workspace_id: &str,
    config: &mut Option<crate::config::OpusConfig>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match matches.subcommand() {
        Some(("list", _sub)) => {
            let cfg = config.as_ref().ok_or("No config loaded")?;
            let workspaces = cfg.get_workspaces();

            if workspaces.is_empty() {
                println!("{DIM}No workspaces configured. Add with: opus workspace add <id> --name <name>{RESET}");
                return Ok(());
            }

            println!("{DIM}   NAME                            ID{RESET}");
            for w in &workspaces {
                let is_current = w.id == current_workspace_id;
                let marker = if is_current { " *" } else { "  " };
                if is_current {
                    println!("{GREEN}{marker}{RESET} {:<30}  {DIM}{}{RESET}", w.name, w.id);
                } else {
                    println!("{marker} {:<30}  {DIM}{}{RESET}", w.name, w.id);
                }
            }
            Ok(())
        }
        Some(("current", _sub)) => {
            let cfg = config.as_ref().ok_or("No config loaded")?;
            let workspaces = cfg.get_workspaces();
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
            let cfg = config.as_mut().ok_or("No config loaded")?;

            let found = cfg.find_workspace(name).cloned();
            match found {
                Some(w) => {
                    cfg.workspace_id = Some(w.id.clone());
                    cfg.save().map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;
                    println!("{GREEN}\u{2713}{RESET} Switched to workspace: {} ({})", w.name, w.id);
                    Ok(())
                }
                None => {
                    eprintln!("Workspace '{}' not found in config. Available:", name);
                    for w in &cfg.get_workspaces() {
                        eprintln!("  - {} ({})", w.name, w.id);
                    }
                    eprintln!("\nAdd with: opus workspace add <id> --name <name>");
                    std::process::exit(1);
                }
            }
        }
        Some(("add", sub)) => {
            let id = sub.get_one::<String>("id").unwrap().clone();
            let name = sub.get_one::<String>("name").unwrap().clone();
            let cfg = config.as_mut().ok_or("No config loaded")?;

            if cfg.find_workspace(&id).is_some() {
                println!("Workspace '{}' already exists in config", id);
                return Ok(());
            }

            cfg.add_workspace(id.clone(), name.clone());
            cfg.save().map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;
            println!("{GREEN}\u{2713}{RESET} Added workspace: {} ({})", name, id);
            Ok(())
        }
        Some(("remove", sub)) => {
            let name = sub.get_one::<String>("name").unwrap();
            let cfg = config.as_mut().ok_or("No config loaded")?;

            if cfg.remove_workspace(name) {
                cfg.save().map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;
                println!("{GREEN}\u{2713}{RESET} Removed workspace: {}", name);
            } else {
                eprintln!("Workspace '{}' not found in config", name);
                std::process::exit(1);
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
