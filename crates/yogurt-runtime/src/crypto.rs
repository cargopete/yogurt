//! Cryptographic utilities.

#[cfg(target_arch = "wasm32")]
use crate::asc::{asc_to_bytes, bytes_to_asc, AscPtr};
use crate::types::Bytes;

/// Compute the Keccak-256 hash of the input data.
///
/// Returns a 32-byte hash.
///
/// # Example
///
/// ```ignore
/// use yogurt_runtime::crypto;
///
/// let hash = crypto::keccak256(b"hello");
/// assert_eq!(hash.len(), 32);
/// ```
#[cfg(target_arch = "wasm32")]
pub fn keccak256(data: &[u8]) -> Bytes {
    let data_ptr = bytes_to_asc(data);
    let result_ptr = unsafe { crate::host::crypto_keccak256(data_ptr.as_i32()) };
    Bytes::from_vec(asc_to_bytes(AscPtr::new(result_ptr as u32)))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn keccak256(data: &[u8]) -> Bytes {
    use sha3::{Digest, Keccak256};

    let mut hasher = Keccak256::new();
    hasher.update(data);
    let result = hasher.finalize();
    Bytes::from_vec(result.to_vec())
}

/// Compute the SHA-256 hash of the input data.
///
/// Returns a 32-byte hash.
///
/// Note: This function is only available in native builds (for testing).
/// Graph-node does not provide a SHA-256 host function - only keccak256.
/// In WASM builds, this will panic. Use `keccak256` instead for production code.
#[cfg(target_arch = "wasm32")]
pub fn sha256(_data: &[u8]) -> Bytes {
    panic!("sha256 is not available in WASM - graph-node only supports keccak256")
}

#[cfg(not(target_arch = "wasm32"))]
pub fn sha256(data: &[u8]) -> Bytes {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    Bytes::from_vec(result.to_vec())
}
