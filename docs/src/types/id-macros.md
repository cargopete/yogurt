# ID Generation Macros

yogurt provides macros for common entity ID patterns, reducing boilerplate and ensuring uniqueness.

## log_id!

Generates a unique ID from an event's transaction hash and log index.

```rust
let id = log_id!(event);  // "0xabc123...-42"
```

This is the most common pattern for event entity IDs. The combination of transaction hash and log index is guaranteed to be unique.

### Usage

```rust
use yogurt_runtime::prelude::*;

#[handler]
fn handle_transfer(event: TransferEvent) {
    let id = log_id!(event);

    Transfer::builder(id)
        .from(event.params.from)
        .to(event.params.to)
        .value(event.params.value)
        .save();
}
```

### Format

```
{transaction_hash_hex}-{log_index}
```

Example: `0x1234abcd...5678-42`

## call_id!

Generates an ID from a call's transaction hash.

```rust
let id = call_id!(call);  // "0xabc123..."
```

### Usage

```rust
#[handler]
fn handle_transfer_call(call: TransferCall) {
    let id = call_id!(call);

    TransferRecord::builder(id)
        .from(call.from.clone())
        .to(call.inputs.to.clone())
        .save();
}
```

### Note on Uniqueness

If multiple calls of the same type occur in one transaction, you'll need to add an index:

```rust
let id = format!("{}-{}", call_id!(call), call_index);
```

## block_id!

Generates an ID from a block's number.

```rust
let id = block_id!(block);  // "15000000"
```

### Usage

```rust
#[handler]
fn handle_block(block: Block) {
    let id = block_id!(block);

    BlockStats::builder(id)
        .number(block.number.clone())
        .timestamp(block.timestamp.clone())
        .gas_used(block.gas_used.clone())
        .save();
}
```

## day_id!

Generates a day-based ID from an event's block timestamp. Returns the number of days since Unix epoch.

```rust
let id = day_id!(event);  // "19724"
```

### Usage

Perfect for daily aggregation entities:

```rust
#[handler]
fn handle_swap(event: SwapEvent) {
    let daily_id = day_id!(event);

    DailyVolume::upsert(&daily_id, |daily| {
        daily.set_volume(daily.volume() + swap_amount);
        daily.set_swap_count(daily.swap_count() + BigInt::from(1));
    });
}
```

### Calculation

```
timestamp / 86400  (seconds per day)
```

## hour_id!

Generates an hour-based ID from an event's block timestamp. Returns the number of hours since Unix epoch.

```rust
let id = hour_id!(event);  // "473376"
```

### Usage

For hourly aggregations:

```rust
#[handler]
fn handle_swap(event: SwapEvent) {
    let hourly_id = hour_id!(event);

    HourlyVolume::upsert(&hourly_id, |hourly| {
        hourly.set_volume(hourly.volume() + swap_amount);
        hourly.set_swap_count(hourly.swap_count() + BigInt::from(1));
    });
}
```

### Calculation

```
timestamp / 3600  (seconds per hour)
```

## Combining IDs

For entity-specific time aggregations, combine IDs:

```rust
// Daily stats per token
let daily_token_id = format!("{}-{}", day_id!(event), token_id);

// Hourly stats per pair
let hourly_pair_id = format!("{}-{}", hour_id!(event), pair_id);

// User's daily activity
let user_daily_id = format!("{}-{}", user_address.to_hex(), day_id!(event));
```

## Comparison with AssemblyScript

| Pattern | AssemblyScript | yogurt |
|---------|---------------|--------|
| Event ID | `event.transaction.hash.toHex() + "-" + event.logIndex.toString()` | `log_id!(event)` |
| Call ID | `call.transaction.hash.toHex()` | `call_id!(call)` |
| Block ID | `block.number.toString()` | `block_id!(block)` |
| Day ID | `(event.block.timestamp.toI32() / 86400).toString()` | `day_id!(event)` |
| Hour ID | `(event.block.timestamp.toI32() / 3600).toString()` | `hour_id!(event)` |
