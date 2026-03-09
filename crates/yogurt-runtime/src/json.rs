//! JSON parsing utilities.
//!
//! Provides JSON parsing for subgraph handlers, commonly used for
//! parsing IPFS metadata and other JSON data sources.
//!
//! # Example
//!
//! ```ignore
//! use yogurt_runtime::json;
//! use yogurt_runtime::types::Bytes;
//!
//! let data = Bytes::from(br#"{"name": "Token", "decimals": 18}"#.to_vec());
//! let value = json::from_bytes(&data);
//!
//! if let Some(name) = value.get("name").and_then(|v| v.as_string()) {
//!     log::info!("Token name: {}", name);
//! }
//! ```

use alloc::string::String;
use alloc::vec::Vec;

use crate::types::{BigInt, Bytes};

/// A JSON value.
#[derive(Clone, Debug, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(JsonNumber),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

/// A JSON number (can be integer or float).
#[derive(Clone, Debug, PartialEq)]
pub enum JsonNumber {
    Int(i64),
    Uint(u64),
    Float(f64),
}

impl JsonValue {
    /// Check if this is a null value.
    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }

    /// Check if this is a boolean.
    pub fn is_bool(&self) -> bool {
        matches!(self, JsonValue::Bool(_))
    }

    /// Check if this is a number.
    pub fn is_number(&self) -> bool {
        matches!(self, JsonValue::Number(_))
    }

    /// Check if this is a string.
    pub fn is_string(&self) -> bool {
        matches!(self, JsonValue::String(_))
    }

    /// Check if this is an array.
    pub fn is_array(&self) -> bool {
        matches!(self, JsonValue::Array(_))
    }

    /// Check if this is an object.
    pub fn is_object(&self) -> bool {
        matches!(self, JsonValue::Object(_))
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

    /// Try to get as an i64.
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            JsonValue::Number(JsonNumber::Int(n)) => Some(*n),
            JsonValue::Number(JsonNumber::Uint(n)) => (*n).try_into().ok(),
            JsonValue::Number(JsonNumber::Float(n)) => Some(*n as i64),
            _ => None,
        }
    }

    /// Try to get as a u64.
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            JsonValue::Number(JsonNumber::Uint(n)) => Some(*n),
            JsonValue::Number(JsonNumber::Int(n)) => (*n).try_into().ok(),
            JsonValue::Number(JsonNumber::Float(n)) => Some(*n as u64),
            _ => None,
        }
    }

    /// Try to get as an f64.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            JsonValue::Number(JsonNumber::Float(n)) => Some(*n),
            JsonValue::Number(JsonNumber::Int(n)) => Some(*n as f64),
            JsonValue::Number(JsonNumber::Uint(n)) => Some(*n as f64),
            _ => None,
        }
    }

    /// Try to get as a BigInt.
    pub fn as_big_int(&self) -> Option<BigInt> {
        match self {
            JsonValue::Number(JsonNumber::Int(n)) => Some(BigInt::from_i64(*n)),
            JsonValue::Number(JsonNumber::Uint(n)) => Some(BigInt::from_u64(*n)),
            JsonValue::String(s) => BigInt::from_string(s),
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

    /// Get an array element by index.
    pub fn get_index(&self, index: usize) -> Option<&JsonValue> {
        match self {
            JsonValue::Array(arr) => arr.get(index),
            _ => None,
        }
    }

    /// Convert to a pretty-printed string (for debugging).
    pub fn to_string(&self) -> String {
        match self {
            JsonValue::Null => String::from("null"),
            JsonValue::Bool(b) => if *b { String::from("true") } else { String::from("false") },
            JsonValue::Number(n) => match n {
                JsonNumber::Int(i) => alloc::format!("{}", i),
                JsonNumber::Uint(u) => alloc::format!("{}", u),
                JsonNumber::Float(f) => alloc::format!("{}", f),
            },
            JsonValue::String(s) => alloc::format!("\"{}\"", s),
            JsonValue::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                alloc::format!("[{}]", items.join(", "))
            }
            JsonValue::Object(obj) => {
                let items: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| alloc::format!("\"{}\": {}", k, v.to_string()))
                    .collect();
                alloc::format!("{{{}}}", items.join(", "))
            }
        }
    }
}

