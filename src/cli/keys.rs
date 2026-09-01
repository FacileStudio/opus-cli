use clap::{Arg, ArgAction, Command};
use std::io::{self, IsTerminal, Write};

use crate::opus::models::CreateKeyRequest;
use crate::opus_client::OpusClient;

pub fn subcommand() -> Command {
    Command::new("keys")
        .about("Manage API keys")
        .subcommand_required(true)
        .subcommand(
            Command::new("list")
                .about("List API keys")
                .arg(
                    Arg::new("app")
                        .long("app")
                        .help("Filter keys by application name"),
                )
                .arg(
                    Arg::new("json")
                        .long("json")
                        .action(ArgAction::SetTrue)
                        .help("Output as JSON"),
                ),
        )
        .subcommand(
            Command::new("create")
                .about("Create a new API key")
                .arg(
                    Arg::new("app")
                        .long("app")
                        .required(true)
                        .help("Application name"),
                )
                .arg(
                    Arg::new("public")
                        .long("public")
                        .action(ArgAction::SetTrue)
                        .help("Create a public browser key instead of a secret key"),
                )
                .arg(
                    Arg::new("origins")
                        .long("origins")
                        .value_delimiter(',')
                        .help("Comma-separated allowed origins (for public keys)"),
                )
                .arg(
                    Arg::new("quota")
                        .long("quota")
                        .value_parser(clap::value_parser!(i32))
                        .help("Daily event quota limit (for public keys)"),
                )
                .arg(
                    Arg::new("json")
                        .long("json")
                        .action(ArgAction::SetTrue)
                        .help("Output as JSON"),
                ),
        )
        .subcommand(
            Command::new("revoke")
                .about("Revoke an API key")
                .arg(
                    Arg::new("id")
                        .required(true)
                        .value_parser(clap::value_parser!(i64))
                        .help("Key ID to revoke"),
                )
                .arg(
                    Arg::new("yes")
                        .short('y')
                        .long("yes")
                        .action(ArgAction::SetTrue)
                        .help("Confirm revocation without prompting"),
                )
                .arg(
                    Arg::new("json")
                        .long("json")
                        .action(ArgAction::SetTrue)
                        .help("Output as JSON"),
                ),
        )
}

pub async fn handle(
    client: &OpusClient,
    matches: &clap::ArgMatches,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match matches.subcommand() {
        Some(("list", sub)) => handle_list(client, sub).await,
        Some(("create", sub)) => handle_create(client, sub).await,
        Some(("revoke", sub)) => handle_revoke(client, sub).await,
        _ => unreachable!(),
    }
}

async fn handle_list(
    client: &OpusClient,
    args: &clap::ArgMatches,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = args.get_one::<String>("app").map(|s| s.as_str());
    let json = args.get_flag("json");

    let keys = client.list_keys(app).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&keys)?);
        return Ok(());
    }

    if keys.is_empty() {
        crate::ui::step("No API keys found");
        return Ok(());
    }

    let id_width = keys
        .iter()
        .map(|k| k.id.to_string().len())
        .max()
        .unwrap_or(2)
        .max(2);
    let app_width = keys.iter().map(|k| k.app.len()).max().unwrap_or(3).max(3);
    let kind_width = keys.iter().map(|k| k.kind.len()).max().unwrap_or(4).max(4);
    let prefix_width = keys
        .iter()
        .map(|k| k.prefix.len())
        .max()
        .unwrap_or(6)
        .max(6);

    println!(
        "{:<id_width$}  {:<app_width$}  {:<kind_width$}  {:<prefix_width$}  {:<8}  {:<24}  CREATED",
        "ID", "APP", "KIND", "PREFIX", "STATUS", "QUOTA"
    );

    for k in &keys {
        let status = if k.revoked_at.is_some() {
            "revoked"
        } else {
            "active"
        };
        let quota = if k.daily_quota > 0 {
            format!("{}/day ({} used)", k.daily_quota, k.used_today)
        } else {
            "unlimited".to_string()
        };
        let created = if k.created_at.len() >= 10 {
            &k.created_at[..10]
        } else {
            &k.created_at
        };
        println!(
            "{:<id_width$}  {:<app_width$}  {:<kind_width$}  {:<prefix_width$}  {:<8}  {:<24}  {}",
            k.id, k.app, k.kind, k.prefix, status, quota, created
        );
    }

    Ok(())
}

async fn handle_create(
    client: &OpusClient,
    args: &clap::ArgMatches,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = args.get_one::<String>("app").unwrap();
    if app.trim().is_empty() {
        return Err("--app is required".into());
    }

    let public = args.get_flag("public");
    let origins: Vec<String> = args
        .get_many::<String>("origins")
        .unwrap_or_default()
        .cloned()
        .collect();
    let quota = args.get_one::<i32>("quota").copied();
    let json = args.get_flag("json");

    let kind = if public { "public" } else { "secret" };
    let req = CreateKeyRequest {
        app: app.clone(),
        kind: kind.to_string(),
        allowed_origins: origins,
        daily_quota: quota.unwrap_or(0),
    };

    let resp = client.create_key(&req).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&resp)?);
        return Ok(());
    }

    crate::ui::success(&format!(
        "Created {} API key #{} for {}",
        resp.key.kind, resp.key.id, resp.key.app
    ));
    println!("{}", resp.token);
    crate::ui::hint("Save this token now, it will not be shown again");
    Ok(())
}

async fn handle_revoke(
    client: &OpusClient,
    args: &clap::ArgMatches,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let id = *args.get_one::<i64>("id").unwrap();
    let yes = args.get_flag("yes");
    let json = args.get_flag("json");

    if id <= 0 {
        return Err("key id must be a positive integer".into());
    }

    if !yes && io::stdin().is_terminal() {
        print!("Revoke API key #{}? [y/N] ", id);
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let trimmed = input.trim();
        if !trimmed.eq_ignore_ascii_case("y") && !trimmed.eq_ignore_ascii_case("yes") {
            crate::ui::step("Revocation aborted");
            return Ok(());
        }
    }

    client.revoke_key(id).await?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "id": id,
                "revoked": true
            }))?
        );
        return Ok(());
    }

    crate::ui::success(&format!("Revoked key {}", id));
    Ok(())
}
