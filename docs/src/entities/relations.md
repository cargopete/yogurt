# Entity Relations

Relations connect entities together, allowing you to model complex data structures.

## One-to-One Relations

```graphql
type Pair @entity {
  id: ID!
  token0: Token!
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
let token = Token::load(&token_address).unwrap();
pair.set_token0(&token.id());

// Getting the relation (returns the ID)
let token0_id: &String = pair.token0();

// Loading the related entity
let token0 = Token::load(pair.token0()).unwrap();
```

## One-to-Many with @derivedFrom

Use `@derivedFrom` for reverse lookups computed at query time:

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

The `transfers` field is:
- **Not stored** — computed by graph-node at query time
- **Not in Rust** — no getter/setter generated
- **Read-only** — you can't set derived fields

To create the relationship, set the "owning" side:

```rust
let mut transfer = Transfer::new(id);
transfer.set_token(&token.id());  // This creates the relation
transfer.save();
```

## Many-to-Many Relations

Model many-to-many with a join entity:

```graphql
type Pool @entity {
  id: ID!
  tokens: [PoolToken!]! @derivedFrom(field: "pool")
}

type Token @entity {
  id: ID!
  pools: [PoolToken!]! @derivedFrom(field: "token")
}

type PoolToken @entity {
  id: ID!
  pool: Pool!
  token: Token!
  balance: BigInt!
}
```

Create the join entities:

```rust
fn add_token_to_pool(pool: &Pool, token: &Token, balance: BigInt) {
    let id = format!("{}-{}", pool.id(), token.id());

    PoolToken::builder(id)
        .pool(&pool.id())
        .token(&token.id())
        .balance(balance)
        .save();
}
```

## Optional Relations

```graphql
type Transfer @entity {
  id: ID!
  token: Token        # Optional relation
}
```

In Rust:

```rust
// Setting
transfer.set_token(&token.id());

// Unsetting
transfer.unset_token();

// Getting (returns Option<&String>)
if let Some(token_id) = transfer.token() {
    let token = Token::load(token_id).unwrap();
}
```

## Self-Referential Relations

Entities can reference themselves:

```graphql
type Category @entity {
  id: ID!
  name: String!
  parent: Category
  children: [Category!]! @derivedFrom(field: "parent")
}
```

```rust
// Create parent
let mut parent = Category::new("parent-id".to_string());
parent.set_name("Parent".to_string());
parent.save();

// Create child with reference to parent
let mut child = Category::new("child-id".to_string());
child.set_name("Child".to_string());
child.set_parent(&parent.id());
child.save();
```

## Patterns

### Loading Related Entities

```rust
fn get_pair_tokens(pair: &Pair) -> (Token, Token) {
    let token0 = Token::load(pair.token0())
        .expect("Token0 must exist");
    let token1 = Token::load(pair.token1())
        .expect("Token1 must exist");
    (token0, token1)
}
```

### Updating Related Entities

```rust
#[handler]
fn handle_swap(event: SwapEvent) {
    let pair_id = data_source::address().to_hex();

    if let Some(pair) = Pair::load(&pair_id) {
        // Update the pair
        Pair::update(&pair_id, |p| {
            p.set_tx_count(p.tx_count() + BigInt::from(1));
        });

        // Update related tokens
        Token::update(pair.token0(), |t| {
            t.set_tx_count(t.tx_count() + BigInt::from(1));
        });

        Token::update(pair.token1(), |t| {
            t.set_tx_count(t.tx_count() + BigInt::from(1));
        });
    }
}
```

### Relation IDs

Common pattern for relation IDs:

```rust
// Pool-Token join entity
let pool_token_id = format!("{}-{}", pool_id, token_id);

// User-Token balance
let balance_id = format!("{}-{}", user_address.to_hex(), token_address.to_hex());

// Daily stats per entity
let daily_token_id = format!("{}-{}", day_id, token_id);
```
