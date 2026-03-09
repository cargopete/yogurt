# Project Structure

A yogurt subgraph project has the following structure:

```
my-subgraph/
├── Cargo.toml           # Rust dependencies
├── subgraph.yaml        # Subgraph manifest
├── schema.graphql       # Entity definitions
├── abis/
│   └── MyContract.json  # Contract ABIs
├── src/
│   ├── lib.rs           # Handler entry points
│   ├── generated/       # Auto-generated types (don't edit)
│   │   ├── mod.rs
│   │   ├── schema.rs    # Entity structs
│   │   └── abi.rs       # Event/contract types
│   └── mappings/        # Your handler logic (optional organization)
│       └── mod.rs
├── tests/               # Native tests
│   └── handler_test.rs
└── build/               # Compiled output
    └── subgraph.wasm
```

## Key Files

### `Cargo.toml`

Your Rust project configuration:

```toml
[package]
name = "my-subgraph"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
yogurt-runtime = { version = "0.1" }

[features]
default = []
std = ["yogurt-runtime/std"]

[dev-dependencies]
# For native testing
```

### `subgraph.yaml`

The manifest defines your data sources, handlers, and schema:

```yaml
specVersion: 0.0.9
schema:
  file: ./schema.graphql
dataSources:
  - kind: ethereum
    name: MyContract
    network: mainnet
    source:
      address: "0x..."
      abi: MyContract
      startBlock: 12345678
    mapping:
      kind: ethereum/events
      apiVersion: 0.0.7
      language: wasm/assemblyscript
      entities:
        - MyEntity
      abis:
        - name: MyContract
          file: ./abis/MyContract.json
      eventHandlers:
        - event: MyEvent(indexed address,uint256)
          handler: handleMyEvent
      file: ./build/subgraph.wasm
```

> **Note:** yogurt uses `language: wasm/assemblyscript` because it produces binary-compatible WASM. Graph-node treats it identically to AssemblyScript subgraphs.

### `schema.graphql`

GraphQL schema defining your entities:

```graphql
type MyEntity @entity {
  id: ID!
  address: Bytes!
  value: BigInt!
  timestamp: BigInt!
}
```

### `src/generated/`

Auto-generated code from `yogurt codegen`. **Don't edit these files** — they're regenerated each time you run codegen.

### `src/lib.rs`

Your handler entry points:

```rust
mod generated;

use yogurt_runtime::prelude::*;
use generated::*;

#[handler]
fn handle_my_event(event: MyEventEvent) {
    // Your logic here
}
```

## Generated Code

Running `yogurt codegen` generates:

1. **Entity structs** with getters, setters, and builders
2. **Event structs** with typed parameters
3. **Contract bindings** for view/pure functions
4. **Data source templates** for dynamic data sources

The generated code provides full type safety — if your schema or ABI changes, you'll get compile errors in your handlers.
