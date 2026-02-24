//! Event handlers for the ERC-20 Transfer subgraph

use alloc::string::ToString;
use yogurt_runtime::prelude::*;
use yogurt_macros::handler;

use crate::generated::{Transfer, TransferEvent};

/// Handle a Transfer event from an ERC-20 contract.
///
/// Creates a Transfer entity with the event data and saves it to the store.
#[handler]
pub fn handle_transfer(event: TransferEvent) {
    // Create unique ID from transaction hash + log index
    let id = alloc::format!(
        "{}-{}",
        event.transaction.hash.to_hex(),
        event.log_index.to_string()
    );

    // Create and populate the Transfer entity
    let mut transfer = Transfer::new(id);

    // Set the transfer details from event params
    transfer.set_from(Bytes::from(event.params.from.as_bytes()));
    transfer.set_to(Bytes::from(event.params.to.as_bytes()));
    transfer.set_value(event.params.value);

    // Set block context
    transfer.set_block_number(event.block.number);
    transfer.set_block_timestamp(event.block.timestamp);
    transfer.set_transaction_hash(event.transaction.hash);

    // Save to the store
    transfer.save();
}
