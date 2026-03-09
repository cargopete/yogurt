# Defining Entities

Entities are defined in your `schema.graphql` file using GraphQL syntax. yogurt generates Rust structs with full type safety.

## Basic Entity

```graphql
type Transfer @entity {
  id: ID!
  from: Bytes!
  to: Bytes!
  value: BigInt!
  timestamp: BigInt!
}
```

The `@entity` directive marks a type for storage. The `id` field is required.

## Field Types

| GraphQL Type | Rust Type | Description |
|--------------|-----------|-------------|
| `ID!` | `String` | Unique identifier (required) |
| `String` | `String` | UTF-8 text |
| `Bytes` | `Bytes` | Arbitrary bytes |
| `BigInt` | `BigInt` | Arbitrary precision integer |
| `BigDecimal` | `BigDecimal` | Arbitrary precision decimal |
| `Int` | `i32` | 32-bit signed integer |
| `Boolean` | `bool` | True/false |

## Required vs Optional Fields

```graphql
type Token @entity {
  id: ID!
  symbol: String!        # Required - Rust: String
  name: String           # Optional - Rust: Option<String>
  decimals: BigInt!      # Required
  totalSupply: BigInt    # Optional
}
```

Optional fields:
- Use `Option<T>` in Rust
- Have `unset_*` methods generated
- Return `None` when not set

## Array Fields

```graphql
type Pool @entity {
  id: ID!
  tokens: [Bytes!]!       # Required array of required Bytes
  amounts: [BigInt!]!     # Required array of required BigInt
  tags: [String!]         # Optional array
}
```

Arrays are `Vec<T>` in Rust:

```rust
let pool = Pool::load(&id).unwrap();
let tokens: &Vec<Bytes> = pool.tokens();

pool.set_tokens(vec![token1.into(), token2.into()]);
```

## Immutable Entities

Mark entities as immutable when they should never change after creation:

```graphql
type Transfer @entity(immutable: true) {
  id: ID!
  from: Bytes!
  to: Bytes!
  value: BigInt!
}
```

Immutable entities:
- Have no setter methods generated
- Can only be created, not updated
- Are more efficient for graph-node

## Derived Fields

Use `@derivedFrom` for reverse lookups:

```graphql
type Token @entity {
  id: ID!
  symbol: String!
  transfers: [Transfer!]! @derivedFrom(field: "token")
}

type Transfer @entity {
  id: ID!
  token: Token!
  value: BigInt!
}
```

Derived fields:
- Are computed by graph-node at query time
- Don't exist in Rust code (no getter/setter)
- Don't need to be set in handlers

## Relations

Reference other entities by ID:

```graphql
type Pair @entity {
  id: ID!
  token0: Token!          # Stores token0.id as String
  token1: Token!
}

type Token @entity {
  id: ID!
  symbol: String!
}
```

In Rust, relations are stored as ID strings:

```rust
// Setting a relation
pair.set_token0(&token.id());

// Getting a relation (returns the ID)
let token0_id: &String = pair.token0();

// Load the related entity
let token0 = Token::load(pair.token0()).unwrap();
```

## Full Example

```graphql
type Factory @entity {
  id: ID!
  pairCount: BigInt!
  totalVolumeUSD: BigDecimal!
}

type Token @entity {
  id: ID!
  symbol: String!
  name: String!
  decimals: BigInt!
  totalSupply: BigInt
  pairs: [Pair!]! @derivedFrom(field: "token0")
}

type Pair @entity {
  id: ID!
  token0: Token!
  token1: Token!
  reserve0: BigDecimal!
  reserve1: BigDecimal!
  swaps: [Swap!]! @derivedFrom(field: "pair")
}

type Swap @entity(immutable: true) {
  id: ID!
  pair: Pair!
  sender: Bytes!
  amount0In: BigDecimal!
  amount1In: BigDecimal!
  amount0Out: BigDecimal!
  amount1Out: BigDecimal!
  timestamp: BigInt!
}
```
