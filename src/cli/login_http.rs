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
