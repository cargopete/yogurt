//! Testing utilities for subgraph mappings.
//!
//! This module provides mock implementations of the runtime that can be used
//! for unit testing handlers without WASM.
//!
//! # Example
//!
//! ```rust,ignore
//! use yogurt_runtime::testing::*;
//! use yogurt_runtime::prelude::*;
//!
//! #[test]
//! fn test_handle_transfer() {
//!     clear_store();
//!
//!     let event: TransferEvent = EventBuilder::new()
//!         .block_number(12345)
//!         .params(TransferParams {
//!             from: Address::from([0x11u8; 20]),
//!             to: Address::from([0x22u8; 20]),
//!             value: BigInt::from_u64(1000),
//!         })
//!         .build();
//!
//!     handle_transfer(event);
//!
//!     assert_entity_exists::<Transfer>("expected-id");
//! }
//! ```

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::ethereum::{Block, Call, Event, Token, Transaction, TransactionReceipt};
use crate::types::{Address, BigInt, Bytes, Entity, EntityData};

// ============================================================================
// Thread-Local Mock Store
// ============================================================================

use std::cell::RefCell;

use std::collections::HashSet;

thread_local! {
    /// The mock entity store: entity_type -> id -> data
    static MOCK_STORE: RefCell<BTreeMap<String, BTreeMap<String, EntityData>>> = RefCell::new(BTreeMap::new());

    /// Entities modified in the current block: (entity_type, id)
    /// This is used for loadInBlock simulation.
    static BLOCK_MODIFIED_ENTITIES: RefCell<HashSet<(String, String)>> = RefCell::new(HashSet::new());

    /// Mock Ethereum call registry: (address, signature) -> result or revert
    static MOCK_ETH_CALLS: RefCell<Vec<MockEthereumCall>> = RefCell::new(Vec::new());

    /// Mock data source state
    static MOCK_DATA_SOURCE: RefCell<MockDataSource> = RefCell::new(MockDataSource::default());

    /// Mock IPFS content: CID -> content
    static MOCK_IPFS: RefCell<BTreeMap<String, Vec<u8>>> = RefCell::new(BTreeMap::new());
}

/// Mocked data source state.
#[derive(Clone, Default)]
pub struct MockDataSource {
    pub address: Option<Address>,
    pub network: Option<String>,
    pub context: Option<EntityData>,
}

/// A mocked Ethereum call.
#[derive(Clone)]
pub struct MockEthereumCall {
    pub address: Address,
    pub signature: String,
    pub returns: Option<Vec<Token>>,
}

// ============================================================================
// Store Operations
// ============================================================================

/// Get an entity from the mock store.
pub fn store_get(entity_type: &str, id: &str) -> Option<EntityData> {
    MOCK_STORE.with(|store| {
        store
            .borrow()
            .get(entity_type)
            .and_then(|entities| entities.get(id))
            .cloned()
    })
}

/// Set an entity in the mock store.
///
/// This also marks the entity as modified in the current block,
/// making it available via `store_get_in_block`.
pub fn store_set(entity_type: &str, id: &str, data: &EntityData) {
    MOCK_STORE.with(|store| {
        store
            .borrow_mut()
            .entry(entity_type.to_string())
            .or_insert_with(BTreeMap::new)
            .insert(id.to_string(), data.clone());
    });

    // Track this entity as modified in the current block
    BLOCK_MODIFIED_ENTITIES.with(|modified| {
        modified.borrow_mut().insert((entity_type.to_string(), id.to_string()));
    });
}

/// Remove an entity from the mock store.
pub fn store_remove(entity_type: &str, id: &str) {
    MOCK_STORE.with(|store| {
        if let Some(entities) = store.borrow_mut().get_mut(entity_type) {
            entities.remove(id);
        }
    });
}

/// Clear all entities from the mock store.
///
/// This also clears the block-modified tracking.
pub fn clear_store() {
    MOCK_STORE.with(|store| {
        store.borrow_mut().clear();
    });
    BLOCK_MODIFIED_ENTITIES.with(|modified| {
        modified.borrow_mut().clear();
    });
}

