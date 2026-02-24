//! Store operations for reading and writing entities.
//!
//! Entities are serialized to AssemblyScript's TypedMap format:
//! - Entity = TypedMap<String, StoreValue>
//! - TypedMap contains an Array of TypedMapEntry pointers
//! - Each TypedMapEntry has a key (String) and value (Enum)
//! - The Enum contains a kind discriminant and payload

use alloc::string::String;
use alloc::vec::Vec;

use crate::allocator::{asc_alloc, class_id, read_rt_size};
use crate::asc::{
    asc_to_bytes, asc_to_string, bytes_to_asc, str_to_asc, AscArrayHeader, AscEnumHeader,
    AscEntity, AscPtr, AscStoreValue, AscString, AscTypedMapEntry, AscTypedMapEntryHeader,
    AscTypedMapHeader, StoreValueKind,
};
use crate::types::{BigDecimal, BigInt, Bytes, EntityData, Value};

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

// ============================================================================
// Serialization: Rust EntityData → AssemblyScript memory
// ============================================================================

/// Serialize entity data to an AssemblyScript TypedMap pointer.
#[cfg(target_arch = "wasm32")]
fn serialize_entity(data: &EntityData) -> AscPtr<AscEntity> {
    // Collect entries
    let entries: Vec<_> = data.iter().collect();
    let entry_count = entries.len();

    // Allocate the array buffer to hold entry pointers
    let buffer_size = (entry_count * 4) as u32; // Each pointer is 4 bytes
    let buffer_ptr = asc_alloc(buffer_size, class_id::ARRAY_BUFFER);

    // Create each entry and write pointer to buffer
    for (i, (key, value)) in entries.iter().enumerate() {
        let entry_ptr = serialize_entry(key, value);

        unsafe {
            let dest = (buffer_ptr as *mut u32).add(i);
            core::ptr::write_unaligned(dest, entry_ptr.as_raw());
        }
    }

    // Allocate the Array struct
    let array_ptr = asc_alloc(
        core::mem::size_of::<AscArrayHeader>() as u32,
        class_id::ARRAY_PTR,
    );

    // Write Array header
    unsafe {
        let header = array_ptr as *mut AscArrayHeader;
        (*header).buffer = buffer_ptr;
        (*header).buffer_data_start = 0;
        (*header).buffer_data_length = buffer_size;
        (*header).length = entry_count as i32;
    }

    // Allocate the TypedMap struct
    let map_ptr = asc_alloc(
        core::mem::size_of::<AscTypedMapHeader>() as u32,
        class_id::TYPED_MAP,
    );

    // Write TypedMap header (just the entries pointer)
    unsafe {
        let header = map_ptr as *mut AscTypedMapHeader;
        (*header).entries = array_ptr;
    }

    AscPtr::new(map_ptr)
}

/// Serialize a single key-value entry to a TypedMapEntry.
#[cfg(target_arch = "wasm32")]
fn serialize_entry(
    key: &String,
    value: &Value,
) -> AscPtr<AscTypedMapEntry<AscString, AscStoreValue>> {
    let key_ptr = str_to_asc(key);
    let value_ptr = serialize_value(value);

    // Allocate entry struct
    let entry_ptr = asc_alloc(
        core::mem::size_of::<AscTypedMapEntryHeader>() as u32,
        class_id::TYPED_MAP_ENTRY,
    );

    // Write entry fields
    unsafe {
        let header = entry_ptr as *mut AscTypedMapEntryHeader;
        (*header).key = key_ptr.as_raw();
        (*header).value = value_ptr.as_raw();
    }

    AscPtr::new(entry_ptr)
}

/// Serialize a Value to an AssemblyScript StoreValue enum.
#[cfg(target_arch = "wasm32")]
fn serialize_value(value: &Value) -> AscPtr<AscStoreValue> {
    let (kind, payload) = match value {
        Value::String(s) => {
            let ptr = str_to_asc(s);
            (StoreValueKind::String, ptr.as_raw() as u64)
        }
        Value::Int(i) => (StoreValueKind::Int, *i as u64),
        Value::Int8(i) => (StoreValueKind::Int8, *i as u64),
        Value::BigInt(bi) => {
            let ptr = bi.as_ptr();
            (StoreValueKind::BigInt, ptr.as_raw() as u64)
        }
        Value::BigDecimal(bd) => {
            let ptr = bd.as_ptr();
            (StoreValueKind::BigDecimal, ptr.as_raw() as u64)
        }
        Value::Bool(b) => (StoreValueKind::Bool, if *b { 1 } else { 0 }),
        Value::Bytes(bytes) => {
            let ptr = bytes_to_asc(&bytes.0);
            (StoreValueKind::Bytes, ptr.as_raw() as u64)
        }
        Value::Array(arr) => {
            let ptr = serialize_value_array(arr);
            (StoreValueKind::Array, ptr.as_raw() as u64)
        }
        Value::Null => (StoreValueKind::Null, 0),
    };

    // Allocate enum struct
    let enum_ptr = asc_alloc(
        core::mem::size_of::<AscEnumHeader>() as u32,
        class_id::STORE_VALUE,
    );

    // Write enum fields
    unsafe {
        let header = enum_ptr as *mut AscEnumHeader;
        (*header).kind = kind as i32;
        (*header)._padding = 0;
        (*header).payload = payload;
    }

    AscPtr::new(enum_ptr)
}

