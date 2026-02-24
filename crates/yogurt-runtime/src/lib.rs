//! yogurt-runtime: The Graph subgraph runtime for Rust
//!
//! This crate provides the Rust equivalent of `@graphprotocol/graph-ts`,
//! enabling developers to write subgraph mapping handlers in idiomatic Rust
//! while producing WASM binaries compatible with graph-node's AssemblyScript runtime.

#![cfg_attr(all(target_arch = "wasm32", not(feature = "std")), no_std)]

extern crate alloc;

mod allocator;
mod asc;
mod host;
mod types;

pub mod crypto;
pub mod data_source;
pub mod ethereum;
pub mod ipfs;
pub mod json;
pub mod log;
pub mod store;

#[cfg(feature = "testing")]
pub mod testing;

pub use types::*;

/// The standard prelude for yogurt subgraph mappings.
///
/// ```rust,ignore
/// use yogurt_runtime::prelude::*;
/// ```
pub mod prelude {
    pub use crate::ethereum::{Block, Event, Transaction, TransactionReceipt};
    pub use crate::types::{Address, BigDecimal, BigInt, Bytes, Entity, Value};
    pub use crate::{data_source, log};

    // Re-export the handler macro when available
    #[cfg(feature = "macros")]
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
    #[no_mangle]
    pub extern "C" fn abort(_msg: u32, _file: u32, _line: u32, _col: u32) -> ! {
        core::arch::wasm32::unreachable()
    }

    #[no_mangle]
    pub extern "C" fn __pin(ptr: u32) -> u32 {
        ptr // No-op: we use bump allocation, no GC
    }

    #[no_mangle]
    pub extern "C" fn __unpin(_ptr: u32) {
        // No-op: we use bump allocation, no GC
    }

    #[no_mangle]
    pub extern "C" fn __collect() {
        // No-op: we use bump allocation, no GC
    }
}
