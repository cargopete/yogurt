//! Ethereum types and contract call functionality.

use alloc::string::String;
use alloc::vec::Vec;

use crate::asc::{asc_to_bytes, asc_to_string, AscPtr, FromAscPtr};
use crate::types::{Address, BigInt, Bytes};

// ============================================================================
// Memory Layout Constants
// ============================================================================

// Field offsets for EthereumBlock (each field is a 4-byte AscPtr)
mod block_offsets {
    pub const HASH: usize = 0;
    pub const PARENT_HASH: usize = 4;
    pub const UNCLES_HASH: usize = 8;
    pub const AUTHOR: usize = 12;
    pub const STATE_ROOT: usize = 16;
    pub const TRANSACTIONS_ROOT: usize = 20;
    pub const RECEIPTS_ROOT: usize = 24;
    pub const NUMBER: usize = 28;
    pub const GAS_USED: usize = 32;
    pub const GAS_LIMIT: usize = 36;
    pub const TIMESTAMP: usize = 40;
    pub const DIFFICULTY: usize = 44;
    pub const TOTAL_DIFFICULTY: usize = 48;
    pub const SIZE: usize = 52;
    pub const BASE_FEE_PER_GAS: usize = 56;
}

// Field offsets for EthereumTransaction
mod tx_offsets {
    pub const HASH: usize = 0;
    pub const INDEX: usize = 4;
    pub const FROM: usize = 8;
    pub const TO: usize = 12;
    pub const VALUE: usize = 16;
    pub const GAS_LIMIT: usize = 20;
    pub const GAS_PRICE: usize = 24;
    pub const INPUT: usize = 28;
    pub const NONCE: usize = 32;
}

// Field offsets for EthereumEvent
mod event_offsets {
    pub const ADDRESS: usize = 0;
    pub const LOG_INDEX: usize = 4;
    pub const TRANSACTION_LOG_INDEX: usize = 8;
    pub const LOG_TYPE: usize = 12;
    pub const BLOCK: usize = 16;
    pub const TRANSACTION: usize = 20;
    pub const PARAMS: usize = 24;
    pub const RECEIPT: usize = 28;
}

// Field offsets for TransactionReceipt
mod receipt_offsets {
    pub const TRANSACTION_HASH: usize = 0;
    pub const TRANSACTION_INDEX: usize = 4;
    pub const BLOCK_HASH: usize = 8;
    pub const BLOCK_NUMBER: usize = 12;
    pub const CUMULATIVE_GAS_USED: usize = 16;
    pub const GAS_USED: usize = 20;
    pub const CONTRACT_ADDRESS: usize = 24;
    pub const STATUS: usize = 28;
    pub const ROOT: usize = 32;
    pub const LOGS_BLOOM: usize = 36;
}

// ============================================================================
// Ethereum Types
// ============================================================================

/// An Ethereum block.
#[derive(Clone, Debug)]
pub struct Block {
    pub hash: Bytes,
    pub parent_hash: Bytes,
    pub uncles_hash: Bytes,
    pub author: Address,
    pub state_root: Bytes,
    pub transactions_root: Bytes,
    pub receipts_root: Bytes,
    pub number: BigInt,
    pub gas_used: BigInt,
    pub gas_limit: BigInt,
    pub timestamp: BigInt,
    pub difficulty: BigInt,
    pub total_difficulty: BigInt,
    pub size: Option<BigInt>,
    pub base_fee_per_gas: Option<BigInt>,
}

/// An Ethereum transaction.
#[derive(Clone, Debug)]
pub struct Transaction {
    pub hash: Bytes,
    pub index: BigInt,
    pub from: Address,
    pub to: Option<Address>,
    pub value: BigInt,
    pub gas_limit: BigInt,
    pub gas_price: BigInt,
    pub input: Bytes,
    pub nonce: BigInt,
}

/// An Ethereum transaction receipt.
#[derive(Clone, Debug)]
pub struct TransactionReceipt {
    pub transaction_hash: Bytes,
    pub transaction_index: BigInt,
    pub block_hash: Bytes,
    pub block_number: BigInt,
    pub cumulative_gas_used: BigInt,
    pub gas_used: BigInt,
    pub contract_address: Option<Address>,
    pub status: BigInt,
    pub root: Bytes,
    pub logs_bloom: Bytes,
}

