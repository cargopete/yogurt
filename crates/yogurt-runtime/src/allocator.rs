//! Bump allocator with AssemblyScript-compatible 20-byte managed object headers.
//!
//! Every object in AssemblyScript's WASM memory is preceded by a 20-byte header
//! at negative offsets from the object pointer:
//!
//! ```text
//! Pointer - 20: mmInfo   (u32) — Memory manager metadata
//! Pointer - 16: gcInfo   (u32) — GC tracking (unused by yogurt)
//! Pointer - 12: gcInfo2  (u32) — Additional GC metadata (unused by yogurt)
//! Pointer -  8: rtId     (u32) — Runtime type class ID
//! Pointer -  4: rtSize   (u32) — Payload byte length
//! Pointer     : [payload bytes...]
//! ```

use core::sync::atomic::{AtomicU32, Ordering};

/// Size of the AssemblyScript managed object header in bytes
pub const HEADER_SIZE: u32 = 20;

/// Hardcoded AssemblyScript runtime type IDs
///
/// The first three are fixed by the AS runtime. Others are assigned
/// sequentially by the compiler, but we use stable IDs for our generated types.
pub mod class_id {
    pub const OBJECT: u32 = 0;
    pub const ARRAY_BUFFER: u32 = 1;
    pub const STRING: u32 = 2;

    // Array types - IDs chosen to avoid conflicts with codegen
    // These start at a high number to leave room for user-defined types
    pub const ARRAY_PTR: u32 = 1000;           // Array<AscPtr<T>>
    pub const TYPED_MAP: u32 = 1001;           // TypedMap<K, V>
    pub const TYPED_MAP_ENTRY: u32 = 1002;     // TypedMapEntry<K, V>
    pub const STORE_VALUE: u32 = 1003;         // Enum<StoreValueKind>
    pub const ARRAY_STORE_VALUE: u32 = 1004;   // Array<StoreValue>
}

/// Current heap pointer for bump allocation
static HEAP_PTR: AtomicU32 = AtomicU32::new(0);

/// Initial heap base (set on first allocation)
static HEAP_BASE: AtomicU32 = AtomicU32::new(0);

/// Initialise the heap pointer from WASM memory size
#[cfg(target_arch = "wasm32")]
fn ensure_heap_initialised() {
    if HEAP_BASE.load(Ordering::Relaxed) == 0 {
        // Start heap after the data segment
        // __heap_base is provided by wasm-ld
        unsafe extern "C" {
            static __heap_base: u8;
        }
        let base = unsafe { &__heap_base as *const u8 as u32 };
        HEAP_BASE.store(base, Ordering::Relaxed);
        HEAP_PTR.store(base, Ordering::Relaxed);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn ensure_heap_initialised() {
    // Native testing: heap simulation not needed
}

/// Allocate memory with an AssemblyScript-compatible header.
///
/// Returns a pointer to the payload (after the 20-byte header).
/// The header fields are initialised as:
/// - mmInfo: 0 (unused)
/// - gcInfo: 0 (unused)
/// - gcInfo2: 0 (unused)
/// - rtId: the provided class_id
/// - rtSize: the provided size
#[cfg(target_arch = "wasm32")]
pub fn asc_alloc(size: u32, class_id: u32) -> u32 {
    ensure_heap_initialised();

    let total_size = HEADER_SIZE + size;
    // Align to 8 bytes
    let aligned_size = (total_size + 7) & !7;

    let base = HEAP_PTR.fetch_add(aligned_size, Ordering::Relaxed);

    // Check if we need to grow memory
    let end = base + aligned_size;
    let pages_needed = (end + 65535) / 65536;
    let current_pages = core::arch::wasm32::memory_size(0) as u32;

    if pages_needed > current_pages {
        let grow = pages_needed - current_pages;
        if core::arch::wasm32::memory_grow(0, grow as usize) == usize::MAX {
            core::arch::wasm32::unreachable();
        }
    }

    // Write the 20-byte header
    let header_ptr = base as *mut u32;
    unsafe {
        // mmInfo at offset 0
        core::ptr::write_unaligned(header_ptr, 0);
        // gcInfo at offset 4
        core::ptr::write_unaligned(header_ptr.add(1), 0);
        // gcInfo2 at offset 8
        core::ptr::write_unaligned(header_ptr.add(2), 0);
        // rtId at offset 12
        core::ptr::write_unaligned(header_ptr.add(3), class_id);
        // rtSize at offset 16
        core::ptr::write_unaligned(header_ptr.add(4), size);
    }

    // Return pointer to payload (after header)
    base + HEADER_SIZE
}

#[cfg(not(target_arch = "wasm32"))]
pub fn asc_alloc(_size: u32, _class_id: u32) -> u32 {
    // Native: not supported, use testing mock
    panic!("asc_alloc not available on native target");
}

/// The `__new` export required by graph-node.
/// Signature: `(size: i32, classId: i32) -> i32`
#[unsafe(no_mangle)]
#[cfg(target_arch = "wasm32")]
pub extern "C" fn __new(size: i32, class_id: i32) -> i32 {
    asc_alloc(size as u32, class_id as u32) as i32
}

/// Read the rtId (runtime type ID) from an object's header
pub unsafe fn read_rt_id(ptr: u32) -> u32 {
    let header_ptr = (ptr - 8) as *const u32;
    unsafe { core::ptr::read_unaligned(header_ptr) }
}

/// Read the rtSize (payload byte length) from an object's header
pub unsafe fn read_rt_size(ptr: u32) -> u32 {
    let header_ptr = (ptr - 4) as *const u32;
    unsafe { core::ptr::read_unaligned(header_ptr) }
}
