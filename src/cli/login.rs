use std::io::{self, BufRead, IsTerminal, Read};

use clap::{Arg, ArgAction, ArgMatches, Command};

use crate::cli::login_http::{exchange, non_empty, offers_sso, verify};
use crate::cli::loopback::{open_browser, random_state, Loopback};
use crate::config::{OpusConfig, DEFAULT_URL};
use crate::ui;

pub fn subcommand() -> Command {
    Command::new("login")
        .about("Sign in through your browser and save the credential")
        .long_about(
            "Sign in through your browser and save the credential.\n\n\
             With no URL this signs in to https://opus.facile.studio. A session already open with \
             another Facile tool completes the login without a prompt.\n\n\
             opus login --no-listener      sign in from a browser on another machine, paste the code\n\
             opus login --token <token>    use a token from Settings > Account > Developer\n\
             opus login --token-stdin      read the token from stdin",
        )
        .arg(Arg::new("url").help("Opus instance URL (default: https://opus.facile.studio)"))
        .arg(Arg::new("token").long("token").help("Save this token instead of signing in"))
        .arg(
            Arg::new("token-stdin")
                .long("token-stdin")
                .action(ArgAction::SetTrue)
                .help("Read the token from stdin"),
        )
        .arg(
            Arg::new("no-listener")
                .long("no-listener")
                .action(ArgAction::SetTrue)
                .help("Show the code on the page and paste it here"),
        )
}

pub fn handle(matches: &ArgMatches, config_path: Option<&str>) -> Result<(), String> {
    let existing = OpusConfig::load_from_path(config_path);
    let base = resolve_url(matches.get_one::<String>("url"), existing.as_ref());
    let token = obtain_token(matches, &base)?;
    verify(&base, &token)?;
    let mut config = existing.unwrap_or_default();
    config.api_url = base.clone();
    config.api_key = Some(token);
    config.save()?;
    ui::success(&format!("Logged in to {base}"));
    ui::hint(&format!("Config saved to {}", config.display_path()));
    Ok(())
}

fn resolve_url(arg: Option<&String>, existing: Option<&OpusConfig>) -> String {
    let raw = arg
        .cloned()
        .or_else(|| std::env::var("OPUS_API_URL").ok())
        .or_else(|| std::env::var("OPUS_SERVER_URL").ok())
        .or_else(|| existing.map(|config| config.api_url.clone()))
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_URL.to_string());
    raw.trim().trim_end_matches('/').trim_end_matches("/api").trim_end_matches('/').to_string()
}

fn obtain_token(matches: &ArgMatches, base: &str) -> Result<String, String> {
    if let Some(token) = matches.get_one::<String>("token") {
        return non_empty(token.clone());
    }
    if matches.get_flag("token-stdin") {
        let mut raw = String::new();
        io::stdin().read_to_string(&mut raw).map_err(|e| format!("failed to read token: {e}"))?;
        return non_empty(raw);
    }
    if !offers_sso(base) {
        return Err(format!("{base} offers no browser sign-in, use --token or --token-stdin"));
    }
    let interactive = io::stdin().is_terminal() && io::stdout().is_terminal();
    if matches.get_flag("no-listener") || !interactive {
        return paste_login(base, interactive);
    }
    browser_login(base)
}

fn browser_login(base: &str) -> Result<String, String> {
    let listener = Loopback::listen()?;
    let state = random_state();
    let url = format!("{base}/api/auth/oidc?flow=cli&port={}&cli_state={state}", listener.port());
    if open_browser(&url) {
        ui::step("Waiting for the browser to finish signing you in");
    } else {
        ui::step(&format!("Open this URL in your browser to sign in: {url}"));
    }
    let code = listener.wait_for_code(&state)?;
    exchange(base, &code)
}

fn paste_login(base: &str, interactive: bool) -> Result<String, String> {
    if !io::stdin().is_terminal() {
        return Err("no terminal to paste a code into, use --token-stdin".to_string());
    }
    let url = format!("{base}/api/auth/oidc?flow=cli");
    ui::step(&format!("Open this URL in your browser to sign in: {url}"));
    if interactive {
        open_browser(&url);
    }
    println!("Paste the code from that page:");
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line).map_err(|e| format!("failed to read the login code: {e}"))?;
    let code = line.trim();
    if code.is_empty() {
        return Err("no login code was pasted, run 'opus login' again".to_string());
    }
    exchange(base, code)
}

#[cfg(test)]
mod tests {
    use super::resolve_url;
    use crate::config::DEFAULT_URL;

    #[test]
    fn no_url_falls_back_to_the_hosted_instance() {
        std::env::remove_var("OPUS_API_URL");
        std::env::remove_var("OPUS_SERVER_URL");
        assert_eq!(resolve_url(None, None), DEFAULT_URL);
    }

    #[test]
    fn an_explicit_url_loses_its_api_suffix() {
        assert_eq!(resolve_url(Some(&"https://x.test/api/".to_string()), None), "https://x.test");
    }
}
