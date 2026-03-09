# Core Types

yogurt provides Rust equivalents of all graph-ts types.

## Overview

| Type | Description | Graph-TS Equivalent |
|------|-------------|---------------------|
| `Address` | 20-byte Ethereum address | `Address` |
| `Bytes` | Dynamic byte array | `Bytes` |
| `BigInt` | Arbitrary precision integer | `BigInt` |
| `BigDecimal` | Arbitrary precision decimal | `BigDecimal` |
| `String` | UTF-8 string | `string` |
| `i32` | 32-bit signed integer | `i32` |
| `bool` | Boolean | `boolean` |

## Importing Types

All core types are available via the prelude:

```rust
use yogurt_runtime::prelude::*;

// Now you have access to:
// Address, Bytes, BigInt, BigDecimal, Value, Entity
```

## Type Conversions

### Address ↔ Bytes

```rust
// Address to Bytes (automatic via From trait)
let address: Address = Address::from([0x11; 20]);
let bytes: Bytes = address.into();

// In setters, coercion is automatic
transfer.set_from(event.params.from);  // Address → Bytes
```

### Address ↔ String

```rust
// Address to hex string
let hex: String = address.to_hex();  // "0x1111..."

// String to Address
let address = Address::from_string("0x1111111111111111111111111111111111111111");
```

### Bytes ↔ String

```rust
// Bytes to hex string
let hex: String = bytes.to_hex();

// Hex string to Bytes
let bytes = Bytes::from_hex_string("0xdeadbeef").unwrap();
```

### BigInt ↔ String

```rust
// BigInt to string
let s: String = big_int.to_string();

// String to BigInt
let big_int = BigInt::from_string("12345678901234567890").unwrap();
```

### BigInt ↔ Primitives

```rust
// From primitives
let a = BigInt::from_i32(42);
let b = BigInt::from_u64(1_000_000);

// To primitives (may overflow)
let n: i32 = big_int.to_i32();
let n: u64 = big_int.to_u64();
```

## Value Type

`Value` is the universal type for entity fields:

```rust
pub enum Value {
    String(String),
    Int(i32),
    BigDecimal(BigDecimal),
    Bool(bool),
    Array(Vec<Value>),
    Null,
    Bytes(Bytes),
    BigInt(BigInt),
}
```

Converting to/from Value:

```rust
// To Value
let v = Value::from("hello");
let v = Value::from(BigInt::from_i32(42));
let v = Value::from(true);

// From Value
let s: Option<String> = value.to_string();
let n: Option<BigInt> = value.to_big_int();
let b: Option<bool> = value.to_bool();
```

## Entity Data

`EntityData` is a map of field names to values:

```rust
let mut data = EntityData::new();
data.set("name", Value::from("Token"));
data.set("decimals", Value::from(BigInt::from_i32(18)));

// Get a value
if let Some(name) = data.get("name") {
    // ...
}
```

This is used internally for entity serialization. You typically don't need to use it directly.

## Zero Values

All numeric types have zero constructors:

```rust
let zero_int = BigInt::zero();
let zero_dec = BigDecimal::zero();

// Check if zero
if big_int.is_zero() {
    // ...
}
```

## Cloning

All types implement `Clone`:

```rust
let a = BigInt::from_i32(42);
let b = a.clone();

let addr = event.params.from.clone();
```

Use `.clone()` when you need to keep a value after moving it.
