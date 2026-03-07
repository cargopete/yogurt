//! Subgraph Studio API client for deploying to The Graph's hosted service.
//!
//! Uses the same JSON-RPC protocol as graph-node but with authentication.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Studio deploy endpoint.
pub const STUDIO_DEPLOY_URL: &str = "https://api.studio.thegraph.com/deploy/";

/// Studio IPFS endpoint (without /api/v0 suffix - added by IpfsClient).
pub const STUDIO_IPFS_URL: &str = "https://api.thegraph.com/ipfs";

/// Subgraph Studio API client.
pub struct StudioClient {
    deploy_url: String,
    deploy_key: String,
    client: reqwest::Client,
}

#[derive(Debug, Serialize)]
struct JsonRpcRequest<T: Serialize> {
    jsonrpc: &'static str,
    id: u32,
    method: &'static str,
    params: T,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse<T> {
    #[allow(dead_code)]
    jsonrpc: String,
    #[allow(dead_code)]
    id: u32,
    result: Option<T>,
    error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

impl StudioClient {
    /// Create a new Studio client with the given deploy key.
    pub fn new(deploy_key: &str) -> Self {
        Self {
            deploy_url: STUDIO_DEPLOY_URL.to_string(),
            deploy_key: deploy_key.to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Deploy a subgraph to Studio.
    ///
    /// Note: The subgraph must be created in the Studio web UI first.
    /// The `name` should match the subgraph slug from Studio.
    pub async fn deploy(&self, name: &str, ipfs_hash: &str, version: &str) -> Result<()> {
        let params = json!({
            "name": name,
            "ipfs_hash": ipfs_hash,
            "version_label": version,
        });

        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id: 1,
            method: "subgraph_deploy",
            params,
        };

        let response = self
            .client
            .post(&self.deploy_url)
            .header("Authorization", format!("Bearer {}", self.deploy_key))
            .json(&request)
            .send()
            .await
            .context("Failed to connect to Subgraph Studio")?;

        let status = response.status();
        let body: JsonRpcResponse<serde_json::Value> = response
            .json()
            .await
            .context("Failed to parse Studio response")?;

        if let Some(error) = body.error {
            anyhow::bail!(
                "Studio deployment failed (code {}): {}",
                error.code,
                error.message
            );
        }

        if !status.is_success() {
            anyhow::bail!("Studio returned status {}", status);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = StudioClient::new("test-key");
        assert_eq!(client.deploy_url, STUDIO_DEPLOY_URL);
        assert_eq!(client.deploy_key, "test-key");
    }

    #[test]
    fn test_constants() {
        assert!(STUDIO_DEPLOY_URL.starts_with("https://"));
        assert!(STUDIO_IPFS_URL.starts_with("https://"));
    }
}
