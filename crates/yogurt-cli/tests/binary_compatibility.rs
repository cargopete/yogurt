//! Binary compatibility tests for graph-node WASM modules.
//!
//! These tests verify that compiled subgraph WASM modules conform to
//! the AssemblyScript ABI expected by graph-node.

use std::path::PathBuf;
use std::process::Command;
use walrus::{ExportItem, Module, ValType};

/// Path to the PoC subgraph WASM (built by CI or manually).
fn poc_wasm_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests/integration/erc20-transfer/target/wasm32-unknown-unknown/release/erc20_transfer.wasm")
}

/// Build the PoC subgraph if it doesn't exist.
fn ensure_poc_built() {
    let wasm_path = poc_wasm_path();
    if wasm_path.exists() {
        return;
    }

    let project_dir = wasm_path
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let status = Command::new("cargo")
        .args([
            "build",
            "--release",
            "--target",
            "wasm32-unknown-unknown",
        ])
        .current_dir(project_dir)
        .status()
        .expect("Failed to run cargo build");

    assert!(status.success(), "Failed to build PoC subgraph");
}

/// Load the PoC WASM module.
fn load_poc_module() -> Module {
    ensure_poc_built();
    let wasm_bytes = std::fs::read(poc_wasm_path()).expect("Failed to read WASM");
    Module::from_buffer(&wasm_bytes).expect("Failed to parse WASM")
}

/// Get the function signature for an exported function.
fn get_export_func_signature(module: &Module, name: &str) -> Option<(Vec<ValType>, Vec<ValType>)> {
    for export in module.exports.iter() {
        if export.name == name {
            if let ExportItem::Function(func_id) = export.item {
                let func = module.funcs.get(func_id);
                let ty = module.types.get(func.ty());
                return Some((ty.params().to_vec(), ty.results().to_vec()));
            }
        }
    }
    None
}

/// Check if an export exists and is a function.
fn has_function_export(module: &Module, name: &str) -> bool {
    module.exports.iter().any(|e| {
        e.name == name && matches!(e.item, ExportItem::Function(_))
    })
}

/// Check if an export exists and is a memory.
fn has_memory_export(module: &Module, name: &str) -> bool {
    module.exports.iter().any(|e| {
        e.name == name && matches!(e.item, ExportItem::Memory(_))
    })
}

// ============================================================================
// Required Export Tests
// ============================================================================

#[test]
fn test_exports_memory() {
    let module = load_poc_module();
    assert!(
        has_memory_export(&module, "memory"),
        "Missing required export: memory"
    );
}

#[test]
fn test_exports_new() {
    let module = load_poc_module();
    assert!(
        has_function_export(&module, "__new"),
        "Missing required export: __new"
    );
}

#[test]
fn test_exports_pin() {
    let module = load_poc_module();
    assert!(
        has_function_export(&module, "__pin"),
        "Missing required export: __pin"
    );
}

#[test]
fn test_exports_unpin() {
    let module = load_poc_module();
    assert!(
        has_function_export(&module, "__unpin"),
        "Missing required export: __unpin"
    );
}

#[test]
fn test_exports_collect() {
    let module = load_poc_module();
    assert!(
        has_function_export(&module, "__collect"),
        "Missing required export: __collect"
    );
}

#[test]
fn test_exports_abort() {
    let module = load_poc_module();
    assert!(
        has_function_export(&module, "abort"),
        "Missing required export: abort"
    );
}

// ============================================================================
// Handler Export Tests
// ============================================================================

#[test]
fn test_handler_exports_exist() {
    let module = load_poc_module();
    assert!(
        has_function_export(&module, "handleTransfer"),
        "Missing handler export: handleTransfer"
    );
}

// ============================================================================
// WASM Module Structure Tests
// ============================================================================

#[test]
fn test_wasm_is_valid() {
    // If we can load it with walrus, it's valid WASM
    let _module = load_poc_module();
}

#[test]
fn test_wasm_size_under_limit() {
    ensure_poc_built();
    let wasm_bytes = std::fs::read(poc_wasm_path()).expect("Failed to read WASM");

    // Release builds should be under 100KB
    let size_kb = wasm_bytes.len() / 1024;
    assert!(
        size_kb < 100,
        "WASM size ({} KB) exceeds 100 KB target",
        size_kb
    );
}

#[test]
fn test_no_start_function() {
    let module = load_poc_module();
    // walrus exposes start function via module.start
    assert!(
        module.start.is_none(),
        "WASM should not have a start function - graph-node doesn't expect one"
    );
}

// ============================================================================
// Export Signature Tests (Function Type Validation)
// ============================================================================

#[test]
fn test_new_signature() {
    let module = load_poc_module();
    let (params, results) = get_export_func_signature(&module, "__new")
        .expect("__new not found");

    // __new: (size: i32, classId: i32) -> i32
    assert_eq!(params.len(), 2, "__new should take 2 parameters");
    assert_eq!(params[0], ValType::I32, "__new first param should be i32");
    assert_eq!(params[1], ValType::I32, "__new second param should be i32");
    assert_eq!(results.len(), 1, "__new should return 1 value");
    assert_eq!(results[0], ValType::I32, "__new should return i32");
}

