//! Data source utilities for dynamic data source creation.

use alloc::string::String;
use alloc::vec::Vec;

use crate::asc::{asc_to_bytes, asc_to_string, str_to_asc, AscPtr};
use crate::types::{Address, EntityData};

/// Create a new data source from a template.
///
/// This is used for dynamically tracking new contracts discovered at runtime,
/// such as new pool contracts created by a factory.
#[cfg(target_arch = "wasm32")]
pub fn create(_name: &str, _params: &[String]) {
    // TODO: Implement data source creation
    // let name_ptr = str_to_asc(name);
    // let params_ptr = strings_to_asc_array(params);
    // unsafe { crate::host::data_source_create(name_ptr.as_i32(), params_ptr.as_i32()); }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn create(_name: &str, _params: &[String]) {}

/// Create a new data source from a template with context.
///
/// The context is an entity that will be available to handlers
/// via `data_source::context()`.
pub fn create_with_context(_name: &str, _params: &[String], _context: EntityData) {
    // TODO: Implement with context support
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
    Address::zero()
}

/// Get the network name of the current data source (e.g., "mainnet", "goerli").
#[cfg(target_arch = "wasm32")]
pub fn network() -> String {
    let ptr = unsafe { crate::host::data_source_network() };
    asc_to_string(AscPtr::new(ptr as u32))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn network() -> String {
    String::from("mainnet")
}

/// Get the context entity for the current data source.
///
/// Only available if the data source was created with `create_with_context`.
#[cfg(target_arch = "wasm32")]
pub fn context() -> EntityData {
    let _ptr = unsafe { crate::host::data_source_context() };
    // TODO: Deserialize context entity
    EntityData::new()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn context() -> EntityData {
    EntityData::new()
}
