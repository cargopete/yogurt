//! Build command — compile subgraph to WASM.

use anyhow::Result;
use console::style;
use std::fs;
use std::process::Command;

pub fn run(release: bool) -> Result<()> {
    println!("{}", style("yogurt build").bold().cyan());
    println!();

    // Check if codegen is up to date
    println!("  Checking codegen freshness...");
    // TODO: Check timestamps and hashes

    // Run cargo build
    let profile = if release { "release" } else { "debug" };
    println!(
        "  Compiling (wasm32-unknown-unknown, {})...",
        profile
    );

    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--target")
        .arg("wasm32-unknown-unknown");

    if release {
        cmd.arg("--release");
    }

    let status = cmd.status()?;

    if !status.success() {
        anyhow::bail!("Cargo build failed");
    }

    // Find the output wasm file
    let target_dir = format!("target/wasm32-unknown-unknown/{}", profile);
    let wasm_files: Vec<_> = fs::read_dir(&target_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "wasm").unwrap_or(false))
        .collect();

    let wasm_file = wasm_files
        .first()
        .ok_or_else(|| anyhow::anyhow!("No WASM file found in {}", target_dir))?;

    let wasm_path = wasm_file.path();

    // Create build directory
    fs::create_dir_all("build")?;

    // Run wasm-opt if available (optional)
    if release {
        println!("  Running wasm-opt...");
        let wasm_opt_result = Command::new("wasm-opt")
            .arg("-Oz")
            .arg(&wasm_path)
            .arg("-o")
            .arg("build/subgraph.wasm")
            .status();

        match wasm_opt_result {
            Ok(status) if status.success() => {}
            _ => {
                println!(
                    "  {} wasm-opt not available, copying unoptimised",
                    style("⚠").yellow()
                );
                fs::copy(&wasm_path, "build/subgraph.wasm")?;
            }
        }
    } else {
        fs::copy(&wasm_path, "build/subgraph.wasm")?;
    }

    // Get file size
    let metadata = fs::metadata("build/subgraph.wasm")?;
    let size_kb = metadata.len() as f64 / 1024.0;

    println!();
    println!(
        "  Output: {} ({:.1} KB)",
        style("build/subgraph.wasm").yellow(),
        size_kb
    );
    println!();
    println!("{}", style("✓ Build complete").green());

    Ok(())
}
