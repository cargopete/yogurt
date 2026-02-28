//! Deploy command — upload and deploy subgraph to a local graph-node.

use anyhow::{Context, Result};
use console::style;
use std::collections::HashMap;
use std::path::Path;

use crate::graph_node::GraphNodeClient;
use crate::ipfs::IpfsClient;

/// Configuration for deployment.
pub struct DeployConfig {
    pub subgraph_name: String,
    pub manifest_path: String,
    pub ipfs_url: String,
    pub node_url: String,
    pub version_label: Option<String>,
}

pub async fn run(
    node_url: Option<String>,
    ipfs_url: Option<String>,
    name: Option<String>,
    version: Option<String>,
) -> Result<()> {
    println!("{}", style("yogurt deploy").bold().cyan());
    println!();

    // Validate inputs
    let subgraph_name = name.ok_or_else(|| {
        anyhow::anyhow!(
            "Subgraph name required. Use: yogurt deploy <name>\n\
             Name format: account/subgraph (e.g., myaccount/erc20-tracker)"
        )
    })?;

    let manifest_path = "subgraph.yaml";
    if !Path::new(manifest_path).exists() {
        anyhow::bail!(
            "No subgraph.yaml found in current directory.\n\
             Run this command from your subgraph project root."
        );
    }

    // Check that build exists
    if !Path::new("build/subgraph.wasm").exists() {
        anyhow::bail!(
            "No build found at build/subgraph.wasm.\n\
             Run `yogurt build` first."
        );
    }

    let config = DeployConfig {
        subgraph_name,
        manifest_path: manifest_path.to_string(),
        ipfs_url: ipfs_url.unwrap_or_else(|| "http://localhost:5001".to_string()),
        node_url: node_url.unwrap_or_else(|| "http://localhost:8020".to_string()),
        version_label: version,
    };

    println!("  Subgraph: {}", style(&config.subgraph_name).yellow());
    println!("  IPFS:     {}", style(&config.ipfs_url).dim());
    println!("  Node:     {}", style(&config.node_url).dim());
    println!();

    deploy_to_node(&config).await
}

async fn deploy_to_node(config: &DeployConfig) -> Result<()> {
    let ipfs = IpfsClient::new(Some(&config.ipfs_url));
    let graph_node = GraphNodeClient::new(Some(&config.node_url));

    // Check connectivity
    print!("  Checking IPFS connection... ");
    ipfs.health_check()
        .await
        .context("IPFS node not reachable. Is `ipfs daemon` running?")?;
    println!("{}", style("ok").green());

    print!("  Checking graph-node connection... ");
    graph_node
        .health_check()
        .await
        .context("Graph-node not reachable. Is graph-node running?")?;
    println!("{}", style("ok").green());

    println!();

    // Parse the manifest
    let manifest_content = std::fs::read_to_string(&config.manifest_path)
        .context("Failed to read subgraph.yaml")?;
    let manifest: serde_yaml::Value =
        serde_yaml::from_str(&manifest_content).context("Failed to parse subgraph.yaml")?;

    let manifest_dir = Path::new(&config.manifest_path)
        .parent()
        .unwrap_or(Path::new("."));

    // Upload files and track their IPFS hashes
    let mut file_to_ipfs: HashMap<String, String> = HashMap::new();

    // Upload schema
    if let Some(schema_file) = manifest
        .get("schema")
        .and_then(|s| s.get("file"))
        .and_then(|f| f.as_str())
    {
        let schema_path = manifest_dir.join(schema_file);
        print!("  Uploading schema... ");
        let hash = ipfs
            .add_file(&schema_path)
            .await
            .context("Failed to upload schema")?;
        println!("{}", style(&hash).dim());
        file_to_ipfs.insert(schema_file.to_string(), hash);
    }

    // Upload ABIs and WASM from data sources
    if let Some(data_sources) = manifest.get("dataSources").and_then(|ds| ds.as_sequence()) {
        for ds in data_sources {
            if let Some(mapping) = ds.get("mapping") {
                // Upload ABIs
                if let Some(abis) = mapping.get("abis").and_then(|a| a.as_sequence()) {
                    for abi in abis {
                        if let Some(abi_file) = abi.get("file").and_then(|f| f.as_str()) {
                            if !file_to_ipfs.contains_key(abi_file) {
                                let abi_path = manifest_dir.join(abi_file);
                                let abi_name = abi
                                    .get("name")
                                    .and_then(|n| n.as_str())
                                    .unwrap_or("ABI");
                                print!("  Uploading {} ABI... ", abi_name);
                                let hash = ipfs
                                    .add_file(&abi_path)
                                    .await
                                    .with_context(|| format!("Failed to upload {}", abi_file))?;
                                println!("{}", style(&hash).dim());
                                file_to_ipfs.insert(abi_file.to_string(), hash);
                            }
                        }
                    }
                }

                // Upload WASM
                if let Some(wasm_file) = mapping.get("file").and_then(|f| f.as_str()) {
                    if !file_to_ipfs.contains_key(wasm_file) {
                        // Use our built WASM, not the path in manifest
                        let wasm_path = Path::new("build/subgraph.wasm");
                        print!("  Uploading WASM... ");
                        let hash = ipfs
                            .add_file(wasm_path)
                            .await
                            .context("Failed to upload WASM")?;
                        println!("{}", style(&hash).dim());
                        file_to_ipfs.insert(wasm_file.to_string(), hash);
                    }
                }
            }
        }
    }

    // Create resolved manifest with IPFS paths
    print!("  Creating resolved manifest... ");
    let resolved_manifest = resolve_manifest(&manifest, &file_to_ipfs)?;
    let resolved_yaml =
        serde_yaml::to_string(&resolved_manifest).context("Failed to serialize manifest")?;
    let manifest_hash = ipfs
        .add_str(&resolved_yaml, "subgraph.yaml")
        .await
        .context("Failed to upload manifest")?;
    println!("{}", style(&manifest_hash).dim());

    println!();

    // Create and deploy subgraph
    print!("  Creating subgraph... ");
    graph_node
        .subgraph_create(&config.subgraph_name)
        .await
        .context("Failed to create subgraph")?;
    println!("{}", style("ok").green());

    print!("  Deploying... ");
    graph_node
        .subgraph_deploy(
            &config.subgraph_name,
            &manifest_hash,
            config.version_label.as_deref(),
        )
        .await
        .context("Failed to deploy subgraph")?;
    println!("{}", style("ok").green());

    println!();
    println!("{}", style("✓ Deployment complete").green());
    println!();
    println!(
        "  Subgraph ID: {}",
        style(format!("/ipfs/{}", manifest_hash)).yellow()
    );
    println!(
        "  GraphQL endpoint: {}/subgraphs/name/{}",
        config.node_url.replace(":8020", ":8000"),
        config.subgraph_name
    );

    Ok(())
}

