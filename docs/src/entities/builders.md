# Entity Builders

Entity builders provide a fluent API for constructing entities. They're generated automatically by `yogurt codegen`.

## Basic Usage

```rust
Transfer::builder("unique-id")
    .from(event.params.from)
    .to(event.params.to)
    .value(event.params.value)
    .block_number(event.block.number.clone())
    .save();
```

## Builder Methods

### Creating a Builder

```rust
let builder = Transfer::builder("unique-id");
```

The ID is required when creating the builder.

### Setting Fields

Each field has a chainable setter:

```rust
Transfer::builder(id)
    .from(address1)        // Takes Address, converts to Bytes
    .to(address2)
    .value(amount)         // Takes BigInt
    .timestamp(ts);        // Takes BigInt
```

### Building Without Saving

Get the entity without persisting:

```rust
let transfer = Transfer::builder(id)
    .from(address1)
    .to(address2)
    .value(amount)
    .build();

// Do something with transfer...
transfer.save();
```

### Building and Saving

Save directly from the builder:

```rust
Transfer::builder(id)
    .from(address1)
    .to(address2)
    .value(amount)
    .save();  // Builds and saves in one call
```

## Type Coercion

Builders accept types that can be converted:

```rust
// Address → Bytes coercion is automatic
Transfer::builder(id)
    .from(event.params.from)  // from is Address, field is Bytes
    .to(event.params.to)
    .save();
```

For references, use `&`:

```rust
Pair::builder(id)
    .token0(&token0.id())  // Pass reference to String
    .token1(&token1.id())
    .save();
```

## Optional Fields

Optional fields can be omitted:

```rust
// description is optional in schema
Token::builder(id)
    .symbol(symbol)
    .name(name)
    // .description(...)  -- can be omitted
    .save();
```

Or explicitly set to None using the entity after building:

```rust
let mut token = Token::builder(id)
    .symbol(symbol)
    .build();

token.unset_description();
token.save();
```

## Complex Example

```rust
#[handler]
fn handle_swap(event: SwapEvent) {
    let pair_id = data_source::address().to_hex();

    // Build swap entity
    Swap::builder(log_id!(event))
        .pair(&pair_id)
        .sender(event.params.sender)
        .to(event.params.to)
        .amount0_in(BigDecimal::from_big_int(&event.params.amount0_in))
        .amount1_in(BigDecimal::from_big_int(&event.params.amount1_in))
        .amount0_out(BigDecimal::from_big_int(&event.params.amount0_out))
        .amount1_out(BigDecimal::from_big_int(&event.params.amount1_out))
        .timestamp(event.block.timestamp.clone())
        .save();

    // Update pair using traditional style
    if let Some(mut pair) = Pair::load(&pair_id) {
        pair.set_tx_count(pair.tx_count() + BigInt::from(1));
        pair.save();
    }
}
```

## When to Use Builders

Builders are ideal for:
- Creating new entities with many fields
- One-shot entity creation (create and save)
- Clean, readable handler code

Traditional `new` + setters is better for:
- Loading and modifying existing entities
- Conditional field updates
- Complex update logic

## Generated Code

For an entity like:

```graphql
type Transfer @entity {
  id: ID!
  from: Bytes!
  to: Bytes!
  value: BigInt!
}
```

yogurt generates:

```rust
pub struct TransferBuilder {
    entity: Transfer,
}

impl TransferBuilder {
    pub fn from(mut self, value: impl Into<Bytes>) -> Self {
        self.entity.set_from(value.into());
        self
    }

    pub fn to(mut self, value: impl Into<Bytes>) -> Self {
        self.entity.set_to(value.into());
        self
    }

    pub fn value(mut self, value: BigInt) -> Self {
        self.entity.set_value(value);
        self
    }

    pub fn build(self) -> Transfer {
        self.entity
    }

    pub fn save(self) {
        self.entity.save();
    }
}

impl Transfer {
    pub fn builder(id: impl Into<String>) -> TransferBuilder {
        TransferBuilder {
            entity: Transfer::new(id.into()),
        }
    }
}
```
