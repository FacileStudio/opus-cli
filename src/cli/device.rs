use std::time::{Duration, Instant};

use serde_json::Value;

use crate::cli::device_poll::{next, text, Next};

const ISSUER: &str = "https://sso.facile.studio";
const CLIENT_ID: &str = "facile-cli";
const SCOPES: &str = "openid email profile";
const GRANT_TYPE: &str = "urn:ietf:params:oauth:grant-type:device_code";
const DEFAULT_INTERVAL: u64 = 5;
const DEFAULT_EXPIRY: u64 = 600;

/// The two endpoints of Registre's RFC 8628 device grant, read from its root
/// discovery document. Per-application discovery paths answer with HTML.
pub struct Provider {
    device_authorization: String,
    token: String,
}

/// The device authorization response, RFC 8628 §3.2. `link` carries the user
/// code inside it, so nobody has to type the code.
pub struct Authorization {
    device_code: String,
    pub user_code: String,
    pub link: String,
    interval: Duration,
    expires: Duration,
}

impl Provider {
    /// Reads the endpoints from `https://sso.facile.studio/.well-known/openid-configuration`.
    pub fn discover() -> Result<Self, String> {
        let doc: Value = client()?
            .get(format!("{ISSUER}/.well-known/openid-configuration"))
            .send()
            .and_then(|response| response.error_for_status())
            .and_then(|response| response.json())
            .map_err(|e| format!("cannot read the identity provider's configuration: {e}"))?;
        let provider = Self {
            device_authorization: text(&doc, "device_authorization_endpoint"),
            token: text(&doc, "token_endpoint"),
        };
        if provider.device_authorization.is_empty() || provider.token.is_empty() {
            return Err(format!("{ISSUER} does not offer a device sign-in"));
        }
        Ok(provider)
    }

    /// Starts a device sign-in as the public `facile-cli` client, which has no
    /// secret. The approval link must be https, because the CLI hands it to the
    /// operating system to open.
    pub fn authorize(&self) -> Result<Authorization, String> {
        let (status, doc) = post_form(&self.device_authorization, &[("client_id", CLIENT_ID), ("scope", SCOPES)])?;
        if !(200..300).contains(&status) {
            return Err(format!("the identity provider would not start a device sign-in ({status})"));
        }
        let mut link = text(&doc, "verification_uri_complete");
        if link.is_empty() {
            link = text(&doc, "verification_uri");
        }
        let auth = Authorization {
            device_code: text(&doc, "device_code"),
            user_code: text(&doc, "user_code"),
            link,
            interval: seconds(&doc, "interval", DEFAULT_INTERVAL),
            expires: seconds(&doc, "expires_in", DEFAULT_EXPIRY),
        };
        if auth.device_code.is_empty() || !auth.link.starts_with("https://") {
            return Err("the identity provider's device authorization is not usable".to_string());
        }
        Ok(auth)
    }

    /// Polls the token endpoint until the user approves, refuses, or the code
    /// expires. A transport error is retried: the deadline already bounds the
    /// loop.
    pub fn await_token(&self, auth: &Authorization) -> Result<String, String> {
        let deadline = Instant::now() + auth.expires;
        let mut interval = auth.interval;
        let form = [("grant_type", GRANT_TYPE), ("device_code", auth.device_code.as_str()), ("client_id", CLIENT_ID)];
        while Instant::now() < deadline {
            std::thread::sleep(interval);
            let Ok((status, doc)) = post_form(&self.token, &form) else {
                continue;
            };
            match next(interval, status, &doc)? {
                Next::Done(token) => return Ok(token),
                Next::Wait(wait) => interval = wait,
            }
        }
        Err("the code expired before it was approved, run 'opus login' again".to_string())
    }
}

fn client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder().timeout(Duration::from_secs(15)).build().map_err(|e| e.to_string())
}

fn post_form(url: &str, form: &[(&str, &str)]) -> Result<(u16, Value), String> {
    let response = client()?
        .post(url)
        .header("Accept", "application/json")
        .form(form)
        .send()
        .map_err(|e| format!("cannot reach the identity provider: {e}"))?;
    let status = response.status().as_u16();
    Ok((status, response.json().unwrap_or(Value::Null)))
}

fn seconds(doc: &Value, key: &str, fallback: u64) -> Duration {
    let value = doc.get(key).and_then(Value::as_u64).filter(|v| *v > 0).unwrap_or(fallback);
    Duration::from_secs(value)
}
