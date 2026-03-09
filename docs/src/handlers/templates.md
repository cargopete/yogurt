# Data Source Templates

Data source templates let you create new data sources dynamically at runtime. This is essential for factory patterns like Uniswap, where new contracts are deployed frequently.

## When to Use Templates

- **Factory patterns** — A factory deploys new contracts (pairs, vaults, pools)
- **Proxy patterns** — New proxy instances are created dynamically
- **Registry patterns** — Contracts are registered at runtime

## Defining Templates

In `subgraph.yaml`:

```yaml
dataSources:
  - kind: ethereum
    name: Factory
    # ... factory config

templates:
  - kind: ethereum
    name: Pair
    network: mainnet
    source:
      abi: Pair
    mapping:
      kind: rust/wasm
      apiVersion: 0.0.7
      language: rust
      entities:
        - Pair
        - Swap
      abis:
        - name: Pair
          file: ./abis/Pair.json
      eventHandlers:
        - event: Swap(indexed address,uint256,uint256,uint256,uint256,indexed address)
          handler: handleSwap
      file: ./build/subgraph.wasm
```

## Creating Instances

Use the generated template module:

```rust
use yogurt_runtime::prelude::*;
use crate::generated::{PairCreatedEvent, Pair};
use crate::generated::templates::Pair as PairTemplate;

#[handler]
fn handle_pair_created(event: PairCreatedEvent) {
    // Create the data source for the new pair
    PairTemplate::create(event.params.pair.clone());

    // Create an entity to track the pair
    Pair::builder(event.params.pair.to_hex())
        .token0(event.params.token0.into())
        .token1(event.params.token1.into())
        .created_at_block(event.block.number.clone())
        .save();
}
```

## Template Context

Pass context data to templates:

```rust
use crate::generated::templates::PairWithContext;

#[handler]
fn handle_pair_created(event: PairCreatedEvent) {
    // Create with context
    let mut context = data_source::DataSourceContext::new();
    context.set("factory", Value::from(data_source::address().to_hex()));

    PairWithContext::create_with_context(
        event.params.pair.clone(),
        context,
    );
}
```

Access context in template handlers:

```rust
#[handler]
fn handle_swap(event: SwapEvent) {
    let factory_address = data_source::context()
        .get("factory")
        .and_then(|v| v.to_string());
}
```

## Example: Uniswap V2 Pattern

### Factory Handler

```rust
#[handler]
fn handle_pair_created(event: PairCreatedEvent) {
    let pair_address = event.params.pair.clone();
    let pair_id = pair_address.to_hex();

    // Create data source for new pair
    PairTemplate::create(pair_address);

    // Initialize pair entity
    let token0 = get_or_create_token(&event.params.token0);
    let token1 = get_or_create_token(&event.params.token1);

    Pair::builder(pair_id)
        .token0(&token0.id())
        .token1(&token1.id())
        .reserve0(BigDecimal::zero())
        .reserve1(BigDecimal::zero())
        .created_at_timestamp(event.block.timestamp.clone())
        .save();

    // Update factory stats
    Factory::update(FACTORY_ADDRESS, |f| {
        f.set_pair_count(f.pair_count() + BigInt::from(1));
    });
}
```

### Pair Handler (Template)

```rust
#[handler]
fn handle_swap(event: SwapEvent) {
    let pair_id = data_source::address().to_hex();

    Swap::builder(log_id!(event))
        .pair(&pair_id)
        .sender(event.params.sender)
        .amount0_in(event.params.amount0_in.clone())
        .amount1_in(event.params.amount1_in.clone())
        .amount0_out(event.params.amount0_out.clone())
        .amount1_out(event.params.amount1_out.clone())
        .save();
}
```

## Getting the Data Source Address

Inside a template handler, get the contract address:

```rust
let contract_address = data_source::address();
```

This returns the address of the specific contract instance that emitted the event.
