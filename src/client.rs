use reqwest::Client;
use serde::{de::DeserializeOwned, Serialize};

const DEFAULT_HTTP_URL: &str = "http://127.0.0.1:7878";

/// Discover the Origin server URL.
/// Priority: CLI flag > HTTP default.
/// Note: UDS discovery disabled — reqwest doesn't support unix:// URLs natively.
/// Origin always binds HTTP on 127.0.0.1:7878 alongside UDS, so HTTP is reliable.
pub fn discover_origin_url(cli_url: Option<String>) -> String {
    if let Some(url) = cli_url {
        return url;
    }

    DEFAULT_HTTP_URL.to_string()
}

/// HTTP client for the Origin REST API.
#[derive(Clone)]
pub struct OriginClient {
    client: Client,
    base_url: String,
}

impl OriginClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    /// GET request, deserialize JSON response.
    pub async fn get<R: DeserializeOwned>(&self, path: &str) -> Result<R, OriginError> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| OriginError::Unreachable(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(OriginError::Api { status, body });
        }

        let bytes = resp.bytes().await.map_err(|e| {
            OriginError::Deserialize(format!("failed to read response body: {e:#}"))
        })?;

        serde_json::from_slice::<R>(&bytes).map_err(|e| {
            let preview = std::str::from_utf8(&bytes)
                .unwrap_or("<non-utf8>")
                .chars()
                .take(512)
                .collect::<String>();
            OriginError::Deserialize(format!("{e} (body preview: {preview})"))
        })
    }

    /// POST request with JSON body, deserialize JSON response.
    pub async fn post<B: Serialize, R: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<R, OriginError> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .post(&url)
            .json(body)
            .send()
            .await
            .map_err(|e| OriginError::Unreachable(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(OriginError::Api { status, body });
        }

        // Collect bytes first so that a body-read failure is distinguished
        // from a JSON parse failure, and the full error chain is preserved.
        let bytes = resp.bytes().await.map_err(|e| {
            OriginError::Deserialize(format!("failed to read response body: {e:#}"))
        })?;

        serde_json::from_slice::<R>(&bytes).map_err(|e| {
            // Include the first 512 bytes of the body to aid debugging without
            // flooding logs with potentially large payloads.
            let preview = std::str::from_utf8(&bytes)
                .unwrap_or("<non-utf8>")
                .chars()
                .take(512)
                .collect::<String>();
            OriginError::Deserialize(format!("{e} (body preview: {preview})"))
        })
    }

    /// DELETE request, deserialize JSON response.
    pub async fn delete<R: DeserializeOwned>(&self, path: &str) -> Result<R, OriginError> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .delete(&url)
            .send()
            .await
            .map_err(|e| OriginError::Unreachable(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(OriginError::Api { status, body });
        }

        let bytes = resp.bytes().await.map_err(|e| {
            OriginError::Deserialize(format!("failed to read response body: {e:#}"))
        })?;

        serde_json::from_slice::<R>(&bytes).map_err(|e| {
            let preview = std::str::from_utf8(&bytes)
                .unwrap_or("<non-utf8>")
                .chars()
                .take(512)
                .collect::<String>();
            OriginError::Deserialize(format!("{e} (body preview: {preview})"))
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum OriginError {
    #[error("Origin is not reachable: {0}")]
    Unreachable(String),

    #[error("Origin API error (HTTP {status}): {body}")]
    Api { status: u16, body: String },

    #[error("Failed to parse Origin response: {0}")]
    Deserialize(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_url_prefers_cli_flag() {
        let url = discover_origin_url(Some("http://localhost:9999".into()));
        assert_eq!(url, "http://localhost:9999");
    }

    #[test]
    fn test_discover_url_falls_back_to_http() {
        // With no CLI flag and no socket, should fall back to default HTTP
        let url = discover_origin_url(None);
        assert_eq!(url, "http://127.0.0.1:7878");
    }
}
