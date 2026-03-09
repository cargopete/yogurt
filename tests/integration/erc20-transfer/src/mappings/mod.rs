//! Event handlers for the ERC-20 Transfer subgraph

use yogurt_runtime::prelude::*;
use yogurt_macros::handler;

use crate::generated::{Transfer, TransferEvent};

/// Handle a Transfer event from an ERC-20 contract.
#[handler]
pub fn handle_transfer(event: TransferEvent) {
    // Create unique ID from transaction hash + log index
    let id = log_id!(event);

    // Build and save the transfer entity
    Transfer::builder(id)
        .from(event.params.from)
        .to(event.params.to)
        .value(event.params.value)
        .block_number(event.block.number)
        .block_timestamp(event.block.timestamp)
        .transaction_hash(event.transaction.hash)
        .save();
}

/// Handle a transfer() function call.
#[handler]
pub fn handle_transfer_call(_call: crate::generated::TransferCall) {
    // Call handlers not needed for this example
}

/// Handle token metadata from IPFS.
#[handler]
pub fn handle_metadata(_content: Bytes) {
    // IPFS handler not needed for this example
}
