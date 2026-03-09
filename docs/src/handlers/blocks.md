# Block Handlers

Block handlers are called for every block (or filtered blocks) on the chain. They're useful for periodic aggregations or tracking chain-level metrics.

## Basic Structure

```rust
use yogurt_runtime::prelude::*;
use crate::generated::{BlockStats, Block};

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

## Block Structure

```rust
pub struct Block {
    pub hash: Bytes,
    pub parent_hash: Bytes,
    pub number: BigInt,
    pub timestamp: BigInt,
    pub gas_limit: BigInt,
    pub gas_used: BigInt,
    pub base_fee_per_gas: Option<BigInt>,
    pub author: Address,
    pub difficulty: BigInt,
    pub total_difficulty: BigInt,
    pub size: Option<BigInt>,
}
```

## Manifest Configuration

```yaml
blockHandlers:
  - handler: handleBlock
```

### Filtered Block Handlers

Only trigger on blocks with matching calls:

```yaml
blockHandlers:
  - handler: handleBlock
    filter:
      kind: call
```

Or with polling interval:

```yaml
blockHandlers:
  - handler: handleBlock
    filter:
      kind: polling
      every: 100  # Every 100 blocks
```

## Use Cases

### Daily Aggregations

```rust
#[handler]
fn handle_block(block: Block) {
    let day = day_id_from_timestamp(&block.timestamp);

    DailyStats::upsert(&day, |stats| {
        stats.set_block_count(stats.block_count() + BigInt::from(1));
        stats.set_last_block(block.number.clone());
    });
}

fn day_id_from_timestamp(timestamp: &BigInt) -> String {
    timestamp.divided_by(&BigInt::from_i32(86400)).to_string()
}
```

### Chain Metrics

```rust
#[handler]
fn handle_block(block: Block) {
    let id = block_id!(block);

    let mut metrics = ChainMetrics::new(id);
    metrics.set_block_number(block.number.clone());
    metrics.set_gas_used(block.gas_used.clone());
    metrics.set_gas_limit(block.gas_limit.clone());

    // Calculate utilization
    let utilization = block.gas_used
        .times(&BigInt::from_i32(100))
        .divided_by(&block.gas_limit);
    metrics.set_utilization_percent(utilization);

    metrics.save();
}
```

## Performance Considerations

Block handlers are called frequently. To avoid performance issues:

1. **Use polling filters** when you don't need every block
2. **Keep handlers lightweight** — avoid complex computations
3. **Use efficient entity patterns** — `upsert` for aggregations

```yaml
# Good: Only run every 100 blocks
blockHandlers:
  - handler: handleBlock
    filter:
      kind: polling
      every: 100
```
