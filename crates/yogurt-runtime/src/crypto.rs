//! Cryptographic utilities.

use crate::asc::{asc_to_bytes, bytes_to_asc, AscPtr};
use crate::types::Bytes;

/// Compute the Keccak-256 hash of the input data.
#[cfg(target_arch = "wasm32")]
pub fn keccak256(data: &[u8]) -> Bytes {
    let data_ptr = bytes_to_asc(data);
    let result_ptr = unsafe { crate::host::crypto_keccak256(data_ptr.as_i32()) };
    Bytes::from_vec(asc_to_bytes(AscPtr::new(result_ptr as u32)))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn keccak256(_data: &[u8]) -> Bytes {
    // Native: could use a real keccak implementation for testing
    Bytes::new()
}
