# Quick Start

Let's create a simple ERC-20 Transfer event indexer in 5 minutes.

## 1. Create a New Project

```bash
yogurt init my-subgraph
cd my-subgraph
```

This creates a project with:
- `subgraph.yaml` — Manifest defining data sources
- `schema.graphql` — GraphQL schema for entities
- `src/` — Rust handler code
- `abis/` — Contract ABIs
- `Cargo.toml` — Rust dependencies

## 2. Define Your Schema

Edit `schema.graphql`:

```graphql
type Transfer @entity {
  id: ID!
  from: Bytes!
  to: Bytes!
  value: BigInt!
  blockNumber: BigInt!
  timestamp: BigInt!
}
```

## 3. Add Your ABI

Place your contract ABI in `abis/ERC20.json`. For ERC-20, you need at least the Transfer event:

```json
[
  {
    "anonymous": false,
    "inputs": [
      {"indexed": true, "name": "from", "type": "address"},
      {"indexed": true, "name": "to", "type": "address"},
      {"indexed": false, "name": "value", "type": "uint256"}
    ],
    "name": "Transfer",
    "type": "event"
  }
]
```

## 4. Configure the Manifest

Edit `subgraph.yaml`:

```yaml
specVersion: 0.0.5
schema:
  file: ./schema.graphql
dataSources:
  - kind: ethereum
    name: ERC20
    network: mainnet
    source:
      address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"  # USDC
      abi: ERC20
      startBlock: 6082465
    mapping:
      kind: rust/wasm
      apiVersion: 0.0.7
      language: rust
      entities:
        - Transfer
      abis:
        - name: ERC20
          file: ./abis/ERC20.json
      eventHandlers:
        - event: Transfer(indexed address,indexed address,uint256)
          handler: handleTransfer
      file: ./build/subgraph.wasm
```

## 5. Generate Types

```bash
yogurt codegen
```

This generates `src/generated/` with:
- Entity structs with getters/setters
- Event structs with typed parameters
- Contract bindings

## 6. Write Your Handler

Edit `src/lib.rs`:

```rust
mod generated;

use yogurt_runtime::prelude::*;
use generated::{Transfer, TransferEvent};

#[handler]
fn handle_transfer(event: TransferEvent) {
    let id = log_id!(event);

    Transfer::builder(id)
        .from(event.params.from)
        .to(event.params.to)
        .value(event.params.value)
        .block_number(event.block.number.clone())
        .timestamp(event.block.timestamp.clone())
        .save();
}
```

## 7. Build

```bash
yogurt build --release
```

This compiles to `build/subgraph.wasm`.

## 8. Validate

```bash
yogurt validate
```

Checks that your WASM exports all required functions.

## 9. Deploy

To a local graph-node:

```bash
yogurt deploy myaccount/my-subgraph
```

To Subgraph Studio:

```bash
yogurt auth <your-deploy-key>
yogurt deploy my-subgraph --studio --version 0.0.1
```

## Next Steps

- [Event Handlers](../handlers/events.md) — Learn more about handling events
- [Entity Builders](../entities/builders.md) — Fluent entity construction
- [Testing](../testing/overview.md) — Write tests for your handlers
