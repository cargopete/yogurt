//! Inspect command — detailed WASM module analysis.

use anyhow::Result;
use console::style;
use std::fs;
use wasmparser::{Parser, Payload};

pub fn run(wasm_file: &str) -> Result<()> {
    println!("{}", style("yogurt inspect").bold().cyan());
    println!();

    let wasm_bytes = fs::read(wasm_file)?;
    let size_kb = wasm_bytes.len() as f64 / 1024.0;

    println!("  File: {}", style(wasm_file).yellow());
    println!("  Size: {:.1} KB", size_kb);
    println!();

    // Parse the WASM module
    let parser = Parser::new(0);
    let mut imports: Vec<(String, String)> = Vec::new();
    let mut exports: Vec<String> = Vec::new();
    let mut function_count = 0;
    let mut memory_pages = 0u32;
    let mut memory_max_pages: Option<u32> = None;
    let mut table_count = 0;
    let mut global_count = 0;
    let mut data_section_size = 0usize;

    for payload in parser.parse_all(&wasm_bytes) {
        match payload? {
            Payload::ImportSection(reader) => {
                for import in reader {
                    let import = import?;
                    imports.push((import.module.to_string(), import.name.to_string()));
                }
            }
            Payload::ExportSection(reader) => {
                for export in reader {
                    let export = export?;
                    exports.push(export.name.to_string());
                }
            }
            Payload::FunctionSection(reader) => {
                function_count = reader.count();
            }
            Payload::MemorySection(reader) => {
                for memory in reader {
                    let memory = memory?;
                    memory_pages = memory.initial as u32;
                    memory_max_pages = memory.maximum.map(|m| m as u32);
                }
            }
            Payload::TableSection(reader) => {
                table_count = reader.count();
            }
            Payload::GlobalSection(reader) => {
                global_count = reader.count();
            }
            Payload::DataSection(reader) => {
                for data in reader {
                    let data = data?;
                    data_section_size += data.data.len();
                }
            }
            _ => {}
        }
    }

    // Memory info
    println!("  {}", style("Memory:").underlined());
    println!(
        "    Initial: {} pages ({} KB)",
        memory_pages,
        memory_pages * 64
    );
    if let Some(max) = memory_max_pages {
        println!("    Maximum: {} pages ({} KB)", max, max * 64);
    } else {
        println!("    Maximum: unbounded");
    }
    println!("    Data section: {} bytes", data_section_size);
    println!();

    // Module stats
    println!("  {}", style("Module stats:").underlined());
    println!("    Functions: {}", function_count);
    println!("    Tables: {}", table_count);
    println!("    Globals: {}", global_count);
    println!();

    // Host imports
    println!("  {}", style("Host imports:").underlined());
    let host_imports: Vec<_> = imports
        .iter()
        .filter(|(module, _)| module == "env")
        .collect();

    if host_imports.is_empty() {
        println!("    None");
    } else {
        // Group by category
        let mut store_ops = Vec::new();
        let mut bigint_ops = Vec::new();
        let mut bigdecimal_ops = Vec::new();
        let mut crypto_ops = Vec::new();
        let mut ethereum_ops = Vec::new();
        let mut log_ops = Vec::new();
        let mut data_source_ops = Vec::new();
        let mut other_ops = Vec::new();

        for (_, name) in &host_imports {
            if name.starts_with("store.") {
                store_ops.push(name.as_str());
            } else if name.starts_with("bigInt.") {
                bigint_ops.push(name.as_str());
            } else if name.starts_with("bigDecimal.") {
                bigdecimal_ops.push(name.as_str());
            } else if name.starts_with("crypto.") {
                crypto_ops.push(name.as_str());
            } else if name.starts_with("ethereum.") {
                ethereum_ops.push(name.as_str());
            } else if name.starts_with("log.") {
                log_ops.push(name.as_str());
            } else if name.starts_with("dataSource.") {
                data_source_ops.push(name.as_str());
            } else {
                other_ops.push(name.as_str());
            }
        }

        print_import_group("Store", &store_ops);
        print_import_group("BigInt", &bigint_ops);
        print_import_group("BigDecimal", &bigdecimal_ops);
        print_import_group("Crypto", &crypto_ops);
        print_import_group("Ethereum", &ethereum_ops);
        print_import_group("Logging", &log_ops);
        print_import_group("DataSource", &data_source_ops);
        print_import_group("Other", &other_ops);
    }
    println!();

    // Exports
    println!("  {}", style("Exports:").underlined());

    // Runtime exports
    let runtime_exports: Vec<_> = exports
        .iter()
        .filter(|e| {
            e.starts_with("__")
                || *e == "memory"
                || *e == "abort"
                || *e == "id_of_type"
        })
        .collect();

    // Handler exports
    let handler_exports: Vec<_> = exports
        .iter()
        .filter(|e| {
            !e.starts_with("__")
                && *e != "memory"
                && *e != "abort"
                && *e != "id_of_type"
        })
        .collect();

    println!("    {}:", style("Runtime").dim());
    for export in &runtime_exports {
        println!("      {}", export);
    }

    println!("    {}:", style("Handlers").green());
    if handler_exports.is_empty() {
        println!("      {}", style("(none found)").yellow());
    } else {
        for export in &handler_exports {
            println!("      {} {}", style("→").cyan(), export);
        }
    }
    println!();

    // Validation summary
    let has_memory = exports.contains(&"memory".to_string());
    let has_new = exports.contains(&"__new".to_string());
    let has_pin = exports.contains(&"__pin".to_string());
    let has_unpin = exports.contains(&"__unpin".to_string());
    let has_collect = exports.contains(&"__collect".to_string());

    println!("  {}", style("Compatibility check:").underlined());
    print_check("memory export", has_memory);
    print_check("__new export", has_new);
    print_check("__pin export", has_pin);
    print_check("__unpin export", has_unpin);
    print_check("__collect export", has_collect);
    print_check("handler exports", !handler_exports.is_empty());
    println!();

    let all_required = has_memory && has_new && has_pin && has_unpin && has_collect;
    if all_required && !handler_exports.is_empty() {
        println!("{}", style("Ready for deployment").green().bold());
    } else {
        println!(
            "{}",
            style("Not ready for deployment (missing requirements)").yellow()
        );
    }

    Ok(())
}

fn print_import_group(name: &str, imports: &[&str]) {
    if !imports.is_empty() {
        println!(
            "    {} ({}): {}",
            style(name).dim(),
            imports.len(),
            imports.join(", ")
        );
    }
}

fn print_check(name: &str, passed: bool) {
    if passed {
        println!("    {} {}", style("✓").green(), name);
    } else {
        println!("    {} {}", style("✗").red(), name);
    }
}
