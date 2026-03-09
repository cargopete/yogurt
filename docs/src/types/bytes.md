# Address and Bytes

Types for handling Ethereum addresses and arbitrary byte data.

## Address

`Address` is a 20-byte Ethereum address.

### Creating Address

```rust
// From fixed array
let addr = Address::from([0x11; 20]);

// From string (hex)
let addr = Address::from_string("0x1111111111111111111111111111111111111111");

// Zero address
let zero = Address::zero();
```

### Methods

```rust
let addr = Address::from([0x11; 20]);

// To hex string (with 0x prefix)
let hex: String = addr.to_hex();  // "0x1111111111111111111111111111111111111111"

// Check if zero address
if addr.is_zero() {
    // This is a burn or mint
}

// Access raw bytes
let bytes: &[u8; 20] = &addr.0;
```

### Zero Address Check

Common pattern for detecting burns/mints:

```rust
#[handler]
fn handle_transfer(event: TransferEvent) {
    if event.params.from.is_zero() {
        // This is a mint
        log::info!("Mint detected");
    }

    if event.params.to.is_zero() {
        // This is a burn
        log::info!("Burn detected");
    }
}
```

### Automatic Coercion

`Address` automatically converts to `Bytes`:

```rust
// In entity setters - just works
transfer.set_from(event.params.from);  // Address → Bytes

// Explicit conversion
let bytes: Bytes = address.into();
```

## Bytes

`Bytes` is a dynamic byte array.

### Creating Bytes

```rust
// From Vec<u8>
let bytes = Bytes::from(vec![0xde, 0xad, 0xbe, 0xef]);

// From slice
let bytes = Bytes::from(&[0xde, 0xad, 0xbe, 0xef][..]);

// From hex string
let bytes = Bytes::from_hex_string("0xdeadbeef").unwrap();

// From fixed arrays
let bytes = Bytes::from([0x11u8; 20]);  // [u8; 20]
let bytes = Bytes::from([0x22u8; 32]);  // [u8; 32]

// Empty
let empty = Bytes::empty();
```

### Methods

```rust
let bytes = Bytes::from_hex_string("0xdeadbeef").unwrap();

// To hex string
let hex: String = bytes.to_hex();  // "0xdeadbeef"

// Length
let len: usize = bytes.len();

// Access raw bytes
let raw: &[u8] = bytes.as_slice();

// Check if empty
if bytes.is_empty() { /* ... */ }
```

### Concatenation

```rust
let a = Bytes::from_hex_string("0xdead").unwrap();
let b = Bytes::from_hex_string("0xbeef").unwrap();

// Concat two Bytes
let combined = a.concat(&b);  // 0xdeadbeef

// Concat with i32 (useful for unique IDs)
let with_index = a.concat_i32(42);  // 0xdead0000002a
```

### Common Patterns

#### Transaction Hash as ID

```rust
let tx_hash: Bytes = event.transaction.hash.clone();
let id = tx_hash.to_hex();
```

#### Combining for Unique IDs

```rust
// Unique ID from address + token ID
fn token_balance_id(owner: &Address, token_id: &BigInt) -> String {
    format!("{}-{}", owner.to_hex(), token_id.to_string())
}

// Or using concat
fn unique_id(prefix: &Bytes, index: i32) -> Bytes {
    prefix.concat_i32(index)
}
```

#### Comparing Bytes

```rust
let a = Bytes::from_hex_string("0xdead").unwrap();
let b = Bytes::from_hex_string("0xdead").unwrap();

if a == b {
    // Equal
}
```

## Type Conversions Summary

| From | To | Method |
|------|-----|--------|
| `Address` | `Bytes` | `.into()` or automatic |
| `Address` | `String` | `.to_hex()` |
| `String` | `Address` | `Address::from_string()` |
| `[u8; 20]` | `Address` | `Address::from()` |
| `[u8; 20]` | `Bytes` | `Bytes::from()` |
| `[u8; 32]` | `Bytes` | `Bytes::from()` |
| `Bytes` | `String` | `.to_hex()` |
| `String` | `Bytes` | `Bytes::from_hex_string()` |
| `Vec<u8>` | `Bytes` | `Bytes::from()` |
| `&[u8]` | `Bytes` | `Bytes::from()` |