#[test]
fn test_pin_signature() {
    let module = load_poc_module();
    let (params, results) = get_export_func_signature(&module, "__pin")
        .expect("__pin not found");

    // __pin: (ptr: i32) -> i32
    assert_eq!(params.len(), 1, "__pin should take 1 parameter");
    assert_eq!(params[0], ValType::I32, "__pin param should be i32");
    assert_eq!(results.len(), 1, "__pin should return 1 value");
    assert_eq!(results[0], ValType::I32, "__pin should return i32");
}

#[test]
fn test_unpin_signature() {
    let module = load_poc_module();
    let (params, results) = get_export_func_signature(&module, "__unpin")
        .expect("__unpin not found");

    // __unpin: (ptr: i32) -> void
    assert_eq!(params.len(), 1, "__unpin should take 1 parameter");
    assert_eq!(params[0], ValType::I32, "__unpin param should be i32");
    assert_eq!(results.len(), 0, "__unpin should return nothing");
}

#[test]
fn test_collect_signature() {
    let module = load_poc_module();
    let (params, results) = get_export_func_signature(&module, "__collect")
        .expect("__collect not found");

    // __collect: () -> void
    assert_eq!(params.len(), 0, "__collect should take 0 parameters");
    assert_eq!(results.len(), 0, "__collect should return nothing");
}

#[test]
fn test_abort_signature() {
    let module = load_poc_module();
    let (params, results) = get_export_func_signature(&module, "abort")
        .expect("abort not found");

    // abort: (msg: i32, file: i32, line: i32, col: i32) -> never (void)
    assert_eq!(params.len(), 4, "abort should take 4 parameters");
    for (i, param) in params.iter().enumerate() {
        assert_eq!(*param, ValType::I32, "abort param {} should be i32", i);
    }
    // abort doesn't return (it traps), so results should be empty
    assert_eq!(results.len(), 0, "abort should not return");
}

#[test]
fn test_handler_signature() {
    let module = load_poc_module();
    let (params, results) = get_export_func_signature(&module, "handleTransfer")
        .expect("handleTransfer not found");

    // Handler: (event_ptr: i32) -> void
    assert_eq!(
        params.len(),
        1,
        "Handler should take 1 parameter (event pointer)"
    );
    assert_eq!(
        params[0],
        ValType::I32,
        "Handler param should be i32 (pointer)"
    );
    assert_eq!(results.len(), 0, "Handler should return nothing");
}

// ============================================================================
// Import Validation Tests
// ============================================================================

#[test]
fn test_imports_use_correct_modules() {
    let module = load_poc_module();

    // All host function imports should be from known graph-node modules
    let allowed_modules = [
        "env",
        "ethereum",
        "store",
        "typeConversion",
        "bigInt",
        "bigDecimal",
        "json",
        "crypto",
        "ipfs",
        "log",
        "dataSource",
        "index", // graph-node uses this for some functions
    ];

    for import in module.imports.iter() {
        assert!(
            allowed_modules.contains(&import.module.as_str()),
            "Unexpected import module: '{}' (function: '{}'). \
             graph-node only supports specific import modules.",
            import.module,
            import.name
        );
    }
}

#[test]
fn test_no_wasi_imports() {
    let module = load_poc_module();

    // WASI imports would indicate non-graph-node-compatible code
    for import in module.imports.iter() {
        assert!(
            !import.module.starts_with("wasi"),
            "WASI import detected: {}::{}. graph-node does not support WASI.",
            import.module,
            import.name
        );
    }
}

// ============================================================================
// Memory Layout Tests
// ============================================================================

#[test]
fn test_single_memory() {
    let module = load_poc_module();

    // graph-node expects exactly one memory
    let memory_count = module.memories.iter().count();
    assert_eq!(
        memory_count, 1,
        "WASM should have exactly 1 memory, found {}",
        memory_count
    );
}

#[test]
fn test_memory_is_exported() {
    let module = load_poc_module();

    let memory_exports: Vec<_> = module
        .exports
        .iter()
        .filter(|e| matches!(e.item, ExportItem::Memory(_)))
        .collect();

    assert_eq!(
        memory_exports.len(),
        1,
        "Should export exactly 1 memory"
    );
    assert_eq!(
        memory_exports[0].name, "memory",
        "Memory export should be named 'memory'"
    );
}

// ============================================================================
// Summary Test
// ============================================================================

#[test]
fn test_full_compatibility_check() {
    let module = load_poc_module();

    // Required runtime exports
    let required_exports = ["memory", "__new", "__pin", "__unpin", "__collect", "abort"];

    let mut missing = Vec::new();
    for &name in &required_exports {
        let found = module.exports.iter().any(|e| e.name == name);
        if !found {
            missing.push(name);
        }
    }

    assert!(
        missing.is_empty(),
        "Missing required exports: {:?}",
        missing
    );

    // At least one handler should be exported
    let handler_count = module
        .exports
        .iter()
        .filter(|e| {
            matches!(e.item, ExportItem::Function(_))
                && !required_exports.contains(&e.name.as_str())
                && !e.name.starts_with("__")
        })
        .count();

    assert!(
        handler_count > 0,
        "No handler exports found. Subgraph needs at least one handler."
    );

    println!("Binary compatibility check passed:");
    println!("  - All {} required exports present", required_exports.len());
    println!("  - {} handler(s) exported", handler_count);
    println!(
        "  - WASM size: {} KB",
        std::fs::read(poc_wasm_path()).unwrap().len() / 1024
    );
}
