use crate::debug::debug_log;
use crate::opus::models::{CreateKeyRequest, CreateKeyResponse, Key, ListKeysResponse};

impl super::OpusClient {
    pub async fn list_keys(
        &self,
        app: Option<&str>,
    ) -> Result<Vec<Key>, Box<dyn std::error::Error + Send + Sync>> {
        let url = match app {
            Some(a) if !a.is_empty() => format!("{}/apikeys?app={}", self.base_url, a),
            _ => format!("{}/apikeys", self.base_url),
        };
        debug_log(&format!("Fetching API keys from: {}", url));

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            let body: ListKeysResponse = response.json().await?;
            debug_log(&format!("Got {} API keys", body.keys.len()));
            Ok(body.keys)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            debug_log(&format!("Failed to list API keys: {} - {}", status, error_text));
            Err(format!("GET /apikeys failed ({}): {}", status, error_text).into())
        }
    }

    pub async fn create_key(
        &self,
        req: &CreateKeyRequest,
    ) -> Result<CreateKeyResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/apikeys", self.base_url);
        debug_log(&format!("Creating API key at: {}", url));

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .json(req)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            let body: CreateKeyResponse = response.json().await?;
            debug_log(&format!("Created API key #{}", body.key.id));
            Ok(body)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            debug_log(&format!("Failed to create API key: {} - {}", status, error_text));
            Err(format!("POST /apikeys failed ({}): {}", status, error_text).into())
        }
    }

    pub async fn revoke_key(
        &self,
        id: i64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/apikeys/{}", self.base_url, id);
        debug_log(&format!("Revoking API key {} at: {}", id, url));

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            debug_log(&format!("Revoked API key {}", id));
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            debug_log(&format!("Failed to revoke API key {}: {} - {}", id, status, error_text));
            Err(format!("DELETE /apikeys/{} failed ({}): {}", id, status, error_text).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opus_client::OpusClient;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn list_keys_with_app_filter() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 1024];
            let n = stream.read(&mut buf).await.unwrap();
            let req = String::from_utf8_lossy(&buf[..n]);
            assert!(req.starts_with("GET /apikeys?app=web HTTP/1.1"));
            assert!(req.contains("authorization: Bearer test-token"));
            let body = r#"{"keys":[{"id":1,"app":"web","kind":"secret","prefix":"opus_sec_","allowed_origins":[],"daily_quota":0,"used_today":0,"created_at":"2026-09-01T00:00:00Z"}]}"#;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(resp.as_bytes()).await.unwrap();
        });

        let client = OpusClient::new(
            format!("http://127.0.0.1:{}", port),
            "test-token".to_string(),
            "ws1".to_string(),
        );
        let keys = client.list_keys(Some("web")).await.unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].id, 1);
        assert_eq!(keys[0].app, "web");
        assert_eq!(keys[0].kind, "secret");
        assert_eq!(keys[0].prefix, "opus_sec_");
    }

    #[tokio::test]
    async fn list_keys_without_filter() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 1024];
            let n = stream.read(&mut buf).await.unwrap();
            let req = String::from_utf8_lossy(&buf[..n]);
            assert!(req.starts_with("GET /apikeys HTTP/1.1"));
            let body = r#"{"keys":[]}"#;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(resp.as_bytes()).await.unwrap();
        });

        let client = OpusClient::new(
            format!("http://127.0.0.1:{}", port),
            "test-token".to_string(),
            "ws1".to_string(),
        );
        let keys = client.list_keys(None).await.unwrap();
        assert_eq!(keys.len(), 0);
    }

    #[tokio::test]
    async fn create_key_success() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 1024];
            let n = stream.read(&mut buf).await.unwrap();
            let req = String::from_utf8_lossy(&buf[..n]);
            assert!(req.starts_with("POST /apikeys HTTP/1.1"));
            assert!(req.contains(r#""app":"testapp""#));
            assert!(req.contains(r#""kind":"public""#));
            assert!(req.contains("authorization: Bearer test-token"));
            let body = r#"{"key":{"id":5,"app":"testapp","kind":"public","prefix":"opus_pub_testapp_","allowed_origins":["https://example.com"],"daily_quota":500,"used_today":0,"created_at":"2026-09-01T00:00:00Z"},"token":"opus_pub_testapp_secret123"}"#;
            let resp = format!(
                "HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(resp.as_bytes()).await.unwrap();
        });

        let client = OpusClient::new(
            format!("http://127.0.0.1:{}", port),
            "test-token".to_string(),
            "ws1".to_string(),
        );
        let req = CreateKeyRequest {
            app: "testapp".to_string(),
            kind: "public".to_string(),
            allowed_origins: vec!["https://example.com".to_string()],
            daily_quota: 500,
        };
        let res = client.create_key(&req).await.unwrap();
        assert_eq!(res.key.id, 5);
        assert_eq!(res.token, "opus_pub_testapp_secret123");
    }

    #[tokio::test]
    async fn revoke_key_success() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 1024];
            let n = stream.read(&mut buf).await.unwrap();
            let req = String::from_utf8_lossy(&buf[..n]);
            assert!(req.starts_with("DELETE /apikeys/42 HTTP/1.1"));
            assert!(req.contains("authorization: Bearer test-token"));
            let resp = "HTTP/1.1 204 No Content\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
            stream.write_all(resp.as_bytes()).await.unwrap();
        });

        let client = OpusClient::new(
            format!("http://127.0.0.1:{}", port),
            "test-token".to_string(),
            "ws1".to_string(),
        );
        let res = client.revoke_key(42).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn endpoint_error_response() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf).await.unwrap();
            let body = r#"{"error":"unauthorized"}"#;
            let resp = format!(
                "HTTP/1.1 401 Unauthorized\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(resp.as_bytes()).await.unwrap();
        });

        let client = OpusClient::new(
            format!("http://127.0.0.1:{}", port),
            "bad-token".to_string(),
            "ws1".to_string(),
        );
        let res = client.list_keys(None).await;
        assert!(res.is_err());
    }
}