/// Start a new block in the test context.
///
/// This clears the set of entities modified in the current block,
/// so `store_get_in_block` will return `None` for previously modified entities.
/// The actual entity data in the store is preserved.
///
/// Use this when simulating multiple blocks in a test:
///
/// ```ignore
/// // Simulate block 1
/// store_set("Transfer", "tx1", &data1);
/// assert!(store_get_in_block("Transfer", "tx1").is_some());
///
/// // Simulate block 2
/// start_block();
/// assert!(store_get_in_block("Transfer", "tx1").is_none()); // Not modified in this block
/// assert!(store_get("Transfer", "tx1").is_some()); // But still in the store
/// ```
pub fn start_block() {
    BLOCK_MODIFIED_ENTITIES.with(|modified| {
        modified.borrow_mut().clear();
    });
}

/// Get an entity from the mock store, but only if it was modified in the current block.
///
/// Returns `None` if:
/// - The entity does not exist, OR
/// - The entity exists but was NOT modified in the current block
///
/// This simulates graph-node's `store.get_in_block` / `Entity.loadInBlock` behavior.
pub fn store_get_in_block(entity_type: &str, id: &str) -> Option<EntityData> {
    // First check if the entity was modified in the current block
    let was_modified = BLOCK_MODIFIED_ENTITIES.with(|modified| {
        modified.borrow().contains(&(entity_type.to_string(), id.to_string()))
    });

    if !was_modified {
        return None;
    }

    // Then get it from the store
    store_get(entity_type, id)
}

/// Get the count of entities of a given type.
pub fn entity_count<E: Entity>() -> usize {
    MOCK_STORE.with(|store| {
        store
            .borrow()
            .get(E::ENTITY_TYPE)
            .map(|entities| entities.len())
            .unwrap_or(0)
    })
}

/// Assert that an entity exists in the store.
pub fn assert_entity_exists<E: Entity>(id: &str) {
    let exists = MOCK_STORE.with(|store| {
        store
            .borrow()
            .get(E::ENTITY_TYPE)
            .map(|entities| entities.contains_key(id))
            .unwrap_or(false)
    });

    assert!(
        exists,
        "Expected entity {}('{}') to exist, but it does not",
        E::ENTITY_TYPE,
        id
    );
}

/// Assert that an entity does not exist in the store.
pub fn assert_entity_not_exists<E: Entity>(id: &str) {
    let exists = MOCK_STORE.with(|store| {
        store
            .borrow()
            .get(E::ENTITY_TYPE)
            .map(|entities| entities.contains_key(id))
            .unwrap_or(false)
    });

    assert!(
        !exists,
        "Expected entity {}('{}') to not exist, but it does",
        E::ENTITY_TYPE,
        id
    );
}

// ============================================================================
// Mock Ethereum Calls
// ============================================================================

/// Register a mock Ethereum call that returns the given values.
pub fn mock_call(address: Address, signature: &str, returns: Vec<Token>) {
    MOCK_ETH_CALLS.with(|calls| {
        calls.borrow_mut().push(MockEthereumCall {
            address,
            signature: signature.to_string(),
            returns: Some(returns),
        });
    });
}

/// Register a mock Ethereum call that reverts.
pub fn mock_call_reverts(address: Address, signature: &str) {
    MOCK_ETH_CALLS.with(|calls| {
        calls.borrow_mut().push(MockEthereumCall {
            address,
            signature: signature.to_string(),
            returns: None,
        });
    });
}

/// Clear all mock Ethereum calls.
pub fn clear_mocks() {
    MOCK_ETH_CALLS.with(|calls| {
        calls.borrow_mut().clear();
    });
}

/// Execute a mock Ethereum call (called internally by ethereum::call).
pub fn execute_mock_call(address: &Address, signature: &str) -> Option<Vec<Token>> {
    MOCK_ETH_CALLS.with(|calls| {
        // Find matching mock (most recent first)
        for mock in calls.borrow().iter().rev() {
            if mock.address == *address && mock.signature == signature {
                return mock.returns.clone();
            }
        }
        // No mock found, return None (simulates revert)
        None
    })
}

