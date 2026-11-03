use std::time::Duration;

use serde::Deserialize;

pub fn offers_sso(base: &str) -> bool {
    #[derive(Deserialize)]
    struct Discovery {
        oidc_enabled: bool,
    }
    let Ok(client) = reqwest::blocking::Client::builder().timeout(Duration::from_secs(10)).build() else {
        return false;
    };
    client
        .get(format!("{base}/api/auth/config"))
        .send()
        .and_then(|response| response.json::<Discovery>())
        .map(|discovery| discovery.oidc_enabled)
        .unwrap_or(false)
}

pub fn exchange(base: &str, code: &str) -> Result<String, String> {
    #[derive(Deserialize)]
    struct Exchanged {
        token: String,
    }
    let response = reqwest::blocking::Client::new()
        .post(format!("{base}/api/auth/oidc/exchange"))
        .json(&serde_json::json!({ "code": code }))
        .send()
        .map_err(|e| format!("connection failed: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("sign-in failed: {}", response.text().unwrap_or_default().trim()));
    }
    let exchanged: Exchanged = response.json().map_err(|e| format!("invalid response: {e}"))?;
    non_empty(exchanged.token)
}

/// Reports whether the server mounts the device exchange. Only a 404 means it
/// does not; the empty probe gets a 401 from a server that does.
pub fn device_exchange_mounted(base: &str) -> Result<bool, String> {
    let response = post_device_exchange(base, "")?;
    Ok(mounted(response.status().as_u16()))
}

fn mounted(status: u16) -> bool {
    status != 404
}

/// Trades a Registre access token for an Opus session token. A 401 almost
/// always means the account never signed in to Opus in a browser, because the
/// exchange looks up existing identities and never provisions one.
pub fn device_exchange(base: &str, access_token: &str) -> Result<String, String> {
    #[derive(Deserialize)]
    struct Exchanged {
        token: String,
    }
    let response = post_device_exchange(base, access_token)?;
    match response.status().as_u16() {
        200..=299 => {}
        401 => return Err(format!("Opus does not know this account yet. Sign in once at {base} in a browser, then run 'opus login' again")),
        _ => return Err(format!("sign-in failed: {}", response.text().unwrap_or_default().trim())),
    }
    let exchanged: Exchanged = response.json().map_err(|e| format!("invalid response: {e}"))?;
    non_empty(exchanged.token)
}

fn post_device_exchange(base: &str, access_token: &str) -> Result<reqwest::blocking::Response, String> {
    reqwest::blocking::Client::new()
        .post(format!("{base}/api/auth/oidc/device/exchange"))
        .json(&serde_json::json!({ "access_token": access_token }))
        .send()
        .map_err(|e| format!("connection failed: {e}"))
}

pub fn verify(base: &str, token: &str) -> Result<(), String> {
    let response = reqwest::blocking::Client::new()
        .get(format!("{base}/api/auth/me"))
        .bearer_auth(token)
        .send()
        .map_err(|e| format!("connection failed: {e}"))?;
    match response.status().as_u16() {
        200 => Ok(()),
        401 | 403 => Err(format!("token rejected by {base}")),
        _ => Err(format!("server returned {}", response.status())),
    }
}

pub fn non_empty(token: String) -> Result<String, String> {
    let token = token.trim().to_string();
    if token.is_empty() {
        return Err("empty token".to_string());
    }
    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::mounted;

    #[test]
    fn only_a_404_falls_back_to_the_listener() {
        assert!(!mounted(404));
        assert!(mounted(401));
        assert!(mounted(400));
    }
}
