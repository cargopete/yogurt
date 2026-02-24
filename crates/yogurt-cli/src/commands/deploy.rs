//! Deploy command — upload and deploy subgraph.

use anyhow::Result;
use console::style;

pub async fn run(target: Option<String>, name: Option<String>) -> Result<()> {
    println!("{}", style("yogurt deploy").bold().cyan());
    println!();

    let target = target.unwrap_or_else(|| "studio".to_string());
    let name = name.ok_or_else(|| anyhow::anyhow!("Subgraph name required"))?;

    println!("  Target: {}", style(&target).yellow());
    println!("  Subgraph: {}", style(&name).yellow());
    println!();

    // Check that build exists
    if !std::path::Path::new("build/subgraph.wasm").exists() {
        anyhow::bail!("No build found. Run `yogurt build` first.");
    }

    println!("  Uploading to IPFS...");
    // TODO: Implement IPFS upload

    println!("  Deploying to {}...", target);
    // TODO: Implement deployment

    println!();
    println!(
        "{}",
        style("⚠ Deployment not yet implemented").yellow()
    );
    println!("  For now, use `graph deploy` with the built WASM file.");

    Ok(())
}
