//! Test command — run mapping handler tests.

use anyhow::Result;
use console::style;
use std::process::Command;

pub fn run(wasm: bool) -> Result<()> {
    println!("{}", style("yogurt test").bold().cyan());
    println!();

    if wasm {
        println!("  Running tests in WASM mode...");
        println!();
        println!(
            "  {} WASM test mode runs handlers in actual WASM for higher fidelity.",
            style("Note:").yellow()
        );
        println!("  This feature is planned but not yet implemented.");
        println!();
        println!("  For now, use native tests (without --wasm flag) which provide");
        println!("  fast iteration with the full testing framework.");
        println!();
        anyhow::bail!("WASM test mode not yet implemented. Use native tests instead.");
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