/// Serialize an array of Values.
#[cfg(target_arch = "wasm32")]
fn serialize_value_array(values: &[Value]) -> AscPtr<crate::asc::AscArray<AscStoreValue>> {
    let count = values.len();

    // Allocate buffer for value pointers
    let buffer_size = (count * 4) as u32;
    let buffer_ptr = asc_alloc(buffer_size, class_id::ARRAY_BUFFER);

    // Serialize each value and store pointer
    for (i, value) in values.iter().enumerate() {
        let value_ptr = serialize_value(value);
        unsafe {
            let dest = (buffer_ptr as *mut u32).add(i);
            core::ptr::write_unaligned(dest, value_ptr.as_raw());
        }
    }

    // Allocate Array struct
    let array_ptr = asc_alloc(
        core::mem::size_of::<AscArrayHeader>() as u32,
        class_id::ARRAY_STORE_VALUE,
    );

    // Write Array header
    unsafe {
        let header = array_ptr as *mut AscArrayHeader;
        (*header).buffer = buffer_ptr;
        (*header).buffer_data_start = 0;
        (*header).buffer_data_length = buffer_size;
        (*header).length = count as i32;
    }

    AscPtr::new(array_ptr)
}

// ============================================================================
// Deserialization: AssemblyScript memory → Rust EntityData
// ============================================================================

/// Deserialize entity data from an AssemblyScript TypedMap pointer.
#[cfg(target_arch = "wasm32")]
fn deserialize_entity(ptr: AscPtr<AscEntity>) -> EntityData {
    let mut data = EntityData::new();

    if ptr.is_null() {
        return data;
    }

    unsafe {
        // Read TypedMap header to get entries array pointer
        let map_header = ptr.as_raw() as *const AscTypedMapHeader;
        let entries_array_ptr = (*map_header).entries;

        if entries_array_ptr == 0 {
            return data;
        }

        // Read Array header
        let array_header = entries_array_ptr as *const AscArrayHeader;
        let buffer_ptr = (*array_header).buffer;
        let length = (*array_header).length;

        if buffer_ptr == 0 || length <= 0 {
            return data;
        }

        // Read each entry pointer from the buffer
        for i in 0..length as usize {
            let entry_ptr_addr = (buffer_ptr as *const u32).add(i);
            let entry_ptr = core::ptr::read_unaligned(entry_ptr_addr);

            if entry_ptr == 0 {
                continue;
            }

            // Read entry header
            let entry_header = entry_ptr as *const AscTypedMapEntryHeader;
            let key_ptr = (*entry_header).key;
            let value_ptr = (*entry_header).value;

            // Deserialize key and value
            let key = asc_to_string(AscPtr::new(key_ptr));
            let value = deserialize_value(AscPtr::new(value_ptr));

            data.set(key, value);
        }
    }

    data
}

/// Deserialize a StoreValue enum to a Rust Value.
#[cfg(target_arch = "wasm32")]
fn deserialize_value(ptr: AscPtr<AscStoreValue>) -> Value {
    if ptr.is_null() {
        return Value::Null;
    }

    unsafe {
        let header = ptr.as_raw() as *const AscEnumHeader;
        let kind = (*header).kind;
        let payload = (*header).payload;

        match kind {
            0 => {
                // STRING
                let str_ptr = AscPtr::new(payload as u32);
                Value::String(asc_to_string(str_ptr))
            }
            1 => {
                // INT
                Value::Int(payload as i32)
            }
            2 => {
                // BIGDECIMAL
                Value::BigDecimal(BigDecimal::from_ptr(AscPtr::new(payload as u32)))
            }
            3 => {
                // BOOL
                Value::Bool(payload != 0)
            }
            4 => {
                // ARRAY
                let arr = deserialize_value_array(AscPtr::new(payload as u32));
                Value::Array(arr)
            }
            5 => {
                // NULL
                Value::Null
            }
            6 => {
                // BYTES
                let bytes = asc_to_bytes(AscPtr::new(payload as u32));
                Value::Bytes(Bytes::from_vec(bytes))
            }
            7 => {
                // BIGINT
                Value::BigInt(BigInt::from_ptr(AscPtr::new(payload as u32)))
            }
            8 => {
                // INT8
                Value::Int8(payload as i64)
            }
            _ => Value::Null, // Unknown type, treat as null
        }
    }
}

/// Deserialize an array of StoreValues.
#[cfg(target_arch = "wasm32")]
fn deserialize_value_array(ptr: AscPtr<crate::asc::AscArray<AscStoreValue>>) -> Vec<Value> {
    let mut values = Vec::new();

    if ptr.is_null() {
        return values;
    }

    unsafe {
        let array_header = ptr.as_raw() as *const AscArrayHeader;
        let buffer_ptr = (*array_header).buffer;
        let length = (*array_header).length;

        if buffer_ptr == 0 || length <= 0 {
            return values;
        }

        values.reserve(length as usize);

        for i in 0..length as usize {
            let value_ptr_addr = (buffer_ptr as *const u32).add(i);
            let value_ptr = core::ptr::read_unaligned(value_ptr_addr);
            let value = deserialize_value(AscPtr::new(value_ptr));
            values.push(value);
        }
    }

    values
}
