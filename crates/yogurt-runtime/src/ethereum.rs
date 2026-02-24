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
pub fn call(call_data: SmartContractCall) -> Option<Vec<Token>> {
    use crate::asc::{str_to_asc, bytes_to_asc, AscArrayHeader};
    use crate::allocator::{asc_alloc, class_id};

    // SmartContractCall layout in AS memory:
    // - contractName: AscPtr<String>     (offset 0)
    // - contractAddress: AscPtr<Bytes>   (offset 4)
    // - functionName: AscPtr<String>     (offset 8)
    // - functionSignature: AscPtr<String> (offset 12)
    // - functionParams: AscPtr<Array<EthereumValue>> (offset 16)

    // Serialize each field
    let contract_name_ptr = str_to_asc(&call_data.contract_name);
    let contract_address_ptr = bytes_to_asc(call_data.contract_address.as_bytes());
    let function_name_ptr = str_to_asc(&call_data.function_name);
    let function_signature_ptr = str_to_asc(&call_data.function_signature);
    let params_array_ptr = serialize_token_array(&call_data.function_params);

    // Allocate the SmartContractCall struct (5 * 4 = 20 bytes)
    let call_ptr = asc_alloc(20, class_id::SMART_CONTRACT_CALL);

    unsafe {
        let base = call_ptr as *mut u32;
        core::ptr::write_unaligned(base, contract_name_ptr.as_raw());
        core::ptr::write_unaligned(base.add(1), contract_address_ptr.as_raw());
        core::ptr::write_unaligned(base.add(2), function_name_ptr.as_raw());
        core::ptr::write_unaligned(base.add(3), function_signature_ptr.as_raw());
        core::ptr::write_unaligned(base.add(4), params_array_ptr);
    }

    // Call the host function
    let result_ptr = unsafe { crate::host::ethereum_call(call_ptr as i32) };

    if result_ptr == 0 {
        return None;
    }

    // Deserialize the result array
    Some(deserialize_token_array(result_ptr as u32))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn call(_call: SmartContractCall) -> Option<Vec<Token>> {
    None
}

/// Serialize an array of Tokens to AS memory.
#[cfg(target_arch = "wasm32")]
fn serialize_token_array(tokens: &[Token]) -> u32 {
    use crate::asc::{str_to_asc, bytes_to_asc, AscArrayHeader};
    use crate::allocator::{asc_alloc, class_id};

    let count = tokens.len();

    // Allocate buffer for token pointers
    let buffer_size = (count * 4) as u32;
    let buffer_ptr = asc_alloc(buffer_size, class_id::ARRAY_BUFFER);

    // Serialize each token
    for (i, token) in tokens.iter().enumerate() {
        let token_ptr = serialize_token(token);
        unsafe {
            let dest = (buffer_ptr as *mut u32).add(i);
            core::ptr::write_unaligned(dest, token_ptr);
        }
    }

    // Allocate Array struct
    let array_ptr = asc_alloc(
        core::mem::size_of::<AscArrayHeader>() as u32,
        class_id::ARRAY_ETHEREUM_VALUE,
    );

    unsafe {
        let header = array_ptr as *mut AscArrayHeader;
        (*header).buffer = buffer_ptr;
        (*header).buffer_data_start = 0;
        (*header).buffer_data_length = buffer_size;
        (*header).length = count as i32;
    }

    array_ptr
}

/// Serialize a single Token to AS memory.
/// Returns a pointer to an EthereumValue enum.
#[cfg(target_arch = "wasm32")]
fn serialize_token(token: &Token) -> u32 {
    use crate::asc::{str_to_asc, bytes_to_asc, AscEnumHeader};
    use crate::allocator::{asc_alloc, class_id};

    // EthereumValue enum layout (same as StoreValue):
    // - kind: i32
    // - _padding: u32
    // - payload: u64

    // EthereumValue kinds (from graph-ts):
    // ADDRESS = 0, FIXED_BYTES = 1, BYTES = 2, INT = 3, UINT = 4,
    // BOOL = 5, STRING = 6, FIXED_ARRAY = 7, ARRAY = 8, TUPLE = 9

    let (kind, payload): (i32, u64) = match token {
        Token::Address(addr) => {
            let ptr = bytes_to_asc(addr.as_bytes());
            (0, ptr.as_raw() as u64)
        }
        Token::FixedBytes(bytes) => {
            let ptr = bytes_to_asc(bytes);
            (1, ptr.as_raw() as u64)
        }
        Token::Bytes(bytes) => {
            let ptr = bytes_to_asc(bytes.as_slice());
            (2, ptr.as_raw() as u64)
        }
        Token::Int(bigint) => {
            (3, bigint.as_ptr().as_raw() as u64)
        }
        Token::Uint(bigint) => {
            (4, bigint.as_ptr().as_raw() as u64)
        }
        Token::Bool(b) => {
            (5, if *b { 1 } else { 0 })
        }
        Token::String(s) => {
            let ptr = str_to_asc(s);
            (6, ptr.as_raw() as u64)
        }
        Token::FixedArray(arr) => {
            let ptr = serialize_token_array(arr);
            (7, ptr as u64)
        }
        Token::Array(arr) => {
            let ptr = serialize_token_array(arr);
            (8, ptr as u64)
        }
        Token::Tuple(arr) => {
            let ptr = serialize_token_array(arr);
            (9, ptr as u64)
        }
    };

    let enum_ptr = asc_alloc(
        core::mem::size_of::<AscEnumHeader>() as u32,
        class_id::ETHEREUM_VALUE,
    );

    unsafe {
        let header = enum_ptr as *mut AscEnumHeader;
        (*header).kind = kind;
        (*header)._padding = 0;
        (*header).payload = payload;
    }

    enum_ptr
}

/// Deserialize an array of Tokens from AS memory.
#[cfg(target_arch = "wasm32")]
fn deserialize_token_array(ptr: u32) -> Vec<Token> {
    use crate::asc::AscArrayHeader;

    if ptr == 0 {
        return Vec::new();
    }

    unsafe {
        let header = ptr as *const AscArrayHeader;
        let buffer_ptr = (*header).buffer;
        let length = (*header).length;

        if buffer_ptr == 0 || length <= 0 {
            return Vec::new();
        }

        let mut tokens = Vec::with_capacity(length as usize);

        for i in 0..length as usize {
            let token_ptr_addr = (buffer_ptr as *const u32).add(i);
            let token_ptr = core::ptr::read_unaligned(token_ptr_addr);
            tokens.push(deserialize_token(token_ptr));
        }

        tokens
    }
}

/// Deserialize a single Token from AS memory.
#[cfg(target_arch = "wasm32")]
fn deserialize_token(ptr: u32) -> Token {
    use crate::asc::{asc_to_bytes, asc_to_string, AscEnumHeader, AscPtr};

    if ptr == 0 {
        return Token::Bool(false); // Default fallback
    }

    unsafe {
        let header = ptr as *const AscEnumHeader;
        let kind = (*header).kind;
        let payload = (*header).payload;

        match kind {
            0 => {
                // ADDRESS
                let bytes = asc_to_bytes(AscPtr::new(payload as u32));
                Token::Address(Address::from(bytes.as_slice()))
            }
            1 => {
                // FIXED_BYTES
                let bytes = asc_to_bytes(AscPtr::new(payload as u32));
                Token::FixedBytes(bytes)
            }
            2 => {
                // BYTES
                let bytes = asc_to_bytes(AscPtr::new(payload as u32));
                Token::Bytes(Bytes::from_vec(bytes))
            }
            3 => {
                // INT
                Token::Int(BigInt::from_ptr(AscPtr::new(payload as u32)))
            }
            4 => {
                // UINT
                Token::Uint(BigInt::from_ptr(AscPtr::new(payload as u32)))
            }
            5 => {
                // BOOL
                Token::Bool(payload != 0)
            }
            6 => {
                // STRING
                let s = asc_to_string(AscPtr::new(payload as u32));
                Token::String(s)
            }
            7 => {
                // FIXED_ARRAY
                let arr = deserialize_token_array(payload as u32);
                Token::FixedArray(arr)
            }
            8 => {
                // ARRAY
                let arr = deserialize_token_array(payload as u32);
                Token::Array(arr)
            }
            9 => {
                // TUPLE
                let arr = deserialize_token_array(payload as u32);
                Token::Tuple(arr)
            }
            _ => Token::Bool(false), // Unknown type
        }
    }
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
