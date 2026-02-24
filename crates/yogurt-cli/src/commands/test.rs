//! Test command — run mapping handler tests.

use anyhow::Result;
use console::style;
use std::process::Command;

pub fn run(wasm: bool) -> Result<()> {
    println!("{}", style("yogurt test").bold().cyan());
    println!();

    if wasm {
        println!("  Running tests in WASM mode...");
        // TODO: Implement WASM test runner
        anyhow::bail!("WASM test mode not yet implemented");
    }

    // Run native tests via cargo test
    println!("  Compiling tests (native target)...");

    let status = Command::new("cargo")
        .arg("test")
        .arg("--features")
        .arg("testing")
        .status()?;

    if !status.success() {
        anyhow::bail!("Tests failed");
    }

    println!();
    println!("{}", style("✓ Tests passed").green());

    Ok(())
}
