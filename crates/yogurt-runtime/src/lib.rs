//! yogurt-runtime: The Graph subgraph runtime for Rust
//!
//! This crate provides the Rust equivalent of `@graphprotocol/graph-ts`,
//! enabling developers to write subgraph mapping handlers in idiomatic Rust
//! while producing WASM binaries compatible with graph-node's AssemblyScript runtime.

#![cfg_attr(all(target_arch = "wasm32", not(feature = "std")), no_std)]

extern crate alloc;

pub mod allocator;
pub mod asc;
mod host;
pub mod types;
// mod type_ids;  // TypeId exports - disabled, need to add via WASM post-processing

pub mod crypto;
pub mod data_source;
pub mod ens;
pub mod ethereum;
pub mod ipfs;
pub mod json;
pub mod log;
pub mod store;

// Testing module is always available on native builds
#[cfg(not(target_arch = "wasm32"))]
pub mod testing;

pub use types::*;

/// Format a BigInt as a decimal string with the given number of decimal places.
///
/// This is a convenience function equivalent to `amount.to_decimals(decimals)`.
///
/// # Example
///
/// ```ignore
/// let wei = BigInt::from_string("1500000000000000000").unwrap();
/// let eth = format_units(&wei, 18);  // "1.5"
/// ```
pub fn format_units(amount: &types::BigInt, decimals: u8) -> alloc::string::String {
    amount.to_decimals(decimals)
}

/// Generate an entity ID from an event's transaction hash and log index.
///
/// This is the most common pattern for entity IDs in subgraphs:
/// `{transaction_hash}-{log_index}`
///
/// # Example
///
/// ```ignore
/// use yogurt_runtime::log_id;
///
/// #[handler]
/// fn handle_transfer(event: TransferEvent) {
///     let id = log_id!(event);  // "0xabc...-42"
///     let transfer = Transfer::new(id);
/// }
/// ```
#[macro_export]
macro_rules! log_id {
    ($event:expr) => {
        alloc::format!(
            "{}-{}",
            $event.transaction.hash.to_hex(),
            $event.log_index.to_string()
        )
    };
}

/// Generate an entity ID from a call's transaction hash.
///
/// For call handlers, we typically use just the transaction hash as the ID
/// since there's no log index.
///
/// # Example
///
/// ```ignore
/// use yogurt_runtime::call_id;
///
/// #[handler]
/// fn handle_transfer_call(call: TransferCall) {
///     let id = call_id!(call);  // "0xabc..."
///     let transfer = Transfer::new(id);
/// }
/// ```
#[macro_export]
macro_rules! call_id {
    ($call:expr) => {
        $call.transaction.hash.to_hex()
    };
}

/// Generate an entity ID from a block's number.
///
/// For block handlers, we typically use the block number as the ID.
/// This creates a predictable, unique ID for each block.
///
/// # Example
///
/// ```ignore
/// use yogurt_runtime::block_id;
///
/// #[handler]
/// fn handle_block(block: Block) {
///     let id = block_id!(block);  // "12345678"
///     let block_entity = BlockEntity::new(id);
/// }
/// ```
#[macro_export]
macro_rules! block_id {
    ($block:expr) => {
        $block.number.to_string()
    };
}

/// Generate a day-based entity ID from an event's block timestamp.
///
/// Useful for creating daily aggregation entities (e.g., DailyVolume, DailyStats).
/// Returns the number of days since Unix epoch.
///
/// # Example
///
/// ```ignore
/// use yogurt_runtime::day_id;
///
/// #[handler]
/// fn handle_swap(event: SwapEvent) {
///     let daily_id = day_id!(event);  // "19724" (days since epoch)
///     let mut daily = DailyVolume::load_or_create(&daily_id, |d| {
///         d.set_volume(BigDecimal::zero());
///     });
/// }
/// ```
#[macro_export]
macro_rules! day_id {
    ($event:expr) => {{
        // Seconds per day = 86400
        let timestamp = &$event.block.timestamp;
        let days = timestamp.divided_by(&$crate::types::BigInt::from_i32(86400));
        days.to_string()
    }};
}

