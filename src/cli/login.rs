use std::io::{self, Read};

use clap::{Arg, ArgAction, ArgMatches, Command};

use crate::cli::device::Provider;
use crate::cli::login_http::{device_exchange, device_exchange_mounted, exchange, non_empty, offers_sso, verify};
use crate::cli::loopback::{open_browser, random_state, Loopback};
use crate::config::{OpusConfig, DEFAULT_URL};
use crate::ui;

pub fn subcommand() -> Command {
    Command::new("login")
        .about("Sign in through a browser approval link and save the credential")
        .long_about(
            "Sign in through a browser approval link and save the credential.\n\n\
             With no URL this signs in to https://opus.facile.studio. The CLI prints a link, tries to \
             open it, and waits until you approve the sign-in in any browser. Nothing listens on this \
             machine and nothing is pasted, so it works over SSH.\n\n\
             opus login --listener         loopback flow, needs a browser on this machine\n\
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
            Arg::new("listener")
                .long("listener")
                .action(ArgAction::SetTrue)
                .conflicts_with("no-listener")
                .help("Use the loopback listener instead of the approval link"),
        )
        .arg(Arg::new("no-listener").long("no-listener").action(ArgAction::SetTrue).hide(true))
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
    if !matches.get_flag("listener") {
        if device_exchange_mounted(base)? {
            return device_login(base);
        }
        ui::hint("This server has no device sign-in yet, falling back to the browser listener.");
    }
    browser_login(base)
}

fn device_login(base: &str) -> Result<String, String> {
    let provider = Provider::discover()?;
    let auth = provider.authorize()?;
    ui::step(&format!("Approve this sign-in in a browser: {}", auth.link));
    ui::hint(&format!("The page shows the code {}. Waiting for approval.", auth.user_code));
    open_browser(&auth.link);
    let access_token = provider.await_token(&auth)?;
    device_exchange(base, &access_token)
}

fn browser_login(base: &str) -> Result<String, String> {
    let listener = Loopback::listen()?;
    let state = random_state();
    let url = format!("{base}/api/auth/oidc?flow=cli&port={}&cli_state={state}", listener.port());
    open_browser(&url);
    ui::step(&format!("Finish signing in in your browser: {url}"));
    let code = listener.wait_for_code(&state)?;
    exchange(base, &code)
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