// ============================================================================
// Data Source Mocking
// ============================================================================

/// Set the mock data source address.
///
/// This value will be returned by `data_source::address()` in tests.
///
/// # Example
///
/// ```ignore
/// mock_data_source_address(Address::from([0xAB; 20]));
/// let addr = data_source::address();
/// assert_eq!(addr, Address::from([0xAB; 20]));
/// ```
pub fn mock_data_source_address(address: Address) {
    MOCK_DATA_SOURCE.with(|ds| {
        ds.borrow_mut().address = Some(address);
    });
}

/// Set the mock data source network.
///
/// This value will be returned by `data_source::network()` in tests.
///
/// # Example
///
/// ```ignore
/// mock_data_source_network("goerli");
/// assert_eq!(data_source::network(), "goerli");
/// ```
pub fn mock_data_source_network(network: impl Into<String>) {
    MOCK_DATA_SOURCE.with(|ds| {
        ds.borrow_mut().network = Some(network.into());
    });
}

/// Set the mock data source context.
///
/// This value will be returned by `data_source::context()` in tests.
pub fn mock_data_source_context(context: EntityData) {
    MOCK_DATA_SOURCE.with(|ds| {
        ds.borrow_mut().context = Some(context);
    });
}

/// Clear all data source mocks.
pub fn clear_data_source_mocks() {
    MOCK_DATA_SOURCE.with(|ds| {
        *ds.borrow_mut() = MockDataSource::default();
    });
}

/// Get the mocked data source address (called internally by data_source::address).
pub fn get_mock_data_source_address() -> Address {
    MOCK_DATA_SOURCE.with(|ds| {
        ds.borrow().address.clone().unwrap_or_else(Address::zero)
    })
}

/// Get the mocked data source network (called internally by data_source::network).
pub fn get_mock_data_source_network() -> String {
    MOCK_DATA_SOURCE.with(|ds| {
        ds.borrow().network.clone().unwrap_or_else(|| String::from("mainnet"))
    })
}

/// Get the mocked data source context (called internally by data_source::context).
pub fn get_mock_data_source_context() -> EntityData {
    MOCK_DATA_SOURCE.with(|ds| {
        ds.borrow().context.clone().unwrap_or_default()
    })
}

// ============================================================================
// IPFS Mocking
// ============================================================================

/// Register mock IPFS content for a given CID.
///
/// This content will be returned by `ipfs::cat()` in tests.
///
/// # Example
///
/// ```ignore
/// mock_ipfs_cat("QmXyz...", b"{ \"name\": \"Token\" }");
/// let content = ipfs::cat("QmXyz...").unwrap();
/// ```
pub fn mock_ipfs_cat(cid: impl Into<String>, content: impl Into<Vec<u8>>) {
    MOCK_IPFS.with(|ipfs| {
        ipfs.borrow_mut().insert(cid.into(), content.into());
    });
}

/// Clear all mock IPFS content.
pub fn clear_ipfs_mocks() {
    MOCK_IPFS.with(|ipfs| {
        ipfs.borrow_mut().clear();
    });
}

/// Get mocked IPFS content (called internally by ipfs::cat).
pub fn get_mock_ipfs_content(cid: &str) -> Option<Vec<u8>> {
    MOCK_IPFS.with(|ipfs| {
        ipfs.borrow().get(cid).cloned()
    })
}

// ============================================================================
// Event Builder
// ============================================================================

/// Builder for constructing test events with sensible defaults.
pub struct EventBuilder<P> {
    address: Address,
    log_index: BigInt,
    transaction_log_index: BigInt,
    log_type: Option<String>,
    block_number: u64,
    block_timestamp: u64,
    transaction_hash: [u8; 32],
    transaction_from: Address,
    transaction_to: Option<Address>,
    params: Option<P>,
}