/// Generate an hour-based entity ID from an event's block timestamp.
///
/// Useful for creating hourly aggregation entities (e.g., HourlyVolume, HourlyStats).
/// Returns the number of hours since Unix epoch.
///
/// # Example
///
/// ```ignore
/// use yogurt_runtime::hour_id;
///
/// #[handler]
/// fn handle_swap(event: SwapEvent) {
///     let hourly_id = hour_id!(event);  // "473376" (hours since epoch)
///     let mut hourly = HourlyVolume::load_or_create(&hourly_id, |h| {
///         h.set_volume(BigDecimal::zero());
///     });
/// }
/// ```
#[macro_export]
macro_rules! hour_id {
    ($event:expr) => {{
        // Seconds per hour = 3600
        let timestamp = &$event.block.timestamp;
        let hours = timestamp.divided_by(&$crate::types::BigInt::from_i32(3600));
        hours.to_string()
    }};
}

/// The standard prelude for yogurt subgraph mappings.
///
/// ```rust,ignore
/// use yogurt_runtime::prelude::*;
/// ```
pub mod prelude {
    pub use crate::asc::FromAscPtr;
    pub use crate::ethereum::{Block, Call, Event, Transaction, TransactionReceipt};
    pub use crate::types::{Address, BigDecimal, BigInt, Bytes, Entity, Value};
    pub use crate::{data_source, log};

    // Re-export ID generation macros
    pub use crate::{block_id, call_id, day_id, hour_id, log_id};

    // Re-export token formatting utilities
    pub use crate::format_units;
    pub use crate::types::parse_units;

    // Re-export the handler macro when available
    #[cfg(feature = "yogurt-macros")]
    pub use yogurt_macros::handler;
}

// WASM-specific panic handler and exports
#[cfg(all(target_arch = "wasm32", not(feature = "std")))]
mod wasm {
    use core::panic::PanicInfo;

    #[panic_handler]
    fn panic(_info: &PanicInfo) -> ! {
        core::arch::wasm32::unreachable()
    }

    // AssemblyScript runtime exports required by graph-node
    #[unsafe(no_mangle)]
    pub extern "C" fn abort(_msg: u32, _file: u32, _line: u32, _col: u32) -> ! {
        core::arch::wasm32::unreachable()
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn __pin(ptr: u32) -> u32 {
        ptr // No-op: we use bump allocation, no GC
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn __unpin(_ptr: u32) {
        // No-op: we use bump allocation, no GC
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn __collect() {
        // No-op: we use bump allocation, no GC
    }

    /// Runtime type information base pointer.
    ///
    /// AssemblyScript exports this for runtime type reflection.
    /// We don't use RTTI, so this returns a null pointer.
    #[unsafe(no_mangle)]
    pub static __rtti_base: u32 = 0;

    /// The `allocate` export required by graph-node.
    /// This is the older allocation API that some versions of graph-node use.
    #[unsafe(no_mangle)]
    pub extern "C" fn allocate(size: i32) -> i32 {
        crate::allocator::asc_alloc(size as u32, 0) as i32
    }

    /// The `id_of_type` export required by graph-node.
    /// Takes a pointer to a type name string and returns the class ID.
    /// This is used by graph-node for runtime type identification.
    #[unsafe(no_mangle)]
    pub extern "C" fn id_of_type(type_name_ptr: i32) -> i32 {
        // Guard against null pointer - graph-node may probe with 0
        if type_name_ptr == 0 {
            return 0;
        }

        // Read the type name string from the pointer
        let type_name = crate::asc::asc_to_string(crate::asc::AscPtr::new(type_name_ptr as u32));

        // Match against known type names and return the class ID
        match type_name.as_str() {
            "ArrayBuffer" => crate::allocator::class_id::ARRAY_BUFFER as i32,
            "String" => crate::allocator::class_id::STRING as i32,
            s if s.starts_with("TypedMap<") => crate::allocator::class_id::TYPED_MAP as i32,
            s if s.starts_with("TypedMapEntry<") => crate::allocator::class_id::TYPED_MAP_ENTRY as i32,
            s if s.starts_with("~lib/array/Array<") && s.contains("store~Value") => {
                crate::allocator::class_id::ARRAY_STORE_VALUE as i32
            }
            s if s.starts_with("~lib/array/Array<") => crate::allocator::class_id::ARRAY_PTR as i32,
            s if s.contains("store~Value") || s.contains("StoreValue") => {
                crate::allocator::class_id::STORE_VALUE as i32
            }
            s if s.contains("ethereum~Value") || s.contains("EthereumValue") => {
                crate::allocator::class_id::ETHEREUM_VALUE as i32
            }
            _ => {
                // Unknown type - return 0 (OBJECT) as fallback
                0
            }
        }
    }

    /// The `_start` export - WASM module entry point.
    /// Graph-node calls this to initialize the module.
    #[unsafe(no_mangle)]
    pub extern "C" fn _start() {
        // No initialization needed
    }
}
