use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub token: String,
    pub user: TokenUser,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenUser {
    pub id: String,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoredCredentials {
    pub api_url: String,
    pub token: String,
    pub user_id: String,
    pub user_name: String,
    pub user_email: String,
}

pub struct DeviceAuth {
    client: reqwest::Client,
    base_url: String,
}

impl DeviceAuth {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    pub async fn request_device_code(&self) -> Result<DeviceCodeResponse, anyhow::Error> {
        let url = format!("{}/api/auth/device/authorize", self.base_url);

        let resp = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "client_id": "opus-cli"
            }))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!(
                "Device authorization request failed ({}): {}",
                status,
                body
            );
        }

        let device_code: DeviceCodeResponse = resp.json().await?;
        Ok(device_code)
    }

    pub async fn poll_for_token(
        &self,
        device_code: &str,
        interval: u64,
    ) -> Result<TokenResponse, anyhow::Error> {
        let url = format!("{}/api/auth/device/token", self.base_url);
        let poll_interval = std::time::Duration::from_secs(interval.max(1));

        loop {
            tokio::time::sleep(poll_interval).await;

            let resp = self
                .client
                .post(&url)
                .json(&serde_json::json!({
                    "client_id": "opus-cli",
                    "device_code": device_code
                }))
                .send()
                .await?;

            if resp.status().is_success() {
                let token_resp: TokenResponse = resp.json().await?;
                return Ok(token_resp);
            }

            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();

            if body.contains("authorization_pending") || body.contains("slow_down") {
                continue;
            }

            if body.contains("expired_token") || body.contains("access_denied") {
                anyhow::bail!("Device authorization failed: {}", body);
            }

            if status.is_server_error() {
                continue;
            }

            anyhow::bail!("Unexpected response ({}): {}", status, body);
        }
    }

    pub fn credentials_path() -> PathBuf {
        match std::env::var("XDG_CONFIG_HOME") {
            Ok(val) => PathBuf::from(val).join("opus-cli/credentials.json"),
            Err(_) => dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config/opus-cli/credentials.json"),
        }
    }

    pub fn save_credentials(creds: &StoredCredentials) -> Result<(), anyhow::Error> {
        let path = Self::credentials_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(creds)?;
        fs::write(&path, json)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
        }

        Ok(())
    }

    pub fn load_credentials() -> Option<StoredCredentials> {
        let path = Self::credentials_path();
        let contents = fs::read_to_string(&path).ok()?;
        serde_json::from_str(&contents).ok()
    }
}

pub async fn run_device_auth_flow(base_url: &str) -> Result<StoredCredentials, anyhow::Error> {
    let auth = DeviceAuth::new(base_url);

    let device_code = auth.request_device_code().await?;

    println!();
    println!(
        "Open this URL in your browser: {}",
        device_code.verification_uri
    );
    println!("Enter code: {}", device_code.user_code);
    println!();

    let _ = open::that(&device_code.verification_uri);

    println!("Waiting for authorization...");

    let token_resp = auth
        .poll_for_token(&device_code.device_code, device_code.interval)
        .await?;

    let creds = StoredCredentials {
        api_url: base_url.trim_end_matches('/').to_string(),
        token: token_resp.token,
        user_id: token_resp.user.id,
        user_name: token_resp.user.name,
        user_email: token_resp.user.email,
    };

    DeviceAuth::save_credentials(&creds)?;

    Ok(creds)
}
