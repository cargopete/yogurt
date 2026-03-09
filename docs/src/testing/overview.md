# Testing Overview

yogurt includes a native testing framework that lets you test handlers without WASM compilation. Tests run instantly on your native target.

## Why Native Testing?

- **Fast iteration** — No WASM compilation, tests run in milliseconds
- **Standard Rust testing** — Use `cargo test`, IDE integration works
- **Full debugging** — Set breakpoints, inspect variables
- **Mocking** — Mock store, data source, IPFS

## Setup

Tests go in `tests/` directory or inline with `#[cfg(test)]`:

```
my-subgraph/
├── src/
│   └── lib.rs
└── tests/
    └── handler_test.rs
```

### Test File Structure

```rust
// tests/handler_test.rs

use yogurt_runtime::prelude::*;
use yogurt_runtime::testing::*;
use my_subgraph::generated::*;
use my_subgraph::handle_transfer;

#[test]
fn test_handle_transfer() {
    clear_store();

    // Build test event
    let event: TransferEvent = EventBuilder::new()
        .block_number(12345678)
        .transaction_hash([0xAB; 32])
        .log_index(0)
        .params(TransferParams {
            from: Address::from([0x11; 20]),
            to: Address::from([0x22; 20]),
            value: BigInt::from_u64(1_000_000),
        })
        .build();

    // Call handler
    handle_transfer(event);

    // Assert results
    assert_entity_exists::<Transfer>("expected-id");
}
```

## Running Tests

```bash
# Run all tests
cargo test --target aarch64-apple-darwin  # macOS ARM
cargo test --target x86_64-unknown-linux-gnu  # Linux

# Run specific test
cargo test test_handle_transfer

# Run with output
cargo test -- --nocapture
```

## Testing Utilities

### Store Operations

```rust
use yogurt_runtime::testing::*;

// Clear all entities
clear_store();

// Check entity count
let count = entity_count::<Transfer>();

// Direct store access
store_set("Transfer", "id-1", entity_data);
let data = store_get("Transfer", "id-1");
store_remove("Transfer", "id-1");
```

### Assertions

```rust
// Entity exists
assert_entity_exists::<Transfer>("some-id");

// Entity doesn't exist
assert_entity_not_exists::<Transfer>("some-id");

// Load and check values
let transfer = Transfer::load("some-id").unwrap();
assert_eq!(transfer.value().to_string(), "1000000");
```

### Event Construction

```rust
let event: TransferEvent = EventBuilder::new()
    .block_number(12345678)
    .block_timestamp(1234567890)
    .transaction_hash([0xAB; 32])
    .log_index(0)
    .params(TransferParams {
        from: Address::from([0x11; 20]),
        to: Address::from([0x22; 20]),
        value: BigInt::from_u64(1_000_000),
    })
    .build();
```

### Mocking

```rust
// Mock data source
mock_data_source_address(Address::from([0x11; 20]));
mock_data_source_network("mainnet");

// Mock IPFS
mock_ipfs_cat("QmHash123", b"file content");
```

## Test Patterns

### Testing Entity Creation

```rust
#[test]
fn test_creates_transfer_entity() {
    clear_store();

    let event = build_transfer_event(
        Address::from([0x11; 20]),
        Address::from([0x22; 20]),
        BigInt::from_u64(1000),
    );

    handle_transfer(event);

    assert_entity_exists::<Transfer>("expected-id");

    let transfer = Transfer::load("expected-id").unwrap();
    assert_eq!(transfer.from().to_hex(), "0x1111111111111111111111111111111111111111");
    assert_eq!(transfer.value().to_string(), "1000");
}
```

### Testing Updates

```rust
#[test]
fn test_updates_token_supply() {
    clear_store();

    // Create initial token
    let mut token = Token::new("token-id".to_string());
    token.set_total_supply(BigInt::from_u64(1000));
    token.save();

    // Trigger mint event
    let event = build_mint_event(BigInt::from_u64(500));
    handle_mint(event);

    // Verify update
    let token = Token::load("token-id").unwrap();
    assert_eq!(token.total_supply().to_string(), "1500");
}
```

### Testing with Templates

```rust
#[test]
fn test_pair_created_spawns_template() {
    clear_store();

    let event = build_pair_created_event(
        Address::from([0xAA; 20]),  // pair address
        Address::from([0x11; 20]),  // token0
        Address::from([0x22; 20]),  // token1
    );

    handle_pair_created(event);

    // Verify pair entity was created
    let pair_id = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    assert_entity_exists::<Pair>(pair_id);
}
```

## Next Steps

- [EventBuilder](./event-builder.md) — Detailed event construction
- [Mock Store](./mock-store.md) — Store mocking details
- [Assertions](./assertions.md) — Available assertions
