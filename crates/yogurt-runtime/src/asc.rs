//! AssemblyScript pointer types and string encoding.
//!
//! graph-node passes and receives data as pointers into WASM linear memory.
//! All strings must be UTF-16LE encoded (AssemblyScript's native format).

use alloc::string::String;
use alloc::vec::Vec;
use core::marker::PhantomData;

use crate::allocator::{asc_alloc, class_id, read_rt_size};

/// An opaque pointer into WASM linear memory with AssemblyScript-compatible layout.
///
/// The generic parameter `T` is a phantom type for type safety — it doesn't
/// affect the runtime representation (always a u32 offset).
#[repr(transparent)]
pub struct AscPtr<T> {
    offset: u32,
    _phantom: PhantomData<T>,
}

// Manual impls to avoid trait bounds on T
impl<T> Clone for AscPtr<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for AscPtr<T> {}

impl<T> core::fmt::Debug for AscPtr<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AscPtr")
            .field("offset", &self.offset)
            .finish()
    }
}

impl<T> PartialEq for AscPtr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.offset == other.offset
    }
}

impl<T> Eq for AscPtr<T> {}

unsafe impl<T> Send for AscPtr<T> {}
unsafe impl<T> Sync for AscPtr<T> {}

impl<T> AscPtr<T> {
    /// Create a new AscPtr from a raw memory offset.
    #[inline]
    pub const fn new(offset: u32) -> Self {
        Self {
            offset,
            _phantom: PhantomData,
        }
    }

    /// Create a null pointer.
    #[inline]
    pub const fn null() -> Self {
        Self::new(0)
    }

    /// Check if this pointer is null.
    #[inline]
    pub const fn is_null(&self) -> bool {
        self.offset == 0
    }

    /// Get the raw memory offset.
    #[inline]
    pub const fn as_raw(&self) -> u32 {
        self.offset
    }

    /// Get the raw offset as i32 for WASM interop.
    #[inline]
    pub const fn as_i32(&self) -> i32 {
        self.offset as i32
    }
}

/// Marker type for AssemblyScript strings (UTF-16LE encoded)
#[derive(Debug)]
pub struct AscString;

/// Marker type for AssemblyScript byte arrays (ArrayBuffer)
#[derive(Debug)]
pub struct AscBytes;

/// Marker type for AssemblyScript typed arrays
#[derive(Debug)]
pub struct AscTypedArray<T>(PhantomData<T>);

/// Marker type for AssemblyScript arrays
#[derive(Debug)]
pub struct AscArray<T>(PhantomData<T>);

/// Marker type for entity data (TypedMap<String, StoreValue>)
#[derive(Debug)]
pub struct AscEntity;

/// Marker type for TypedMapEntry<K, V>
#[derive(Debug)]
pub struct AscTypedMapEntry<K, V>(PhantomData<(K, V)>);

/// Marker type for StoreValue enum
#[derive(Debug)]
pub struct AscStoreValue;

// ============================================================================
// AssemblyScript Memory Layout Structures
// ============================================================================

/// AssemblyScript Array layout (API version 0.0.5+)
///
/// Memory layout (after 20-byte header):
/// - buffer: AscPtr<ArrayBuffer>  (4 bytes)
/// - buffer_data_start: u32       (4 bytes)
/// - buffer_data_length: u32      (4 bytes)
/// - length: i32                  (4 bytes)
#[repr(C)]
pub struct AscArrayHeader {
    pub buffer: u32,
    pub buffer_data_start: u32,
    pub buffer_data_length: u32,
    pub length: i32,
}

/// AssemblyScript TypedMap layout
///
/// Memory layout (after 20-byte header):
/// - entries: AscPtr<Array<AscPtr<TypedMapEntry>>>  (4 bytes)
#[repr(C)]
pub struct AscTypedMapHeader {
    pub entries: u32,
}

/// AssemblyScript TypedMapEntry layout
///
/// Memory layout (after 20-byte header):
/// - key: AscPtr<K>    (4 bytes)
/// - value: AscPtr<V>  (4 bytes)
#[repr(C)]
pub struct AscTypedMapEntryHeader {
    pub key: u32,
    pub value: u32,
}

