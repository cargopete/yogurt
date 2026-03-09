# yogurt

**Write The Graph subgraphs in Rust.**

yogurt is a Rust toolchain that lets you write subgraph mapping handlers in Rust instead of AssemblyScript. It compiles to WASM modules that are binary-compatible with graph-node's existing runtime — zero modifications required.

## Why Rust?

- **Type Safety** — Rust's type system catches bugs at compile time
- **Better Tooling** — rust-analyzer, cargo, proper IDE support
- **Memory Safety** — The borrow checker prevents common bugs
- **Native Testing** — Run tests instantly without WASM compilation
- **Developer Experience** — Helpful compiler errors, no AS quirks

## Features

yogurt provides full feature parity with AssemblyScript subgraphs:

- Event handlers, block handlers, call handlers
- Data source templates
- File data sources (IPFS/Arweave)
- Contract calls (`ethereum.call`)
- Entity storage (CRUD operations)
- All graph-ts types (BigInt, BigDecimal, Bytes, Address)

Plus extras that make development more pleasant:

- `log_id!`, `day_id!`, `hour_id!` macros for common ID patterns
- Entity builders for fluent construction
- `load_or_create`, `update`, `upsert`, `exists` helpers
- `safe_div` for division without panics
- `format_units` / `parse_units` for token amounts
- Native testing framework with mock store

## Quick Example

```rust
use yogurt_runtime::prelude::*;
use crate::generated::{Transfer, TransferEvent};

#[handler]
fn handle_transfer(event: TransferEvent) {
    let id = log_id!(event);

    Transfer::builder(id)
        .from(event.params.from)
        .to(event.params.to)
        .value(event.params.value)
        .block_number(event.block.number.clone())
        .save();
}
```

## Getting Started

Ready to write your first Rust subgraph? Head to the [Installation](./getting-started/installation.md) guide.