/// Replace local file paths with IPFS paths in the manifest.
fn resolve_manifest(
    manifest: &serde_yaml::Value,
    file_to_ipfs: &HashMap<String, String>,
) -> Result<serde_yaml::Value> {
    let mut resolved = manifest.clone();

    // Helper to replace a file path with IPFS path
    let to_ipfs_path = |path: &str| -> String {
        file_to_ipfs
            .get(path)
            .map(|hash| format!("/ipfs/{}", hash))
            .unwrap_or_else(|| path.to_string())
    };

    // Replace schema file
    if let Some(schema) = resolved.get_mut("schema") {
        if let Some(file) = schema.get_mut("file") {
            if let Some(path) = file.as_str() {
                *file = serde_yaml::Value::String(to_ipfs_path(path));
            }
        }
    }

    // Replace data source files
    if let Some(data_sources) = resolved.get_mut("dataSources") {
        if let Some(ds_seq) = data_sources.as_sequence_mut() {
            for ds in ds_seq {
                if let Some(mapping) = ds.get_mut("mapping") {
                    // Replace ABI files
                    if let Some(abis) = mapping.get_mut("abis") {
                        if let Some(abi_seq) = abis.as_sequence_mut() {
                            for abi in abi_seq {
                                if let Some(file) = abi.get_mut("file") {
                                    if let Some(path) = file.as_str() {
                                        *file = serde_yaml::Value::String(to_ipfs_path(path));
                                    }
                                }
                            }
                        }
                    }

                    // Replace WASM file
                    if let Some(file) = mapping.get_mut("file") {
                        if let Some(path) = file.as_str() {
                            *file = serde_yaml::Value::String(to_ipfs_path(path));
                        }
                    }
                }
            }
        }
    }

    // Replace template files (if any)
    if let Some(templates) = resolved.get_mut("templates") {
        if let Some(tmpl_seq) = templates.as_sequence_mut() {
            for tmpl in tmpl_seq {
                if let Some(mapping) = tmpl.get_mut("mapping") {
                    // Replace ABI files
                    if let Some(abis) = mapping.get_mut("abis") {
                        if let Some(abi_seq) = abis.as_sequence_mut() {
                            for abi in abi_seq {
                                if let Some(file) = abi.get_mut("file") {
                                    if let Some(path) = file.as_str() {
                                        *file = serde_yaml::Value::String(to_ipfs_path(path));
                                    }
                                }
                            }
                        }
                    }

                    // Replace WASM file
                    if let Some(file) = mapping.get_mut("file") {
                        if let Some(path) = file.as_str() {
                            *file = serde_yaml::Value::String(to_ipfs_path(path));
                        }
                    }
                }
            }
        }
    }

    Ok(resolved)
}
