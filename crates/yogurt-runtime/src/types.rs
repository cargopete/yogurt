//! Core types for yogurt subgraph mappings.
//!
//! These are the Rust equivalents of graph-ts types like `Address`, `BigInt`,
//! `BigDecimal`, `Bytes`, and `Entity`.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::asc::AscPtr;

/// A 20-byte Ethereum address.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Address(pub [u8; 20]);

impl Address {
    /// Create an address from a 20-byte array.
    pub const fn new(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }

    /// Create a zero address.
    pub const fn zero() -> Self {
        Self([0u8; 20])
    }

    /// Get the address as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Convert to a hex string with 0x prefix.
    pub fn to_hex(&self) -> String {
        let mut s = String::with_capacity(42);
        s.push_str("0x");
        for byte in &self.0 {
            s.push(HEX_CHARS[(byte >> 4) as usize]);
            s.push(HEX_CHARS[(byte & 0xf) as usize]);
        }
        s
    }
}

impl From<[u8; 20]> for Address {
    fn from(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }
}

impl From<&[u8]> for Address {
    fn from(bytes: &[u8]) -> Self {
        let mut arr = [0u8; 20];
        let len = bytes.len().min(20);
        arr[20 - len..].copy_from_slice(&bytes[..len]);
        Self(arr)
    }
}

/// Variable-length byte array.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct Bytes(pub Vec<u8>);

impl Bytes {
    /// Create an empty byte array.
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// Create from a vector.
    pub fn from_vec(v: Vec<u8>) -> Self {
        Self(v)
    }

    /// Get as a byte slice.
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// Convert to a hex string with 0x prefix.
    pub fn to_hex(&self) -> String {
        let mut s = String::with_capacity(2 + self.0.len() * 2);
        s.push_str("0x");
        for byte in &self.0 {
            s.push(HEX_CHARS[(byte >> 4) as usize]);
            s.push(HEX_CHARS[(byte & 0xf) as usize]);
        }
        s
    }

    /// Get the length in bytes.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(v: Vec<u8>) -> Self {
        Self(v)
    }
}

impl From<&[u8]> for Bytes {
    fn from(slice: &[u8]) -> Self {
        Self(slice.to_vec())
    }
}

const HEX_CHARS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

/// Arbitrary-precision signed integer.
///
/// Backed by graph-node host calls for arithmetic operations.
#[derive(Clone, Debug)]
pub struct BigInt {
    ptr: AscPtr<crate::asc::AscBytes>,
}

impl BigInt {
    /// Create a BigInt from an AscPtr (internal use).
    pub fn from_ptr(ptr: AscPtr<crate::asc::AscBytes>) -> Self {
        Self { ptr }
    }

    /// Get the internal pointer.
    pub fn as_ptr(&self) -> AscPtr<crate::asc::AscBytes> {
        self.ptr
    }

    /// Create a BigInt with value zero.
    pub fn zero() -> Self {
        Self::from_i32(0)
    }

    /// Create a BigInt with value one.
    pub fn one() -> Self {
        Self::from_i32(1)
    }

