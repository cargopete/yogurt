//! IPFS utilities for fetching content.

use crate::asc::{asc_to_bytes, str_to_asc, AscPtr};
use crate::types::Bytes;

/// Fetch content from IPFS by hash.
///
/// Returns `None` if the content cannot be fetched.
#[cfg(target_arch = "wasm32")]
pub fn cat(hash: &str) -> Option<Bytes> {
    let hash_ptr = str_to_asc(hash);
    let result_ptr = unsafe { crate::host::ipfs_cat(hash_ptr.as_i32()) };

    if result_ptr == 0 {
        None
    } else {
        Some(Bytes::from_vec(asc_to_bytes(AscPtr::new(result_ptr as u32))))
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn cat(_hash: &str) -> Option<Bytes> {
    None
}