impl Default for JsonValue {
    fn default() -> Self {
        JsonValue::Null
    }
}

// ============================================================================
// Native JSON Parsing (using serde_json)
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
fn serde_to_json_value(value: serde_json::Value) -> JsonValue {
    match value {
        serde_json::Value::Null => JsonValue::Null,
        serde_json::Value::Bool(b) => JsonValue::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                JsonValue::Number(JsonNumber::Int(i))
            } else if let Some(u) = n.as_u64() {
                JsonValue::Number(JsonNumber::Uint(u))
            } else if let Some(f) = n.as_f64() {
                JsonValue::Number(JsonNumber::Float(f))
            } else {
                JsonValue::Null
            }
        }
        serde_json::Value::String(s) => JsonValue::String(s),
        serde_json::Value::Array(arr) => {
            JsonValue::Array(arr.into_iter().map(serde_to_json_value).collect())
        }
        serde_json::Value::Object(obj) => {
            JsonValue::Object(
                obj.into_iter()
                    .map(|(k, v)| (k, serde_to_json_value(v)))
                    .collect(),
            )
        }
    }
}

/// Parse JSON from bytes.
///
/// Returns `JsonValue::Null` if parsing fails.
#[cfg(not(target_arch = "wasm32"))]
pub fn from_bytes(data: &Bytes) -> JsonValue {
    match serde_json::from_slice(data.as_slice()) {
        Ok(value) => serde_to_json_value(value),
        Err(_) => JsonValue::Null,
    }
}

/// Parse JSON from a string.
///
/// Returns `JsonValue::Null` if parsing fails.
#[cfg(not(target_arch = "wasm32"))]
pub fn from_string(data: &str) -> JsonValue {
    match serde_json::from_str(data) {
        Ok(value) => serde_to_json_value(value),
        Err(_) => JsonValue::Null,
    }
}

/// Try to parse JSON from bytes, returning an error on failure.
#[cfg(not(target_arch = "wasm32"))]
pub fn try_from_bytes(data: &Bytes) -> Result<JsonValue, String> {
    serde_json::from_slice(data.as_slice())
        .map(serde_to_json_value)
        .map_err(|e| alloc::format!("JSON parse error: {}", e))
}

/// Try to parse JSON from a string, returning an error on failure.
#[cfg(not(target_arch = "wasm32"))]
pub fn try_from_string(data: &str) -> Result<JsonValue, String> {
    serde_json::from_str(data)
        .map(serde_to_json_value)
        .map_err(|e| alloc::format!("JSON parse error: {}", e))
}

// ============================================================================
// WASM JSON Parsing (via host functions)
// ============================================================================

#[cfg(target_arch = "wasm32")]
use crate::asc::{bytes_to_asc, AscPtr};

/// Parse JSON from bytes.
///
/// Returns `JsonValue::Null` if parsing fails.
#[cfg(target_arch = "wasm32")]
pub fn from_bytes(data: &Bytes) -> JsonValue {
    let data_ptr = bytes_to_asc(data.as_slice());
    let result_ptr = unsafe { crate::host::json_from_bytes(data_ptr.as_i32()) };

    if result_ptr == 0 {
        return JsonValue::Null;
    }

    deserialize_json_value(result_ptr as u32)
}

/// Parse JSON from a string.
#[cfg(target_arch = "wasm32")]
pub fn from_string(data: &str) -> JsonValue {
    from_bytes(&Bytes::from(data.as_bytes()))
}

