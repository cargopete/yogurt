//! Ethereum types and contract call functionality.

use alloc::string::String;
use alloc::vec::Vec;

use crate::types::{Address, BigInt, Bytes};

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
