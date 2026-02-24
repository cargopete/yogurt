//! Error types for code generation.

use std::io;
use thiserror::Error;

/// Result type for codegen operations.
pub type Result<T> = std::result::Result<T, CodegenError>;

/// Errors that can occur during code generation.
#[derive(Debug, Error)]
pub enum CodegenError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("GraphQL parsing error: {0}")]
    GraphQL(String),

    #[error("ABI parsing error: {0}")]
    Abi(String),

    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),

    #[error("Unsupported type: {0}")]
    UnsupportedType(String),
}
