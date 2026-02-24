//! Testing utilities for subgraph mappings.
//!
//! This module provides mock implementations of the runtime
//! that can be used for unit testing handlers without WASM.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::ethereum::{Block, Transaction, TransactionReceipt};
use crate::types::{Address, BigInt, Bytes, EntityData};

/// A mock context for testing subgraph handlers.
pub struct MockContext {
    store: BTreeMap<String, BTreeMap<String, EntityData>>,
}

impl MockContext {
    /// Create a new empty mock context.
    pub fn new() -> Self {
        Self {
            store: BTreeMap::new(),
        }
    }

    /// Store an entity in the mock store.
    pub fn store<E: crate::types::Entity>(&mut self, entity: &E) {
        let type_name = E::ENTITY_TYPE.to_string();
        let id = entity.id().to_string();

        // TODO: Extract EntityData from entity
        let data = EntityData::new();

        self.store
            .entry(type_name)
            .or_insert_with(BTreeMap::new)
            .insert(id, data);
    }

    /// Load an entity from the mock store.
    pub fn load<E: crate::types::Entity>(&self, id: &str) -> Option<E> {
        let type_name = E::ENTITY_TYPE;
        self.store
            .get(type_name)
            .and_then(|entities| entities.get(id))
            .and_then(|_data| {
                // TODO: Construct entity from EntityData
                None
            })
    }

    /// Check if an entity exists in the mock store.
    pub fn exists<E: crate::types::Entity>(&self, id: &str) -> bool {
        let type_name = E::ENTITY_TYPE;
        self.store
            .get(type_name)
            .map(|entities| entities.contains_key(id))
            .unwrap_or(false)
    }

    /// Clear all entities of a given type.
    pub fn clear<E: crate::types::Entity>(&mut self) {
        self.store.remove(E::ENTITY_TYPE);
    }

    /// Clear the entire mock store.
    pub fn clear_all(&mut self) {
        self.store.clear();
    }
}

impl Default for MockContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a mock Ethereum block for testing.
pub fn mock_block(number: u64, timestamp: u64) -> Block {
    Block {
        hash: Bytes::from(vec![0u8; 32]),
        parent_hash: Bytes::from(vec![0u8; 32]),
        uncles_hash: Bytes::from(vec![0u8; 32]),
        author: Address::zero(),
        state_root: Bytes::from(vec![0u8; 32]),
        transactions_root: Bytes::from(vec![0u8; 32]),
        receipts_root: Bytes::from(vec![0u8; 32]),
        number: BigInt::from_u64(number),
        gas_used: BigInt::zero(),
        gas_limit: BigInt::from_u64(30_000_000),
        timestamp: BigInt::from_u64(timestamp),
        difficulty: BigInt::zero(),
        total_difficulty: BigInt::zero(),
        size: None,
        base_fee_per_gas: None,
    }
}

/// Create a mock Ethereum transaction for testing.
pub fn mock_transaction(hash: [u8; 32], from: Address, to: Option<Address>) -> Transaction {
    Transaction {
        hash: Bytes::from(hash.as_slice()),
        index: BigInt::zero(),
        from,
        to,
        value: BigInt::zero(),
        gas_limit: BigInt::from_u64(21_000),
        gas_price: BigInt::from_u64(1_000_000_000),
        input: Bytes::new(),
        nonce: BigInt::zero(),
    }
}

/// Create a mock transaction receipt for testing.
pub fn mock_receipt(tx_hash: [u8; 32], block_number: u64) -> TransactionReceipt {
    TransactionReceipt {
        transaction_hash: Bytes::from(tx_hash.as_slice()),
        transaction_index: BigInt::zero(),
        block_hash: Bytes::from(vec![0u8; 32]),
        block_number: BigInt::from_u64(block_number),
        cumulative_gas_used: BigInt::from_u64(21_000),
        gas_used: BigInt::from_u64(21_000),
        contract_address: None,
        status: BigInt::one(),
        root: Bytes::new(),
        logs_bloom: Bytes::from(vec![0u8; 256]),
    }
}
