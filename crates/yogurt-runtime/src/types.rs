//! Core types for yogurt subgraph mappings.
//!
//! These are the Rust equivalents of graph-ts types like `Address`, `BigInt`,
//! `BigDecimal`, `Bytes`, and `Entity`.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

#[cfg(target_arch = "wasm32")]
use crate::asc::AscPtr;

use alloc::string::ToString;

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

    /// Check if this is the zero address.
    ///
    /// The zero address (0x0000...0000) is commonly used to represent:
    /// - Token mints (transfer from zero)
    /// - Token burns (transfer to zero)
    /// - Uninitialized addresses
    ///
    /// # Example
    ///
    /// ```ignore
    /// if event.params.from.is_zero() {
    ///     // This is a mint event
    /// }
    /// if event.params.to.is_zero() {
    ///     // This is a burn event
    /// }
    /// ```
    pub fn is_zero(&self) -> bool {
        self.0 == [0u8; 20]
    }

    /// Parse an address from a hex string (with or without 0x prefix).
    ///
    /// Returns `None` if the string is not a valid 20-byte hex address.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let addr = Address::from_string("0xdead000000000000000000000000000000000000").unwrap();
    /// ```
    pub fn from_string(hex: &str) -> Option<Self> {
        let bytes = hex_to_bytes(hex)?;
        if bytes.len() != 20 {
            return None;
        }
        let mut arr = [0u8; 20];
        arr.copy_from_slice(&bytes);
        Some(Self(arr))
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

    /// Create from a hex string (with or without 0x prefix).
    ///
    /// Returns `None` if the string is not valid hex.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let bytes = Bytes::from_hex_string("0xdeadbeef").unwrap();
    /// assert_eq!(bytes.as_slice(), &[0xde, 0xad, 0xbe, 0xef]);
    /// ```
    pub fn from_hex_string(hex: &str) -> Option<Self> {
        hex_to_bytes(hex).map(Self)
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

    /// Reverse the bytes (for endianness conversion).
    pub fn reverse(&self) -> Self {
        let mut reversed = self.0.clone();
        reversed.reverse();
        Self(reversed)
    }

    /// Get the length in bytes.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Concatenate this byte array with another.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let a = Bytes::from_vec(vec![0x01, 0x02]);
    /// let b = Bytes::from_vec(vec![0x03, 0x04]);
    /// let c = a.concat(&b);
    /// assert_eq!(c.as_slice(), &[0x01, 0x02, 0x03, 0x04]);
    /// ```
    pub fn concat(&self, other: &Bytes) -> Bytes {
        let mut result = Vec::with_capacity(self.0.len() + other.0.len());
        result.extend_from_slice(&self.0);
        result.extend_from_slice(&other.0);
        Bytes(result)
    }

    /// Concatenate this byte array with an i32 (appends 4 bytes, big-endian).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let bytes = Bytes::from_vec(vec![0x01]);
    /// let result = bytes.concat_i32(256);
    /// assert_eq!(result.as_slice(), &[0x01, 0x00, 0x00, 0x01, 0x00]);
    /// ```
    pub fn concat_i32(&self, value: i32) -> Bytes {
        let mut result = Vec::with_capacity(self.0.len() + 4);
        result.extend_from_slice(&self.0);
        result.extend_from_slice(&value.to_be_bytes());
        Bytes(result)
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

impl From<Address> for Bytes {
    fn from(addr: Address) -> Self {
        Self(addr.0.to_vec())
    }
}

impl From<&Address> for Bytes {
    fn from(addr: &Address) -> Self {
        Self(addr.0.to_vec())
    }
}

impl From<[u8; 20]> for Bytes {
    fn from(arr: [u8; 20]) -> Self {
        Self(arr.to_vec())
    }
}

impl From<[u8; 32]> for Bytes {
    fn from(arr: [u8; 32]) -> Self {
        Self(arr.to_vec())
    }
}

const HEX_CHARS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

/// Parse a hex character to its numeric value.
fn hex_char_to_nibble(c: char) -> Option<u8> {
    match c {
        '0'..='9' => Some(c as u8 - b'0'),
        'a'..='f' => Some(c as u8 - b'a' + 10),
        'A'..='F' => Some(c as u8 - b'A' + 10),
        _ => None,
    }
}

/// Parse a hex string to bytes.
fn hex_to_bytes(hex: &str) -> Option<Vec<u8>> {
    let hex = hex.strip_prefix("0x").unwrap_or(hex);
    let hex = hex.strip_prefix("0X").unwrap_or(hex);

    if hex.len() % 2 != 0 {
        return None;
    }

    let mut bytes = Vec::with_capacity(hex.len() / 2);
    let chars: Vec<char> = hex.chars().collect();

    for chunk in chars.chunks(2) {
        let high = hex_char_to_nibble(chunk[0])?;
        let low = hex_char_to_nibble(chunk[1])?;
        bytes.push((high << 4) | low);
    }

    Some(bytes)
}

// ============================================================================
// BigInt - WASM implementation (backed by host calls)
// ============================================================================

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Debug)]
pub struct BigInt {
    ptr: AscPtr<crate::asc::AscBytes>,
}

#[cfg(target_arch = "wasm32")]
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
    pub fn from_i32(value: i32) -> Self {
        let bytes = value.to_le_bytes();
        let ptr = crate::asc::bytes_to_asc(&bytes);
        Self {
            ptr: AscPtr::new(ptr.as_raw()),
        }
    }

    /// Create a BigInt from a u64.
    pub fn from_u64(value: u64) -> Self {
        let bytes = value.to_le_bytes();
        let ptr = crate::asc::bytes_to_asc(&bytes);
        Self {
            ptr: AscPtr::new(ptr.as_raw()),
        }
    }

    /// Create a BigInt from an i64.
    pub fn from_i64(value: i64) -> Self {
        let bytes = value.to_le_bytes();
        let ptr = crate::asc::bytes_to_asc(&bytes);
        Self {
            ptr: AscPtr::new(ptr.as_raw()),
        }
    }

    /// Convert to a decimal string representation.
    pub fn to_string(&self) -> String {
        let str_ptr = unsafe { crate::host::big_int_to_string(self.ptr.as_i32()) };
        crate::asc::asc_to_string(AscPtr::new(str_ptr as u32))
    }

    /// Parse a BigInt from a decimal string.
    pub fn from_string(s: &str) -> Option<Self> {
        let str_ptr = crate::asc::str_to_asc(s);
        let result_ptr = unsafe { crate::host::big_int_from_string(str_ptr.as_i32()) };
        if result_ptr == 0 {
            None
        } else {
            Some(Self {
                ptr: AscPtr::new(result_ptr as u32),
            })
        }
    }

    /// Create a BigInt from signed bytes (little-endian, two's complement).
    pub fn from_signed_bytes(bytes: &[u8]) -> Self {
        // graph-node expects signed bytes in little-endian two's complement
        let ptr = crate::asc::bytes_to_asc(bytes);
        Self {
            ptr: AscPtr::new(ptr.as_raw()),
        }
    }

    /// Create a BigInt from unsigned bytes (little-endian).
    pub fn from_unsigned_bytes(bytes: &[u8]) -> Self {
        // For unsigned, we might need to add a leading zero byte
        // to prevent interpretation as negative
        if bytes.is_empty() {
            return Self::zero();
        }

        // If high bit is set in the last byte (most significant in LE),
        // add a zero byte to keep it positive
        if bytes.last().map_or(false, |b| (*b & 0x80) != 0) {
            let mut extended = bytes.to_vec();
            extended.push(0x00);
            let ptr = crate::asc::bytes_to_asc(&extended);
            Self {
                ptr: AscPtr::new(ptr.as_raw()),
            }
        } else {
            let ptr = crate::asc::bytes_to_asc(bytes);
            Self {
                ptr: AscPtr::new(ptr.as_raw()),
            }
        }
    }

    /// Convert to signed bytes (little-endian, two's complement).
    pub fn to_signed_bytes(&self) -> Vec<u8> {
        crate::asc::asc_to_bytes(self.ptr)
    }

    /// Convert to unsigned bytes (little-endian).
    pub fn to_unsigned_bytes(&self) -> Vec<u8> {
        crate::asc::asc_to_bytes(self.ptr)
    }
}

