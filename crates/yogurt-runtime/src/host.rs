//! Host function imports from graph-node.
//!
//! graph-node provides approximately 50 host functions that WASM modules can import.
//! Critical finding from graph-node source: import resolution is by name only,
//! ignoring the WASM module path. We use "env" as the module name.

#[cfg(target_arch = "wasm32")]
mod imports {
    #[link(wasm_import_module = "env")]
    extern "C" {
        // Store operations
        #[link_name = "store.get"]
        pub fn store_get(entity_type: i32, id: i32) -> i32;

        #[link_name = "store.set"]
        pub fn store_set(entity_type: i32, id: i32, data: i32);

        #[link_name = "store.remove"]
        pub fn store_remove(entity_type: i32, id: i32);

        // Ethereum
        #[link_name = "ethereum.call"]
        pub fn ethereum_call(call: i32) -> i32;

        #[link_name = "ethereum.encode"]
        pub fn ethereum_encode(params: i32) -> i32;

        #[link_name = "ethereum.decode"]
        pub fn ethereum_decode(types: i32, data: i32) -> i32;

        // Type conversions
        #[link_name = "typeConversion.bytesToString"]
        pub fn bytes_to_string(bytes: i32) -> i32;

        #[link_name = "typeConversion.bytesToHex"]
        pub fn bytes_to_hex(bytes: i32) -> i32;

        #[link_name = "typeConversion.bigIntToString"]
        pub fn big_int_to_string(bigint: i32) -> i32;

        #[link_name = "typeConversion.bigIntToHex"]
        pub fn big_int_to_hex(bigint: i32) -> i32;

        #[link_name = "typeConversion.stringToH160"]
        pub fn string_to_h160(s: i32) -> i32;

        #[link_name = "typeConversion.bytesToBase58"]
        pub fn bytes_to_base58(bytes: i32) -> i32;

        // BigInt arithmetic
        #[link_name = "bigInt.plus"]
        pub fn big_int_plus(a: i32, b: i32) -> i32;

        #[link_name = "bigInt.minus"]
        pub fn big_int_minus(a: i32, b: i32) -> i32;

        #[link_name = "bigInt.times"]
        pub fn big_int_times(a: i32, b: i32) -> i32;

        #[link_name = "bigInt.dividedBy"]
        pub fn big_int_divided_by(a: i32, b: i32) -> i32;

        #[link_name = "bigInt.mod"]
        pub fn big_int_mod(a: i32, b: i32) -> i32;

        #[link_name = "bigInt.pow"]
        pub fn big_int_pow(base: i32, exp: i32) -> i32;

        #[link_name = "bigInt.bitOr"]
        pub fn big_int_bit_or(a: i32, b: i32) -> i32;

        #[link_name = "bigInt.bitAnd"]
        pub fn big_int_bit_and(a: i32, b: i32) -> i32;

        #[link_name = "bigInt.leftShift"]
        pub fn big_int_left_shift(a: i32, bits: i32) -> i32;

        #[link_name = "bigInt.rightShift"]
        pub fn big_int_right_shift(a: i32, bits: i32) -> i32;

        // BigDecimal
        #[link_name = "bigDecimal.plus"]
        pub fn big_decimal_plus(a: i32, b: i32) -> i32;

        #[link_name = "bigDecimal.minus"]
        pub fn big_decimal_minus(a: i32, b: i32) -> i32;

        #[link_name = "bigDecimal.times"]
        pub fn big_decimal_times(a: i32, b: i32) -> i32;

        #[link_name = "bigDecimal.dividedBy"]
        pub fn big_decimal_divided_by(a: i32, b: i32) -> i32;

        #[link_name = "bigDecimal.equals"]
        pub fn big_decimal_equals(a: i32, b: i32) -> i32;

        #[link_name = "bigDecimal.toString"]
        pub fn big_decimal_to_string(val: i32) -> i32;

        #[link_name = "bigDecimal.fromString"]
        pub fn big_decimal_from_string(s: i32) -> i32;

        // Crypto
        #[link_name = "crypto.keccak256"]
        pub fn crypto_keccak256(data: i32) -> i32;

        // JSON
        #[link_name = "json.fromBytes"]
        pub fn json_from_bytes(bytes: i32) -> i32;

        #[link_name = "json.toI64"]
        pub fn json_to_i64(value: i32) -> i64;

        #[link_name = "json.toU64"]
        pub fn json_to_u64(value: i32) -> u64;

        #[link_name = "json.toF64"]
        pub fn json_to_f64(value: i32) -> f64;

        #[link_name = "json.toBigInt"]
        pub fn json_to_big_int(value: i32) -> i32;

        // IPFS
        #[link_name = "ipfs.cat"]
        pub fn ipfs_cat(hash: i32) -> i32;

        #[link_name = "ipfs.map"]
        pub fn ipfs_map(hash: i32, callback: i32, user_data: i32, flags: i32);

        // Logging
        #[link_name = "log.log"]
        pub fn log_log(level: i32, msg: i32);

        // Data source
        #[link_name = "dataSource.create"]
        pub fn data_source_create(name: i32, params: i32);

        #[link_name = "dataSource.address"]
        pub fn data_source_address() -> i32;

        #[link_name = "dataSource.network"]
        pub fn data_source_network() -> i32;

        #[link_name = "dataSource.context"]
        pub fn data_source_context() -> i32;

        // ENS
        #[link_name = "ens.nameByHash"]
        pub fn ens_name_by_hash(hash: i32) -> i32;
    }
}

#[cfg(target_arch = "wasm32")]
pub use imports::*;

// Native stubs for testing
#[cfg(not(target_arch = "wasm32"))]
pub mod imports {
    pub unsafe fn store_get(_entity_type: i32, _id: i32) -> i32 {
        0
    }
    pub unsafe fn store_set(_entity_type: i32, _id: i32, _data: i32) {}
    pub unsafe fn store_remove(_entity_type: i32, _id: i32) {}
    pub unsafe fn log_log(_level: i32, _msg: i32) {}
    // Add other stubs as needed
}

#[cfg(not(target_arch = "wasm32"))]
pub use imports::*;
