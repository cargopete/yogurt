//! IPFS HTTP API client for subgraph deployment.
//!
//! Uploads files to a local IPFS node and returns content identifiers (CIDs).

use anyhow::{Context, Result};
use reqwest::multipart;
use serde::Deserialize;

/// IPFS HTTP API client.
pub struct IpfsClient {
    base_url: String,
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct AddResponse {
    #[serde(rename = "Hash")]
    hash: String,
    #[serde(rename = "Name")]
    #[allow(dead_code)]
    name: String,
    #[serde(rename = "Size")]
    #[allow(dead_code)]
    size: String,
}

impl IpfsClient {
    /// Create a new IPFS client.
    ///
    /// Default URL is `http://localhost:5001` (standard IPFS daemon port).
    pub fn new(base_url: Option<&str>) -> Self {
        Self {
            base_url: base_url
                .unwrap_or("http://localhost:5001")
                .trim_end_matches('/')
                .to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Check if the IPFS node is reachable.
    pub async fn health_check(&self) -> Result<()> {
        let url = format!("{}/api/v0/id", self.base_url);
        self.client
            .post(&url)
            .send()
            .await
            .context("Failed to connect to IPFS node")?
            .error_for_status()
            .context("IPFS node returned error")?;
        Ok(())
    }

    /// Upload raw bytes to IPFS.
    ///
    /// Returns the CID (content identifier) of the uploaded content.
    pub async fn add_bytes(&self, data: Vec<u8>, filename: &str) -> Result<String> {
        let url = format!("{}/api/v0/add", self.base_url);

        let part = multipart::Part::bytes(data).file_name(filename.to_string());
        let form = multipart::Form::new().part("file", part);

        let response = self
            .client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .context("Failed to upload to IPFS")?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("IPFS upload failed ({}): {}", status, body);
        }

        let add_response: AddResponse = response
            .json()
            .await
            .context("Failed to parse IPFS response")?;

        Ok(add_response.hash)
    }

    /// Upload a string to IPFS.
    pub async fn add_str(&self, content: &str, filename: &str) -> Result<String> {
        self.add_bytes(content.as_bytes().to_vec(), filename).await
    }

    /// Upload a file from disk to IPFS.
    pub async fn add_file(&self, path: &std::path::Path) -> Result<String> {
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("file");

        let data = std::fs::read(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        self.add_bytes(data, filename).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = IpfsClient::new(None);
        assert_eq!(client.base_url, "http://localhost:5001");

        let client = IpfsClient::new(Some("http://127.0.0.1:5001/"));
        assert_eq!(client.base_url, "http://127.0.0.1:5001");
    }
}
