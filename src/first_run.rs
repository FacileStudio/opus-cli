use crate::config::OpusConfig;
use regex::Regex;
use std::fs;
use std::io::{self, Write};

pub fn first_run_wizard() -> Option<OpusConfig> {
    println!("Welcome to opus-cli setup!");
    println!();

    let url_re = Regex::new(r"^https?://[\w.-]+(:\d+)?(/[\w.-]*)*$").unwrap();

    let mut base_url = String::new();
    loop {
        print!("Enter your Opus instance URL (default: http://localhost:1337): ");
        io::stdout().flush().unwrap();
        base_url.clear();
        io::stdin().read_line(&mut base_url).unwrap();
        base_url = base_url.trim().to_string();

        if base_url.is_empty() {
            base_url = "http://localhost:1337".to_string();
            break;
        }

        base_url = base_url.trim_end_matches('/').to_string();

        if !url_re.is_match(&base_url) {
            println!("Invalid URL. Please enter a valid http(s) URL.");
            continue;
        }
        break;
    }

    println!();
    println!("Choose authentication method:");
    println!("  1) API key");
    println!("  2) Device authorization (browser login)");
    print!("Selection [1]: ");
    io::stdout().flush().unwrap();

    let mut auth_choice = String::new();
    io::stdin().read_line(&mut auth_choice).unwrap();
    let auth_choice = auth_choice.trim();

    let mut api_key: Option<String> = None;

    match auth_choice {
        "2" => {
            println!();
            println!("Starting device authorization flow...");
            let rt = tokio::runtime::Runtime::new().unwrap();
            match rt.block_on(crate::auth::run_device_auth_flow(&base_url)) {
                Ok(creds) => {
                    println!("Authenticated as {} ({})", creds.user_name, creds.user_email);
                    api_key = Some(creds.token);
                }
                Err(e) => {
                    println!("Device auth failed: {}", e);
                    println!("You can set an API key manually in the config file later.");
                }
            }
        }
        _ => {
            print!("Paste your API key: ");
            io::stdout().flush().unwrap();
            let mut key_input = String::new();
            io::stdin().read_line(&mut key_input).unwrap();
            let key_input = key_input.trim().to_string();
            if !key_input.is_empty() {
                api_key = Some(key_input);
            }
        }
    }

    println!();
    print!("Workspace ID (leave blank to skip): ");
    io::stdout().flush().unwrap();
    let mut workspace_id = String::new();
    io::stdin().read_line(&mut workspace_id).unwrap();
    let workspace_id = workspace_id.trim().to_string();
    let workspace_id = if workspace_id.is_empty() {
        None
    } else {
        Some(workspace_id)
    };

    print!("Default project name (default: Inbox): ");
    io::stdout().flush().unwrap();
    let mut default_project = String::new();
    io::stdin().read_line(&mut default_project).unwrap();
    let default_project = default_project.trim().to_string();
    let default_project = if default_project.is_empty() {
        "Inbox".to_string()
    } else {
        default_project
    };

    let config = OpusConfig {
        api_url: base_url,
        api_key,
        workspace_id,
        default_project: Some(default_project),
        default_filter: None,
        auto_refresh: None,
        refresh_interval_seconds: None,
        quick_actions: None,
        table_columns: None,
        column_layouts: None,
        active_layout: None,
    };

    let config_path = {
        let mut home = dirs::home_dir().unwrap();
        home.push(".opus.yml");
        home
    };

    if let Some(parent) = config_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    if config_path.exists() {
        print!("Config file already exists. Backup and overwrite? (Y/n): ");
        io::stdout().flush().unwrap();
        let mut backup_answer = String::new();
        io::stdin().read_line(&mut backup_answer).unwrap();
        let backup_answer = backup_answer.trim().to_lowercase();
        if backup_answer.is_empty() || backup_answer == "y" {
            let backup_path = config_path.with_extension("yaml.bak");
            if let Err(e) = fs::copy(&config_path, &backup_path) {
                println!("Failed to backup config: {}", e);
            } else {
                println!("Backed up existing config to {}", backup_path.display());
            }
        } else {
            println!("Aborting wizard. No changes made.");
            return None;
        }
    }

    let yaml = serde_yaml::to_string(&config).unwrap();
    fs::write(&config_path, yaml).unwrap();
    println!("Config saved to {}", config_path.display());

    Some(config)
}
