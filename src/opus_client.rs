use reqwest::{Client, Result as ReqwestResult};
use crate::opus_parser::QuickAddParser;
use crate::debug::debug_log;

pub mod tasks;
pub mod projects;
pub mod filters;
pub mod users;
pub mod labels;
pub mod attachments;
pub mod relations;

pub struct OpusClient {
    client: Client,
    base_url: String,
    auth_token: String,
    workspace_id: String,
    parser: QuickAddParser,
}

impl OpusClient {
    pub fn new(base_url: String, auth_token: String, workspace_id: String) -> Self {
        let base_url = normalize_base_url(&base_url);
        debug_log(&format!("Creating OpusClient with URL: {}", base_url));
        debug_log(&format!("Auth token length: {}", auth_token.len()));
        debug_log(&format!("Workspace ID: {}", workspace_id));
        let client = Client::new();
        Self {
            client,
            base_url,
            auth_token,
            workspace_id,
            parser: QuickAddParser::new(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn auth_token(&self) -> &str {
        &self.auth_token
    }

    pub fn workspace_id(&self) -> &str {
        &self.workspace_id
    }

    pub async fn test_connection(&self) -> ReqwestResult<bool> {
        debug_log(&format!("Testing connection to {}", self.base_url));
        let url = format!(
            "{}/api/project?workspaceId={}",
            self.base_url, self.workspace_id
        );
        debug_log(&format!("Testing with URL: {}", url));
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await;
        match response {
            Ok(resp) => {
                debug_log(&format!("Connection test - Status: {}", resp.status()));
                if resp.status().is_success() {
                    debug_log("Connection successful!");
                    Ok(true)
                } else {
                    debug_log(&format!("Connection failed with status: {}", resp.status()));
                    Ok(false)
                }
            }
            Err(e) => {
                debug_log(&format!("Connection test failed: {:?}", e));
                if e.is_connect() {
                    debug_log(&format!(
                        "Cannot connect to Opus at {}. Is it running?",
                        self.base_url
                    ));
                }
                Err(e)
            }
        }
    }
}

fn normalize_base_url(url: &str) -> String {
    url.trim()
        .trim_end_matches('/')
        .trim_end_matches("/api")
        .trim_end_matches('/')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::normalize_base_url;

    #[test]
    fn accepts_plain_base_url() {
        assert_eq!(
            normalize_base_url("https://opus.example.com"),
            "https://opus.example.com"
        );
    }

    #[test]
    fn accepts_api_config_url() {
        assert_eq!(
            normalize_base_url("https://opus.example.com/api"),
            "https://opus.example.com"
        );
    }

    #[test]
    fn strips_trailing_slashes() {
        assert_eq!(
            normalize_base_url("https://opus.example.com/api/"),
            "https://opus.example.com"
        );
    }
}
