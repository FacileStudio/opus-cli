#![allow(dead_code, unused_variables, unreachable_patterns, unused_assignments)]

mod ui;
use clap::{Arg, Command};

mod tui;
mod opus;
mod opus_client;
mod opus_parser;
mod debug;
mod config;
mod cli;

mod ui_loop;
mod url_utils;

use crate::debug::debug_log;
use crate::ui_loop::run_ui;

fn resolve_settings(
    matches: &clap::ArgMatches,
    workspace_override: Option<&str>,
) -> config::Settings {
    match config::Settings::resolve(
        matches.get_one::<String>("config").map(|s| s.as_str()),
        workspace_override,
        matches.get_flag("dev-env"),
    ) {
        Ok(settings) => settings,
        Err(e) => {
            ui::error(&e);
            std::process::exit(1);
        }
    }
}

fn main() {
    dotenv::dotenv().ok();

    let matches = Command::new("opus")
        .about("Terminal client for Opus project management")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::new("config")
                .long("config")
                .short('c')
                .help("Path to config file")
                .value_name("FILE")
        )
        .arg(
            Arg::new("dev-env")
                .long("dev-env")
                .help("Run without a config file, falling back to localhost defaults")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("quick")
                .long("quick")
                .help("Quick add a task: --quick \"new task *label +project due:tomorrow\"")
                .value_name("TASK_STRING")
                .num_args(1)
        )
        .arg(
            Arg::new("workspace")
                .long("workspace")
                .short('w')
                .help("Override workspace ID for this session")
                .value_name("ID")
        )
        .arg(
            Arg::new("no-color")
                .long("no-color")
                .help("Disable colored output")
                .global(true)
                .action(clap::ArgAction::SetTrue)
        )
        .subcommand(cli::task::subcommand())
        .subcommand(cli::workspace::subcommand())
        .subcommand(cli::keys::subcommand())
        .get_matches();

    if matches.get_flag("no-color") {
        ui::disable_color();
    }

    let workspace_override = matches.get_one::<String>("workspace").cloned();

    if let Some(quick_str) = matches.get_one::<String>("quick") {
        let config::Settings {
            api_url,
            api_key,
            workspace_id,
            default_project,
            ..
        } = resolve_settings(&matches, workspace_override.as_deref());

        let result = tokio::runtime::Runtime::new().unwrap().block_on(async {
            let api_client = crate::opus_client::OpusClient::new(
                api_url.clone(),
                api_key.clone(),
                workspace_id.clone(),
            );
            let default_project_id = match api_client.find_or_get_project_id(&default_project).await {
                Ok(Some(id)) => id,
                _ => String::new(),
            };
            match api_client.create_task_with_magic(quick_str, &default_project_id).await {
                Ok(task) => {
                    println!("Task created: {} (ID: {})", task.title, task.id);
                    Ok(())
                },
                Err(e) => {
                    eprintln!("Failed to create task: {}", e);
                    Err(())
                }
            }
        });
        std::process::exit(if result.is_ok() { 0 } else { 1 });
    }

    if let Some(("task", sub_matches)) = matches.subcommand() {
        let config::Settings {
            api_url,
            api_key,
            workspace_id,
            default_project,
            ..
        } = resolve_settings(&matches, workspace_override.as_deref());

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let client = crate::opus_client::OpusClient::new(api_url, api_key, workspace_id);
            cli::task::handle(&client, sub_matches, &default_project).await
        });
        if let Err(e) = &result {
            ui::error(&format!("{e}"));
        }
        std::process::exit(if result.is_ok() { 0 } else { 1 });
    }

    if let Some(("workspace", sub_matches)) = matches.subcommand() {
        let config::Settings {
            api_url,
            api_key,
            workspace_id,
            mut config,
            ..
        } = resolve_settings(&matches, workspace_override.as_deref());

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let client = crate::opus_client::OpusClient::new(api_url, api_key, workspace_id.clone());
            cli::workspace::handle(&client, sub_matches, &workspace_id, &mut config).await
        });
        if let Err(e) = &result {
            ui::error(&format!("{e}"));
        }
        std::process::exit(if result.is_ok() { 0 } else { 1 });
    }

    if let Some(("keys", sub_matches)) = matches.subcommand() {
        let config::Settings {
            api_url,
            api_key,
            workspace_id,
            ..
        } = resolve_settings(&matches, workspace_override.as_deref());

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let client = crate::opus_client::OpusClient::new(api_url, api_key, workspace_id);
            cli::keys::handle(&client, sub_matches).await
        });
        if let Err(e) = &result {
            ui::error(&format!("{e}"));
        }
        std::process::exit(if result.is_ok() { 0 } else { 1 });
    }

    crate::debug::clear_debug_log();
    debug_log("Starting Opus application");
    debug_log(&format!("Environment variables:"));
    debug_log(&format!("  OPUS_API_URL: {:?}", std::env::var("OPUS_API_URL")));
    debug_log(&format!("  OPUS_API_KEY: {:?}", std::env::var("OPUS_API_KEY").map(|t| format!("{}...", &t[..t.len().min(8)]))));
    debug_log(&format!("  OPUS_WORKSPACE_ID: {:?}", std::env::var("OPUS_WORKSPACE_ID")));
    debug_log(&format!("  OPUS_DEFAULT_PROJECT: {:?}", std::env::var("OPUS_DEFAULT_PROJECT")));

    let config::Settings {
        api_url,
        api_key,
        workspace_id,
        default_project,
        config,
    } = resolve_settings(&matches, workspace_override.as_deref());
    debug_log(&format!("Resolved api_url={api_url}, api_key=***"));

    if let Err(e) = tokio_main(api_url, api_key, workspace_id, default_project, config) {
        ui::error(&format!("{e}"));
        std::process::exit(1);
    }
}

