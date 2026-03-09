# Event Handlers

Event handlers are the most common type of handler. They're called whenever a smart contract emits a matching event.

## Basic Structure

```rust
use yogurt_runtime::prelude::*;
use crate::generated::{Transfer, TransferEvent};

#[handler]
fn handle_transfer(event: TransferEvent) {
    let id = log_id!(event);

    let mut transfer = Transfer::new(id);
    transfer.set_from(event.params.from);
    transfer.set_to(event.params.to);
    transfer.set_value(event.params.value);
    transfer.save();
}
```

## The `#[handler]` Attribute

The `#[handler]` attribute:
- Exports the function with the correct WASM signature
- Handles parameter deserialization from graph-node

You can customize the exported name:

```rust
#[handler(name = "handleERC20Transfer")]
fn handle_transfer(event: TransferEvent) {
    // ...
}
```

## Event Structure

Generated event types have this structure:

```rust
pub struct TransferEvent {
    pub params: TransferParams,
    pub block: Block,
    pub transaction: Transaction,
    pub log_index: BigInt,
}

pub struct TransferParams {
    pub from: Address,
    pub to: Address,
    pub value: BigInt,
}
```

### Accessing Event Data

```rust
#[handler]
fn handle_transfer(event: TransferEvent) {
    // Event parameters
    let from = &event.params.from;
    let to = &event.params.to;
    let value = &event.params.value;

    // Block data
    let block_number = &event.block.number;
    let block_timestamp = &event.block.timestamp;
    let block_hash = &event.block.hash;

    // Transaction data
    let tx_hash = &event.transaction.hash;
    let tx_from = &event.transaction.from;
    let gas_price = &event.transaction.gas_price;

    // Log index (position in block)
    let log_index = &event.log_index;
}
```

## ID Generation

Use the `log_id!` macro for unique event IDs:

```rust
let id = log_id!(event);  // "0xabc...123-42"
```

This combines the transaction hash and log index, guaranteeing uniqueness.

## Manifest Configuration

In `subgraph.yaml`:

```yaml
eventHandlers:
  - event: Transfer(indexed address,indexed address,uint256)
    handler: handleTransfer
```

The event signature must match your ABI exactly, including `indexed` annotations.

## Multiple Events

Handle multiple events from the same contract:

```rust
#[handler]
fn handle_transfer(event: TransferEvent) {
    // Handle transfers
}

#[handler]
fn handle_approval(event: ApprovalEvent) {
    // Handle approvals
}
```

## Filtering Events

Graph-node supports event filters in the manifest:

```yaml
eventHandlers:
  - event: Transfer(indexed address,indexed address,uint256)
    handler: handleTransfer
    filter:
      kind: call
```

This only triggers when the event is emitted from a direct call (not internal).

## Receipt Data

Access transaction receipt data if available:

```rust
#[handler]
fn handle_transfer(event: TransferEvent) {
    if let Some(receipt) = &event.transaction.receipt {
        let gas_used = &receipt.gas_used;
        let status = receipt.status;  // 1 = success, 0 = revert
    }
}
```

> Note: Receipt data requires `receipt: true` in your manifest and may not be available on all networks.
