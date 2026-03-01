# yogurt

A Rust toolchain for The Graph subgraphs.

Write your subgraph mapping handlers in Rust instead of AssemblyScript. yogurt compiles to WASM modules that are binary-compatible with graph-node's existing runtime — zero modifications required.

## Status

🚧 **Early development** — not yet production-ready.

**Working:**
- `yogurt init` — project scaffolding
- `yogurt codegen` — generate Rust types from schema/ABIs
- `yogurt build` — compile to WASM with optional wasm-opt
- `yogurt validate` — check WASM binary compatibility
- `yogurt deploy` — deploy to local graph-node
- Event handlers, block handlers, call handlers
- Data source templates (`dataSource.create()`)
- File data sources (IPFS/Arweave)
- Contract calls (`ethereum.call`)

**Coming soon:**
- Subgraph Studio deployment
- Testing framework improvements

## Installation

```bash
cargo install --path crates/yogurt-cli
```

Requires:
- Rust 1.80+ with `wasm32-unknown-unknown` target
- (Optional) `wasm-opt` for release builds

```bash
rustup target add wasm32-unknown-unknown
```

## Quick Start

```bash
# Create a new subgraph project
yogurt init

# Generate Rust types from schema and ABIs
yogurt codegen

# Compile to WASM
yogurt build --release

# Validate WASM exports
yogurt validate

# Deploy to local graph-node
yogurt deploy myaccount/my-subgraph
```

## Example Handler

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

## Local Development

Test against a local graph-node using the included scripts:

```bash
# Start IPFS, Postgres, and graph-node
./scripts/test-deploy.sh --up

# Build and deploy the test subgraph
./scripts/test-deploy.sh --deploy

# Check graph-node logs
./scripts/test-deploy.sh --logs

# Tear down
./scripts/test-deploy.sh --down
```

For a custom Ethereum RPC:

```bash
ETHEREUM_RPC="https://mainnet.infura.io/v3/YOUR_KEY" ./scripts/test-deploy.sh --up
```

## Deploy Command

```bash
# Deploy to local graph-node (default ports)
yogurt deploy myaccount/my-subgraph

# Custom endpoints
yogurt deploy myaccount/my-subgraph \
  --node http://localhost:8020 \
  --ipfs http://localhost:5001 \
  --version v1.0.0
```

Requirements:
- IPFS node running (default: `http://localhost:5001`)
- graph-node admin API (default: `http://localhost:8020`)

## Architecture

```
yogurt/
├── crates/
│   ├── yogurt-cli/       # The `yogurt` command
│   ├── yogurt-runtime/   # Rust equivalent of graph-ts
│   ├── yogurt-codegen/   # Schema/ABI → Rust code generation
│   └── yogurt-macros/    # #[handler] and other proc macros
├── scripts/              # Local dev/test scripts
└── tests/
    └── integration/      # End-to-end tests
```

## How It Works

yogurt produces WASM binaries that are indistinguishable from AssemblyScript output at the binary level:

- **20-byte managed object headers** matching AS memory layout
- **UTF-16LE string encoding** for all string data
- **Runtime type IDs** compatible with graph-node's expectations
- **Exported functions** (`__new`, `__pin`, `__unpin`, `__collect`) that satisfy the AS runtime contract

The complexity lives in the toolchain, not your code. You write idiomatic Rust; yogurt handles the rest.

## Codegen Features

**Schema (GraphQL → Rust):**
- Entity structs with getters/setters
- `@entity(immutable: true)` support (no setters generated)
- `@derivedFrom` directive handling
- Nullable fields (`Option<T>`)
- Array fields (`Vec<T>`)
- Relation fields

**ABI (JSON → Rust):**
- Event structs with typed parameters
- Call handler structs with typed inputs/outputs
- Contract bindings for view/pure functions
- Tuple parameter support
- Fixed-size array support (`[T; N]`)
- Automatic `FromAscPtr` implementation

**Templates:**
- Data source template generation
- File data source support (`file/ipfs`, `file/arweave`)
- `Template::create()` methods for spawning new data sources

## Licence

MIT OR Apache-2.0