#[tokio::main]
async fn tokio_main(
    api_url: String,
    api_key: String,
    workspace_id: String,
    default_project: String,
    config: Option<crate::config::OpusConfig>,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use crate::tui::app::state::App;
    use crate::opus_client::OpusClient;
    use crate::debug::debug_log;

    let api_client = Arc::new(Mutex::new(OpusClient::new(api_url, api_key, workspace_id)));

    let config_clone = config.clone();
    let app = Arc::new(Mutex::new(App::new_with_config(config.unwrap_or_default(), default_project.clone())));

    {
        let api_client_guard = api_client.lock().await;
        match api_client_guard.test_connection().await {
            Ok(true) => {
                debug_log("SUCCESS: Connected to Opus API");
            }
            Ok(false) => {
                debug_log("WARNING: Failed to connect to Opus API");
                debug_log("The app requires a connection to the api.");
            }
            Err(e) => {
                debug_log(&format!("WARNING: Failed to connect to Opus API: {}", e));
                debug_log("The app requires a connection to the api.");
            }
        }
    }

    let client_clone = api_client.clone();

    let workspaces = client_clone.lock().await.get_workspaces().await.unwrap_or_default();
    debug_log(&format!("Fetched {} workspaces from API", workspaces.len()));
    {
        let mut app_guard = app.lock().await;
        app_guard.set_available_workspaces(workspaces);
    }

    let (tasks, project_map, project_colors) = client_clone.lock().await.get_tasks_with_projects().await.unwrap_or_default();
    debug_log(&format!("Fetched {} tasks from API", tasks.len()));
    let all_labels = client_clone.lock().await.get_all_labels().await.unwrap_or_default();
    debug_log(&format!("Fetched {} labels from API", all_labels.len()));
    if let Some(first) = tasks.get(0) {
        debug_log(&format!("First task: {:?}", first));
    } else {
        debug_log("No tasks returned from API");
    }
    let filters = client_clone.lock().await.get_saved_filters().await.unwrap_or_default();
    debug_log(&format!("Fetched {} saved filters from backend", filters.len()));
    {
        let mut app_guard = app.lock().await;
        app_guard.update_all_tasks(tasks);
        app_guard.project_map = project_map;
        app_guard.project_colors = project_colors;
        app_guard.set_filters(filters);
        for label in all_labels {
            app_guard.label_map.insert(label.id.clone(), label.name.clone());
            app_guard.label_colors.insert(label.id.clone(), label.color.clone());
        }

        if let Some(ref config) = config_clone {
            drop(app_guard);
            let mut app_guard = app.lock().await;
            app_guard.apply_default_filter_from_config(config, &client_clone).await;
        }

        let app_guard = app.lock().await;
        debug_log(&format!("App all_tasks count: {}", app_guard.all_tasks.len()));
        debug_log(&format!("App tasks count after filter: {}", app_guard.tasks.len()));
        debug_log(&format!("App project_map: {:?}", app_guard.project_map));
        debug_log(&format!("App filters: {:?}", app_guard.filters));
    }

    debug_log("=== ABOUT TO CALL run_ui ===");
    run_ui(app.clone(), client_clone.clone()).await?;
    debug_log("=== run_ui RETURNED ===");

    Ok(())
}
