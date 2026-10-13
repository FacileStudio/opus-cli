use clap::{Arg, ArgAction, Command};
use std::io::{self, IsTerminal, Write};

use crate::opus::models::CreateTokenRequest;
use crate::opus_client::OpusClient;

pub fn subcommand() -> Command {
    Command::new("token")
        .about("Manage personal API tokens")
        .subcommand_required(true)
        .subcommand(
            Command::new("list")
                .about("List your API tokens")
                .arg(
                    Arg::new("json")
                        .long("json")
                        .action(ArgAction::SetTrue)
                        .help("Output as JSON"),
                ),
        )
        .subcommand(
            Command::new("create")
                .about("Create a token that authenticates as you")
                .arg(
                    Arg::new("name")
                        .long("name")
                        .short('n')
                        .required(true)
                        .help("A name that makes the token recognisable later"),
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
                .about("Revoke an API token")
                .arg(
                    Arg::new("id")
                        .required(true)
                        .value_parser(clap::value_parser!(i64))
                        .help("Token ID to revoke"),
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
    let json = args.get_flag("json");
    let tokens = client.list_tokens().await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&tokens)?);
        return Ok(());
    }

    if tokens.is_empty() {
        crate::ui::step("No API tokens found");
        crate::ui::hint("Create one with `opus token create --name <name>`");
        return Ok(());
    }

    let id_width = tokens
        .iter()
        .map(|token| token.id.to_string().len())
        .max()
        .unwrap_or(2)
        .max(2);
    let name_width = tokens
        .iter()
        .map(|token| token.name.len())
        .max()
        .unwrap_or(4)
        .max(4);

    println!(
        "{:<id_width$}  {:<name_width$}  {:<12}  LAST USED",
        "ID", "NAME", "CREATED"
    );

    for token in &tokens {
        println!(
            "{:<id_width$}  {:<name_width$}  {:<12}  {}",
            token.id,
            token.name,
            date_prefix(&token.created_at),
            date_prefix(&token.last_used_at)
        );
    }

    Ok(())
}

async fn handle_create(
    client: &OpusClient,
    args: &clap::ArgMatches,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let name = args.get_one::<String>("name").unwrap();
    if name.trim().is_empty() {
        return Err("--name is required".into());
    }

    let json = args.get_flag("json");
    let req = CreateTokenRequest {
        name: name.trim().to_string(),
    };
    let token = client.create_token(&req).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&token)?);
        return Ok(());
    }

    crate::ui::success(&format!("Created API token #{} ({})", token.id, token.name));
    if let Some(secret) = &token.token {
        println!("{}", secret);
    }
    crate::ui::hint("Save this token now, it will not be shown again");
    crate::ui::hint("Add it to ~/.opus.yml as api_key, or export OPUS_TOKEN");
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
        return Err("token id must be a positive integer".into());
    }

    if !yes && io::stdin().is_terminal() {
        print!("Revoke API token #{}? [y/N] ", id);
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let trimmed = input.trim();
        if !trimmed.eq_ignore_ascii_case("y") && !trimmed.eq_ignore_ascii_case("yes") {
            crate::ui::step("Revocation aborted");
            return Ok(());
        }
    }

    client.revoke_token(id).await?;

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

    crate::ui::success(&format!("Revoked token {}", id));
    Ok(())
}

fn date_prefix(value: &str) -> String {
    if value.len() >= 10 {
        value[..10].to_string()
    } else if value.is_empty() {
        "never".to_string()
    } else {
        value.to_string()
    }
}