    /// Create a BigInt from an i32.
    #[cfg(target_arch = "wasm32")]
    pub fn from_i32(value: i32) -> Self {
        // Encode as little-endian signed bytes
        let bytes = value.to_le_bytes();
        let ptr = crate::asc::bytes_to_asc(&bytes);
        Self {
            ptr: AscPtr::new(ptr.as_raw()),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_i32(_value: i32) -> Self {
        Self {
            ptr: AscPtr::null(),
        }
    }

    /// Create a BigInt from a u64.
    #[cfg(target_arch = "wasm32")]
    pub fn from_u64(value: u64) -> Self {
        let bytes = value.to_le_bytes();
        let ptr = crate::asc::bytes_to_asc(&bytes);
        Self {
            ptr: AscPtr::new(ptr.as_raw()),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_u64(_value: u64) -> Self {
        Self {
            ptr: AscPtr::null(),
        }
    }

    /// Convert to a decimal string representation.
    #[cfg(target_arch = "wasm32")]
    pub fn to_string(&self) -> String {
        let str_ptr = unsafe { crate::host::big_int_to_string(self.ptr.as_i32()) };
        crate::asc::asc_to_string(AscPtr::new(str_ptr as u32))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn to_string(&self) -> String {
        String::from("0")
    }
}

impl PartialEq for BigInt {
    fn eq(&self, other: &Self) -> bool {
        // TODO: Use host comparison when available
        self.to_string() == other.to_string()
    }
}

impl Eq for BigInt {}

// ============================================================================
// BigInt Arithmetic Operations
// ============================================================================

impl BigInt {
    /// Add two BigInts.
    #[cfg(target_arch = "wasm32")]
    pub fn plus(&self, other: &BigInt) -> BigInt {
        let result = unsafe { crate::host::big_int_plus(self.ptr.as_i32(), other.ptr.as_i32()) };
        BigInt::from_ptr(AscPtr::new(result as u32))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn plus(&self, _other: &BigInt) -> BigInt {
        BigInt::zero()
    }

    /// Subtract two BigInts.
    #[cfg(target_arch = "wasm32")]
    pub fn minus(&self, other: &BigInt) -> BigInt {
        let result = unsafe { crate::host::big_int_minus(self.ptr.as_i32(), other.ptr.as_i32()) };
        BigInt::from_ptr(AscPtr::new(result as u32))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn minus(&self, _other: &BigInt) -> BigInt {
        BigInt::zero()
    }

    /// Multiply two BigInts.
    #[cfg(target_arch = "wasm32")]
    pub fn times(&self, other: &BigInt) -> BigInt {
        let result = unsafe { crate::host::big_int_times(self.ptr.as_i32(), other.ptr.as_i32()) };
        BigInt::from_ptr(AscPtr::new(result as u32))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn times(&self, _other: &BigInt) -> BigInt {
        BigInt::zero()
    }

    /// Divide two BigInts (integer division).
    #[cfg(target_arch = "wasm32")]
    pub fn divided_by(&self, other: &BigInt) -> BigInt {
        let result = unsafe { crate::host::big_int_divided_by(self.ptr.as_i32(), other.ptr.as_i32()) };
        BigInt::from_ptr(AscPtr::new(result as u32))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn divided_by(&self, _other: &BigInt) -> BigInt {
        BigInt::zero()
    }

    /// Modulo operation.
    #[cfg(target_arch = "wasm32")]
    pub fn modulo(&self, other: &BigInt) -> BigInt {
        let result = unsafe { crate::host::big_int_mod(self.ptr.as_i32(), other.ptr.as_i32()) };
        BigInt::from_ptr(AscPtr::new(result as u32))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn modulo(&self, _other: &BigInt) -> BigInt {
        BigInt::zero()
    }

    /// Raise to a power.
    #[cfg(target_arch = "wasm32")]
    pub fn pow(&self, exp: u8) -> BigInt {
        let result = unsafe { crate::host::big_int_pow(self.ptr.as_i32(), exp as i32) };
        BigInt::from_ptr(AscPtr::new(result as u32))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn pow(&self, _exp: u8) -> BigInt {
        BigInt::zero()
    }

    /// Bitwise OR.
    #[cfg(target_arch = "wasm32")]
    pub fn bit_or(&self, other: &BigInt) -> BigInt {
        let result = unsafe { crate::host::big_int_bit_or(self.ptr.as_i32(), other.ptr.as_i32()) };
        BigInt::from_ptr(AscPtr::new(result as u32))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn bit_or(&self, _other: &BigInt) -> BigInt {
        BigInt::zero()
    }

    /// Bitwise AND.
    #[cfg(target_arch = "wasm32")]
    pub fn bit_and(&self, other: &BigInt) -> BigInt {
        let result = unsafe { crate::host::big_int_bit_and(self.ptr.as_i32(), other.ptr.as_i32()) };
        BigInt::from_ptr(AscPtr::new(result as u32))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn bit_and(&self, _other: &BigInt) -> BigInt {
        BigInt::zero()
    }

    /// Left shift.
    #[cfg(target_arch = "wasm32")]
    pub fn left_shift(&self, bits: u8) -> BigInt {
        let result = unsafe { crate::host::big_int_left_shift(self.ptr.as_i32(), bits as i32) };
        BigInt::from_ptr(AscPtr::new(result as u32))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn left_shift(&self, _bits: u8) -> BigInt {
        BigInt::zero()
    }

    /// Right shift.
    #[cfg(target_arch = "wasm32")]
    pub fn right_shift(&self, bits: u8) -> BigInt {
        let result = unsafe { crate::host::big_int_right_shift(self.ptr.as_i32(), bits as i32) };
        BigInt::from_ptr(AscPtr::new(result as u32))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn right_shift(&self, _bits: u8) -> BigInt {
        BigInt::zero()
    }

    /// Check if this BigInt is zero.
    pub fn is_zero(&self) -> bool {
        *self == BigInt::zero()
    }

    /// Convert to hex string.
    #[cfg(target_arch = "wasm32")]
    pub fn to_hex(&self) -> String {
        let str_ptr = unsafe { crate::host::big_int_to_hex(self.ptr.as_i32()) };
        crate::asc::asc_to_string(AscPtr::new(str_ptr as u32))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn to_hex(&self) -> String {
        String::from("0x0")
    }
}

// Rust operator trait implementations for BigInt
impl core::ops::Add for BigInt {
    type Output = BigInt;
    fn add(self, other: BigInt) -> BigInt {
        self.plus(&other)
    }
}

impl core::ops::Add<&BigInt> for BigInt {
    type Output = BigInt;
    fn add(self, other: &BigInt) -> BigInt {
        self.plus(other)
    }
}

impl core::ops::Add for &BigInt {
    type Output = BigInt;
    fn add(self, other: &BigInt) -> BigInt {
        self.plus(other)
    }
}

impl core::ops::Sub for BigInt {
    type Output = BigInt;
    fn sub(self, other: BigInt) -> BigInt {
        self.minus(&other)
    }
}

impl core::ops::Sub<&BigInt> for BigInt {
    type Output = BigInt;
    fn sub(self, other: &BigInt) -> BigInt {
        self.minus(other)
    }
}

impl core::ops::Sub for &BigInt {
    type Output = BigInt;
    fn sub(self, other: &BigInt) -> BigInt {
        self.minus(other)
    }
}

impl core::ops::Mul for BigInt {
    type Output = BigInt;
    fn mul(self, other: BigInt) -> BigInt {
        self.times(&other)
    }
}

impl core::ops::Mul<&BigInt> for BigInt {
    type Output = BigInt;
    fn mul(self, other: &BigInt) -> BigInt {
        self.times(other)
    }
}

impl core::ops::Mul for &BigInt {
    type Output = BigInt;
    fn mul(self, other: &BigInt) -> BigInt {
        self.times(other)
    }
}

impl core::ops::Div for BigInt {
    type Output = BigInt;
    fn div(self, other: BigInt) -> BigInt {
        self.divided_by(&other)
    }
}

impl core::ops::Div<&BigInt> for BigInt {
    type Output = BigInt;
    fn div(self, other: &BigInt) -> BigInt {
        self.divided_by(other)
    }
}

impl core::ops::Div for &BigInt {
    type Output = BigInt;
    fn div(self, other: &BigInt) -> BigInt {
        self.divided_by(other)
    }
}

impl core::ops::Rem for BigInt {
    type Output = BigInt;
    fn rem(self, other: BigInt) -> BigInt {
        self.modulo(&other)
    }
}

impl core::ops::Rem<&BigInt> for BigInt {
    type Output = BigInt;
    fn rem(self, other: &BigInt) -> BigInt {
        self.modulo(other)
    }
}

impl core::ops::BitOr for BigInt {
    type Output = BigInt;
    fn bitor(self, other: BigInt) -> BigInt {
        self.bit_or(&other)
    }
}

impl core::ops::BitAnd for BigInt {
    type Output = BigInt;
    fn bitand(self, other: BigInt) -> BigInt {
        self.bit_and(&other)
    }
}

impl core::ops::Shl<u8> for BigInt {
    type Output = BigInt;
    fn shl(self, bits: u8) -> BigInt {
        self.left_shift(bits)
    }
}

impl core::ops::Shr<u8> for BigInt {
    type Output = BigInt;
    fn shr(self, bits: u8) -> BigInt {
        self.right_shift(bits)
    }
}

impl From<i32> for BigInt {
    fn from(value: i32) -> Self {
        BigInt::from_i32(value)
    }
}

impl From<u64> for BigInt {
    fn from(value: u64) -> Self {
        BigInt::from_u64(value)
    }
}

/// Arbitrary-precision decimal number.
///
/// Backed by graph-node host calls for arithmetic operations.
#[derive(Clone, Debug)]
pub struct BigDecimal {
    ptr: AscPtr<crate::asc::AscBytes>,
}

impl BigDecimal {
    /// Create a BigDecimal from an AscPtr (internal use).
    pub fn from_ptr(ptr: AscPtr<crate::asc::AscBytes>) -> Self {
        Self { ptr }
    }

    /// Get the internal pointer.
    pub fn as_ptr(&self) -> AscPtr<crate::asc::AscBytes> {
        self.ptr
    }

    /// Create a BigDecimal with value zero.
    pub fn zero() -> Self {
        Self::from_string("0")
    }

    /// Create a BigDecimal with value one.
    pub fn one() -> Self {
        Self::from_string("1")
    }

    /// Create a BigDecimal from a string representation.
    #[cfg(target_arch = "wasm32")]
    pub fn from_string(s: &str) -> Self {
        let str_ptr = crate::asc::str_to_asc(s);
        let ptr = unsafe { crate::host::big_decimal_from_string(str_ptr.as_i32()) };
        Self {
            ptr: AscPtr::new(ptr as u32),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_string(_s: &str) -> Self {
        Self {
            ptr: AscPtr::null(),
        }
    }

    /// Convert to a string representation.
    #[cfg(target_arch = "wasm32")]
    pub fn to_string(&self) -> String {
        let str_ptr = unsafe { crate::host::big_decimal_to_string(self.ptr.as_i32()) };
        crate::asc::asc_to_string(AscPtr::new(str_ptr as u32))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn to_string(&self) -> String {
        String::from("0")
    }
}

impl PartialEq for BigDecimal {
    #[cfg(target_arch = "wasm32")]
    fn eq(&self, other: &Self) -> bool {
        unsafe { crate::host::big_decimal_equals(self.ptr.as_i32(), other.ptr.as_i32()) != 0 }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl Eq for BigDecimal {}

// ============================================================================
// BigDecimal Arithmetic Operations
// ============================================================================

impl BigDecimal {
    /// Add two BigDecimals.
    #[cfg(target_arch = "wasm32")]
    pub fn plus(&self, other: &BigDecimal) -> BigDecimal {
        let result = unsafe { crate::host::big_decimal_plus(self.ptr.as_i32(), other.ptr.as_i32()) };
        BigDecimal::from_ptr(AscPtr::new(result as u32))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn plus(&self, _other: &BigDecimal) -> BigDecimal {
        BigDecimal::zero()
    }

    /// Subtract two BigDecimals.
    #[cfg(target_arch = "wasm32")]
    pub fn minus(&self, other: &BigDecimal) -> BigDecimal {
        let result = unsafe { crate::host::big_decimal_minus(self.ptr.as_i32(), other.ptr.as_i32()) };
        BigDecimal::from_ptr(AscPtr::new(result as u32))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn minus(&self, _other: &BigDecimal) -> BigDecimal {
        BigDecimal::zero()
    }

    /// Multiply two BigDecimals.
    #[cfg(target_arch = "wasm32")]
    pub fn times(&self, other: &BigDecimal) -> BigDecimal {
        let result = unsafe { crate::host::big_decimal_times(self.ptr.as_i32(), other.ptr.as_i32()) };
        BigDecimal::from_ptr(AscPtr::new(result as u32))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn times(&self, _other: &BigDecimal) -> BigDecimal {
        BigDecimal::zero()
    }

    /// Divide two BigDecimals.
    #[cfg(target_arch = "wasm32")]
    pub fn divided_by(&self, other: &BigDecimal) -> BigDecimal {
        let result = unsafe { crate::host::big_decimal_divided_by(self.ptr.as_i32(), other.ptr.as_i32()) };
        BigDecimal::from_ptr(AscPtr::new(result as u32))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn divided_by(&self, _other: &BigDecimal) -> BigDecimal {
        BigDecimal::zero()
    }

    /// Check if this BigDecimal is zero.
    pub fn is_zero(&self) -> bool {
        *self == BigDecimal::zero()
    }

    /// Create a BigDecimal from a BigInt.
    #[cfg(target_arch = "wasm32")]
    pub fn from_big_int(value: &BigInt) -> BigDecimal {
        // Convert BigInt to string, then parse as BigDecimal
        let s = value.to_string();
        BigDecimal::from_string(&s)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_big_int(_value: &BigInt) -> BigDecimal {
        BigDecimal::zero()
    }
}

// Rust operator trait implementations for BigDecimal
impl core::ops::Add for BigDecimal {
    type Output = BigDecimal;
    fn add(self, other: BigDecimal) -> BigDecimal {
        self.plus(&other)
    }
}

impl core::ops::Add<&BigDecimal> for BigDecimal {
    type Output = BigDecimal;
    fn add(self, other: &BigDecimal) -> BigDecimal {
        self.plus(other)
    }
}

impl core::ops::Add for &BigDecimal {
    type Output = BigDecimal;
    fn add(self, other: &BigDecimal) -> BigDecimal {
        self.plus(other)
    }
}

impl core::ops::Sub for BigDecimal {
    type Output = BigDecimal;
    fn sub(self, other: BigDecimal) -> BigDecimal {
        self.minus(&other)
    }
}

impl core::ops::Sub<&BigDecimal> for BigDecimal {
    type Output = BigDecimal;
    fn sub(self, other: &BigDecimal) -> BigDecimal {
        self.minus(other)
    }
}

impl core::ops::Sub for &BigDecimal {
    type Output = BigDecimal;
    fn sub(self, other: &BigDecimal) -> BigDecimal {
        self.minus(other)
    }
}

impl core::ops::Mul for BigDecimal {
    type Output = BigDecimal;
    fn mul(self, other: BigDecimal) -> BigDecimal {
        self.times(&other)
    }
}

impl core::ops::Mul<&BigDecimal> for BigDecimal {
    type Output = BigDecimal;
    fn mul(self, other: &BigDecimal) -> BigDecimal {
        self.times(other)
    }
}

impl core::ops::Mul for &BigDecimal {
    type Output = BigDecimal;
    fn mul(self, other: &BigDecimal) -> BigDecimal {
        self.times(other)
    }
}

impl core::ops::Div for BigDecimal {
    type Output = BigDecimal;
    fn div(self, other: BigDecimal) -> BigDecimal {
        self.divided_by(&other)
    }
}

impl core::ops::Div<&BigDecimal> for BigDecimal {
    type Output = BigDecimal;
    fn div(self, other: &BigDecimal) -> BigDecimal {
        self.divided_by(other)
    }
}

impl core::ops::Div for &BigDecimal {
    type Output = BigDecimal;
    fn div(self, other: &BigDecimal) -> BigDecimal {
        self.divided_by(other)
    }
}

impl From<&str> for BigDecimal {
    fn from(s: &str) -> Self {
        BigDecimal::from_string(s)
    }
}

impl From<&BigInt> for BigDecimal {
    fn from(value: &BigInt) -> Self {
        BigDecimal::from_big_int(value)
    }
}

/// A value that can be stored in an entity field.
#[derive(Clone, Debug)]
pub enum Value {
    String(String),
    Int(i32),
    Int8(i64),
    BigInt(BigInt),
    BigDecimal(BigDecimal),
    Bool(bool),
    Bytes(Bytes),
    Array(Vec<Value>),
    Null,
}

impl Value {
    /// Try to get as a string reference.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get as bytes.
    pub fn as_bytes(&self) -> Option<&Bytes> {
        match self {
            Value::Bytes(b) => Some(b),
            _ => None,
        }
    }

    /// Try to get as BigInt.
    pub fn as_big_int(&self) -> Option<&BigInt> {
        match self {
            Value::BigInt(bi) => Some(bi),
            _ => None,
        }
    }

    /// Try to get as BigDecimal.
    pub fn as_big_decimal(&self) -> Option<&BigDecimal> {
        match self {
            Value::BigDecimal(bd) => Some(bd),
            _ => None,
        }
    }

    /// Check if this value is null.
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
}

/// Entity data storage — a map of field names to values.
#[derive(Clone, Debug, Default)]
pub struct EntityData {
    fields: BTreeMap<String, Value>,
}

impl EntityData {
    /// Create a new empty entity data container.
    pub fn new() -> Self {
        Self {
            fields: BTreeMap::new(),
        }
    }

    /// Set a field value.
    pub fn set(&mut self, key: impl Into<String>, value: Value) {
        self.fields.insert(key.into(), value);
    }

    /// Get a field value.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.fields.get(key)
    }

    /// Get a string field or panic.
    pub fn get_string(&self, key: &str) -> &str {
        self.get(key)
            .and_then(|v| v.as_string())
            .expect("expected string field")
    }

    /// Get a bytes field or panic.
    pub fn get_bytes(&self, key: &str) -> Bytes {
        self.get(key)
            .and_then(|v| v.as_bytes())
            .cloned()
            .expect("expected bytes field")
    }

    /// Get a BigInt field or panic.
    pub fn get_bigint(&self, key: &str) -> BigInt {
        self.get(key)
            .and_then(|v| v.as_big_int())
            .cloned()
            .expect("expected bigint field")
    }

    /// Get a BigDecimal field or panic.
    pub fn get_big_decimal(&self, key: &str) -> BigDecimal {
        self.get(key)
            .and_then(|v| v.as_big_decimal())
            .cloned()
            .expect("expected bigdecimal field")
    }

    /// Get an optional string field.
    pub fn get_string_opt(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|v| v.as_string())
    }

    /// Iterate over all fields.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.fields.iter()
    }
}

/// Trait that all generated entity types implement.
pub trait Entity: Sized {
    /// The entity type name as it appears in the GraphQL schema.
    const ENTITY_TYPE: &'static str;

    /// Get the entity's ID.
    fn id(&self) -> &str;

    /// Save the entity to the store.
    fn save(&self);

    /// Load an entity from the store by ID.
    fn load(id: &str) -> Option<Self>;

    /// Remove an entity from the store.
    fn remove(id: &str);
}
