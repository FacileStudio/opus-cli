use crate::debug::debug_log;
use crate::opus::models::{ApiToken, CreateTokenRequest, ListTokensResponse};

impl super::OpusClient {
    pub async fn list_tokens(
        &self,
    ) -> Result<Vec<ApiToken>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/auth/api-tokens", self.base_url);
        debug_log(&format!("Fetching API tokens from: {}", url));

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            let body: ListTokensResponse = response.json().await?;
            debug_log(&format!("Got {} API tokens", body.tokens.len()));
            Ok(body.tokens)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            debug_log(&format!("Failed to list API tokens: {} - {}", status, error_text));
            Err(format!("GET /api/auth/api-tokens failed ({}): {}", status, error_text).into())
        }
    }

    pub async fn create_token(
        &self,
        req: &CreateTokenRequest,
    ) -> Result<ApiToken, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/auth/api-tokens", self.base_url);
        debug_log(&format!("Creating API token at: {}", url));

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .json(req)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            let body: ApiToken = response.json().await?;
            debug_log(&format!("Created API token #{}", body.id));
            Ok(body)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            debug_log(&format!("Failed to create API token: {} - {}", status, error_text));
            Err(format!("POST /api/auth/api-tokens failed ({}): {}", status, error_text).into())
        }
    }

    pub async fn revoke_token(
        &self,
        id: i64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/auth/api-tokens/{}", self.base_url, id);
        debug_log(&format!("Revoking API token {} at: {}", id, url));

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            debug_log(&format!("Revoked API token {}", id));
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            debug_log(&format!(
                "Failed to revoke API token {}: {} - {}",
                id, status, error_text
            ));
            Err(format!(
                "DELETE /api/auth/api-tokens/{} failed ({}): {}",
                id, status, error_text
            )
            .into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opus_client::OpusClient;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    async fn serve_once(
        body: &'static str,
        status: &'static str,
    ) -> (u16, tokio::task::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 2048];
            let n = stream.read(&mut buf).await.unwrap();
            let request = String::from_utf8_lossy(&buf[..n]).to_string();
            let resp = format!(
                "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                status,
                body.len(),
                body
            );
            stream.write_all(resp.as_bytes()).await.unwrap();
            request
        });
        (port, handle)
    }

    #[tokio::test]
    async fn list_tokens_hits_the_go_backend_path() {
        let body = r#"{"tokens":[{"id":1,"name":"laptop","createdAt":"2026-09-01T00:00:00Z","lastUsedAt":"2026-09-02T00:00:00Z"}]}"#;
        let (port, handle) = serve_once(body, "200 OK").await;

        let client = OpusClient::new(
            format!("http://127.0.0.1:{}", port),
            "test-token".to_string(),
            "ws1".to_string(),
        );
        let tokens = client.list_tokens().await.unwrap();
        let request = handle.await.unwrap();

        assert!(request.starts_with("GET /api/auth/api-tokens HTTP/1.1"));
        assert!(request.contains("authorization: Bearer test-token"));
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].id, 1);
        assert_eq!(tokens[0].name, "laptop");
        assert_eq!(tokens[0].created_at, "2026-09-01T00:00:00Z");
        assert_eq!(tokens[0].last_used_at, "2026-09-02T00:00:00Z");
    }

    #[tokio::test]
    async fn create_token_returns_the_one_time_secret() {
        let body = r#"{"id":5,"name":"ci","createdAt":"2026-09-01T00:00:00Z","lastUsedAt":"","token":"opus-secret-value"}"#;
        let (port, handle) = serve_once(body, "201 Created").await;

        let client = OpusClient::new(
            format!("http://127.0.0.1:{}", port),
            "test-token".to_string(),
            "ws1".to_string(),
        );
        let req = CreateTokenRequest {
            name: "ci".to_string(),
        };
        let token = client.create_token(&req).await.unwrap();
        let request = handle.await.unwrap();

        assert!(request.starts_with("POST /api/auth/api-tokens HTTP/1.1"));
        assert!(request.contains(r#""name":"ci""#));
        assert_eq!(token.id, 5);
        assert_eq!(token.token.as_deref(), Some("opus-secret-value"));
    }

    #[tokio::test]
    async fn revoke_token_deletes_by_id() {
        let (port, handle) = serve_once(r#"{"deleted":true}"#, "200 OK").await;

        let client = OpusClient::new(
            format!("http://127.0.0.1:{}", port),
            "test-token".to_string(),
            "ws1".to_string(),
        );
        client.revoke_token(42).await.unwrap();
        let request = handle.await.unwrap();

        assert!(request.starts_with("DELETE /api/auth/api-tokens/42 HTTP/1.1"));
        assert!(request.contains("authorization: Bearer test-token"));
    }

    #[tokio::test]
    async fn an_error_status_is_an_error() {
        let (port, _handle) = serve_once(r#"{"message":"unauthenticated"}"#, "401 Unauthorized").await;

        let client = OpusClient::new(
            format!("http://127.0.0.1:{}", port),
            "bad-token".to_string(),
            "ws1".to_string(),
        );
        assert!(client.list_tokens().await.is_err());
    }
}
