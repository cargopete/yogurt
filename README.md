# yogurt

A Rust toolchain for The Graph subgraphs.

Write your subgraph mapping handlers in Rust instead of AssemblyScript. yogurt compiles to WASM modules that are binary-compatible with graph-node's existing runtime — zero modifications required.

> **March 2026:** The first-ever Rust subgraph was deployed to Subgraph Studio.

## Status

⚠️ This software is a POC and is purely for demo purposes.

**Features:**
- `yogurt init` — project scaffolding
- `yogurt codegen` — generate Rust types from schema/ABIs
- `yogurt build` — compile to WASM with optional wasm-opt
- `yogurt validate` — check WASM binary compatibility
- `yogurt deploy` — deploy to local graph-node or Subgraph Studio
- `yogurt dev` — file watching with auto-rebuild
- `yogurt inspect` — WASM module analysis
- `yogurt auth` — store Studio deploy key
- Event handlers, block handlers, call handlers
- Data source templates (`dataSource.create()`)
- File data sources (IPFS/Arweave)
- Contract calls (`ethereum.call`)
- Native testing framework with mock store, EventBuilder, CallBuilder
- JSON parsing utilities
- Native crypto (keccak256)
- DX helpers: `log_id!`, `call_id!`, `block_id!`, `day_id!`, `hour_id!` macros
- Entity builders, `load_or_create`, `update`, `upsert`, `exists`
- Token formatting: `format_units`, `parse_units` (wei ↔ ETH conversion)
- Safe arithmetic: `BigInt::safe_div`, `BigDecimal::safe_div`
- Address utilities: `Address::is_zero()`
- Automatic `Address` → `Bytes` coercion

**Coming soon:**
- Documentation site

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
    // log_id! macro generates unique ID from tx hash + log index
    let id = log_id!(event);

    let mut transfer = Transfer::new(id);
    // Address → Bytes coercion is automatic
    transfer.set_from(event.params.from);
    transfer.set_to(event.params.to);
    transfer.set_value(event.params.value);
    transfer.save();
}
```

## Developer Experience

yogurt is designed to be *better* than AssemblyScript, not just "Rust instead of AS".

### ID Generation Macros

Common ID patterns are a single macro call:

```rust
// Event ID from tx hash + log index
let id = log_id!(event);  // "0xabc...123-42"

// Call ID from tx hash
let id = call_id!(call);  // "0xabc...123"

// Block ID from block number
let id = block_id!(block);  // "15000000"

// Time-based IDs for analytics entities
let daily_id = day_id!(event);   // "19724" (days since epoch)
let hourly_id = hour_id!(event); // "473376" (hours since epoch)
```

### Automatic Type Coercion

`Address` converts to `Bytes` automatically — no explicit conversion needed:

```rust
// Just works — Address → Bytes coercion is automatic
transfer.set_from(event.params.from);
transfer.set_to(event.params.to);
```

### Entity Builder Pattern

Fluent builders for clean entity construction:

```rust
// Create entity with builder — chainable setters
let swap = Swap::builder(log_id!(event))
    .pair(&pair_id)
    .sender(event.params.sender)
    .amount0_in(amount0)
    .amount1_out(amount1)
    .save();  // .save() builds and persists in one go
```

### Entity Helper Methods

Reduce boilerplate with `load_or_create`, `update`, `upsert`, and `exists`:

```rust
// Check existence without loading
if !Token::exists(&token_id) {
    // Create new token...
}

// Load or create with initializer
let token = Token::load_or_create(&address, |t| {
    t.set_decimals(BigInt::from_i32(18));
    t.set_total_supply(BigInt::zero());
});

// Update existing entity (no-op if not found)
Token::update(&address, |t| {
    t.set_total_supply(t.total_supply() + amount);
});

// Upsert — load or create, then update and save
Token::upsert(&address, |t| {
    t.set_last_updated(event.block.timestamp.clone());
});
```

### Token Amount Formatting

Convert between wei and human-readable amounts:

```rust
// Format wei as ETH (with 18 decimals)
let wei = BigInt::from_string("1500000000000000000").unwrap();
let eth = format_units(&wei, 18);  // "1.5"

// Parse human-readable back to wei
let wei = parse_units("1.5", 18);  // 1500000000000000000
```

### Safe Arithmetic

Division that returns zero instead of panicking:

```rust
// safe_div returns zero on divide-by-zero
let result = amount.safe_div(&divisor);  // No panic if divisor is zero

// Check for zero address
if event.params.to.is_zero() {
    // Burn transaction
}
```

## Development Workflow

### Watch Mode

Auto-rebuild on file changes:

```bash
yogurt dev
```

This watches `src/**/*.rs`, `schema.graphql`, `abis/**/*.json`, and `subgraph.yaml`.

### Inspect WASM

Analyze your compiled WASM module:

```bash
yogurt inspect build/subgraph.wasm
```

Shows memory usage, host imports, exports, and compatibility status.

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

### Subgraph Studio

```bash
# Store your deploy key (from Studio dashboard)
yogurt auth <your-deploy-key>

# Deploy to Studio
yogurt deploy my-subgraph --studio --version 0.0.1
```

The subgraph must be created in the [Studio web UI](https://thegraph.com/studio/) first.

## Testing

Write tests for your handlers using the native testing framework:

```rust
use yogurt_runtime::prelude::*;
use yogurt_runtime::testing::*;

#[test]
fn test_handle_transfer() {
    clear_store();

    // Build a test event
    let event: TransferEvent = EventBuilder::new()
        .block_number(12345678)
        .transaction_hash([0xAB; 32])
        .params(TransferParams {
            from: Address::from([0x11; 20]),
            to: Address::from([0x22; 20]),
            value: BigInt::from_u64(1_000_000),
        })
        .build();

    // Call your handler
    handle_transfer(event);

    // Assert entity was created
    assert_entity_exists::<Transfer>("expected-id");

    let transfer = Transfer::load("expected-id").unwrap();
    assert_eq!(transfer.value().to_string(), "1000000");
}
```

Run tests with the native target:

```bash
cargo test --target aarch64-apple-darwin  # macOS ARM
cargo test --target x86_64-unknown-linux-gnu  # Linux
```

### Testing Features

- **EventBuilder** — construct events with custom block/tx data
- **CallBuilder** — construct call handler inputs
- **BlockBuilder** — construct blocks for block handlers
- **Mock store** — thread-local entity storage
- **Mock data source** — `mock_data_source_address()`, `mock_data_source_network()`
- **Mock IPFS** — `mock_ipfs_cat()` for file data source testing
- **Native crypto** — `keccak256` works in tests
- **JSON parsing** — `json::from_string()` works in tests

## Example Subgraphs

The `tests/integration/` directory contains working examples:

- **erc20-transfer** — Simple ERC-20 Transfer event indexer
- **uniswap-v2** — Full Uniswap V2 clone with Factory, Pair, Token entities, data source templates, and contract calls

Test the Uniswap V2 example:

```bash
# Set your RPC (optional, defaults to public endpoint)
export ETHEREUM_RPC="https://mainnet.infura.io/v3/YOUR_KEY"

# Run the full test (starts infra, builds, deploys)
./scripts/test-uniswap-v2.sh

# Monitor indexing progress
./scripts/test-uniswap-v2.sh --status
```

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
    └── integration/      # End-to-end example subgraphs
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