/// An Ethereum event (log) with typed parameters.
#[derive(Clone, Debug)]
pub struct Event<P> {
    pub address: Address,
    pub log_index: BigInt,
    pub transaction_log_index: BigInt,
    pub log_type: Option<String>,
    pub block: Block,
    pub transaction: Transaction,
    pub params: P,
    pub receipt: Option<TransactionReceipt>,
}

/// An Ethereum smart contract call.
#[derive(Clone, Debug)]
pub struct SmartContractCall {
    pub contract_name: String,
    pub contract_address: Address,
    pub function_name: String,
    pub function_signature: String,
    pub function_params: Vec<Token>,
}

/// ABI token types for encoding/decoding function calls.
#[derive(Clone, Debug)]
pub enum Token {
    Address(Address),
    FixedBytes(Vec<u8>),
    Bytes(Bytes),
    Int(BigInt),
    Uint(BigInt),
    Bool(bool),
    String(String),
    Array(Vec<Token>),
    FixedArray(Vec<Token>),
    Tuple(Vec<Token>),
}

// ============================================================================
// FromAscPtr Implementations
// ============================================================================

#[cfg(target_arch = "wasm32")]
impl FromAscPtr for Address {
    fn from_asc_ptr(ptr: u32) -> Self {
        if ptr == 0 {
            return Address::zero();
        }
        let bytes = asc_to_bytes(AscPtr::new(ptr));
        Address::from(bytes.as_slice())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl FromAscPtr for Address {
    fn from_asc_ptr(_ptr: u32) -> Self {
        Address::zero()
    }
}

#[cfg(target_arch = "wasm32")]
impl FromAscPtr for Bytes {
    fn from_asc_ptr(ptr: u32) -> Self {
        if ptr == 0 {
            return Bytes::new();
        }
        Bytes::from_vec(asc_to_bytes(AscPtr::new(ptr)))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl FromAscPtr for Bytes {
    fn from_asc_ptr(_ptr: u32) -> Self {
        Bytes::new()
    }
}

#[cfg(target_arch = "wasm32")]
impl FromAscPtr for BigInt {
    fn from_asc_ptr(ptr: u32) -> Self {
        BigInt::from_ptr(AscPtr::new(ptr))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl FromAscPtr for BigInt {
    fn from_asc_ptr(_ptr: u32) -> Self {
        BigInt::zero()
    }
}

#[cfg(target_arch = "wasm32")]
impl FromAscPtr for Block {
    fn from_asc_ptr(ptr: u32) -> Self {
        use crate::asc::read_u32_at;

        if ptr == 0 {
            return Block::default();
        }

        unsafe {
            Block {
                hash: Bytes::from_asc_ptr(read_u32_at(ptr, block_offsets::HASH)),
                parent_hash: Bytes::from_asc_ptr(read_u32_at(ptr, block_offsets::PARENT_HASH)),
                uncles_hash: Bytes::from_asc_ptr(read_u32_at(ptr, block_offsets::UNCLES_HASH)),
                author: Address::from_asc_ptr(read_u32_at(ptr, block_offsets::AUTHOR)),
                state_root: Bytes::from_asc_ptr(read_u32_at(ptr, block_offsets::STATE_ROOT)),
                transactions_root: Bytes::from_asc_ptr(read_u32_at(ptr, block_offsets::TRANSACTIONS_ROOT)),
                receipts_root: Bytes::from_asc_ptr(read_u32_at(ptr, block_offsets::RECEIPTS_ROOT)),
                number: BigInt::from_asc_ptr(read_u32_at(ptr, block_offsets::NUMBER)),
                gas_used: BigInt::from_asc_ptr(read_u32_at(ptr, block_offsets::GAS_USED)),
                gas_limit: BigInt::from_asc_ptr(read_u32_at(ptr, block_offsets::GAS_LIMIT)),
                timestamp: BigInt::from_asc_ptr(read_u32_at(ptr, block_offsets::TIMESTAMP)),
                difficulty: BigInt::from_asc_ptr(read_u32_at(ptr, block_offsets::DIFFICULTY)),
                total_difficulty: BigInt::from_asc_ptr(read_u32_at(ptr, block_offsets::TOTAL_DIFFICULTY)),
                size: {
                    let p = read_u32_at(ptr, block_offsets::SIZE);
                    if p == 0 { None } else { Some(BigInt::from_asc_ptr(p)) }
                },
                base_fee_per_gas: {
                    let p = read_u32_at(ptr, block_offsets::BASE_FEE_PER_GAS);
                    if p == 0 { None } else { Some(BigInt::from_asc_ptr(p)) }
                },
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl FromAscPtr for Block {
    fn from_asc_ptr(_ptr: u32) -> Self {
        Block::default()
    }
}

impl Default for Block {
    fn default() -> Self {
        Block {
            hash: Bytes::new(),
            parent_hash: Bytes::new(),
            uncles_hash: Bytes::new(),
            author: Address::zero(),
            state_root: Bytes::new(),
            transactions_root: Bytes::new(),
            receipts_root: Bytes::new(),
            number: BigInt::zero(),
            gas_used: BigInt::zero(),
            gas_limit: BigInt::zero(),
            timestamp: BigInt::zero(),
            difficulty: BigInt::zero(),
            total_difficulty: BigInt::zero(),
            size: None,
            base_fee_per_gas: None,
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl FromAscPtr for Transaction {
    fn from_asc_ptr(ptr: u32) -> Self {
        use crate::asc::read_u32_at;

        if ptr == 0 {
            return Transaction::default();
        }

        unsafe {
            Transaction {
                hash: Bytes::from_asc_ptr(read_u32_at(ptr, tx_offsets::HASH)),
                index: BigInt::from_asc_ptr(read_u32_at(ptr, tx_offsets::INDEX)),
                from: Address::from_asc_ptr(read_u32_at(ptr, tx_offsets::FROM)),
                to: {
                    let p = read_u32_at(ptr, tx_offsets::TO);
                    if p == 0 { None } else { Some(Address::from_asc_ptr(p)) }
                },
                value: BigInt::from_asc_ptr(read_u32_at(ptr, tx_offsets::VALUE)),
                gas_limit: BigInt::from_asc_ptr(read_u32_at(ptr, tx_offsets::GAS_LIMIT)),
                gas_price: BigInt::from_asc_ptr(read_u32_at(ptr, tx_offsets::GAS_PRICE)),
                input: Bytes::from_asc_ptr(read_u32_at(ptr, tx_offsets::INPUT)),
                nonce: BigInt::from_asc_ptr(read_u32_at(ptr, tx_offsets::NONCE)),
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl FromAscPtr for Transaction {
    fn from_asc_ptr(_ptr: u32) -> Self {
        Transaction::default()
    }
}

impl Default for Transaction {
    fn default() -> Self {
        Transaction {
            hash: Bytes::new(),
            index: BigInt::zero(),
            from: Address::zero(),
            to: None,
            value: BigInt::zero(),
            gas_limit: BigInt::zero(),
            gas_price: BigInt::zero(),
            input: Bytes::new(),
            nonce: BigInt::zero(),
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl FromAscPtr for TransactionReceipt {
    fn from_asc_ptr(ptr: u32) -> Self {
        use crate::asc::read_u32_at;

        if ptr == 0 {
            return TransactionReceipt::default();
        }

        unsafe {
            TransactionReceipt {
                transaction_hash: Bytes::from_asc_ptr(read_u32_at(ptr, receipt_offsets::TRANSACTION_HASH)),
                transaction_index: BigInt::from_asc_ptr(read_u32_at(ptr, receipt_offsets::TRANSACTION_INDEX)),
                block_hash: Bytes::from_asc_ptr(read_u32_at(ptr, receipt_offsets::BLOCK_HASH)),
                block_number: BigInt::from_asc_ptr(read_u32_at(ptr, receipt_offsets::BLOCK_NUMBER)),
                cumulative_gas_used: BigInt::from_asc_ptr(read_u32_at(ptr, receipt_offsets::CUMULATIVE_GAS_USED)),
                gas_used: BigInt::from_asc_ptr(read_u32_at(ptr, receipt_offsets::GAS_USED)),
                contract_address: {
                    let p = read_u32_at(ptr, receipt_offsets::CONTRACT_ADDRESS);
                    if p == 0 { None } else { Some(Address::from_asc_ptr(p)) }
                },
                status: BigInt::from_asc_ptr(read_u32_at(ptr, receipt_offsets::STATUS)),
                root: Bytes::from_asc_ptr(read_u32_at(ptr, receipt_offsets::ROOT)),
                logs_bloom: Bytes::from_asc_ptr(read_u32_at(ptr, receipt_offsets::LOGS_BLOOM)),
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl FromAscPtr for TransactionReceipt {
    fn from_asc_ptr(_ptr: u32) -> Self {
        TransactionReceipt::default()
    }
}

impl Default for TransactionReceipt {
    fn default() -> Self {
        TransactionReceipt {
            transaction_hash: Bytes::new(),
            transaction_index: BigInt::zero(),
            block_hash: Bytes::new(),
            block_number: BigInt::zero(),
            cumulative_gas_used: BigInt::zero(),
            gas_used: BigInt::zero(),
            contract_address: None,
            status: BigInt::zero(),
            root: Bytes::new(),
            logs_bloom: Bytes::new(),
        }
    }
}

/// Deserialize an Event from an AssemblyScript pointer.
///
/// The params type `P` must implement `FromAscPtr` — this is typically
/// generated by `yogurt codegen` based on the ABI.
#[cfg(target_arch = "wasm32")]
impl<P: FromAscPtr> FromAscPtr for Event<P> {
    fn from_asc_ptr(ptr: u32) -> Self {
        use crate::asc::read_u32_at;

        if ptr == 0 {
            panic!("Cannot deserialize Event from null pointer");
        }

        unsafe {
            Event {
                address: Address::from_asc_ptr(read_u32_at(ptr, event_offsets::ADDRESS)),
                log_index: BigInt::from_asc_ptr(read_u32_at(ptr, event_offsets::LOG_INDEX)),
                transaction_log_index: BigInt::from_asc_ptr(read_u32_at(ptr, event_offsets::TRANSACTION_LOG_INDEX)),
                log_type: {
                    let p = read_u32_at(ptr, event_offsets::LOG_TYPE);
                    if p == 0 { None } else { Some(asc_to_string(AscPtr::new(p))) }
                },
                block: Block::from_asc_ptr(read_u32_at(ptr, event_offsets::BLOCK)),
                transaction: Transaction::from_asc_ptr(read_u32_at(ptr, event_offsets::TRANSACTION)),
                params: P::from_asc_ptr(read_u32_at(ptr, event_offsets::PARAMS)),
                receipt: {
                    let p = read_u32_at(ptr, event_offsets::RECEIPT);
                    if p == 0 { None } else { Some(TransactionReceipt::from_asc_ptr(p)) }
                },
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<P: FromAscPtr + Default> FromAscPtr for Event<P> {
    fn from_asc_ptr(_ptr: u32) -> Self {
        panic!("Event deserialization not available on native target")
    }
}

// ============================================================================
// Contract Call Functions
// ============================================================================

/// Execute an Ethereum contract call.
///
/// Returns `None` if the call reverts.
#[cfg(target_arch = "wasm32")]
pub fn call(_call: SmartContractCall) -> Option<Vec<Token>> {
    // TODO: Implement contract call serialization and host function invocation
    None
}

#[cfg(not(target_arch = "wasm32"))]
pub fn call(_call: SmartContractCall) -> Option<Vec<Token>> {
    None
}

/// Encode parameters for a contract call.
#[cfg(target_arch = "wasm32")]
pub fn encode(_params: &[Token]) -> Bytes {
    // TODO: Implement ABI encoding via host function
    Bytes::new()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn encode(_params: &[Token]) -> Bytes {
    Bytes::new()
}

/// Decode return data from a contract call.
#[cfg(target_arch = "wasm32")]
pub fn decode(_types: &str, _data: &Bytes) -> Option<Vec<Token>> {
    // TODO: Implement ABI decoding via host function
    None
}

#[cfg(not(target_arch = "wasm32"))]
pub fn decode(_types: &str, _data: &Bytes) -> Option<Vec<Token>> {
    None
}
