//! Build command — compile subgraph to WASM.

use anyhow::Result;
use console::style;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use walrus::{ir::Value, ConstExpr, Module, ValType};

/// Marker file that records the yogurt build metadata.
/// Used to detect if another tool (like graph-cli) has overwritten our WASM.
pub const BUILD_MARKER_FILE: &str = "build/.yogurt-build";

/// Maximum expected size for a Rust subgraph WASM (in bytes).
/// AssemblyScript builds are typically much larger (1MB+).
/// Rust release builds are usually under 100KB.
const MAX_EXPECTED_WASM_SIZE: u64 = 500 * 1024; // 500 KB

pub fn run(release: bool) -> Result<()> {
    println!("{}", style("yogurt build").bold().cyan());
    println!();

    // Check if codegen is up to date
    let manifest_path = Path::new("subgraph.yaml");
    let output_dir = Path::new("src/generated");

    if manifest_path.exists() && output_dir.exists() {
        print!("  Checking codegen freshness... ");
        match yogurt_codegen::is_codegen_fresh(manifest_path, output_dir) {
            Ok(true) => {
                println!("{}", style("up to date").green());
            }
            Ok(false) => {
                println!("{}", style("stale, regenerating").yellow());
                yogurt_codegen::generate(manifest_path, output_dir)?;
                println!("  {} Codegen complete", style("✓").green());
            }
            Err(e) => {
                println!("{}", style(format!("error: {}", e)).red());
                // Continue with build anyway
            }
        }
    }

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

    // Run wasm-opt with bulk memory lowering for graph-node compatibility
    // Graph-node doesn't support WASM bulk memory operations (memory.copy, memory.fill)
    // which modern Rust compilers emit. The --llvm-memory-copy-fill-lowering pass
    // converts these to MVP-compatible loop-based implementations.
    println!("  Running wasm-opt (bulk memory lowering)...");
    let wasm_opt_result = Command::new("wasm-opt")
        .arg("--enable-bulk-memory-opt")
        .arg("--llvm-memory-copy-fill-lowering")
        .arg(if release { "-Oz" } else { "-O1" })
        .arg(&wasm_path)
        .arg("-o")
        .arg("build/subgraph.wasm")
        .status();

    match wasm_opt_result {
        Ok(status) if status.success() => {
            println!("  {} WASM optimised for graph-node compatibility", style("✓").green());
        }
        _ => {
            println!(
                "  {} wasm-opt not available — install binaryen for graph-node compatibility",
                style("✗").red()
            );
            println!(
                "    {}",
                style("brew install binaryen  # or apt-get install binaryen").dim()
            );
            fs::copy(&wasm_path, "build/subgraph.wasm")?;
        }
    }

    // Inject TypeId globals required by graph-node
    println!("  Injecting TypeId globals...");
    inject_type_id_globals("build/subgraph.wasm")?;
    println!("  {} TypeId globals added", style("✓").green());

    // Get file size and compute hash
    let metadata = fs::metadata("build/subgraph.wasm")?;
    let size = metadata.len();
    let size_kb = size as f64 / 1024.0;

    // Compute SHA256 hash for integrity verification
    let hash = compute_wasm_hash("build/subgraph.wasm")?;

    // Write build marker file for deploy verification
    write_build_marker(size, &hash)?;

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

/// Compute SHA256 hash of a file.
fn compute_wasm_hash(path: &str) -> Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Write a marker file recording the build metadata.
fn write_build_marker(size: u64, hash: &str) -> Result<()> {
    let marker_content = format!(
        "# Yogurt build marker — do not edit\n\
         # This file is used to verify the WASM hasn't been overwritten.\n\
         size={}\n\
         hash={}\n\
         timestamp={}\n",
        size,
        hash,
        chrono::Utc::now().to_rfc3339()
    );
    fs::write(BUILD_MARKER_FILE, marker_content)?;
    Ok(())
}

/// Verify the WASM file matches the build marker.
/// Returns Ok(()) if valid, Err with explanation if not.
pub fn verify_wasm_integrity() -> Result<()> {
    let wasm_path = Path::new("build/subgraph.wasm");
    let marker_path = Path::new(BUILD_MARKER_FILE);

    if !wasm_path.exists() {
        anyhow::bail!(
            "No WASM found at build/subgraph.wasm.\n\
             Run `yogurt build` first."
        );
    }

    // Check file size first (quick sanity check)
    let metadata = fs::metadata(wasm_path)?;
    let current_size = metadata.len();

    if current_size > MAX_EXPECTED_WASM_SIZE {
        anyhow::bail!(
            "WASM file is suspiciously large ({:.1} MB).\n\
             Expected < 500 KB for a Rust build.\n\n\
             This usually means `graph build` or `graph deploy` overwrote your Rust WASM\n\
             with an AssemblyScript build.\n\n\
             Fix: Run `yogurt build --release` to rebuild.",
            current_size as f64 / (1024.0 * 1024.0)
        );
    }

    // If marker file exists, verify hash
    if marker_path.exists() {
        let marker_content = fs::read_to_string(marker_path)?;
        let mut expected_size: Option<u64> = None;
        let mut expected_hash: Option<String> = None;

        for line in marker_content.lines() {
            if let Some(value) = line.strip_prefix("size=") {
                expected_size = value.parse().ok();
            } else if let Some(value) = line.strip_prefix("hash=") {
                expected_hash = Some(value.to_string());
            }
        }

        if let (Some(exp_size), Some(exp_hash)) = (expected_size, expected_hash) {
            if current_size != exp_size {
                anyhow::bail!(
                    "WASM file size changed since last yogurt build.\n\
                     Expected: {} bytes, Found: {} bytes\n\n\
                     This may indicate another tool modified build/subgraph.wasm.\n\n\
                     Fix: Run `yogurt build --release` to rebuild.",
                    exp_size,
                    current_size
                );
            }

            let current_hash = compute_wasm_hash("build/subgraph.wasm")?;
            if current_hash != exp_hash {
                anyhow::bail!(
                    "WASM file hash changed since last yogurt build.\n\n\
                     This may indicate another tool modified build/subgraph.wasm.\n\n\
                     Fix: Run `yogurt build --release` to rebuild."
                );
            }
        }
    }

    Ok(())
}

/// Inject TypeId global exports required by graph-node.
///
/// Graph-node reads these exported globals to map type names to runtime class IDs.
/// The values must match graph-node's IndexForAscTypeId enum exactly.
fn inject_type_id_globals(wasm_path: &str) -> Result<()> {
    // TypeId names and their values from graph-node's IndexForAscTypeId enum
    let type_ids: &[(&str, i32)] = &[
        ("TypeId.String", 0),
        ("TypeId.ArrayBuffer", 1),
        ("TypeId.Int8Array", 2),
        ("TypeId.Int16Array", 3),
        ("TypeId.Int32Array", 4),
        ("TypeId.Int64Array", 5),
        ("TypeId.Uint8Array", 6),
        ("TypeId.Uint16Array", 7),
        ("TypeId.Uint32Array", 8),
        ("TypeId.Uint64Array", 9),
        ("TypeId.Float32Array", 10),
        ("TypeId.Float64Array", 11),
        ("TypeId.BigDecimal", 12),
        ("TypeId.ArrayBool", 13),
        ("TypeId.ArrayUint8Array", 14),
        ("TypeId.ArrayEthereumValue", 15),
        ("TypeId.ArrayStoreValue", 16),
        ("TypeId.ArrayJsonValue", 17),
        ("TypeId.ArrayString", 18),
        ("TypeId.ArrayEventParam", 19),
        ("TypeId.ArrayTypedMapEntryStringJsonValue", 20),
        ("TypeId.ArrayTypedMapEntryStringStoreValue", 21),
        ("TypeId.SmartContractCall", 22),
        ("TypeId.EventParam", 23),
        ("TypeId.EthereumTransaction", 24),
        ("TypeId.EthereumBlock", 25),
        ("TypeId.EthereumCall", 26),
        ("TypeId.WrappedTypedMapStringJsonValue", 27),
        ("TypeId.WrappedBool", 28),
        ("TypeId.WrappedJsonValue", 29),
        ("TypeId.EthereumValue", 30),
        ("TypeId.StoreValue", 31),
        ("TypeId.JsonValue", 32),
        ("TypeId.EthereumEvent", 33),
        ("TypeId.TypedMapEntryStringStoreValue", 34),
        ("TypeId.TypedMapEntryStringJsonValue", 35),
        ("TypeId.TypedMapStringStoreValue", 36),
        ("TypeId.TypedMapStringJsonValue", 37),
        ("TypeId.TypedMapStringTypedMapStringJsonValue", 38),
        ("TypeId.ResultTypedMapStringJsonValueBool", 39),
        ("TypeId.ResultJsonValueBool", 40),
        ("TypeId.ArrayU8", 41),
        ("TypeId.ArrayU16", 42),
        ("TypeId.ArrayU32", 43),
        ("TypeId.ArrayU64", 44),
        ("TypeId.ArrayI8", 45),
        ("TypeId.ArrayI16", 46),
        ("TypeId.ArrayI32", 47),
        ("TypeId.ArrayI64", 48),
        ("TypeId.ArrayF32", 49),
        ("TypeId.ArrayF64", 50),
        ("TypeId.ArrayBigDecimal", 51),
    ];

    // Load the WASM module
    let wasm_bytes = fs::read(wasm_path)?;
    let mut module = Module::from_buffer(&wasm_bytes)?;

    // Add each TypeId global and export it
    for (name, value) in type_ids {
        let global_id = module.globals.add_local(
            ValType::I32,
            false, // not mutable
            false, // not shared
            ConstExpr::Value(Value::I32(*value)),
        );
        module.exports.add(name, global_id);
    }

    // Write the modified WASM back
    module.emit_wasm_file(wasm_path)?;

    Ok(())
}
