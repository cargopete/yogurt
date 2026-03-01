//! Event handlers for the ERC-20 Transfer subgraph

use alloc::string::ToString;
use yogurt_runtime::prelude::*;
use yogurt_runtime::data_source;
use yogurt_macros::handler;

use crate::generated::{Transfer, TransferCall, TransferEvent};
use crate::generated::templates::TokenMetadata;

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

    // Example: spawn a file data source to fetch token metadata
    // In a real subgraph, you'd get the IPFS hash from contract state
    // TokenMetadata::create("QmExampleCID...");
}

/// Handle a transfer() function call.
///
/// This demonstrates call handlers - triggered when the function is called,
/// not when an event is emitted.
#[handler]
pub fn handle_transfer_call(call: TransferCall) {
    // Create unique ID from transaction hash
    let id = alloc::format!(
        "call-{}",
        call.transaction.hash.to_hex()
    );

    // Create and populate the Transfer entity
    let mut transfer = Transfer::new(id);

    // Set transfer details from call inputs
    transfer.set_from(Bytes::from(call.from.as_bytes())); // caller
    transfer.set_to(Bytes::from(call.inputs.to.as_bytes()));
    transfer.set_value(call.inputs.value);

    // Set block context
    transfer.set_block_number(call.block.number);
    transfer.set_block_timestamp(call.block.timestamp);
    transfer.set_transaction_hash(call.transaction.hash);

    // Save to the store
    transfer.save();
}

/// Handle token metadata from IPFS.
///
/// This is a file data source handler - it receives the raw file content
/// and can access the content ID via data_source::string_param().
#[handler]
pub fn handle_metadata(content: Bytes) {
    // Get the IPFS CID that triggered this handler
    let ipfs_cid = data_source::string_param();

    yogurt_runtime::log_info!("Processing metadata from IPFS: {}", ipfs_cid);

    // In a real handler, you'd parse the content as JSON
    // and create/update entities based on the metadata
    let _content_length = content.len();
}
