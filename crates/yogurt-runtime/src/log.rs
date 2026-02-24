//! Logging utilities for subgraph mappings.

use crate::asc::str_to_asc;

/// Log severity levels matching graph-node expectations.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Critical = 0,
    Error = 1,
    Warning = 2,
    Info = 3,
    Debug = 4,
}

/// Log a message at the specified level.
#[cfg(target_arch = "wasm32")]
pub fn log(level: Level, msg: &str) {
    let msg_ptr = str_to_asc(msg);
    unsafe {
        crate::host::log_log(level as i32, msg_ptr.as_i32());
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn log(_level: Level, _msg: &str) {
    // Native: no-op or could print to stderr for testing
}

/// Log a critical message.
#[inline]
pub fn critical(msg: &str) {
    log(Level::Critical, msg);
}

/// Log an error message.
#[inline]
pub fn error(msg: &str) {
    log(Level::Error, msg);
}

/// Log a warning message.
#[inline]
pub fn warning(msg: &str) {
    log(Level::Warning, msg);
}

/// Log an info message.
#[inline]
pub fn info(msg: &str) {
    log(Level::Info, msg);
}

/// Log a debug message.
#[inline]
pub fn debug(msg: &str) {
    log(Level::Debug, msg);
}

// Convenience macros for formatted logging
// Note: Using format! pulls in std::fmt machinery which increases WASM size
// For size-critical subgraphs, prefer string concatenation or pre-formatted strings

/// Log an info message with formatting.
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {{
        let msg = alloc::format!($($arg)*);
        $crate::log::info(&msg);
    }};
}

/// Log an error message with formatting.
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {{
        let msg = alloc::format!($($arg)*);
        $crate::log::error(&msg);
    }};
}

/// Log a warning message with formatting.
#[macro_export]
macro_rules! log_warning {
    ($($arg:tt)*) => {{
        let msg = alloc::format!($($arg)*);
        $crate::log::warning(&msg);
    }};
}

/// Log a debug message with formatting.
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {{
        let msg = alloc::format!($($arg)*);
        $crate::log::debug(&msg);
    }};
}