/// AssemblyScript Enum layout (for StoreValue)
///
/// Memory layout (after 20-byte header):
/// - kind: i32         (4 bytes) - discriminant
/// - _padding: u32     (4 bytes) - alignment padding
/// - payload: u64      (8 bytes) - value (pointer or inline primitive)
#[repr(C)]
pub struct AscEnumHeader {
    pub kind: i32,
    pub _padding: u32,
    pub payload: u64,
}

/// StoreValue discriminant values (from graph-ts ValueKind)
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoreValueKind {
    String = 0,
    Int = 1,
    BigDecimal = 2,
    Bool = 3,
    Array = 4,
    Null = 5,
    Bytes = 6,
    BigInt = 7,
    Int8 = 8,
    Timestamp = 9,
}

/// Convert a Rust string to an AssemblyScript string in WASM memory.
///
/// AssemblyScript strings are UTF-16LE encoded with a 20-byte header.
/// Returns a pointer to the string data (after the header).
#[cfg(target_arch = "wasm32")]
pub fn str_to_asc(s: &str) -> AscPtr<AscString> {
    let utf16: Vec<u16> = s.encode_utf16().collect();
    let byte_len = utf16.len() * 2;

    let ptr = asc_alloc(byte_len as u32, class_id::STRING);

    unsafe {
        let dest = ptr as *mut u8;
        for (i, &unit) in utf16.iter().enumerate() {
            core::ptr::write_unaligned(dest.add(i * 2) as *mut u16, unit.to_le());
        }
    }

    AscPtr::new(ptr)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn str_to_asc(_s: &str) -> AscPtr<AscString> {
    panic!("str_to_asc not available on native target");
}

/// Convert an AssemblyScript string from WASM memory to a Rust String.
///
/// Reads the UTF-16LE data and converts to UTF-8.
#[cfg(target_arch = "wasm32")]
pub fn asc_to_string(ptr: AscPtr<AscString>) -> String {
    if ptr.is_null() {
        return String::new();
    }

    unsafe {
        let raw = ptr.as_raw();
        let rt_size = read_rt_size(raw);
        let len = rt_size as usize / 2;

        let mut units = Vec::with_capacity(len);
        let base = raw as *const u16;

        for i in 0..len {
            units.push(u16::from_le(core::ptr::read_unaligned(base.add(i))));
        }

        String::from_utf16_lossy(&units)
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn asc_to_string(_ptr: AscPtr<AscString>) -> String {
    panic!("asc_to_string not available on native target");
}

/// Convert a byte slice to an AssemblyScript Bytes (Uint8Array) in WASM memory.
#[cfg(target_arch = "wasm32")]
pub fn bytes_to_asc(data: &[u8]) -> AscPtr<AscBytes> {
    let len = data.len() as u32;
    let ptr = asc_alloc(len, class_id::ARRAY_BUFFER);

    unsafe {
        let dest = ptr as *mut u8;
        core::ptr::copy_nonoverlapping(data.as_ptr(), dest, data.len());
    }

    AscPtr::new(ptr)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn bytes_to_asc(_data: &[u8]) -> AscPtr<AscBytes> {
    panic!("bytes_to_asc not available on native target");
}

/// Convert an AssemblyScript Bytes from WASM memory to a Rust Vec<u8>.
#[cfg(target_arch = "wasm32")]
pub fn asc_to_bytes(ptr: AscPtr<AscBytes>) -> Vec<u8> {
    if ptr.is_null() {
        return Vec::new();
    }

    unsafe {
        let raw = ptr.as_raw();
        let rt_size = read_rt_size(raw);
        let len = rt_size as usize;

        let mut bytes = Vec::with_capacity(len);
        let src = raw as *const u8;

        for i in 0..len {
            bytes.push(core::ptr::read(src.add(i)));
        }

        bytes
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn asc_to_bytes(_ptr: AscPtr<AscBytes>) -> Vec<u8> {
    panic!("asc_to_bytes not available on native target");
}
