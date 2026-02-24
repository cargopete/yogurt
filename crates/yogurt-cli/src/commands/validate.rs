//! Validate command — check WASM exports for graph-node compatibility.

use anyhow::Result;
use console::style;
use std::fs;

/// Required exports for graph-node compatibility.
const REQUIRED_EXPORTS: &[&str] = &[
    "memory",
    "__new",
    "__pin",
    "__unpin",
    "__collect",
];

pub fn run(wasm_file: &str) -> Result<()> {
    println!("{}", style("yogurt validate").bold().cyan());
    println!();

    let wasm_bytes = fs::read(wasm_file)?;

    println!("  Validating {}...", wasm_file);
    println!();

    // Parse WASM module
    let module = walrus::Module::from_buffer(&wasm_bytes)?;

    // Check required exports
    let mut missing = Vec::new();
    let mut found = Vec::new();

    for &name in REQUIRED_EXPORTS {
        if module.exports.iter().any(|e| e.name == name) {
            found.push(name);
        } else {
            missing.push(name);
        }
    }

    // Report findings
    for name in &found {
        println!("    {} {}", style("✓").green(), name);
    }

    for name in &missing {
        println!("    {} {} (missing)", style("✗").red(), name);
    }

    // List handler exports
    println!();
    println!("  Handler exports:");

    let handlers: Vec<_> = module
        .exports
        .iter()
        .filter(|e| {
            !REQUIRED_EXPORTS.contains(&e.name.as_str())
                && !e.name.starts_with("__")
                && e.name != "memory"
        })
        .collect();

    if handlers.is_empty() {
        println!("    {} No handlers found", style("⚠").yellow());
    } else {
        for export in handlers {
            println!("    {} {}", style("✓").green(), export.name);
        }
    }

    println!();

    if missing.is_empty() {
        println!("{}", style("✓ Validation passed").green());
        Ok(())
    } else {
        println!(
            "{}",
            style("✗ Validation failed — missing required exports").red()
        );
        anyhow::bail!("Missing required exports: {:?}", missing);
    }
}
