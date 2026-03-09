# EventBuilder

`EventBuilder` constructs test events with customizable block and transaction data.

## Basic Usage

```rust
use yogurt_runtime::testing::EventBuilder;

let event: TransferEvent = EventBuilder::new()
    .params(TransferParams {
        from: Address::from([0x11; 20]),
        to: Address::from([0x22; 20]),
        value: BigInt::from_u64(1_000_000),
    })
    .build();
```

## Builder Methods

### params

Set the event-specific parameters (required):

```rust
.params(TransferParams {
    from: Address::from([0x11; 20]),
    to: Address::from([0x22; 20]),
    value: BigInt::from_u64(1_000_000),
})
```

### block_number

Set the block number:

```rust
.block_number(12345678)
```

Default: `1`

### block_timestamp

Set the block timestamp (Unix seconds):

```rust
.block_timestamp(1704067200)  // 2024-01-01 00:00:00 UTC
```

Default: `0`

### block_hash

Set the block hash:

```rust
.block_hash([0xBB; 32])
```

Default: `[0; 32]`

### transaction_hash

Set the transaction hash:

```rust
.transaction_hash([0xAB; 32])
```

Default: `[0; 32]`

### transaction_from

Set the transaction sender:

```rust
.transaction_from(Address::from([0x11; 20]))
```

Default: Zero address

### log_index

Set the log index within the block:

```rust
.log_index(5)
```

Default: `0`

## Complete Example

```rust
let event: SwapEvent = EventBuilder::new()
    // Block data
    .block_number(15_000_000)
    .block_timestamp(1660000000)
    .block_hash([0xBB; 32])
    // Transaction data
    .transaction_hash([0xAB; 32])
    .transaction_from(Address::from([0x99; 20]))
    // Event data
    .log_index(3)
    .params(SwapParams {
        sender: Address::from([0x11; 20]),
        amount0_in: BigInt::from_u64(1_000_000),
        amount1_in: BigInt::zero(),
        amount0_out: BigInt::zero(),
        amount1_out: BigInt::from_u64(500_000),
        to: Address::from([0x22; 20]),
    })
    .build();
```

## CallBuilder

For call handlers, use `CallBuilder`:

```rust
use yogurt_runtime::testing::CallBuilder;

let call: TransferCall = CallBuilder::new()
    .block_number(12345678)
    .transaction_hash([0xAB; 32])
    .from(Address::from([0x11; 20]))
    .to(Address::from([0x22; 20]))
    .inputs(TransferInputs {
        recipient: Address::from([0x33; 20]),
        amount: BigInt::from_u64(1_000_000),
    })
    .outputs(TransferOutputs {
        success: true,
    })
    .build();
```

### CallBuilder Methods

- `.block_number(u64)` — Block number
- `.block_timestamp(u64)` — Block timestamp
- `.transaction_hash([u8; 32])` — Transaction hash
- `.from(Address)` — Caller address
- `.to(Address)` — Contract address
- `.inputs(I)` — Function inputs
- `.outputs(O)` — Function outputs

## BlockBuilder

For block handlers, use `BlockBuilder`:

```rust
use yogurt_runtime::testing::BlockBuilder;

let block: Block = BlockBuilder::new()
    .number(15_000_000)
    .timestamp(1660000000)
    .hash([0xBB; 32])
    .parent_hash([0xAA; 32])
    .gas_used(15_000_000)
    .gas_limit(30_000_000)
    .author(Address::from([0x11; 20]))
    .build();
```

### BlockBuilder Methods

- `.number(u64)` — Block number
- `.timestamp(u64)` — Block timestamp
- `.hash([u8; 32])` — Block hash
- `.parent_hash([u8; 32])` — Parent block hash
- `.gas_used(u64)` — Gas used
- `.gas_limit(u64)` — Gas limit
- `.author(Address)` — Block author/miner
- `.difficulty(BigInt)` — Block difficulty
- `.base_fee_per_gas(BigInt)` — EIP-1559 base fee

## Helper Functions

Create test data quickly:

```rust
fn test_address(seed: u8) -> Address {
    Address::from([seed; 20])
}

fn test_bytes32(seed: u8) -> [u8; 32] {
    [seed; 32]
}

fn test_transfer_event(from: u8, to: u8, value: u64) -> TransferEvent {
    EventBuilder::new()
        .block_number(12345678)
        .transaction_hash(test_bytes32(0xAB))
        .params(TransferParams {
            from: test_address(from),
            to: test_address(to),
            value: BigInt::from_u64(value),
        })
        .build()
}

// Usage
let event = test_transfer_event(0x11, 0x22, 1_000_000);
```
