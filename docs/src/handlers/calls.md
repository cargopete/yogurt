# Call Handlers

Call handlers are triggered when a specific function is called on a contract. They're useful for tracking state changes that don't emit events.

## Basic Structure

```rust
use yogurt_runtime::prelude::*;
use crate::generated::{TransferCall, TransferRecord};

#[handler]
fn handle_transfer_call(call: TransferCall) {
    let id = call_id!(call);

    TransferRecord::builder(id)
        .from(call.from.clone())
        .to(call.inputs.to.clone())
        .amount(call.inputs.amount.clone())
        .block_number(call.block.number.clone())
        .save();
}
```

## Call Structure

Generated call types have this structure:

```rust
pub struct TransferCall {
    pub inputs: TransferInputs,
    pub outputs: TransferOutputs,
    pub block: Block,
    pub transaction: Transaction,
    pub from: Address,        // Caller address
    pub to: Address,          // Contract address
}

pub struct TransferInputs {
    pub to: Address,
    pub amount: BigInt,
}

pub struct TransferOutputs {
    pub success: bool,
}
```

## Manifest Configuration

```yaml
callHandlers:
  - function: transfer(address,uint256)
    handler: handleTransferCall
```

The function signature must match your ABI exactly.

## Accessing Call Data

```rust
#[handler]
fn handle_transfer_call(call: TransferCall) {
    // Input parameters
    let recipient = &call.inputs.to;
    let amount = &call.inputs.amount;

    // Output values (if any)
    let success = call.outputs.success;

    // Call context
    let caller = &call.from;
    let contract = &call.to;

    // Block/transaction data
    let block_number = &call.block.number;
    let tx_hash = &call.transaction.hash;
}
```

## ID Generation

Use the `call_id!` macro:

```rust
let id = call_id!(call);  // "0xabc...123"
```

This returns the transaction hash. If you need uniqueness for multiple calls in one transaction, combine with an index:

```rust
let id = format!("{}-{}", call_id!(call), call_index);
```

## Use Cases

### Tracking Proxy Calls

```rust
#[handler]
fn handle_upgrade_call(call: UpgradeToCall) {
    let mut proxy = Proxy::load_or_create(&call.to.to_hex(), |p| {
        p.set_created_at(call.block.timestamp.clone());
    });

    proxy.set_implementation(call.inputs.new_implementation.into());
    proxy.set_upgraded_at(call.block.timestamp.clone());
    proxy.save();
}
```

### Admin Functions

```rust
#[handler]
fn handle_set_fee_call(call: SetFeeCall) {
    let id = call_id!(call);

    FeeChange::builder(id)
        .old_fee(call.inputs.old_fee.clone())
        .new_fee(call.inputs.new_fee.clone())
        .changed_by(call.from.clone())
        .block_number(call.block.number.clone())
        .save();
}
```

## Limitations

- Call handlers only work with **call traces**, not all networks support them
- Internal calls may not be captured
- They're generally slower than event handlers

When possible, prefer event handlers. Use call handlers when:
- The contract doesn't emit events for the action
- You need to track function inputs/outputs that events don't include
