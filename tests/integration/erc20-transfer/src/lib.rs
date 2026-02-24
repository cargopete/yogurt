//! ERC-20 Transfer Tracker — yogurt proof of concept subgraph
//!
//! This minimal subgraph demonstrates yogurt's ability to:
//! 1. Handle Ethereum events with typed parameters
//! 2. Create and save entities to the graph-node store
//! 3. Compile to AS-compatible WASM

#![no_std]
extern crate alloc;

mod generated;
mod mappings;

// Re-export the runtime's WASM exports
pub use yogurt_runtime::*;
