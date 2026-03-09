# Mock Store

The mock store provides thread-local entity storage for testing. It mimics graph-node's store behavior without requiring a database.

## Clearing the Store

Always clear the store at the start of each test:

```rust
use yogurt_runtime::testing::clear_store;

#[test]
fn test_something() {
    clear_store();  // Start fresh

    // Your test...
}
```

This ensures tests don't interfere with each other.

## Store Operations

### Direct Access

```rust
use yogurt_runtime::testing::{store_get, store_set, store_remove};

// Set an entity
let mut data = EntityData::new();
data.set("from", Value::from(Bytes::from([0x11; 20])));
data.set("value", Value::from(BigInt::from_u64(1000)));
store_set("Transfer", "transfer-1", data);

// Get an entity
if let Some(data) = store_get("Transfer", "transfer-1") {
    let value = data.get("value").unwrap();
}

// Remove an entity
store_remove("Transfer", "transfer-1");
```

### Via Entity Methods

Entities work normally with the mock store:

```rust
// Create and save
let mut transfer = Transfer::new("id-1".to_string());
transfer.set_from(Bytes::from([0x11; 20]));
transfer.set_value(BigInt::from_u64(1000));
transfer.save();

// Load
let loaded = Transfer::load("id-1").unwrap();

// Update
if let Some(mut t) = Transfer::load("id-1") {
    t.set_value(BigInt::from_u64(2000));
    t.save();
}

// Delete
Transfer::remove("id-1");

// Check existence
if Transfer::exists("id-1") {
    // ...
}

// Load or create
let token = Token::load_or_create("token-1", |t| {
    t.set_symbol("TEST".to_string());
    t.set_decimals(BigInt::from_i32(18));
});
```

## Pre-populating Data

Set up initial state before testing:

```rust
#[test]
fn test_updates_existing_entity() {
    clear_store();

    // Pre-populate with existing entity
    let mut token = Token::new("token-1".to_string());
    token.set_symbol("TEST".to_string());
    token.set_total_supply(BigInt::from_u64(1_000_000));
    token.save();

    // Now test updating it
    let event = build_mint_event(BigInt::from_u64(500_000));
    handle_mint(event);

    let token = Token::load("token-1").unwrap();
    assert_eq!(token.total_supply().to_string(), "1500000");
}
```

## Entity Count

Check how many entities of a type exist:

```rust
use yogurt_runtime::testing::entity_count;

#[test]
fn test_creates_multiple_entities() {
    clear_store();

    assert_eq!(entity_count::<Transfer>(), 0);

    handle_transfer(event1);
    handle_transfer(event2);
    handle_transfer(event3);

    assert_eq!(entity_count::<Transfer>(), 3);
}
```

## Mock Data Source

Mock the data source context:

```rust
use yogurt_runtime::testing::*;

#[test]
fn test_with_data_source() {
    clear_store();

    // Mock data source address (for templates)
    mock_data_source_address(Address::from([0xAA; 20]));

    // Mock network
    mock_data_source_network("mainnet");

    // Mock context
    let mut context = data_source::DataSourceContext::new();
    context.set("factory", Value::from("0x1234..."));
    mock_data_source_context(context);

    // Now data_source::address() returns the mocked address
    let addr = data_source::address();
    assert_eq!(addr.to_hex(), "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
}
```

## Mock IPFS

Mock IPFS content for file data source testing:

```rust
use yogurt_runtime::testing::{mock_ipfs_cat, clear_ipfs_mocks};

#[test]
fn test_ipfs_handler() {
    clear_store();
    clear_ipfs_mocks();

    // Mock file content
    let metadata = r#"{"name": "Cool NFT", "image": "ipfs://QmXyz"}"#;
    mock_ipfs_cat("QmAbc123", metadata.as_bytes());

    // Your handler can now read from IPFS
    let content = ipfs::cat("QmAbc123").unwrap();
    assert_eq!(content.len(), metadata.len());
}

#[test]
fn test_multiple_ipfs_files() {
    clear_ipfs_mocks();

    mock_ipfs_cat("QmFile1", b"content 1");
    mock_ipfs_cat("QmFile2", b"content 2");
    mock_ipfs_cat("QmFile3", b"content 3");

    // All files are accessible
    assert!(ipfs::cat("QmFile1").is_some());
    assert!(ipfs::cat("QmFile2").is_some());
    assert!(ipfs::cat("QmFile3").is_some());
    assert!(ipfs::cat("QmNotMocked").is_none());
}
```

## Thread Safety

The mock store is thread-local, so tests can run in parallel:

```rust
// These tests can run simultaneously without conflicts
#[test]
fn test_a() {
    clear_store();
    // Test A's entities...
}

#[test]
fn test_b() {
    clear_store();
    // Test B's entities (isolated from A)
}
```

## Common Patterns

### Test Fixtures

```rust
fn setup_token(id: &str, symbol: &str, supply: u64) -> Token {
    let mut token = Token::new(id.to_string());
    token.set_symbol(symbol.to_string());
    token.set_decimals(BigInt::from_i32(18));
    token.set_total_supply(BigInt::from_u64(supply));
    token.save();
    token
}

fn setup_pair(token0: &Token, token1: &Token) -> Pair {
    let id = format!("{}-{}", token0.id(), token1.id());
    let mut pair = Pair::new(id);
    pair.set_token0(token0.id());
    pair.set_token1(token1.id());
    pair.set_reserve0(BigDecimal::zero());
    pair.set_reserve1(BigDecimal::zero());
    pair.save();
    pair
}

#[test]
fn test_swap() {
    clear_store();

    let token0 = setup_token("token-a", "TKNA", 1_000_000);
    let token1 = setup_token("token-b", "TKNB", 2_000_000);
    let pair = setup_pair(&token0, &token1);

    // Test swap handler...
}
```