impl<P: Default> EventBuilder<P> {
    /// Create a new event builder with default values.
    pub fn new() -> Self {
        Self {
            address: Address::zero(),
            log_index: BigInt::zero(),
            transaction_log_index: BigInt::zero(),
            log_type: None,
            block_number: 1,
            block_timestamp: 1000000000,
            transaction_hash: [0u8; 32],
            transaction_from: Address::zero(),
            transaction_to: None,
            params: None,
        }
    }

    /// Set the contract address that emitted the event.
    pub fn address(mut self, addr: Address) -> Self {
        self.address = addr;
        self
    }

    /// Set the log index.
    pub fn log_index(mut self, index: u64) -> Self {
        self.log_index = BigInt::from_u64(index);
        self
    }

    /// Set the block number.
    pub fn block_number(mut self, num: u64) -> Self {
        self.block_number = num;
        self
    }

    /// Set the block timestamp.
    pub fn block_timestamp(mut self, ts: u64) -> Self {
        self.block_timestamp = ts;
        self
    }

    /// Set the transaction hash.
    pub fn transaction_hash(mut self, hash: [u8; 32]) -> Self {
        self.transaction_hash = hash;
        self
    }

    /// Set the transaction sender.
    pub fn transaction_from(mut self, from: Address) -> Self {
        self.transaction_from = from;
        self
    }

    /// Set the transaction recipient.
    pub fn transaction_to(mut self, to: Address) -> Self {
        self.transaction_to = Some(to);
        self
    }

    /// Set the event parameters.
    pub fn params(mut self, params: P) -> Self {
        self.params = Some(params);
        self
    }

    /// Build the event.
    pub fn build(self) -> Event<P> {
        Event {
            address: self.address,
            log_index: self.log_index,
            transaction_log_index: self.transaction_log_index,
            log_type: self.log_type,
            block: mock_block(self.block_number, self.block_timestamp),
            transaction: mock_transaction(
                self.transaction_hash,
                self.transaction_from,
                self.transaction_to,
            ),
            params: self.params.unwrap_or_default(),
            receipt: None,
        }
    }
}

impl<P: Default> Default for EventBuilder<P> {
    fn default() -> Self {
        Self::new()
    }
}

/// Create an event with default values and the given params.
pub fn create_event<P: Default>(params: P) -> Event<P> {
    EventBuilder::new().params(params).build()
}

// ============================================================================
// Mock Block/Transaction Helpers
// ============================================================================

/// Create a mock Ethereum block for testing.
pub fn mock_block(number: u64, timestamp: u64) -> Block {
    Block {
        hash: Bytes::from(alloc::vec![0u8; 32]),
        parent_hash: Bytes::from(alloc::vec![0u8; 32]),
        uncles_hash: Bytes::from(alloc::vec![0u8; 32]),
        author: Address::zero(),
        state_root: Bytes::from(alloc::vec![0u8; 32]),
        transactions_root: Bytes::from(alloc::vec![0u8; 32]),
        receipts_root: Bytes::from(alloc::vec![0u8; 32]),
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
        block_hash: Bytes::from(alloc::vec![0u8; 32]),
        block_number: BigInt::from_u64(block_number),
        cumulative_gas_used: BigInt::from_u64(21_000),
        gas_used: BigInt::from_u64(21_000),
        contract_address: None,
        status: BigInt::one(),
        root: Bytes::new(),
        logs_bloom: Bytes::from(alloc::vec![0u8; 256]),
    }
}

// ============================================================================
// Call Builder
// ============================================================================

/// Builder for constructing test contract calls with sensible defaults.
pub struct CallBuilder<I, O> {
    to: Address,
    from: Address,
    block_number: u64,
    block_timestamp: u64,
    transaction_hash: [u8; 32],
    inputs: Option<I>,
    outputs: Option<O>,
}

impl<I: Default, O: Default> CallBuilder<I, O> {
    /// Create a new call builder with default values.
    pub fn new() -> Self {
        Self {
            to: Address::zero(),
            from: Address::zero(),
            block_number: 1,
            block_timestamp: 1000000000,
            transaction_hash: [0u8; 32],
            inputs: None,
            outputs: None,
        }
    }

