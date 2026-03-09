# Migrating from AssemblyScript

This guide helps you convert existing AssemblyScript subgraphs to yogurt.

## Overview

The migration process:
1. Create new Rust project structure
2. Copy schema and ABIs (unchanged)
3. Update manifest for Rust
4. Convert handlers to Rust
5. Convert tests (if any)

## Project Structure

**AssemblyScript:**
```
my-subgraph/
├── package.json
├── subgraph.yaml
├── schema.graphql
├── abis/
└── src/
    └── mapping.ts
```

**yogurt:**
```
my-subgraph/
├── Cargo.toml
├── subgraph.yaml
├── schema.graphql
├── abis/
├── src/
│   ├── lib.rs
│   └── generated/
└── tests/
```

## Manifest Changes

Update `subgraph.yaml`:

```yaml
# Change the file path only — the rest stays the same!
mapping:
  kind: ethereum/events
  apiVersion: 0.0.7
  language: wasm/assemblyscript   # Keep this! yogurt is binary-compatible
  file: ./build/subgraph.wasm     # Was: ./src/mapping.ts
```

> **Key insight:** yogurt produces WASM that's binary-compatible with AssemblyScript, so graph-node treats it identically. You don't need to change `kind` or `language` — just point `file` to your compiled WASM.

## Type Mapping

| AssemblyScript | yogurt (Rust) |
|----------------|---------------|
| `Address` | `Address` |
| `Bytes` | `Bytes` |
| `BigInt` | `BigInt` |
| `BigDecimal` | `BigDecimal` |
| `string` | `String` |
| `i32` | `i32` |
| `boolean` | `bool` |
| `Array<T>` | `Vec<T>` |
| `T \| null` | `Option<T>` |

## Handler Conversion

### AssemblyScript

```typescript
import { Transfer } from "../generated/schema"
import { Transfer as TransferEvent } from "../generated/ERC20/ERC20"

export function handleTransfer(event: TransferEvent): void {
  let id = event.transaction.hash.toHex() + "-" + event.logIndex.toString()

  let transfer = new Transfer(id)
  transfer.from = event.params.from
  transfer.to = event.params.to
  transfer.value = event.params.value
  transfer.blockNumber = event.block.number
  transfer.save()
}
```

### yogurt (Rust)

```rust
use yogurt_runtime::prelude::*;
use crate::generated::{Transfer, TransferEvent};

#[handler]
fn handle_transfer(event: TransferEvent) {
    let id = log_id!(event);

    Transfer::builder(id)
        .from(event.params.from)
        .to(event.params.to)
        .value(event.params.value)
        .block_number(event.block.number.clone())
        .save();
}
```

## Common Patterns

### Entity Loading

**AS:**
```typescript
let token = Token.load(id)
if (token == null) {
  token = new Token(id)
  token.symbol = "UNKNOWN"
}
```

**Rust:**
```rust
let token = Token::load_or_create(&id, |t| {
    t.set_symbol("UNKNOWN".to_string());
});
```

### Null Checks

**AS:**
```typescript
if (entity.field != null) {
  // use field
}
```

**Rust:**
```rust
if let Some(field) = entity.field() {
    // use field
}
```

### BigInt Arithmetic

**AS:**
```typescript
let sum = a.plus(b)
let product = a.times(b)
let quotient = a.div(b)
```

**Rust:**
```rust
let sum = a.plus(&b);       // or &a + &b
let product = a.times(&b);  // or &a * &b
let quotient = a.divided_by(&b);  // or &a / &b
```

### String Concatenation

**AS:**
```typescript
let id = address.toHex() + "-" + index.toString()
```

**Rust:**
```rust
let id = format!("{}-{}", address.to_hex(), index.to_string());
```

### Bytes to Hex

**AS:**
```typescript
let hex = bytes.toHexString()
```

**Rust:**
```rust
let hex = bytes.to_hex();
```

### Address Comparison

**AS:**
```typescript
if (address.equals(Address.zero())) {
  // zero address
}
```

**Rust:**
```rust
if address.is_zero() {
    // zero address
}
```

### Contract Calls

**AS:**
```typescript
let contract = ERC20.bind(address)
let result = contract.try_symbol()
if (!result.reverted) {
  token.symbol = result.value
}
```

**Rust:**
```rust
let contract = ERC20Contract::bind(address.clone());
if let Some(symbol) = contract.try_symbol() {
    token.set_symbol(symbol);
}
```

### Data Source Templates

**AS:**
```typescript
import { Pair as PairTemplate } from "../generated/templates"

PairTemplate.create(pairAddress)
```

**Rust:**
```rust
use crate::generated::templates::Pair as PairTemplate;

PairTemplate::create(pair_address);
```

### Logging

**AS:**
```typescript
log.info("Transfer: {} -> {}", [from.toHex(), to.toHex()])
```

**Rust:**
```rust
log::info!("Transfer: {} -> {}", from.to_hex(), to.to_hex());
```

## What's Better in yogurt

### ID Generation

**AS:**
```typescript
let id = event.transaction.hash.toHex() + "-" + event.logIndex.toString()
```

**Rust:**
```rust
let id = log_id!(event);
```

### Entity Builders

**AS:**
```typescript
let swap = new Swap(id)
swap.pair = pairId
swap.sender = event.params.sender
swap.amount0In = event.params.amount0In
swap.save()
```

**Rust:**
```rust
Swap::builder(id)
    .pair(&pair_id)
    .sender(event.params.sender)
    .amount0_in(event.params.amount0_in)
    .save();
```

### Safe Division

**AS:**
```typescript
// Must check manually
let price = reserve0.gt(BigInt.zero())
  ? reserve1.div(reserve0)
  : BigInt.zero()
```

**Rust:**
```rust
let price = reserve1.safe_div(&reserve0);  // Returns zero if reserve0 is zero
```

### Address Coercion

**AS:**
```typescript
transfer.from = event.params.from  // Error: Address vs Bytes
transfer.from = Bytes.fromHexString(event.params.from.toHex())  // Tedious
```

**Rust:**
```rust
transfer.set_from(event.params.from);  // Just works
```

## Testing Migration

**AS (matchstick):**
```typescript
test("handleTransfer creates entity", () => {
  let event = createTransferEvent(from, to, value)
  handleTransfer(event)
  assert.fieldEquals("Transfer", id, "value", "1000")
})
```

**Rust:**
```rust
#[test]
fn test_handle_transfer() {
    clear_store();

    let event: TransferEvent = EventBuilder::new()
        .params(TransferParams { from, to, value })
        .build();

    handle_transfer(event);

    let transfer = Transfer::load(&id).unwrap();
    assert_eq!(transfer.value().to_string(), "1000");
}
```

## Migration Checklist

- [ ] Create `Cargo.toml` with dependencies
- [ ] Copy `schema.graphql` (no changes needed)
- [ ] Copy `abis/` directory (no changes needed)
- [ ] Update `subgraph.yaml` mapping section
- [ ] Run `yogurt codegen`
- [ ] Convert each handler function
- [ ] Convert helper functions
- [ ] Add `#[handler]` attributes
- [ ] Run `yogurt build`
- [ ] Run `yogurt validate`
- [ ] Convert tests to Rust
- [ ] Test locally with `yogurt deploy`