/// Try to parse JSON from bytes.
#[cfg(target_arch = "wasm32")]
pub fn try_from_bytes(data: &Bytes) -> Result<JsonValue, String> {
    let value = from_bytes(data);
    if value.is_null() && !data.is_empty() {
        Err(String::from("JSON parse error"))
    } else {
        Ok(value)
    }
}

/// Try to parse JSON from a string.
#[cfg(target_arch = "wasm32")]
pub fn try_from_string(data: &str) -> Result<JsonValue, String> {
    try_from_bytes(&Bytes::from(data.as_bytes()))
}

/// Deserialize a JsonValue from an AS memory pointer.
#[cfg(target_arch = "wasm32")]
fn deserialize_json_value(ptr: u32) -> JsonValue {
    use crate::asc::{asc_to_string, read_u32_at, AscArrayHeader, AscEnumHeader};

    if ptr == 0 {
        return JsonValue::Null;
    }

    // JSON values in graph-node are represented as an enum
    // Kind values:
    // NULL = 0, BOOL = 1, NUMBER = 2, STRING = 3, ARRAY = 4, OBJECT = 5
    unsafe {
        let header = ptr as *const AscEnumHeader;
        let kind = (*header).kind;
        let payload = (*header).payload;

        match kind {
            0 => JsonValue::Null,
            1 => JsonValue::Bool(payload != 0),
            2 => {
                // Number - payload is pointer to BigDecimal string representation
                let s = asc_to_string(AscPtr::new(payload as u32));
                if let Ok(i) = s.parse::<i64>() {
                    JsonValue::Number(JsonNumber::Int(i))
                } else if let Ok(f) = s.parse::<f64>() {
                    JsonValue::Number(JsonNumber::Float(f))
                } else {
                    JsonValue::Null
                }
            }
            3 => {
                // String
                let s = asc_to_string(AscPtr::new(payload as u32));
                JsonValue::String(s)
            }
            4 => {
                // Array
                let array_ptr = payload as u32;
                let array_header = array_ptr as *const AscArrayHeader;
                let buffer_ptr = (*array_header).buffer;
                let length = (*array_header).length;

                let mut items = Vec::with_capacity(length as usize);
                for i in 0..length as usize {
                    let item_ptr = read_u32_at(buffer_ptr, i * 4);
                    items.push(deserialize_json_value(item_ptr));
                }
                JsonValue::Array(items)
            }
            5 => {
                // Object - represented as TypedMap<String, JsonValue>
                let map_ptr = payload as u32;
                // TypedMap has an entries array
                let entries_ptr = read_u32_at(map_ptr, 0);
                let entries_header = entries_ptr as *const AscArrayHeader;
                let buffer_ptr = (*entries_header).buffer;
                let length = (*entries_header).length;

                let mut entries = Vec::with_capacity(length as usize);
                for i in 0..length as usize {
                    let entry_ptr = read_u32_at(buffer_ptr, i * 4);
                    // Entry has key (offset 0) and value (offset 4)
                    let key_ptr = read_u32_at(entry_ptr, 0);
                    let value_ptr = read_u32_at(entry_ptr, 4);
                    let key = asc_to_string(AscPtr::new(key_ptr));
                    let value = deserialize_json_value(value_ptr);
                    entries.push((key, value));
                }
                JsonValue::Object(entries)
            }
            _ => JsonValue::Null,
        }
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Try to parse a JSON value as i64.
pub fn to_i64(value: &JsonValue) -> Option<i64> {
    value.as_i64()
}

/// Try to parse a JSON value as u64.
pub fn to_u64(value: &JsonValue) -> Option<u64> {
    value.as_u64()
}

/// Try to parse a JSON value as f64.
pub fn to_f64(value: &JsonValue) -> Option<f64> {
    value.as_f64()
}

/// Try to parse a JSON value as BigInt.
pub fn to_big_int(value: &JsonValue) -> Option<BigInt> {
    value.as_big_int()
}
