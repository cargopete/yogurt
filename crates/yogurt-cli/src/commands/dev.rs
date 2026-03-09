//! Dev command — watch for changes and auto-rebuild.

use anyhow::Result;
use console::style;
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode, DebouncedEventKind};
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

use super::{build, codegen};

/// Patterns to watch for changes.
const WATCH_PATTERNS: &[&str] = &[
    "src/**/*.rs",
    "schema.graphql",
    "abis/**/*.json",
    "subgraph.yaml",
];

pub fn run(manifest: &str) -> Result<()> {
    println!("{}", style("yogurt dev").bold().cyan());
    println!();
    println!("  Watching for changes in:");
    for pattern in WATCH_PATTERNS {
        println!("    {}", style(pattern).dim());
    }
    println!();

    // Initial build
    println!("{}", style("Running initial build...").yellow());
    if let Err(e) = run_build_cycle(manifest) {
        println!("  {} Initial build failed: {}", style("✗").red(), e);
    }
    println!();
    println!(
        "{}",
        style("Watching for changes... (Ctrl+C to stop)").cyan()
    );
    println!();

    // Set up file watcher with debouncing
    let (tx, rx) = channel();
    let mut debouncer = new_debouncer(Duration::from_millis(500), tx)?;

    // Watch the relevant directories
    let paths_to_watch = ["src", "abis", "schema.graphql", "subgraph.yaml"];

    for path_str in paths_to_watch {
        let path = Path::new(path_str);
        if path.exists() {
            let mode = if path.is_dir() {
                RecursiveMode::Recursive
            } else {
                RecursiveMode::NonRecursive
            };
            debouncer.watcher().watch(path, mode)?;
        }
    }

    // Main watch loop
    loop {
        match rx.recv() {
            Ok(Ok(events)) => {
                // Filter for actual file changes (not directories)
                let relevant_changes: Vec<_> = events
                    .iter()
                    .filter(|e| matches!(e.kind, DebouncedEventKind::Any))
                    .filter(|e| is_relevant_path(&e.path))
                    .collect();

                if !relevant_changes.is_empty() {
                    // Get unique changed files for display
                    let changed_files: Vec<_> = relevant_changes
                        .iter()
                        .filter_map(|e| e.path.file_name())
                        .map(|n| n.to_string_lossy().to_string())
                        .collect::<std::collections::HashSet<_>>()
                        .into_iter()
                        .take(3)
                        .collect();

                    let changes_desc = if changed_files.len() <= 3 {
                        changed_files.join(", ")
                    } else {
                        format!("{} and {} more", changed_files.join(", "), changed_files.len() - 3)
                    };

                    println!();
                    println!(
                        "  {} Changes detected: {}",
                        style("→").cyan(),
                        style(&changes_desc).dim()
                    );

                    if let Err(e) = run_build_cycle(manifest) {
                        println!("  {} Build failed: {}", style("✗").red(), e);
                    }
                }
            }
            Ok(Err(e)) => {
                println!("  {} Watch error: {:?}", style("⚠").yellow(), e);
            }
            Err(e) => {
                println!("  {} Channel error: {}", style("✗").red(), e);
                break;
            }
        }
    }

    Ok(())
}

/// Run a full codegen + build cycle.
fn run_build_cycle(manifest: &str) -> Result<()> {
    let start = std::time::Instant::now();

    // Run codegen if manifest exists
    if Path::new(manifest).exists() {
        print!("  Running codegen... ");
        match codegen::run(manifest) {
            Ok(_) => println!("{}", style("✓").green()),
            Err(e) => {
                println!("{}", style("✗").red());
                return Err(e);
            }
        }
    }

    // Run debug build
    print!("  Building (debug)... ");
    match build::run(false) {
        Ok(_) => {
            let elapsed = start.elapsed();
            println!(
                "{} ({:.1}s)",
                style("✓").green(),
                elapsed.as_secs_f64()
            );
            Ok(())
        }
        Err(e) => {
            println!("{}", style("✗").red());
            Err(e)
        }
    }
}

/// Check if a path is relevant for triggering a rebuild.
fn is_relevant_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();

    // Skip hidden files and directories
    if path_str.contains("/.") || path_str.contains("\\.") {
        return false;
    }

    // Skip target directory
    if path_str.contains("/target/") || path_str.contains("\\target\\") {
        return false;
    }

    // Check for relevant extensions
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        matches!(ext_str.as_str(), "rs" | "graphql" | "json" | "yaml" | "yml")
    } else if let Some(name) = path.file_name() {
        // Files without extension (like schema.graphql)
        let name_str = name.to_string_lossy();
        name_str == "schema.graphql" || name_str == "subgraph.yaml"
    } else {
        false
    }
}
