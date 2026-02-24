//! JSON parsing utilities.

use alloc::string::String;
use alloc::vec::Vec;

use crate::asc::{bytes_to_asc, AscPtr};
use crate::types::{BigInt, Bytes};

/// A JSON value.
#[derive(Clone, Debug)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(JsonNumber),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

/// A JSON number (can be integer or float).
#[derive(Clone, Debug)]
pub enum JsonNumber {
    Int(i64),
    Uint(u64),
    Float(f64),
    BigInt(BigInt),
}

impl JsonValue {
    /// Check if this is a null value.
    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }

    /// Try to get as a bool.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Try to get as a string.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            JsonValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get as an array.
    pub fn as_array(&self) -> Option<&[JsonValue]> {
        match self {
            JsonValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Try to get as an object (list of key-value pairs).
    pub fn as_object(&self) -> Option<&[(String, JsonValue)]> {
        match self {
            JsonValue::Object(obj) => Some(obj),
            _ => None,
        }
    }

    /// Get a field from a JSON object by key.
    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        match self {
            JsonValue::Object(obj) => obj.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
}

/// Parse JSON from bytes.
#[cfg(target_arch = "wasm32")]
pub fn from_bytes(_data: &Bytes) -> JsonValue {
    // TODO: Implement JSON parsing via host function
    // let data_ptr = bytes_to_asc(&data.0);
    // let result_ptr = unsafe { crate::host::json_from_bytes(data_ptr.as_i32()) };
    // deserialize_json_value(AscPtr::new(result_ptr as u32))
    JsonValue::Null
}

#[cfg(not(target_arch = "wasm32"))]
pub fn from_bytes(_data: &Bytes) -> JsonValue {
    JsonValue::Null
}

/// Try to parse a JSON value as i64.
pub fn to_i64(_value: &JsonValue) -> Option<i64> {
    // TODO: Implement via host function
    None
}

/// Try to parse a JSON value as u64.
pub fn to_u64(_value: &JsonValue) -> Option<u64> {
    // TODO: Implement via host function
    None
}

/// Try to parse a JSON value as f64.
pub fn to_f64(_value: &JsonValue) -> Option<f64> {
    // TODO: Implement via host function
    None
}

/// Try to parse a JSON value as BigInt.
pub fn to_big_int(_value: &JsonValue) -> Option<BigInt> {
    // TODO: Implement via host function
    None
}