    /// Set the contract address being called.
    pub fn to(mut self, addr: Address) -> Self {
        self.to = addr;
        self
    }

    /// Set the caller address.
    pub fn from(mut self, addr: Address) -> Self {
        self.from = addr;
        self
    }

    /// Set the block number.
    pub fn block_number(mut self, num: u64) -> Self {
        self.block_number = num;
        self
    }

    /// Set the block timestamp.
    pub fn block_timestamp(mut self, ts: u64) -> Self {
        self.block_timestamp = ts;
        self
    }

    /// Set the transaction hash.
    pub fn transaction_hash(mut self, hash: [u8; 32]) -> Self {
        self.transaction_hash = hash;
        self
    }

    /// Set the call inputs.
    pub fn inputs(mut self, inputs: I) -> Self {
        self.inputs = Some(inputs);
        self
    }

    /// Set the call outputs.
    pub fn outputs(mut self, outputs: O) -> Self {
        self.outputs = Some(outputs);
        self
    }

    /// Build the call.
    pub fn build(self) -> Call<I, O> {
        Call {
            to: self.to.clone(),
            from: self.from.clone(),
            block: mock_block(self.block_number, self.block_timestamp),
            transaction: mock_transaction(self.transaction_hash, self.from, Some(self.to)),
            inputs: self.inputs.unwrap_or_default(),
            outputs: self.outputs.unwrap_or_default(),
        }
    }
}

impl<I: Default, O: Default> Default for CallBuilder<I, O> {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a call with default values and the given inputs/outputs.
pub fn create_call<I: Default, O: Default>(inputs: I, outputs: O) -> Call<I, O> {
    CallBuilder::new().inputs(inputs).outputs(outputs).build()
}

// ============================================================================
// Block Builder
// ============================================================================

/// Builder for constructing test blocks with sensible defaults.
pub struct BlockBuilder {
    hash: [u8; 32],
    parent_hash: [u8; 32],
    number: u64,
    timestamp: u64,
    author: Address,
    gas_used: u64,
    gas_limit: u64,
    difficulty: u64,
    total_difficulty: u64,
    size: Option<u64>,
    base_fee_per_gas: Option<u64>,
}

impl BlockBuilder {
    /// Create a new block builder with default values.
    pub fn new() -> Self {
        Self {
            hash: [0u8; 32],
            parent_hash: [0u8; 32],
            number: 1,
            timestamp: 1000000000,
            author: Address::zero(),
            gas_used: 0,
            gas_limit: 30_000_000,
            difficulty: 0,
            total_difficulty: 0,
            size: None,
            base_fee_per_gas: None,
        }
    }

    /// Set the block hash.
    pub fn hash(mut self, hash: [u8; 32]) -> Self {
        self.hash = hash;
        self
    }

    /// Set the parent block hash.
    pub fn parent_hash(mut self, hash: [u8; 32]) -> Self {
        self.parent_hash = hash;
        self
    }

    /// Set the block number.
    pub fn number(mut self, num: u64) -> Self {
        self.number = num;
        self
    }

    /// Set the block timestamp.
    pub fn timestamp(mut self, ts: u64) -> Self {
        self.timestamp = ts;
        self
    }

    /// Set the block author (miner/validator).
    pub fn author(mut self, addr: Address) -> Self {
        self.author = addr;
        self
    }

    /// Set the gas used in this block.
    pub fn gas_used(mut self, gas: u64) -> Self {
        self.gas_used = gas;
        self
    }

    /// Set the block gas limit.
    pub fn gas_limit(mut self, limit: u64) -> Self {
        self.gas_limit = limit;
        self
    }

    /// Set the block difficulty.
    pub fn difficulty(mut self, diff: u64) -> Self {
        self.difficulty = diff;
        self
    }

    /// Set the total difficulty.
    pub fn total_difficulty(mut self, diff: u64) -> Self {
        self.total_difficulty = diff;
        self
    }

