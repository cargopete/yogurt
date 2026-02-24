//! Subgraph manifest (subgraph.yaml) parsing.

use serde::Deserialize;

use crate::error::{CodegenError, Result};

/// The top-level subgraph manifest.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub spec_version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
    pub schema: Schema,
    pub data_sources: Vec<DataSource>,
    #[serde(default)]
    pub templates: Vec<DataSourceTemplate>,
}

impl Manifest {
    /// Parse a manifest from YAML content.
    pub fn parse(content: &str) -> Result<Self> {
        serde_yaml::from_str(content).map_err(Into::into)
    }
}

/// Schema file reference.
#[derive(Debug, Deserialize)]
pub struct Schema {
    pub file: String,
}

/// A data source definition.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSource {
    pub kind: String,
    pub name: String,
    pub network: String,
    pub source: Source,
    pub mapping: Mapping,
}

/// Contract source information.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    pub address: String,
    pub abi: String,
    #[serde(default)]
    pub start_block: Option<u64>,
}

/// Mapping configuration.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mapping {
    pub kind: String,
    pub api_version: String,
    #[serde(default)]
    pub language: String,
    pub entities: Vec<String>,
    pub abis: Vec<AbiRef>,
    #[serde(default)]
    pub event_handlers: Vec<EventHandler>,
    #[serde(default)]
    pub call_handlers: Vec<CallHandler>,
    #[serde(default)]
    pub block_handlers: Vec<BlockHandler>,
    pub file: String,
}

/// ABI file reference.
#[derive(Debug, Deserialize)]
pub struct AbiRef {
    pub name: String,
    pub file: String,
}

/// Event handler definition.
#[derive(Debug, Deserialize)]
pub struct EventHandler {
    pub event: String,
    pub handler: String,
    #[serde(default)]
    pub receipt: bool,
}

/// Call handler definition.
#[derive(Debug, Deserialize)]
pub struct CallHandler {
    pub function: String,
    pub handler: String,
}

/// Block handler definition.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockHandler {
    pub handler: String,
    #[serde(default)]
    pub filter: Option<BlockFilter>,
}

/// Block filter (for block handlers).
#[derive(Debug, Deserialize)]
pub struct BlockFilter {
    pub kind: String,
}

/// Data source template for dynamic data sources.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceTemplate {
    pub kind: String,
    pub name: String,
    pub network: String,
    pub source: TemplateSource,
    pub mapping: Mapping,
}

/// Template source (no address, as it's provided at runtime).
#[derive(Debug, Deserialize)]
pub struct TemplateSource {
    pub abi: String,
}
