# Assertions

yogurt provides assertion helpers for common testing patterns.

## Entity Assertions

### assert_entity_exists

Verify an entity exists in the store:

```rust
use yogurt_runtime::testing::assert_entity_exists;

#[test]
fn test_creates_entity() {
    clear_store();

    handle_transfer(event);

    // Passes if entity exists
    assert_entity_exists::<Transfer>("expected-id");
}
```

Panics with a helpful message if the entity doesn't exist:

```
Entity Transfer with id 'expected-id' does not exist
```

### assert_entity_not_exists

Verify an entity does NOT exist:

```rust
use yogurt_runtime::testing::assert_entity_not_exists;

#[test]
fn test_removes_entity() {
    clear_store();

    // Create entity
    let mut transfer = Transfer::new("id-1".to_string());
    transfer.save();

    // Remove it
    Transfer::remove("id-1");

    // Verify it's gone
    assert_entity_not_exists::<Transfer>("id-1");
}
```

### entity_count

Get the number of entities of a type:

```rust
use yogurt_runtime::testing::entity_count;

#[test]
fn test_batch_creation() {
    clear_store();

    assert_eq!(entity_count::<Transfer>(), 0);

    handle_batch(events);

    assert_eq!(entity_count::<Transfer>(), 10);
}
```

## Value Assertions

Use standard Rust assertions on entity values:

```rust
#[test]
fn test_transfer_values() {
    clear_store();

    handle_transfer(event);

    let transfer = Transfer::load("id-1").unwrap();

    // String comparison
    assert_eq!(
        transfer.from().to_hex(),
        "0x1111111111111111111111111111111111111111"
    );

    // BigInt comparison
    assert_eq!(transfer.value().to_string(), "1000000");

    // BigInt numeric comparison
    assert!(transfer.value().gt(&BigInt::zero()));
    assert!(transfer.value().lt(&BigInt::from_u64(2_000_000)));

    // Boolean
    assert!(transfer.value().gt(&BigInt::zero()));
}
```

## Pattern: Assert Entity Fields

Create a helper for common assertions:

```rust
fn assert_transfer(
    id: &str,
    expected_from: &str,
    expected_to: &str,
    expected_value: &str,
) {
    let transfer = Transfer::load(id)
        .unwrap_or_else(|| panic!("Transfer {} not found", id));

    assert_eq!(
        transfer.from().to_hex(),
        expected_from,
        "Transfer {} has wrong 'from'",
        id
    );
    assert_eq!(
        transfer.to().to_hex(),
        expected_to,
        "Transfer {} has wrong 'to'",
        id
    );
    assert_eq!(
        transfer.value().to_string(),
        expected_value,
        "Transfer {} has wrong 'value'",
        id
    );
}

#[test]
fn test_transfer() {
    clear_store();
    handle_transfer(event);

    assert_transfer(
        "0xabc...-0",
        "0x1111111111111111111111111111111111111111",
        "0x2222222222222222222222222222222222222222",
        "1000000",
    );
}
```

## Pattern: Assert State Changes

Test that values change correctly:

```rust
#[test]
fn test_balance_update() {
    clear_store();

    // Initial state
    let mut account = Account::new("user-1".to_string());
    account.set_balance(BigInt::from_u64(1000));
    account.save();

    let initial_balance = Account::load("user-1").unwrap().balance().clone();

    // Action
    handle_deposit(deposit_event);

    // Assert change
    let final_balance = Account::load("user-1").unwrap().balance().clone();
    let expected = initial_balance.plus(&BigInt::from_u64(500));

    assert_eq!(final_balance.to_string(), expected.to_string());
}
```

## Pattern: Assert Multiple Entities

```rust
#[test]
fn test_creates_related_entities() {
    clear_store();

    handle_pair_created(event);

    // Verify all related entities
    assert_entity_exists::<Pair>("pair-id");
    assert_entity_exists::<Token>("token0-id");
    assert_entity_exists::<Token>("token1-id");

    // Verify relationships
    let pair = Pair::load("pair-id").unwrap();
    assert_eq!(pair.token0(), "token0-id");
    assert_eq!(pair.token1(), "token1-id");
}
```

## Pattern: Assert No Side Effects

```rust
#[test]
fn test_no_duplicate_creation() {
    clear_store();

    // Create initial token
    let mut token = Token::new("token-1".to_string());
    token.set_symbol("ORIGINAL".to_string());
    token.save();

    // Handler should not overwrite existing token
    handle_transfer(event_involving_token_1);

    // Symbol should be unchanged
    let token = Token::load("token-1").unwrap();
    assert_eq!(token.symbol(), "ORIGINAL");
}
```

## Debugging Failed Assertions

When assertions fail, load and inspect the entity:

```rust
#[test]
fn test_with_debug() {
    clear_store();
    handle_transfer(event);

    // Debug: print entity state
    if let Some(transfer) = Transfer::load("some-id") {
        println!("Transfer state:");
        println!("  from: {}", transfer.from().to_hex());
        println!("  to: {}", transfer.to().to_hex());
        println!("  value: {}", transfer.value().to_string());
    } else {
        println!("Transfer not found!");

        // Check what entities DO exist
        println!("Entity count: {}", entity_count::<Transfer>());
    }

    // Now the actual assertion
    assert_entity_exists::<Transfer>("some-id");
}
```

Run with output visible:

```bash
cargo test test_with_debug -- --nocapture
```
