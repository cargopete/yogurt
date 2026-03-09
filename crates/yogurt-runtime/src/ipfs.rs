//! IPFS utilities for fetching content.

#[cfg(target_arch = "wasm32")]
use crate::asc::{asc_to_bytes, str_to_asc, AscPtr};
use crate::types::Bytes;

#[cfg(target_arch = "wasm32")]
use crate::store::serialize_entity;
use crate::types::{EntityData, Value};

/// Flags for `ipfs::map` processing.
pub struct MapFlags;

impl MapFlags {
    /// Process the file as JSON (one JSON value per line).
    pub const JSON: i32 = 0;
}

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
pub fn cat(hash: &str) -> Option<Bytes> {
    crate::testing::get_mock_ipfs_content(hash).map(Bytes::from_vec)
}

/// Process an IPFS file by streaming its contents through a callback function.
///
/// This is useful for processing large files without loading them entirely into memory.
/// Each line/entry in the file is passed to the callback function.
///
/// # Arguments
///
/// * `hash` - The IPFS hash of the file to process
/// * `callback` - The name of an exported callback function. The function must have
///   the signature `fn(value: JsonValue, user_data: Value)` and be exported from
///   the WASM module (typically via `#[handler(name = "...")]`)
/// * `user_data` - Arbitrary data to pass to each callback invocation
/// * `flags` - Processing flags (use `MapFlags::JSON` for JSON processing)
///
/// # Example
///
/// ```ignore
/// use yogurt_runtime::ipfs::{self, MapFlags};
/// use yogurt_runtime::json::JsonValue;
/// use yogurt_runtime::types::Value;
///
/// // Define and export a callback function
/// #[handler(name = "processNftMetadata")]
/// pub fn process_nft_metadata(value: JsonValue, user_data: Value) {
///     let obj = value.as_object().unwrap();
///     let name = obj.get("name").unwrap().as_string().unwrap();
///     // ... create entity from metadata
/// }
///
/// // Then in your event handler:
/// let mut user_data = EntityData::new();
/// user_data.set("tokenId", Value::String("123".into()));
/// ipfs::map("QmHash...", "processNftMetadata", user_data, MapFlags::JSON);
/// ```
///
/// # Note
///
/// The callback function must be exported from the WASM module. Use the `#[handler]`
/// macro with a `name` parameter to ensure proper export naming.
#[cfg(target_arch = "wasm32")]
pub fn map(hash: &str, callback: &str, user_data: EntityData, flags: i32) {
    let hash_ptr = str_to_asc(hash);
    let callback_ptr = str_to_asc(callback);
    let user_data_ptr = serialize_entity(&user_data);

    unsafe {
        crate::host::ipfs_map(
            hash_ptr.as_i32(),
            callback_ptr.as_i32(),
            user_data_ptr.as_i32(),
            flags,
        );
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn map(_hash: &str, _callback: &str, _user_data: EntityData, _flags: i32) {
    // ipfs.map is not available in native test mode
    // It requires graph-node to invoke callbacks into WASM
    panic!("ipfs::map is not available in native test mode - it requires graph-node runtime")
}
