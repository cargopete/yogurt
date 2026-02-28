//! Graph-node admin API client for subgraph deployment.
//!
//! Communicates with graph-node's JSON-RPC admin API to create and deploy subgraphs.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Graph-node admin API client.
pub struct GraphNodeClient {
    base_url: String,
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

impl GraphNodeClient {
    /// Create a new graph-node client.
    ///
    /// Default URL is `http://localhost:8020` (standard graph-node admin port).
    pub fn new(base_url: Option<&str>) -> Self {
        Self {
            base_url: base_url
                .unwrap_or("http://localhost:8020")
                .trim_end_matches('/')
                .to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Check if graph-node is reachable.
    pub async fn health_check(&self) -> Result<()> {
        // Try to get index node info - this should work on any running graph-node
        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id: 1,
            method: "subgraph_list",
            params: json!({}),
        };

        self.client
            .post(&self.base_url)
            .json(&request)
            .send()
            .await
            .context("Failed to connect to graph-node")?
            .error_for_status()
            .context("Graph-node returned error")?;

        Ok(())
    }

    /// Create a new subgraph.
    ///
    /// This registers the subgraph name but doesn't deploy any version yet.
    /// The name should be in the format "account/subgraph-name".
    pub async fn subgraph_create(&self, name: &str) -> Result<()> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id: 1,
            method: "subgraph_create",
            params: json!({ "name": name }),
        };

        let response = self
            .client
            .post(&self.base_url)
            .json(&request)
            .send()
            .await
            .context("Failed to create subgraph")?;

        let status = response.status();
        let body: JsonRpcResponse<serde_json::Value> = response
            .json()
            .await
            .context("Failed to parse graph-node response")?;

        if let Some(error) = body.error {
            // Code -32000 with "already exists" is fine - we'll just deploy over it
            if error.message.contains("already exists") {
                return Ok(());
            }
            anyhow::bail!(
                "Failed to create subgraph (code {}): {}",
                error.code,
                error.message
            );
        }

        if !status.is_success() {
            anyhow::bail!("Graph-node returned status {}", status);
        }

        Ok(())
    }

    /// Deploy a subgraph version.
    ///
    /// The `ipfs_hash` should be the CID of the subgraph manifest on IPFS.
    pub async fn subgraph_deploy(
        &self,
        name: &str,
        ipfs_hash: &str,
        version_label: Option<&str>,
    ) -> Result<()> {
        let mut params = json!({
            "name": name,
            "ipfs_hash": ipfs_hash,
        });

        if let Some(label) = version_label {
            params["version_label"] = json!(label);
        }

        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id: 1,
            method: "subgraph_deploy",
            params,
        };

        let response = self
            .client
            .post(&self.base_url)
            .json(&request)
            .send()
            .await
            .context("Failed to deploy subgraph")?;

        let status = response.status();
        let body: JsonRpcResponse<serde_json::Value> = response
            .json()
            .await
            .context("Failed to parse graph-node response")?;

        if let Some(error) = body.error {
            anyhow::bail!(
                "Failed to deploy subgraph (code {}): {}",
                error.code,
                error.message
            );
        }

        if !status.is_success() {
            anyhow::bail!("Graph-node returned status {}", status);
        }

        Ok(())
    }

    /// Remove a subgraph deployment.
    pub async fn subgraph_remove(&self, name: &str) -> Result<()> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id: 1,
            method: "subgraph_remove",
            params: json!({ "name": name }),
        };

        let response = self
            .client
            .post(&self.base_url)
            .json(&request)
            .send()
            .await
            .context("Failed to remove subgraph")?;

        let body: JsonRpcResponse<serde_json::Value> = response
            .json()
            .await
            .context("Failed to parse graph-node response")?;

        if let Some(error) = body.error {
            anyhow::bail!(
                "Failed to remove subgraph (code {}): {}",
                error.code,
                error.message
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = GraphNodeClient::new(None);
        assert_eq!(client.base_url, "http://localhost:8020");

        let client = GraphNodeClient::new(Some("http://127.0.0.1:8020/"));
        assert_eq!(client.base_url, "http://127.0.0.1:8020");
    }
}