#[cfg(target_arch = "wasm32")]
impl PartialEq for BigInt {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

#[cfg(target_arch = "wasm32")]
impl Eq for BigInt {}

// ============================================================================
// BigInt - Native implementation (backed by num-bigint)
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Debug)]
pub struct BigInt {
    inner: num_bigint::BigInt,
}

#[cfg(not(target_arch = "wasm32"))]
impl BigInt {
    /// Create a BigInt from the native inner type (for testing).
    pub fn from_inner(inner: num_bigint::BigInt) -> Self {
        Self { inner }
    }

    /// Get a reference to the inner num_bigint::BigInt.
    pub fn inner(&self) -> &num_bigint::BigInt {
        &self.inner
    }

    /// Create a BigInt with value zero.
    pub fn zero() -> Self {
        Self {
            inner: num_bigint::BigInt::from(0),
        }
    }

    /// Create a BigInt with value one.
    pub fn one() -> Self {
        Self {
            inner: num_bigint::BigInt::from(1),
        }
    }

    /// Create a BigInt from an i32.
    pub fn from_i32(value: i32) -> Self {
        Self {
            inner: num_bigint::BigInt::from(value),
        }
    }

    /// Create a BigInt from a u64.
    pub fn from_u64(value: u64) -> Self {
        Self {
            inner: num_bigint::BigInt::from(value),
        }
    }

    /// Create a BigInt from an i64.
    pub fn from_i64(value: i64) -> Self {
        Self {
            inner: num_bigint::BigInt::from(value),
        }
    }

    /// Convert to a decimal string representation.
    pub fn to_string(&self) -> String {
        self.inner.to_string()
    }

    /// Parse a BigInt from a decimal string.
    pub fn from_string(s: &str) -> Option<Self> {
        s.parse::<num_bigint::BigInt>()
            .ok()
            .map(|inner| Self { inner })
    }

    /// Create a BigInt from signed bytes (little-endian, two's complement).
    ///
    /// This interprets the bytes as a signed integer.
    pub fn from_signed_bytes(bytes: &[u8]) -> Self {
        use num_bigint::Sign;

        if bytes.is_empty() {
            return Self::zero();
        }

        // Convert from little-endian to big-endian for num-bigint
        let mut be_bytes: Vec<u8> = bytes.iter().rev().copied().collect();

        // Check if negative (high bit set in most significant byte)
        let is_negative = !be_bytes.is_empty() && (be_bytes[0] & 0x80) != 0;

        if is_negative {
            // Two's complement: invert all bits and add 1
            let mut carry = true;
            for byte in be_bytes.iter_mut().rev() {
                *byte = !*byte;
                if carry {
                    let (new_val, overflow) = byte.overflowing_add(1);
                    *byte = new_val;
                    carry = overflow;
                }
            }
            Self {
                inner: num_bigint::BigInt::from_bytes_be(Sign::Minus, &be_bytes),
            }
        } else {
            Self {
                inner: num_bigint::BigInt::from_bytes_be(Sign::Plus, &be_bytes),
            }
        }
    }

    /// Create a BigInt from unsigned bytes (little-endian).
    ///
    /// This interprets the bytes as an unsigned integer.
    pub fn from_unsigned_bytes(bytes: &[u8]) -> Self {
        use num_bigint::Sign;

        if bytes.is_empty() {
            return Self::zero();
        }

        // Convert from little-endian to big-endian for num-bigint
        let be_bytes: Vec<u8> = bytes.iter().rev().copied().collect();
        Self {
            inner: num_bigint::BigInt::from_bytes_be(Sign::Plus, &be_bytes),
        }
    }

    /// Convert to signed bytes (little-endian, two's complement).
    pub fn to_signed_bytes(&self) -> Vec<u8> {
        let (sign, be_bytes) = self.inner.to_bytes_be();

        if be_bytes.is_empty() || (be_bytes.len() == 1 && be_bytes[0] == 0) {
            return alloc::vec![0u8];
        }

        let mut le_bytes: Vec<u8> = be_bytes.into_iter().rev().collect();

        if sign == num_bigint::Sign::Minus {
            // Two's complement for negative numbers
            let mut carry = true;
            for byte in le_bytes.iter_mut() {
                *byte = !*byte;
                if carry {
                    let (new_val, overflow) = byte.overflowing_add(1);
                    *byte = new_val;
                    carry = overflow;
                }
            }
            // Extend with 0xFF if needed to preserve sign
            if le_bytes.last().map_or(false, |b| (*b & 0x80) == 0) {
                le_bytes.push(0xFF);
            }
        } else {
            // Extend with 0x00 if high bit is set (would be interpreted as negative)
            if le_bytes.last().map_or(false, |b| (*b & 0x80) != 0) {
                le_bytes.push(0x00);
            }
        }

        le_bytes
    }

