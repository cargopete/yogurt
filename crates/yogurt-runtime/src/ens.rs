//! ENS (Ethereum Name Service) utilities.
//!
//! Provides reverse lookup from ENS name hash to human-readable name.

use alloc::string::String;

#[cfg(target_arch = "wasm32")]
use crate::asc::{asc_to_string, bytes_to_asc, AscPtr};

/// Look up an ENS name by its namehash.
///
/// Returns `None` if no name is registered for this hash.
///
/// # Example
///
/// ```ignore
/// use yogurt_runtime::ens;
/// use yogurt_runtime::crypto;
///
/// // Look up the name for a given namehash
/// let namehash = Bytes::from_hex_string("0x...").unwrap();
/// if let Some(name) = ens::name_by_hash(&namehash.0) {
///     log::info!("ENS name: {}", name);
/// }
/// ```
#[cfg(target_arch = "wasm32")]
pub fn name_by_hash(hash: &[u8]) -> Option<String> {
    let hash_ptr = bytes_to_asc(hash);
    let result_ptr = unsafe { crate::host::ens_name_by_hash(hash_ptr.as_i32()) };

    if result_ptr == 0 {
        None
    } else {
        Some(asc_to_string(AscPtr::new(result_ptr as u32)))
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn name_by_hash(_hash: &[u8]) -> Option<String> {
    // ENS lookups are not available in native test mode
    // Users can mock this if needed via testing utilities
    None
}
