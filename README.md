# yogurt

A Rust toolchain for The Graph subgraphs.

Write your subgraph mapping handlers in Rust instead of AssemblyScript. yogurt compiles to WASM modules that are binary-compatible with graph-node's existing runtime — zero modifications required.

## Status

🚧 **Early development** — not yet production-ready.

## Installation

```bash
cargo install --path crates/yogurt-cli
```

## Usage

```bash
# Create a new subgraph project
yogurt init

# Generate Rust types from schema and ABIs
yogurt codegen

# Compile to WASM
yogurt build --release

# Validate WASM exports
yogurt validate

# Deploy (coming soon)
yogurt deploy --studio my-subgraph
```

## Example

```rust
use yogurt_runtime::prelude::*;
use crate::generated::{Transfer, TransferEvent};

#[handler]
fn handle_transfer(event: TransferEvent) {
    let id = event.transaction.hash.to_hex();
    let mut transfer = Transfer::new(id);
    transfer.set_from(event.params.from);
    transfer.set_to(event.params.to);
    transfer.set_value(event.params.value);
    transfer.save();
}
```

## Architecture

```
yogurt/
├── crates/
│   ├── yogurt-cli/       # The `yogurt` command
│   ├── yogurt-runtime/   # Rust equivalent of graph-ts
│   ├── yogurt-codegen/   # Schema/ABI → Rust code generation
│   └── yogurt-macros/    # #[handler] and other proc macros
├── templates/            # Project scaffolding templates
└── tests/
    ├── integration/      # End-to-end tests
    └── compat/           # AS binary compatibility tests
```

## How It Works

yogurt produces WASM binaries that are indistinguishable from AssemblyScript output at the binary level:

- **20-byte managed object headers** matching AS memory layout
- **UTF-16LE string encoding** for all string data
- **Runtime type IDs** compatible with graph-node's expectations
- **Exported functions** (`__new`, `__pin`, `__unpin`, `__collect`) that satisfy the AS runtime contract

The complexity lives in the toolchain, not your code. You write idiomatic Rust; yogurt handles the rest.

## Licence

MIT OR Apache-2.0
