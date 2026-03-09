# CRUD Operations

yogurt provides several ways to create, read, update, and delete entities.

## Create

### Using `new` and `save`

```rust
let mut transfer = Transfer::new("unique-id".to_string());
transfer.set_from(event.params.from);
transfer.set_to(event.params.to);
transfer.set_value(event.params.value);
transfer.save();
```

### Using Builder Pattern

```rust
Transfer::builder("unique-id")
    .from(event.params.from)
    .to(event.params.to)
    .value(event.params.value)
    .save();
```

See [Entity Builders](./builders.md) for more details.

## Read

### Load by ID

```rust
// Returns Option<Entity>
if let Some(transfer) = Transfer::load("some-id") {
    let value = transfer.value();
    // Use the entity...
}
```

### Check Existence

```rust
if Transfer::exists("some-id") {
    // Entity exists
}
```

### Load or Create

Load an existing entity, or create a new one with default values:

```rust
let token = Token::load_or_create(&address, |t| {
    // This closure runs only if creating a new entity
    t.set_symbol("UNKNOWN".to_string());
    t.set_decimals(BigInt::from_i32(18));
    t.set_total_supply(BigInt::zero());
});

// Token is now loaded or newly created
token.save();
```

## Update

### Load, Modify, Save

```rust
if let Some(mut token) = Token::load(&id) {
    let new_supply = token.total_supply() + amount;
    token.set_total_supply(new_supply);
    token.save();
}
```

### Using `update`

Update an entity if it exists (no-op if not found):

```rust
Token::update(&id, |token| {
    token.set_total_supply(token.total_supply() + amount);
});
```

The entity is automatically saved after the closure.

### Using `upsert`

Load or create, then update and save in one call:

```rust
Token::upsert(&id, |token| {
    // Runs for both new and existing entities
    token.set_last_updated(block_timestamp.clone());
});
```

For new entities, fields are initialized to defaults (zero for numbers, empty for strings/bytes).

## Delete

### Remove by ID

```rust
store::remove("Transfer", "some-id");
```

Or using the entity type:

```rust
Transfer::remove("some-id");
```

## Patterns

### Initialize on First Access

```rust
fn get_or_create_token(address: &Address) -> Token {
    let id = address.to_hex();

    Token::load_or_create(&id, |t| {
        // Fetch token info from contract
        let contract = ERC20Contract::bind(address.clone());

        if let Some(symbol) = contract.try_symbol() {
            t.set_symbol(symbol);
        }
        if let Some(name) = contract.try_name() {
            t.set_name(name);
        }
        if let Some(decimals) = contract.try_decimals() {
            t.set_decimals(BigInt::from_i32(decimals as i32));
        }

        t.set_total_supply(BigInt::zero());
    })
}
```

### Increment Counter

```rust
Factory::update(FACTORY_ADDRESS, |f| {
    f.set_pair_count(f.pair_count() + BigInt::from(1));
});
```

### Aggregate Values

```rust
DailyVolume::upsert(&day_id, |daily| {
    daily.set_volume(daily.volume() + swap_volume);
    daily.set_swap_count(daily.swap_count() + BigInt::from(1));
});
```

### Safe Division

When updating with division, use `safe_div` to avoid panics:

```rust
Pair::update(&pair_id, |pair| {
    let price = reserve1.safe_div(&reserve0);  // Returns zero if reserve0 is zero
    pair.set_token0_price(price);
});
```

## Load in Block

Load an entity only if it was modified in the current block:

```rust
if let Some(transfer) = Transfer::load_in_block("some-id") {
    // Entity was created or modified in this block
}
```

This is useful for detecting and handling entities created earlier in the same block.
