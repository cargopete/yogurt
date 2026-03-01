//! Auto-generated data source templates — do not edit

use alloc::string::ToString;
use yogurt_runtime::prelude::Address;
use yogurt_runtime::data_source;

/// Data source template: `Pair`
///
/// Kind: `ethereum`
pub struct Pair;

impl Pair {
    /// Create a new data source instance for the given contract address.
pub fn create(address: &Address) {
data_source::create("Pair", &[address.to_hex()]);
}
}