    /// Set the block size.
    pub fn size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    /// Set the base fee per gas (EIP-1559).
    pub fn base_fee_per_gas(mut self, fee: u64) -> Self {
        self.base_fee_per_gas = Some(fee);
        self
    }

    /// Build the block.
    pub fn build(self) -> Block {
        Block {
            hash: Bytes::from(self.hash.as_slice()),
            parent_hash: Bytes::from(self.parent_hash.as_slice()),
            uncles_hash: Bytes::from(alloc::vec![0u8; 32]),
            author: self.author,
            state_root: Bytes::from(alloc::vec![0u8; 32]),
            transactions_root: Bytes::from(alloc::vec![0u8; 32]),
            receipts_root: Bytes::from(alloc::vec![0u8; 32]),
            number: BigInt::from_u64(self.number),
            gas_used: BigInt::from_u64(self.gas_used),
            gas_limit: BigInt::from_u64(self.gas_limit),
            timestamp: BigInt::from_u64(self.timestamp),
            difficulty: BigInt::from_u64(self.difficulty),
            total_difficulty: BigInt::from_u64(self.total_difficulty),
            size: self.size.map(BigInt::from_u64),
            base_fee_per_gas: self.base_fee_per_gas.map(BigInt::from_u64),
        }
    }
}

impl Default for BlockBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a block with default values.
pub fn create_block(number: u64, timestamp: u64) -> Block {
    BlockBuilder::new().number(number).timestamp(timestamp).build()
}

// ============================================================================
// TestableEntity Trait
// ============================================================================

/// Trait for entities that can be inspected and constructed in tests.
///
/// This is automatically implemented by yogurt-codegen for all entities
/// when building for native targets.
pub trait TestableEntity: Entity {
    /// Get a reference to the underlying entity data.
    fn as_data(&self) -> &EntityData;

    /// Construct an entity from raw data.
    fn from_data(data: EntityData) -> Self;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Value;

    #[test]
    fn test_store_get_in_block() {
        clear_store();

        let mut data = EntityData::new();
        data.set("id", Value::String("test-1".into()));

        // Initially, nothing in current block
        assert!(store_get_in_block("TestEntity", "test-1").is_none());

        // Set the entity
        store_set("TestEntity", "test-1", &data);

        // Now it should be available via get_in_block
        assert!(store_get_in_block("TestEntity", "test-1").is_some());

        // And also via regular get
        assert!(store_get("TestEntity", "test-1").is_some());
    }

    #[test]
    fn test_store_get_in_block_after_start_block() {
        clear_store();

        let mut data = EntityData::new();
        data.set("id", Value::String("test-1".into()));

        // Set entity in "block 1"
        store_set("TestEntity", "test-1", &data);
        assert!(store_get_in_block("TestEntity", "test-1").is_some());

        // Start "block 2"
        start_block();

        // Entity should NOT be available via get_in_block (wasn't modified this block)
        assert!(store_get_in_block("TestEntity", "test-1").is_none());

        // But should still be available via regular get
        assert!(store_get("TestEntity", "test-1").is_some());
    }

    #[test]
    fn test_store_get_in_block_modified_in_new_block() {
        clear_store();

        let mut data1 = EntityData::new();
        data1.set("id", Value::String("test-1".into()));
        data1.set("value", Value::Int(100));

        // Set entity in "block 1"
        store_set("TestEntity", "test-1", &data1);

        // Start "block 2"
        start_block();
        assert!(store_get_in_block("TestEntity", "test-1").is_none());

        // Modify entity in "block 2"
        let mut data2 = EntityData::new();
        data2.set("id", Value::String("test-1".into()));
        data2.set("value", Value::Int(200));
        store_set("TestEntity", "test-1", &data2);

        // Now it should be available via get_in_block again
        let loaded = store_get_in_block("TestEntity", "test-1").unwrap();
        assert_eq!(loaded.get_int_opt("value"), Some(200));
    }
}
