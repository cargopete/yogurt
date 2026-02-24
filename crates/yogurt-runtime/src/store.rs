//! Store operations for reading and writing entities.

use alloc::string::String;

use crate::asc::{str_to_asc, AscPtr};
use crate::types::EntityData;

/// Load an entity by type name and ID.
///
/// Returns `None` if the entity does not exist.
#[cfg(target_arch = "wasm32")]
pub fn get(entity_type: &str, id: &str) -> Option<EntityData> {
    let type_ptr = str_to_asc(entity_type);
    let id_ptr = str_to_asc(id);

    let result = unsafe { crate::host::store_get(type_ptr.as_i32(), id_ptr.as_i32()) };

    if result == 0 {
        None
    } else {
        // TODO: Deserialize entity data from AscPtr
        Some(deserialize_entity(AscPtr::new(result as u32)))
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get(_entity_type: &str, _id: &str) -> Option<EntityData> {
    None
}

/// Write an entity to the store.
#[cfg(target_arch = "wasm32")]
pub fn set(entity_type: &str, id: &str, data: &EntityData) {
    let type_ptr = str_to_asc(entity_type);
    let id_ptr = str_to_asc(id);
    let data_ptr = serialize_entity(data);

    unsafe {
        crate::host::store_set(type_ptr.as_i32(), id_ptr.as_i32(), data_ptr.as_i32());
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set(_entity_type: &str, _id: &str, _data: &EntityData) {}

/// Remove an entity from the store.
#[cfg(target_arch = "wasm32")]
pub fn remove(entity_type: &str, id: &str) {
    let type_ptr = str_to_asc(entity_type);
    let id_ptr = str_to_asc(id);

    unsafe {
        crate::host::store_remove(type_ptr.as_i32(), id_ptr.as_i32());
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn remove(_entity_type: &str, _id: &str) {}

/// Deserialize entity data from an AssemblyScript pointer.
///
/// The entity is stored as an array of key-value pairs in AS memory.
#[cfg(target_arch = "wasm32")]
fn deserialize_entity(_ptr: AscPtr<crate::asc::AscEntity>) -> EntityData {
    // TODO: Implement full deserialization from AS entity format
    // For now, return empty data
    EntityData::new()
}

/// Serialize entity data to an AssemblyScript pointer.
///
/// Creates an AS-compatible entity representation in WASM memory.
#[cfg(target_arch = "wasm32")]
fn serialize_entity(_data: &EntityData) -> AscPtr<crate::asc::AscEntity> {
    // TODO: Implement full serialization to AS entity format
    AscPtr::null()
}