    /// Convert to unsigned bytes (little-endian).
    pub fn to_unsigned_bytes(&self) -> Vec<u8> {
        let (_, be_bytes) = self.inner.to_bytes_be();

        if be_bytes.is_empty() {
            return alloc::vec![0u8];
        }

        be_bytes.into_iter().rev().collect()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl PartialEq for BigInt {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Eq for BigInt {}

// ============================================================================
// BigInt Arithmetic Operations - WASM
// ============================================================================

#[cfg(target_arch = "wasm32")]
impl BigInt {
    /// Add two BigInts.
    pub fn plus(&self, other: &BigInt) -> BigInt {
        let result = unsafe { crate::host::big_int_plus(self.ptr.as_i32(), other.ptr.as_i32()) };
        BigInt::from_ptr(AscPtr::new(result as u32))
    }

    /// Subtract two BigInts.
    pub fn minus(&self, other: &BigInt) -> BigInt {
        let result = unsafe { crate::host::big_int_minus(self.ptr.as_i32(), other.ptr.as_i32()) };
        BigInt::from_ptr(AscPtr::new(result as u32))
    }

    /// Multiply two BigInts.
    pub fn times(&self, other: &BigInt) -> BigInt {
        let result = unsafe { crate::host::big_int_times(self.ptr.as_i32(), other.ptr.as_i32()) };
        BigInt::from_ptr(AscPtr::new(result as u32))
    }

    /// Divide two BigInts (integer division).
    pub fn divided_by(&self, other: &BigInt) -> BigInt {
        let result = unsafe { crate::host::big_int_divided_by(self.ptr.as_i32(), other.ptr.as_i32()) };
        BigInt::from_ptr(AscPtr::new(result as u32))
    }

    /// Modulo operation.
    pub fn modulo(&self, other: &BigInt) -> BigInt {
        let result = unsafe { crate::host::big_int_mod(self.ptr.as_i32(), other.ptr.as_i32()) };
        BigInt::from_ptr(AscPtr::new(result as u32))
    }

    /// Raise to a power.
    pub fn pow(&self, exp: u8) -> BigInt {
        let result = unsafe { crate::host::big_int_pow(self.ptr.as_i32(), exp as i32) };
        BigInt::from_ptr(AscPtr::new(result as u32))
    }

    /// Bitwise OR.
    pub fn bit_or(&self, other: &BigInt) -> BigInt {
        let result = unsafe { crate::host::big_int_bit_or(self.ptr.as_i32(), other.ptr.as_i32()) };
        BigInt::from_ptr(AscPtr::new(result as u32))
    }

    /// Bitwise AND.
    pub fn bit_and(&self, other: &BigInt) -> BigInt {
        let result = unsafe { crate::host::big_int_bit_and(self.ptr.as_i32(), other.ptr.as_i32()) };
        BigInt::from_ptr(AscPtr::new(result as u32))
    }

    /// Left shift.
    pub fn left_shift(&self, bits: u8) -> BigInt {
        let result = unsafe { crate::host::big_int_left_shift(self.ptr.as_i32(), bits as i32) };
        BigInt::from_ptr(AscPtr::new(result as u32))
    }

    /// Right shift.
    pub fn right_shift(&self, bits: u8) -> BigInt {
        let result = unsafe { crate::host::big_int_right_shift(self.ptr.as_i32(), bits as i32) };
        BigInt::from_ptr(AscPtr::new(result as u32))
    }

    /// Check if this BigInt is zero.
    pub fn is_zero(&self) -> bool {
        *self == BigInt::zero()
    }

    /// Divide two BigInts, returning zero if the divisor is zero.
    ///
    /// This is useful in DeFi calculations where division by zero
    /// should result in zero rather than a panic.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let price = reserve1.safe_div(&reserve0);  // Returns zero if reserve0 is zero
    /// ```
    pub fn safe_div(&self, other: &BigInt) -> BigInt {
        if other.is_zero() {
            BigInt::zero()
        } else {
            self.divided_by(other)
        }
    }

    /// Convert to hex string.
    pub fn to_hex(&self) -> String {
        let str_ptr = unsafe { crate::host::big_int_to_hex(self.ptr.as_i32()) };
        crate::asc::asc_to_string(AscPtr::new(str_ptr as u32))
    }

    /// Compute the integer square root using Newton's method.
    ///
    /// Returns the largest integer `r` such that `r * r <= self`.
    /// Panics if `self` is negative.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let n = BigInt::from_i32(16);
    /// assert_eq!(n.sqrt(), BigInt::from_i32(4));
    ///
    /// let n = BigInt::from_i32(17);
    /// assert_eq!(n.sqrt(), BigInt::from_i32(4)); // floor(sqrt(17)) = 4
    /// ```
    pub fn sqrt(&self) -> BigInt {
        let zero = BigInt::zero();
        let one = BigInt::one();
        let two = BigInt::from_i32(2);

        if self.lt(&zero) {
            panic!("sqrt of negative number");
        }
        if self.lt(&two) {
            return self.clone();
        }

        // Newton's method: x_{n+1} = (x_n + n/x_n) / 2
        let mut x = self.clone();
        let mut y = x.plus(&one).divided_by(&two);

        while y.lt(&x) {
            x = y.clone();
            y = x.plus(&self.divided_by(&x)).divided_by(&two);
        }

        x
    }

    /// Format this BigInt as a decimal string with the given number of decimal places.
    ///
    /// This is commonly used to convert raw token amounts (in wei) to human-readable
    /// format (in ETH/tokens).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let wei = BigInt::from_string("1500000000000000000").unwrap();  // 1.5 ETH in wei
    /// let eth = wei.to_decimals(18);  // "1.5"
    ///
    /// let usdc = BigInt::from_string("1500000").unwrap();  // 1.5 USDC (6 decimals)
    /// let formatted = usdc.to_decimals(6);  // "1.5"
    /// ```
    pub fn to_decimals(&self, decimals: u8) -> String {
        if decimals == 0 {
            return self.to_string();
        }

        let s = self.to_string();
        let is_negative = s.starts_with('-');
        let s = if is_negative { &s[1..] } else { &s };
        let decimals = decimals as usize;

        let result = if s.len() <= decimals {
            // Need leading zeros: "123" with 6 decimals -> "0.000123"
            let zeros = "0".repeat(decimals - s.len());
            let fractional = alloc::format!("{}{}", zeros, s);
            let trimmed = fractional.trim_end_matches('0');
            if trimmed.is_empty() {
                "0".to_string()
            } else {
                alloc::format!("0.{}", trimmed)
            }
        } else {
            // Split: "1500000000000000000" with 18 decimals -> "1.5"
            let split_pos = s.len() - decimals;
            let integer_part = &s[..split_pos];
            let fractional_part = s[split_pos..].trim_end_matches('0');

            if fractional_part.is_empty() {
                integer_part.to_string()
            } else {
                alloc::format!("{}.{}", integer_part, fractional_part)
            }
        };

        if is_negative && result != "0" {
            alloc::format!("-{}", result)
        } else {
            result
        }
    }
}

/// Parse a decimal string into a BigInt with the given number of decimal places.
///
/// This is commonly used to convert human-readable token amounts to raw amounts.
///
/// # Example
///
/// ```ignore
/// let eth = parse_units("1.5", 18);  // 1500000000000000000 (wei)
/// let usdc = parse_units("1.5", 6);  // 1500000 (USDC smallest unit)
/// ```
#[cfg(target_arch = "wasm32")]
pub fn parse_units(value: &str, decimals: u8) -> BigInt {
    let value = value.trim();
    let is_negative = value.starts_with('-');
    let value = if is_negative {
        &value[1..]
    } else {
        value
    };

    let (integer_part, fractional_part) = if let Some(dot_pos) = value.find('.') {
        (&value[..dot_pos], &value[dot_pos + 1..])
    } else {
        (value, "")
    };

    // Pad or truncate fractional part to match decimals
    let decimals = decimals as usize;
    let fractional_padded = if fractional_part.len() >= decimals {
        &fractional_part[..decimals]
    } else {
        &alloc::format!("{}{}", fractional_part, "0".repeat(decimals - fractional_part.len()))
    };

    let combined = alloc::format!("{}{}", integer_part, fractional_padded);
    let result = BigInt::from_string(&combined).unwrap_or_else(BigInt::zero);

    if is_negative {
        BigInt::zero().minus(&result)
    } else {
        result
    }
}

// ============================================================================
// BigInt Arithmetic Operations - Native
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
impl BigInt {
    /// Add two BigInts.
    pub fn plus(&self, other: &BigInt) -> BigInt {
        BigInt {
            inner: &self.inner + &other.inner,
        }
    }

    /// Subtract two BigInts.
    pub fn minus(&self, other: &BigInt) -> BigInt {
        BigInt {
            inner: &self.inner - &other.inner,
        }
    }

    /// Multiply two BigInts.
    pub fn times(&self, other: &BigInt) -> BigInt {
        BigInt {
            inner: &self.inner * &other.inner,
        }
    }

    /// Divide two BigInts (integer division).
    pub fn divided_by(&self, other: &BigInt) -> BigInt {
        BigInt {
            inner: &self.inner / &other.inner,
        }
    }

    /// Modulo operation.
    pub fn modulo(&self, other: &BigInt) -> BigInt {
        BigInt {
            inner: &self.inner % &other.inner,
        }
    }

    /// Raise to a power.
    pub fn pow(&self, exp: u8) -> BigInt {
        use num_traits::Pow;
        BigInt {
            inner: Pow::pow(&self.inner, exp as usize),
        }
    }

    /// Bitwise OR.
    pub fn bit_or(&self, other: &BigInt) -> BigInt {
        BigInt {
            inner: &self.inner | &other.inner,
        }
    }

    /// Bitwise AND.
    pub fn bit_and(&self, other: &BigInt) -> BigInt {
        BigInt {
            inner: &self.inner & &other.inner,
        }
    }

    /// Left shift.
    pub fn left_shift(&self, bits: u8) -> BigInt {
        BigInt {
            inner: &self.inner << (bits as usize),
        }
    }

    /// Right shift.
    pub fn right_shift(&self, bits: u8) -> BigInt {
        BigInt {
            inner: &self.inner >> (bits as usize),
        }
    }

    /// Check if this BigInt is zero.
    pub fn is_zero(&self) -> bool {
        use num_traits::Zero;
        self.inner.is_zero()
    }

    /// Divide two BigInts, returning zero if the divisor is zero.
    ///
    /// This is useful in DeFi calculations where division by zero
    /// should result in zero rather than a panic.
    pub fn safe_div(&self, other: &BigInt) -> BigInt {
        if other.is_zero() {
            BigInt::zero()
        } else {
            self.divided_by(other)
        }
    }

    /// Convert to hex string.
    pub fn to_hex(&self) -> String {
        use num_traits::Signed;
        if self.inner.is_negative() {
            alloc::format!("-0x{:x}", self.inner.magnitude())
        } else {
            alloc::format!("0x{:x}", self.inner.magnitude())
        }
    }

    /// Compute the integer square root.
    ///
    /// Returns the largest integer `r` such that `r * r <= self`.
    /// Panics if `self` is negative.
    pub fn sqrt(&self) -> BigInt {
        use num_integer::Roots;
        use num_traits::Signed;

        if self.inner.is_negative() {
            panic!("sqrt of negative number");
        }

        BigInt {
            inner: self.inner.sqrt(),
        }
    }

    /// Format this BigInt as a decimal string with the given number of decimal places.
    ///
    /// This is commonly used to convert raw token amounts (in wei) to human-readable
    /// format (in ETH/tokens).
    pub fn to_decimals(&self, decimals: u8) -> String {
        use num_traits::Signed;

        if decimals == 0 {
            return self.to_string();
        }

        let is_negative = self.inner.is_negative();
        let s = self.inner.magnitude().to_string();
        let decimals = decimals as usize;

        let result = if s.len() <= decimals {
            // Need leading zeros: "123" with 6 decimals -> "0.000123"
            let zeros = "0".repeat(decimals - s.len());
            let fractional = alloc::format!("{}{}", zeros, s);
            let trimmed = fractional.trim_end_matches('0');
            if trimmed.is_empty() {
                "0".to_string()
            } else {
                alloc::format!("0.{}", trimmed)
            }
        } else {
            // Split: "1500000000000000000" with 18 decimals -> "1.5"
            let split_pos = s.len() - decimals;
            let integer_part = &s[..split_pos];
            let fractional_part = s[split_pos..].trim_end_matches('0');

            if fractional_part.is_empty() {
                integer_part.to_string()
            } else {
                alloc::format!("{}.{}", integer_part, fractional_part)
            }
        };

        if is_negative && result != "0" {
            alloc::format!("-{}", result)
        } else {
            result
        }
    }
}

/// Parse a decimal string into a BigInt with the given number of decimal places.
///
/// This is commonly used to convert human-readable token amounts to raw amounts.
#[cfg(not(target_arch = "wasm32"))]
pub fn parse_units(value: &str, decimals: u8) -> BigInt {
    let value = value.trim();
    let is_negative = value.starts_with('-');
    let value = if is_negative {
        &value[1..]
    } else {
        value
    };

    let (integer_part, fractional_part) = if let Some(dot_pos) = value.find('.') {
        (&value[..dot_pos], &value[dot_pos + 1..])
    } else {
        (value, "")
    };

    // Pad or truncate fractional part to match decimals
    let decimals = decimals as usize;
    let fractional_padded = if fractional_part.len() >= decimals {
        fractional_part[..decimals].to_string()
    } else {
        alloc::format!("{}{}", fractional_part, "0".repeat(decimals - fractional_part.len()))
    };

    let combined = alloc::format!("{}{}", integer_part, fractional_padded);
    let result = BigInt::from_string(&combined).unwrap_or_else(BigInt::zero);

    if is_negative {
        BigInt::zero().minus(&result)
    } else {
        result
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

// ============================================================================
// BigInt Comparison Operations - WASM
// ============================================================================

#[cfg(target_arch = "wasm32")]
impl BigInt {
    /// Check if this BigInt is negative.
    fn is_negative(&self) -> bool {
        let bytes = self.to_signed_bytes();
        // In two's complement little-endian, check high bit of last (most significant) byte
        bytes.last().map_or(false, |b| (*b & 0x80) != 0)
    }

    /// Compare two BigInts, returning -1, 0, or 1.
    fn cmp_value(&self, other: &BigInt) -> core::cmp::Ordering {
        let diff = self.minus(other);
        if diff.is_zero() {
            core::cmp::Ordering::Equal
        } else if diff.is_negative() {
            core::cmp::Ordering::Less
        } else {
            core::cmp::Ordering::Greater
        }
    }

    /// Returns true if self < other.
    pub fn lt(&self, other: &BigInt) -> bool {
        self.cmp_value(other) == core::cmp::Ordering::Less
    }

    /// Returns true if self <= other.
    pub fn le(&self, other: &BigInt) -> bool {
        self.cmp_value(other) != core::cmp::Ordering::Greater
    }

    /// Returns true if self > other.
    pub fn gt(&self, other: &BigInt) -> bool {
        self.cmp_value(other) == core::cmp::Ordering::Greater
    }

    /// Returns true if self >= other.
    pub fn ge(&self, other: &BigInt) -> bool {
        self.cmp_value(other) != core::cmp::Ordering::Less
    }

    /// Returns the absolute value.
    pub fn abs(&self) -> BigInt {
        if self.is_negative() {
            BigInt::zero().minus(self)
        } else {
            self.clone()
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl PartialOrd for BigInt {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp_value(other))
    }
}

#[cfg(target_arch = "wasm32")]
impl Ord for BigInt {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.cmp_value(other)
    }
}

// ============================================================================
// BigInt Comparison Operations - Native
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
impl BigInt {
    /// Returns true if self < other.
    pub fn lt(&self, other: &BigInt) -> bool {
        self.inner < other.inner
    }

    /// Returns true if self <= other.
    pub fn le(&self, other: &BigInt) -> bool {
        self.inner <= other.inner
    }

    /// Returns true if self > other.
    pub fn gt(&self, other: &BigInt) -> bool {
        self.inner > other.inner
    }

    /// Returns true if self >= other.
    pub fn ge(&self, other: &BigInt) -> bool {
        self.inner >= other.inner
    }

    /// Returns the absolute value.
    pub fn abs(&self) -> BigInt {
        use num_traits::Signed;
        BigInt {
            inner: self.inner.abs(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl PartialOrd for BigInt {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.inner.cmp(&other.inner))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Ord for BigInt {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.inner.cmp(&other.inner)
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

// ============================================================================
// BigDecimal - WASM implementation (backed by host calls)
// ============================================================================

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Debug)]
pub struct BigDecimal {
    ptr: AscPtr<crate::asc::AscBytes>,
}

#[cfg(target_arch = "wasm32")]
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
    pub fn from_string(s: &str) -> Self {
        let str_ptr = crate::asc::str_to_asc(s);
        let ptr = unsafe { crate::host::big_decimal_from_string(str_ptr.as_i32()) };
        Self {
            ptr: AscPtr::new(ptr as u32),
        }
    }

    /// Convert to a string representation.
    pub fn to_string(&self) -> String {
        let str_ptr = unsafe { crate::host::big_decimal_to_string(self.ptr.as_i32()) };
        crate::asc::asc_to_string(AscPtr::new(str_ptr as u32))
    }
}

#[cfg(target_arch = "wasm32")]
impl PartialEq for BigDecimal {
    fn eq(&self, other: &Self) -> bool {
        unsafe { crate::host::big_decimal_equals(self.ptr.as_i32(), other.ptr.as_i32()) != 0 }
    }
}

#[cfg(target_arch = "wasm32")]
impl Eq for BigDecimal {}

// ============================================================================
// BigDecimal - Native implementation (BigInt + scale)
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Debug)]
pub struct BigDecimal {
    /// The unscaled value (significand).
    digits: num_bigint::BigInt,
    /// The scale (number of decimal places).
    scale: i64,
}

#[cfg(not(target_arch = "wasm32"))]
impl BigDecimal {
    /// Create a BigDecimal with value zero.
    pub fn zero() -> Self {
        Self {
            digits: num_bigint::BigInt::from(0),
            scale: 0,
        }
    }

    /// Create a BigDecimal with value one.
    pub fn one() -> Self {
        Self {
            digits: num_bigint::BigInt::from(1),
            scale: 0,
        }
    }

    /// Create a BigDecimal from a string representation.
    pub fn from_string(s: &str) -> Self {
        let s = s.trim();
        if s.is_empty() {
            return Self::zero();
        }

        // Handle negative sign
        let (is_negative, s) = if s.starts_with('-') {
            (true, &s[1..])
        } else if s.starts_with('+') {
            (false, &s[1..])
        } else {
            (false, s)
        };

        // Find decimal point
        if let Some(dot_pos) = s.find('.') {
            let integer_part = &s[..dot_pos];
            let fractional_part = &s[dot_pos + 1..];
            let scale = fractional_part.len() as i64;

            // Combine integer and fractional parts
            let combined = alloc::format!("{}{}", integer_part, fractional_part);
            let digits = combined
                .parse::<num_bigint::BigInt>()
                .unwrap_or_else(|_| num_bigint::BigInt::from(0));

            Self {
                digits: if is_negative { -digits } else { digits },
                scale,
            }
        } else {
            // No decimal point
            let digits = s
                .parse::<num_bigint::BigInt>()
                .unwrap_or_else(|_| num_bigint::BigInt::from(0));
            Self {
                digits: if is_negative { -digits } else { digits },
                scale: 0,
            }
        }
    }

    /// Convert to a string representation.
    pub fn to_string(&self) -> String {
        use num_traits::{Signed, Zero};

        if self.digits.is_zero() {
            return String::from("0");
        }

        let is_negative = self.digits.is_negative();
        let abs_digits = self.digits.magnitude().to_string();

        if self.scale <= 0 {
            // No decimal point, or we need trailing zeros
            let zeros = "0".repeat((-self.scale) as usize);
            if is_negative {
                alloc::format!("-{}{}", abs_digits, zeros)
            } else {
                alloc::format!("{}{}", abs_digits, zeros)
            }
        } else {
            let scale = self.scale as usize;
            if abs_digits.len() <= scale {
                // Need leading zeros after decimal point
                let leading_zeros = "0".repeat(scale - abs_digits.len());
                if is_negative {
                    alloc::format!("-0.{}{}", leading_zeros, abs_digits)
                } else {
                    alloc::format!("0.{}{}", leading_zeros, abs_digits)
                }
            } else {
                // Split the string
                let split_pos = abs_digits.len() - scale;
                let integer_part = &abs_digits[..split_pos];
                let fractional_part = &abs_digits[split_pos..];
                if is_negative {
                    alloc::format!("-{}.{}", integer_part, fractional_part)
                } else {
                    alloc::format!("{}.{}", integer_part, fractional_part)
                }
            }
        }
    }

    /// Get the digits (for internal use).
    pub fn digits(&self) -> &num_bigint::BigInt {
        &self.digits
    }

    /// Get the scale (for internal use).
    pub fn scale(&self) -> i64 {
        self.scale
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl PartialEq for BigDecimal {
    fn eq(&self, other: &Self) -> bool {
        // Normalize scales for comparison
        if self.scale == other.scale {
            self.digits == other.digits
        } else if self.scale < other.scale {
            let diff = (other.scale - self.scale) as u32;
            let scaled = &self.digits * num_bigint::BigInt::from(10).pow(diff);
            scaled == other.digits
        } else {
            let diff = (self.scale - other.scale) as u32;
            let scaled = &other.digits * num_bigint::BigInt::from(10).pow(diff);
            self.digits == scaled
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Eq for BigDecimal {}

// ============================================================================
// BigDecimal Arithmetic Operations - WASM
// ============================================================================

#[cfg(target_arch = "wasm32")]
impl BigDecimal {
    /// Add two BigDecimals.
    pub fn plus(&self, other: &BigDecimal) -> BigDecimal {
        let result = unsafe { crate::host::big_decimal_plus(self.ptr.as_i32(), other.ptr.as_i32()) };
        BigDecimal::from_ptr(AscPtr::new(result as u32))
    }

    /// Subtract two BigDecimals.
    pub fn minus(&self, other: &BigDecimal) -> BigDecimal {
        let result = unsafe { crate::host::big_decimal_minus(self.ptr.as_i32(), other.ptr.as_i32()) };
        BigDecimal::from_ptr(AscPtr::new(result as u32))
    }

    /// Multiply two BigDecimals.
    pub fn times(&self, other: &BigDecimal) -> BigDecimal {
        let result = unsafe { crate::host::big_decimal_times(self.ptr.as_i32(), other.ptr.as_i32()) };
        BigDecimal::from_ptr(AscPtr::new(result as u32))
    }

    /// Divide two BigDecimals.
    pub fn divided_by(&self, other: &BigDecimal) -> BigDecimal {
        let result = unsafe { crate::host::big_decimal_divided_by(self.ptr.as_i32(), other.ptr.as_i32()) };
        BigDecimal::from_ptr(AscPtr::new(result as u32))
    }

    /// Check if this BigDecimal is zero.
    pub fn is_zero(&self) -> bool {
        *self == BigDecimal::zero()
    }

    /// Divide two BigDecimals, returning zero if the divisor is zero.
    ///
    /// This is useful in DeFi calculations where division by zero
    /// should result in zero rather than a panic.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let price = token1_reserve.safe_div(&token0_reserve);
    /// ```
    pub fn safe_div(&self, other: &BigDecimal) -> BigDecimal {
        if other.is_zero() {
            BigDecimal::zero()
        } else {
            self.divided_by(other)
        }
    }

    /// Create a BigDecimal from a BigInt.
    pub fn from_big_int(value: &BigInt) -> BigDecimal {
        let s = value.to_string();
        BigDecimal::from_string(&s)
    }
}

// ============================================================================
// BigDecimal Arithmetic Operations - Native
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
impl BigDecimal {
    /// Add two BigDecimals.
    pub fn plus(&self, other: &BigDecimal) -> BigDecimal {
        // Align scales
        if self.scale == other.scale {
            BigDecimal {
                digits: &self.digits + &other.digits,
                scale: self.scale,
            }
        } else if self.scale < other.scale {
            let diff = (other.scale - self.scale) as u32;
            let scaled = &self.digits * num_bigint::BigInt::from(10).pow(diff);
            BigDecimal {
                digits: scaled + &other.digits,
                scale: other.scale,
            }
        } else {
            let diff = (self.scale - other.scale) as u32;
            let scaled = &other.digits * num_bigint::BigInt::from(10).pow(diff);
            BigDecimal {
                digits: &self.digits + scaled,
                scale: self.scale,
            }
        }
    }

    /// Subtract two BigDecimals.
    pub fn minus(&self, other: &BigDecimal) -> BigDecimal {
        // Align scales
        if self.scale == other.scale {
            BigDecimal {
                digits: &self.digits - &other.digits,
                scale: self.scale,
            }
        } else if self.scale < other.scale {
            let diff = (other.scale - self.scale) as u32;
            let scaled = &self.digits * num_bigint::BigInt::from(10).pow(diff);
            BigDecimal {
                digits: scaled - &other.digits,
                scale: other.scale,
            }
        } else {
            let diff = (self.scale - other.scale) as u32;
            let scaled = &other.digits * num_bigint::BigInt::from(10).pow(diff);
            BigDecimal {
                digits: &self.digits - scaled,
                scale: self.scale,
            }
        }
    }

    /// Multiply two BigDecimals.
    pub fn times(&self, other: &BigDecimal) -> BigDecimal {
        BigDecimal {
            digits: &self.digits * &other.digits,
            scale: self.scale + other.scale,
        }
    }

    /// Divide two BigDecimals.
    pub fn divided_by(&self, other: &BigDecimal) -> BigDecimal {
        use num_traits::Zero;

        if other.digits.is_zero() {
            panic!("Division by zero");
        }

        // Use fixed precision (18 decimal places, matching graph-node)
        let precision = 18i64;
        let scale_diff = self.scale - other.scale;
        let target_scale = precision;

        // Scale up dividend for precision
        let scale_up = target_scale - scale_diff;
        let scaled_dividend = if scale_up >= 0 {
            &self.digits * num_bigint::BigInt::from(10).pow(scale_up as u32)
        } else {
            &self.digits / num_bigint::BigInt::from(10).pow((-scale_up) as u32)
        };

        BigDecimal {
            digits: scaled_dividend / &other.digits,
            scale: target_scale,
        }
    }

    /// Check if this BigDecimal is zero.
    pub fn is_zero(&self) -> bool {
        use num_traits::Zero;
        self.digits.is_zero()
    }

    /// Divide two BigDecimals, returning zero if the divisor is zero.
    ///
    /// This is useful in DeFi calculations where division by zero
    /// should result in zero rather than a panic.
    pub fn safe_div(&self, other: &BigDecimal) -> BigDecimal {
        if other.is_zero() {
            BigDecimal::zero()
        } else {
            self.divided_by(other)
        }
    }

    /// Create a BigDecimal from a BigInt.
    pub fn from_big_int(value: &BigInt) -> BigDecimal {
        BigDecimal {
            digits: value.inner.clone(),
            scale: 0,
        }
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

// ============================================================================
// BigDecimal Comparison Operations - WASM
// ============================================================================

#[cfg(target_arch = "wasm32")]
impl BigDecimal {
    /// Check if this BigDecimal is negative by examining the string representation.
    fn is_negative(&self) -> bool {
        let s = self.to_string();
        s.starts_with('-')
    }

    /// Compare two BigDecimals, returning -1, 0, or 1.
    fn cmp_value(&self, other: &BigDecimal) -> core::cmp::Ordering {
        let diff = self.minus(other);
        if diff.is_zero() {
            core::cmp::Ordering::Equal
        } else if diff.is_negative() {
            core::cmp::Ordering::Less
        } else {
            core::cmp::Ordering::Greater
        }
    }

    /// Returns true if self < other.
    pub fn lt(&self, other: &BigDecimal) -> bool {
        self.cmp_value(other) == core::cmp::Ordering::Less
    }

    /// Returns true if self <= other.
    pub fn le(&self, other: &BigDecimal) -> bool {
        self.cmp_value(other) != core::cmp::Ordering::Greater
    }

    /// Returns true if self > other.
    pub fn gt(&self, other: &BigDecimal) -> bool {
        self.cmp_value(other) == core::cmp::Ordering::Greater
    }

    /// Returns true if self >= other.
    pub fn ge(&self, other: &BigDecimal) -> bool {
        self.cmp_value(other) != core::cmp::Ordering::Less
    }

    /// Truncate to a given number of decimal places.
    pub fn truncate(&self, decimal_places: i32) -> BigDecimal {
        // Use string manipulation: parse, truncate decimal portion
        let s = self.to_string();
        if let Some(dot_pos) = s.find('.') {
            let target_len = dot_pos + 1 + decimal_places as usize;
            if target_len < s.len() {
                let truncated = &s[..target_len];
                BigDecimal::from_string(truncated)
            } else {
                self.clone()
            }
        } else {
            self.clone()
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl PartialOrd for BigDecimal {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp_value(other))
    }
}

#[cfg(target_arch = "wasm32")]
impl Ord for BigDecimal {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.cmp_value(other)
    }
}

// ============================================================================
// BigDecimal Comparison Operations - Native
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
impl BigDecimal {
    /// Check if this BigDecimal is negative.
    fn is_negative(&self) -> bool {
        use num_traits::Signed;
        self.digits.is_negative()
    }

    /// Compare two BigDecimals by normalizing their scales.
    fn cmp_value(&self, other: &BigDecimal) -> core::cmp::Ordering {
        use num_traits::Zero;

        // Align scales for comparison
        if self.scale == other.scale {
            self.digits.cmp(&other.digits)
        } else if self.scale < other.scale {
            let diff = (other.scale - self.scale) as u32;
            let scaled = &self.digits * num_bigint::BigInt::from(10).pow(diff);
            scaled.cmp(&other.digits)
        } else {
            let diff = (self.scale - other.scale) as u32;
            let scaled = &other.digits * num_bigint::BigInt::from(10).pow(diff);
            self.digits.cmp(&scaled)
        }
    }

    /// Returns true if self < other.
    pub fn lt(&self, other: &BigDecimal) -> bool {
        self.cmp_value(other) == core::cmp::Ordering::Less
    }

    /// Returns true if self <= other.
    pub fn le(&self, other: &BigDecimal) -> bool {
        self.cmp_value(other) != core::cmp::Ordering::Greater
    }

    /// Returns true if self > other.
    pub fn gt(&self, other: &BigDecimal) -> bool {
        self.cmp_value(other) == core::cmp::Ordering::Greater
    }

    /// Returns true if self >= other.
    pub fn ge(&self, other: &BigDecimal) -> bool {
        self.cmp_value(other) != core::cmp::Ordering::Less
    }

    /// Truncate to a given number of decimal places.
    pub fn truncate(&self, decimal_places: i32) -> BigDecimal {
        use num_traits::Zero;

        if decimal_places as i64 >= self.scale {
            return self.clone();
        }

        let scale_reduction = self.scale - decimal_places as i64;
        let divisor = num_bigint::BigInt::from(10).pow(scale_reduction as u32);
        let truncated_digits = &self.digits / &divisor;

        BigDecimal {
            digits: truncated_digits,
            scale: decimal_places as i64,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl PartialOrd for BigDecimal {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp_value(other))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Ord for BigDecimal {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.cmp_value(other)
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

    /// Get an optional bytes field.
    pub fn get_bytes_opt(&self, key: &str) -> Option<Bytes> {
        self.get(key).and_then(|v| v.as_bytes()).cloned()
    }

    /// Get an optional BigInt field.
    pub fn get_bigint_opt(&self, key: &str) -> Option<BigInt> {
        self.get(key).and_then(|v| v.as_big_int()).cloned()
    }

    /// Get an optional BigDecimal field.
    pub fn get_big_decimal_opt(&self, key: &str) -> Option<BigDecimal> {
        self.get(key).and_then(|v| v.as_big_decimal()).cloned()
    }

    /// Get an optional i32 field.
    pub fn get_int_opt(&self, key: &str) -> Option<i32> {
        self.get(key).and_then(|v| match v {
            Value::Int(i) => Some(*i),
            _ => None,
        })
    }

    /// Get an optional i64 field.
    pub fn get_int8_opt(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(|v| match v {
            Value::Int8(i) => Some(*i),
            _ => None,
        })
    }

    /// Get an optional bool field.
    pub fn get_bool_opt(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(|v| match v {
            Value::Bool(b) => Some(*b),
            _ => None,
        })
    }

    /// Iterate over all fields.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.fields.iter()
    }

    /// Get an array field.
    pub fn get_array(&self, key: &str) -> Option<&Vec<Value>> {
        match self.get(key) {
            Some(Value::Array(arr)) => Some(arr),
            _ => None,
        }
    }

    /// Get a string array field.
    pub fn get_string_array(&self, key: &str) -> Vec<String> {
        self.get_array(key)
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| match v {
                        Value::String(s) => Some(s.clone()),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get a bytes array field.
    pub fn get_bytes_array(&self, key: &str) -> Vec<Bytes> {
        self.get_array(key)
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| match v {
                        Value::Bytes(b) => Some(b.clone()),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get a BigInt array field.
    pub fn get_bigint_array(&self, key: &str) -> Vec<BigInt> {
        self.get_array(key)
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| match v {
                        Value::BigInt(bi) => Some(bi.clone()),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get a BigDecimal array field.
    pub fn get_big_decimal_array(&self, key: &str) -> Vec<BigDecimal> {
        self.get_array(key)
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| match v {
                        Value::BigDecimal(bd) => Some(bd.clone()),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get an i32 array field.
    pub fn get_int_array(&self, key: &str) -> Vec<i32> {
        self.get_array(key)
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| match v {
                        Value::Int(i) => Some(*i),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get a bool array field.
    pub fn get_bool_array(&self, key: &str) -> Vec<bool> {
        self.get_array(key)
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| match v {
                        Value::Bool(b) => Some(*b),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default()
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

    /// Check if an entity exists in the store without loading it.
    ///
    /// This is more efficient than `load().is_some()` when you only need
    /// to check existence.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if !Token::exists(&token_id) {
    ///     // Create the token...
    /// }
    /// ```
    fn exists(id: &str) -> bool {
        Self::load(id).is_some()
    }

    /// Create a new entity with the given ID.
    ///
    /// This is used by `load_or_create` to construct new entities.
    /// Generated entities implement this by delegating to `new()`.
    fn create(id: impl Into<String>) -> Self;

    /// Load an entity if it exists, or create a new one with the given initializer.
    ///
    /// This is a common pattern in subgraphs when you want to ensure an entity
    /// exists and initialize it with default values if it doesn't.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let factory = Factory::load_or_create(&factory_id, |f| {
    ///     f.set_pair_count(BigInt::zero());
    ///     f.set_total_volume(BigDecimal::zero());
    /// });
    /// ```
    fn load_or_create(id: &str, init: impl FnOnce(&mut Self)) -> Self {
        match Self::load(id) {
            Some(entity) => entity,
            None => {
                let mut entity = Self::create(id);
                init(&mut entity);
                entity
            }
        }
    }

    /// Load and update an entity in one operation.
    ///
    /// If the entity exists, applies the updater function and saves it.
    /// Does nothing if the entity doesn't exist.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Increment a counter
    /// Pair::update(&pair_id, |p| {
    ///     p.set_tx_count(p.tx_count() + BigInt::from(1));
    /// });
    /// ```
    fn update(id: &str, updater: impl FnOnce(&mut Self)) {
        if let Some(mut entity) = Self::load(id) {
            updater(&mut entity);
            entity.save();
        }
    }

    /// Load and update an entity, or create it if it doesn't exist.
    ///
    /// Combines `load_or_create` and `update` - always applies the function
    /// and saves the result.
    ///
    /// # Example
    ///
    /// ```ignore
    /// Token::upsert(&token_id, |t| {
    ///     t.set_last_updated(event.block.timestamp);
    ///     t.set_tx_count(t.tx_count() + BigInt::from(1));
    /// });
    /// ```
    fn upsert(id: &str, updater: impl FnOnce(&mut Self)) {
        let mut entity = Self::load(id).unwrap_or_else(|| Self::create(id));
        updater(&mut entity);
        entity.save();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bigint_comparisons() {
        let a = BigInt::from_i32(100);
        let b = BigInt::from_i32(200);
        let c = BigInt::from_i32(100);

        // Less than
        assert!(a.lt(&b));
        assert!(!b.lt(&a));
        assert!(!a.lt(&c));

        // Greater than
        assert!(b.gt(&a));
        assert!(!a.gt(&b));
        assert!(!a.gt(&c));

        // Less than or equal
        assert!(a.le(&b));
        assert!(a.le(&c));
        assert!(!b.le(&a));

        // Greater than or equal
        assert!(b.ge(&a));
        assert!(a.ge(&c));
        assert!(!a.ge(&b));

        // Rust operators
        assert!(a < b);
        assert!(b > a);
        assert!(a <= c);
        assert!(a >= c);
    }

    #[test]
    fn test_bigint_negative_comparisons() {
        let neg = BigInt::from_i32(-50);
        let zero = BigInt::zero();
        let pos = BigInt::from_i32(50);

        assert!(neg.lt(&zero));
        assert!(neg.lt(&pos));
        assert!(zero.lt(&pos));

        assert!(pos.gt(&zero));
        assert!(pos.gt(&neg));
        assert!(zero.gt(&neg));
    }

    #[test]
    fn test_bigint_abs() {
        let pos = BigInt::from_i32(42);
        let neg = BigInt::from_i32(-42);

        assert_eq!(pos.abs(), pos);
        assert_eq!(neg.abs(), pos);
        assert_eq!(BigInt::zero().abs(), BigInt::zero());
    }

    #[test]
    fn test_bigdecimal_comparisons() {
        let a = BigDecimal::from_string("1.5");
        let b = BigDecimal::from_string("2.5");
        let c = BigDecimal::from_string("1.5");

        assert!(a.lt(&b));
        assert!(!b.lt(&a));
        assert!(!a.lt(&c));

        assert!(b.gt(&a));
        assert!(!a.gt(&b));
        assert!(!a.gt(&c));

        assert!(a.le(&b));
        assert!(a.le(&c));
        assert!(b.ge(&a));
        assert!(a.ge(&c));

        // Rust operators
        assert!(a < b);
        assert!(b > a);
    }

    #[test]
    fn test_bigdecimal_truncate() {
        let d = BigDecimal::from_string("3.14159265");

        let t2 = d.truncate(2);
        assert_eq!(t2.to_string(), "3.14");

        let t0 = d.truncate(0);
        assert_eq!(t0.to_string(), "3");
    }

    #[test]
    fn test_bytes_concat() {
        let a = Bytes::from_vec(vec![0x01, 0x02]);
        let b = Bytes::from_vec(vec![0x03, 0x04]);
        let c = a.concat(&b);

        assert_eq!(c.as_slice(), &[0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_bytes_concat_i32() {
        let bytes = Bytes::from_vec(vec![0xFF]);
        let result = bytes.concat_i32(256);

        // 256 in big-endian is 0x00, 0x00, 0x01, 0x00
        assert_eq!(result.as_slice(), &[0xFF, 0x00, 0x00, 0x01, 0x00]);
    }

    #[test]
    fn test_bigint_sqrt() {
        // Perfect squares
        assert_eq!(BigInt::from_i32(0).sqrt(), BigInt::from_i32(0));
        assert_eq!(BigInt::from_i32(1).sqrt(), BigInt::from_i32(1));
        assert_eq!(BigInt::from_i32(4).sqrt(), BigInt::from_i32(2));
        assert_eq!(BigInt::from_i32(9).sqrt(), BigInt::from_i32(3));
        assert_eq!(BigInt::from_i32(16).sqrt(), BigInt::from_i32(4));
        assert_eq!(BigInt::from_i32(100).sqrt(), BigInt::from_i32(10));

        // Non-perfect squares (floor)
        assert_eq!(BigInt::from_i32(2).sqrt(), BigInt::from_i32(1));
        assert_eq!(BigInt::from_i32(3).sqrt(), BigInt::from_i32(1));
        assert_eq!(BigInt::from_i32(5).sqrt(), BigInt::from_i32(2));
        assert_eq!(BigInt::from_i32(17).sqrt(), BigInt::from_i32(4));
        assert_eq!(BigInt::from_i32(99).sqrt(), BigInt::from_i32(9));

        // Large number
        let large = BigInt::from_string("1000000000000").unwrap();
        assert_eq!(large.sqrt(), BigInt::from_i32(1000000));
    }

    #[test]
    #[should_panic(expected = "sqrt of negative")]
    fn test_bigint_sqrt_negative_panics() {
        let neg = BigInt::from_i32(-4);
        neg.sqrt();
    }
}
