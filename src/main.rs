#![allow(dead_code, unused_variables, unreachable_patterns, unused_assignments)]
use clap::{Arg, Command};

mod tui;
mod opus;
mod opus_client;
mod opus_parser;
mod debug;
mod config;
mod auth;
mod first_run;
mod ui_loop;
mod url_utils;

use crate::debug::debug_log;
use crate::ui_loop::run_ui;

fn main() {
    dotenv::dotenv().ok();

    let matches = Command::new("opus")
        .about("opus-cli - Terminal User Interface for Opus project management")
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
                .help("Use environment variables instead of config file")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("login")
                .long("login")
                .help("Run the device auth flow")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("quick")
                .long("quick")
                .help("Quick add a task: --quick \"new task *label +project due:tomorrow\"")
                .value_name("TASK_STRING")
                .num_args(1)
        )
        .get_matches();

    if matches.get_flag("login") {
        let config_path = matches.get_one::<String>("config");
        let config = crate::config::OpusConfig::load_from_path(config_path.map(|s| s.as_str()));
        let api_url = config
            .as_ref()
            .map(|c| c.api_url.clone())
            .unwrap_or_else(|| {
                std::env::var("OPUS_API_URL").unwrap_or_else(|_| "http://localhost:1337".to_string())
            });
        let rt = tokio::runtime::Runtime::new().unwrap();
        match rt.block_on(crate::auth::run_device_auth_flow(&api_url)) {
            Ok(creds) => {
                println!("Login successful as {} ({})", creds.user_name, creds.user_email);
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("Login failed: {}", e);
                std::process::exit(1);
            }
        }
    }

    if let Some(quick_str) = matches.get_one::<String>("quick") {
        let use_env = matches.get_flag("dev-env");
        let config_path = matches.get_one::<String>("config");
        let (api_url, api_key, workspace_id, default_project, _config) = if use_env {
            (
                std::env::var("OPUS_API_URL").unwrap_or_else(|_| "http://localhost:1337".to_string()),
                std::env::var("OPUS_API_KEY").unwrap_or_else(|_| "demo-token".to_string()),
                std::env::var("OPUS_WORKSPACE_ID").unwrap_or_else(|_| String::new()),
                std::env::var("OPUS_DEFAULT_PROJECT").unwrap_or_else(|_| "Inbox".to_string()),
                None
            )
        } else {
            match crate::config::OpusConfig::load_from_path(config_path.map(|s| s.as_str())) {
                Some(cfg) => {
                    if cfg.has_api_key_config() {
                        match cfg.get_api_key() {
                            Ok(api_key) => (
                                cfg.api_url.clone(),
                                api_key,
                                cfg.workspace_id.clone().unwrap_or_default(),
                                cfg.default_project.clone().unwrap_or_else(|| "Inbox".to_string()),
                                Some(cfg),
                            ),
                            Err(e) => {
                                eprintln!("Error loading API key: {}", e);
                                std::process::exit(1);
                            }
                        }
                    } else {
                        eprintln!("Config exists but no API key configured");
                        std::process::exit(1);
                    }
                },
                None => {
                    eprintln!("Config file not found");
                    std::process::exit(1);
                }
            }
        };

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

    crate::debug::clear_debug_log();
    debug_log("Starting Opus application");
    debug_log(&format!("Environment variables:"));
    debug_log(&format!("  OPUS_API_URL: {:?}", std::env::var("OPUS_API_URL")));
    debug_log(&format!("  OPUS_API_KEY: {:?}", std::env::var("OPUS_API_KEY").map(|t| format!("{}...", &t[..t.len().min(8)]))));
    debug_log(&format!("  OPUS_WORKSPACE_ID: {:?}", std::env::var("OPUS_WORKSPACE_ID")));
    debug_log(&format!("  OPUS_DEFAULT_PROJECT: {:?}", std::env::var("OPUS_DEFAULT_PROJECT")));

    let use_env = matches.get_flag("dev-env");
    let config_path = matches.get_one::<String>("config");

    let (api_url, api_key, workspace_id, default_project, config) = if use_env {
        debug_log("Using environment variables for API config");
        (
            std::env::var("OPUS_API_URL").unwrap_or_else(|_| "http://localhost:1337".to_string()),
            std::env::var("OPUS_API_KEY").unwrap_or_else(|_| "demo-token".to_string()),
            std::env::var("OPUS_WORKSPACE_ID").unwrap_or_else(|_| String::new()),
            std::env::var("OPUS_DEFAULT_PROJECT").unwrap_or_else(|_| "Inbox".to_string()),
            None
        )
    } else {
        match crate::config::OpusConfig::load_from_path(config_path.map(|s| s.as_str())) {
            Some(cfg) => {
                let config_source = if let Some(path) = config_path {
                    format!("custom path: {}", path)
                } else {
                    "default location".to_string()
                };
                debug_log(&format!("Loaded config from {}: api_url={}, api_key=***", config_source, cfg.api_url));
                if cfg.has_api_key_config() {
                    match cfg.get_api_key() {
                        Ok(api_key) => (
                            cfg.api_url.clone(),
                            api_key,
                            cfg.workspace_id.clone().unwrap_or_default(),
                            cfg.default_project.clone().unwrap_or_else(|| "Inbox".to_string()),
                            Some(cfg),
                        ),
                        Err(e) => {
                            eprintln!("Error loading API key: {}", e);
                            std::process::exit(1);
                        }
                    }
                } else {
                    debug_log("Config exists but no API key configured");
                    eprintln!("No API key configured. Run `opus --login` to authenticate.");
                    std::process::exit(1);
                }
            },
            None => {
                let error_msg = if let Some(path) = config_path {
                    format!("Config file not found at: {}", path)
                } else {
                    "No config found at default location".to_string()
                };
                debug_log(&error_msg);

                if config_path.is_some() {
                    eprintln!("Error: {}", error_msg);
                    std::process::exit(1);
                } else {
                    eprintln!("No config found. Run `opus --login` to authenticate.");
                    std::process::exit(1);
                }
            }
        }
    };

    if let Err(e) = tokio_main(api_url, api_key, workspace_id, default_project, config) {
        eprintln!("Application error: {e}");
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
    let app = Arc::new(Mutex::new(App::new_with_config(config.expect("Config required"), default_project.clone())));

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
