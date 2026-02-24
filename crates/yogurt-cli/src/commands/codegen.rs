//! Code generation command.

use anyhow::Result;
use console::style;
use std::path::Path;

pub fn run(manifest_path: &str) -> Result<()> {
    println!("{}", style("yogurt codegen").bold().cyan());
    println!();

    let manifest = Path::new(manifest_path);
    if !manifest.exists() {
        anyhow::bail!("Manifest not found: {}", manifest_path);
    }

    let output_dir = Path::new("src/generated");

    println!("  Reading {}...", manifest_path);

    yogurt_codegen::generate(manifest, output_dir)?;

    println!();
    println!("{}", style("✓ Code generation complete").green());

    Ok(())
}
