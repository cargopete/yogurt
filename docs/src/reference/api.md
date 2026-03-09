# API Reference

Quick reference for yogurt-runtime types and functions.

## Types

### Address

```rust
// Construction
Address::from([u8; 20])
Address::from_string("0x...")
Address::zero()

// Methods
address.to_hex() -> String
address.is_zero() -> bool
address.0 -> [u8; 20]  // Access raw bytes
```

### Bytes

```rust
// Construction
Bytes::from(Vec<u8>)
Bytes::from(&[u8])
Bytes::from([u8; 20])
Bytes::from([u8; 32])
Bytes::from_hex_string("0x...") -> Result<Bytes, _>
Bytes::empty()

// Methods
bytes.to_hex() -> String
bytes.len() -> usize
bytes.is_empty() -> bool
bytes.as_slice() -> &[u8]
bytes.concat(&other) -> Bytes
bytes.concat_i32(n) -> Bytes
```

### BigInt

```rust
// Construction
BigInt::from_i32(i32)
BigInt::from_u64(u64)
BigInt::from_string("...") -> Option<BigInt>
BigInt::from_bytes(&[u8])
BigInt::zero()

// Arithmetic
a.plus(&b) -> BigInt       // or &a + &b
a.minus(&b) -> BigInt      // or &a - &b
a.times(&b) -> BigInt      // or &a * &b
a.divided_by(&b) -> BigInt // or &a / &b
a.mod_op(&b) -> BigInt
a.pow(exp) -> BigInt
a.abs() -> BigInt
a.sqrt() -> BigInt
a.safe_div(&b) -> BigInt   // Returns zero if b is zero

// Comparison
a.lt(&b) -> bool   // a < b
a.le(&b) -> bool   // a <= b
a.gt(&b) -> bool   // a > b
a.ge(&b) -> bool   // a >= b
a.is_zero() -> bool

// Conversion
big_int.to_string() -> String
big_int.to_i32() -> i32
big_int.to_u64() -> u64
big_int.to_bytes() -> Vec<u8>
big_int.to_decimals(decimals) -> String
```

### BigDecimal

```rust
// Construction
BigDecimal::from_big_int(&BigInt)
BigDecimal::from_string("...") -> Option<BigDecimal>
BigDecimal::zero()

// Arithmetic
a.plus(&b) -> BigDecimal
a.minus(&b) -> BigDecimal
a.times(&b) -> BigDecimal
a.divided_by(&b) -> BigDecimal
a.safe_div(&b) -> BigDecimal

// Comparison
a.lt(&b) -> bool
a.le(&b) -> bool
a.gt(&b) -> bool
a.ge(&b) -> bool
a.is_zero() -> bool

// Methods
decimal.truncate(n) -> BigDecimal
decimal.to_string() -> String
```

## Macros

```rust
log_id!(event)   -> String  // "{tx_hash}-{log_index}"
call_id!(call)   -> String  // "{tx_hash}"
block_id!(block) -> String  // "{block_number}"
day_id!(event)   -> String  // "{days_since_epoch}"
hour_id!(event)  -> String  // "{hours_since_epoch}"
```

## Functions

### Token Formatting

```rust
format_units(&BigInt, decimals: u8) -> String
parse_units(value: &str, decimals: u8) -> BigInt
```

### Store Operations

```rust
store::set(entity_type: &str, id: &str, data: EntityData)
store::get(entity_type: &str, id: &str) -> Option<EntityData>
store::remove(entity_type: &str, id: &str)
store::get_in_block(entity_type: &str, id: &str) -> Option<EntityData>
```

### Data Source

```rust
data_source::address() -> Address
data_source::network() -> String
data_source::context() -> DataSourceContext
data_source::string_param() -> String  // For file data sources
```

### Logging

```rust
log::info!("message {}", arg)
log::warning!("message {}", arg)
log::error!("message {}", arg)
log::debug!("message {}", arg)
```

### Crypto

```rust
crypto::keccak256(&[u8]) -> [u8; 32]
```

### JSON

```rust
json::from_bytes(&Bytes) -> Result<JsonValue, _>
json::from_string(&str) -> Result<JsonValue, _>

// JsonValue methods
json_value.get("key") -> Option<&JsonValue>
json_value.to_string() -> Option<String>
json_value.to_i64() -> Option<i64>
json_value.to_bool() -> Option<bool>
json_value.to_big_int() -> Option<BigInt>
```

### IPFS

```rust
ipfs::cat(hash: &str) -> Option<Bytes>
ipfs::map(hash: &str, callback: &str, flags: &[&str])
```

### ENS

```rust
ens::name_by_hash(hash: &str) -> Option<String>
```

## Entity Trait

Generated entities implement:

```rust
// Construction
Entity::new(id: String) -> Self
Entity::builder(id: impl Into<String>) -> EntityBuilder

// CRUD
Entity::load(id: &str) -> Option<Self>
Entity::exists(id: &str) -> bool
Entity::load_or_create(id: &str, init: impl FnOnce(&mut Self)) -> Self
Entity::update(id: &str, f: impl FnOnce(&mut Self))
Entity::upsert(id: &str, f: impl FnOnce(&mut Self))
entity.save()
Entity::remove(id: &str)

// Field access
entity.id() -> &String
entity.field() -> &T           // For required fields
entity.field() -> Option<&T>   // For optional fields
entity.set_field(value: T)
entity.unset_field()           // For optional fields
```

## Testing

```rust
// Store
clear_store()
store_get(entity_type: &str, id: &str) -> Option<EntityData>
store_set(entity_type: &str, id: &str, data: EntityData)
store_remove(entity_type: &str, id: &str)
entity_count::<E>() -> usize

// Assertions
assert_entity_exists::<E>(id: &str)
assert_entity_not_exists::<E>(id: &str)

// Mocking
mock_data_source_address(Address)
mock_data_source_network(&str)
mock_data_source_context(DataSourceContext)
mock_ipfs_cat(hash: &str, content: &[u8])
clear_ipfs_mocks()

// Builders
EventBuilder::new()
    .block_number(u64)
    .block_timestamp(u64)
    .block_hash([u8; 32])
    .transaction_hash([u8; 32])
    .transaction_from(Address)
    .log_index(u64)
    .params(P)
    .build() -> Event<P>

CallBuilder::new()
    .block_number(u64)
    .transaction_hash([u8; 32])
    .from(Address)
    .to(Address)
    .inputs(I)
    .outputs(O)
    .build() -> Call<I, O>

BlockBuilder::new()
    .number(u64)
    .timestamp(u64)
    .hash([u8; 32])
    .parent_hash([u8; 32])
    .gas_used(u64)
    .gas_limit(u64)
    .author(Address)
    .build() -> Block
```

## Prelude

Import everything commonly needed:

```rust
use yogurt_runtime::prelude::*;

// Includes:
// - Address, Bytes, BigInt, BigDecimal, Value, Entity
// - Block, Transaction, Event, Call
// - data_source, log
// - log_id!, call_id!, block_id!, day_id!, hour_id!
// - format_units, parse_units
// - FromAscPtr (for custom types)
// - #[handler] macro (with feature)
```
