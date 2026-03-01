//! Uniswap V2 Subgraph — yogurt example
//!
//! Demonstrates:
//! - Data source templates (Factory spawns Pair watchers)
//! - Contract calls (token0, token1, getReserves)
//! - Multiple related entities
//! - Immutable entities (Swap, Mint, Burn)

#![no_std]

extern crate alloc;

mod generated;
mod mappings;
