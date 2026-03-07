//! Credential storage for Subgraph Studio authentication.
//!
//! Stores deploy keys in `~/.yogurt/credentials.json`.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Stored credentials for various services.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Credentials {
    /// Deploy key for Subgraph Studio.
    pub studio_deploy_key: Option<String>,
}

impl Credentials {
    /// Get the path to the credentials file.
    fn credentials_path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        Ok(home.join(".yogurt").join("credentials.json"))
    }

    /// Load credentials from disk.
    ///
    /// Returns empty credentials if the file doesn't exist.
    pub fn load() -> Result<Self> {
        let path = Self::credentials_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read credentials from {}", path.display()))?;

        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse credentials from {}", path.display()))
    }

    /// Save credentials to disk.
    ///
    /// Creates the `~/.yogurt/` directory if it doesn't exist.
    pub fn save(&self) -> Result<()> {
        let path = Self::credentials_path()?;

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }

        let content = serde_json::to_string_pretty(self).context("Failed to serialize credentials")?;

        fs::write(&path, content)
            .with_context(|| format!("Failed to write credentials to {}", path.display()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_credentials() {
        let creds = Credentials::default();
        assert!(creds.studio_deploy_key.is_none());
    }

    #[test]
    fn test_serialization() {
        let creds = Credentials {
            studio_deploy_key: Some("test-key-123".to_string()),
        };
        let json = serde_json::to_string(&creds).unwrap();
        let parsed: Credentials = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.studio_deploy_key, Some("test-key-123".to_string()));
    }
}
