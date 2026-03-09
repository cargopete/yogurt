//! Data source utilities for dynamic data source creation.

use alloc::string::String;

#[cfg(target_arch = "wasm32")]
use alloc::vec::Vec;

#[cfg(target_arch = "wasm32")]
use crate::asc::{asc_to_bytes, asc_to_string, str_to_asc, AscPtr};
#[cfg(not(target_arch = "wasm32"))]
use crate::asc::{asc_to_bytes, asc_to_string, AscPtr};
use crate::types::{Address, EntityData};

/// Create a new data source from a template.
///
/// This is used for dynamically tracking new contracts discovered at runtime,
/// such as new pool contracts created by a factory.
///
/// # Example
///
/// ```ignore
/// // When a factory creates a new pool contract:
/// data_source::create("Pool", &[pool_address.to_hex()]);
/// ```
#[cfg(target_arch = "wasm32")]
pub fn create(name: &str, params: &[String]) {
    let name_ptr = str_to_asc(name);
    let params_ptr = strings_to_asc_array(params);
    unsafe {
        crate::host::data_source_create(name_ptr.as_i32(), params_ptr as i32);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn create(_name: &str, _params: &[String]) {
    // No-op in tests - templates don't spawn real data sources
}

/// Create a new data source from a template with context.
///
/// The context is an entity that will be available to handlers
/// via `data_source::context()`.
///
/// # Example
///
/// ```ignore
/// let mut context = EntityData::new();
/// context.set("poolId", Value::String("pool-123".into()));
/// data_source::create_with_context("Pool", &[pool_address.to_hex()], context);
/// ```
#[cfg(target_arch = "wasm32")]
pub fn create_with_context(name: &str, params: &[String], context: EntityData) {
    let name_ptr = str_to_asc(name);
    let params_ptr = strings_to_asc_array(params);
    let context_ptr = crate::store::serialize_entity(&context);
    unsafe {
        crate::host::data_source_create_with_context(
            name_ptr.as_i32(),
            params_ptr as i32,
            context_ptr.as_i32(),
        );
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn create_with_context(_name: &str, _params: &[String], _context: EntityData) {
    // No-op in tests - templates don't spawn real data sources
}

/// Get the address of the current data source.
#[cfg(target_arch = "wasm32")]
pub fn address() -> Address {
    let ptr = unsafe { crate::host::data_source_address() };
    let bytes = asc_to_bytes(AscPtr::new(ptr as u32));
    Address::from(bytes.as_slice())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn address() -> Address {
    crate::testing::get_mock_data_source_address()
}

/// Get the network name of the current data source (e.g., "mainnet", "goerli").
#[cfg(target_arch = "wasm32")]
pub fn network() -> String {
    let ptr = unsafe { crate::host::data_source_network() };
    asc_to_string(AscPtr::new(ptr as u32))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn network() -> String {
    crate::testing::get_mock_data_source_network()
}

/// Get the context entity for the current data source.
///
/// Only available if the data source was created with `create_with_context`.
/// Returns an empty EntityData if no context was set.
#[cfg(target_arch = "wasm32")]
pub fn context() -> EntityData {
    let ptr = unsafe { crate::host::data_source_context() };
    if ptr == 0 {
        EntityData::new()
    } else {
        crate::store::deserialize_entity(AscPtr::new(ptr as u32))
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn context() -> EntityData {
    crate::testing::get_mock_data_source_context()
}

/// Get the string parameter for a file data source.
///
/// For file data sources (kind: file/ipfs or file/arweave), this returns
/// the content identifier (IPFS CID or Arweave transaction ID) that was
/// passed when the data source was created.
///
/// # Example
///
/// ```ignore
/// #[handler]
/// pub fn handle_metadata(content: Bytes) {
///     let ipfs_hash = data_source::string_param();
///     log::info!("Processing file: {}", ipfs_hash);
/// }
/// ```
#[cfg(target_arch = "wasm32")]
pub fn string_param() -> String {
    // For file data sources, the "address" field contains the content identifier
    // encoded as bytes. We decode it as UTF-8 to get the string.
    let addr = address();
    String::from_utf8_lossy(addr.as_bytes()).into_owned()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn string_param() -> String {
    // For file data sources, the "address" field contains the content identifier
    // encoded as bytes. We decode it as UTF-8 to get the string.
    let addr = address();
    String::from_utf8_lossy(addr.as_bytes()).into_owned()
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert a slice of Strings to an ASC array pointer.
#[cfg(target_arch = "wasm32")]
fn strings_to_asc_array(strings: &[String]) -> u32 {
    use crate::asc::AscArrayHeader;
    use crate::allocator::{asc_alloc, class_id};

    let count = strings.len();

    // Allocate buffer for string pointers
    let buffer_size = (count * 4) as u32;
    let buffer_ptr = asc_alloc(buffer_size, class_id::ARRAY_BUFFER);

    // Convert each string and store pointer
    for (i, s) in strings.iter().enumerate() {
        let str_ptr = str_to_asc(s);
        unsafe {
            let dest = (buffer_ptr as *mut u32).add(i);
            core::ptr::write_unaligned(dest, str_ptr.as_raw());
        }
    }

    // Allocate Array struct
    let array_ptr = asc_alloc(
        core::mem::size_of::<AscArrayHeader>() as u32,
        class_id::ARRAY_PTR,
    );

    unsafe {
        let header = array_ptr as *mut AscArrayHeader;
        (*header).buffer = buffer_ptr;
        (*header).buffer_data_start = 0;
        (*header).buffer_data_length = buffer_size;
        (*header).length = count as i32;
    }

    array_ptr
}
