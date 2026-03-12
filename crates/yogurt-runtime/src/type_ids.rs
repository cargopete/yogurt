//! TypeId exports for graph-node compatibility.
//!
//! Graph-node reads these exported globals to map type names to runtime class IDs.
//! The values must match graph-node's IndexForAscTypeId enum exactly.
//!
//! We use inline assembly to create true WASM globals with constant initializers,
//! since Rust statics are placed in data memory.

#[cfg(target_arch = "wasm32")]
core::arch::global_asm!(
    r#"
    .globaltype TYPE_ID_STRING, i32
    .globl TYPE_ID_STRING
    TYPE_ID_STRING:
    .int32 0
    .export_name TYPE_ID_STRING, "TypeId.String"

    .globaltype TYPE_ID_ARRAY_BUFFER, i32
    .globl TYPE_ID_ARRAY_BUFFER
    TYPE_ID_ARRAY_BUFFER:
    .int32 1
    .export_name TYPE_ID_ARRAY_BUFFER, "TypeId.ArrayBuffer"

    .globaltype TYPE_ID_INT8_ARRAY, i32
    .globl TYPE_ID_INT8_ARRAY
    TYPE_ID_INT8_ARRAY:
    .int32 2
    .export_name TYPE_ID_INT8_ARRAY, "TypeId.Int8Array"

    .globaltype TYPE_ID_INT16_ARRAY, i32
    .globl TYPE_ID_INT16_ARRAY
    TYPE_ID_INT16_ARRAY:
    .int32 3
    .export_name TYPE_ID_INT16_ARRAY, "TypeId.Int16Array"

    .globaltype TYPE_ID_INT32_ARRAY, i32
    .globl TYPE_ID_INT32_ARRAY
    TYPE_ID_INT32_ARRAY:
    .int32 4
    .export_name TYPE_ID_INT32_ARRAY, "TypeId.Int32Array"

    .globaltype TYPE_ID_INT64_ARRAY, i32
    .globl TYPE_ID_INT64_ARRAY
    TYPE_ID_INT64_ARRAY:
    .int32 5
    .export_name TYPE_ID_INT64_ARRAY, "TypeId.Int64Array"

    .globaltype TYPE_ID_UINT8_ARRAY, i32
    .globl TYPE_ID_UINT8_ARRAY
    TYPE_ID_UINT8_ARRAY:
    .int32 6
    .export_name TYPE_ID_UINT8_ARRAY, "TypeId.Uint8Array"

    .globaltype TYPE_ID_UINT16_ARRAY, i32
    .globl TYPE_ID_UINT16_ARRAY
    TYPE_ID_UINT16_ARRAY:
    .int32 7
    .export_name TYPE_ID_UINT16_ARRAY, "TypeId.Uint16Array"

    .globaltype TYPE_ID_UINT32_ARRAY, i32
    .globl TYPE_ID_UINT32_ARRAY
    TYPE_ID_UINT32_ARRAY:
    .int32 8
    .export_name TYPE_ID_UINT32_ARRAY, "TypeId.Uint32Array"

    .globaltype TYPE_ID_UINT64_ARRAY, i32
    .globl TYPE_ID_UINT64_ARRAY
    TYPE_ID_UINT64_ARRAY:
    .int32 9
    .export_name TYPE_ID_UINT64_ARRAY, "TypeId.Uint64Array"

    .globaltype TYPE_ID_FLOAT32_ARRAY, i32
    .globl TYPE_ID_FLOAT32_ARRAY
    TYPE_ID_FLOAT32_ARRAY:
    .int32 10
    .export_name TYPE_ID_FLOAT32_ARRAY, "TypeId.Float32Array"

    .globaltype TYPE_ID_FLOAT64_ARRAY, i32
    .globl TYPE_ID_FLOAT64_ARRAY
    TYPE_ID_FLOAT64_ARRAY:
    .int32 11
    .export_name TYPE_ID_FLOAT64_ARRAY, "TypeId.Float64Array"

    .globaltype TYPE_ID_BIG_DECIMAL, i32
    .globl TYPE_ID_BIG_DECIMAL
    TYPE_ID_BIG_DECIMAL:
    .int32 12
    .export_name TYPE_ID_BIG_DECIMAL, "TypeId.BigDecimal"

    .globaltype TYPE_ID_ARRAY_BOOL, i32
    .globl TYPE_ID_ARRAY_BOOL
    TYPE_ID_ARRAY_BOOL:
    .int32 13
    .export_name TYPE_ID_ARRAY_BOOL, "TypeId.ArrayBool"

    .globaltype TYPE_ID_ARRAY_UINT8_ARRAY, i32
    .globl TYPE_ID_ARRAY_UINT8_ARRAY
    TYPE_ID_ARRAY_UINT8_ARRAY:
    .int32 14
    .export_name TYPE_ID_ARRAY_UINT8_ARRAY, "TypeId.ArrayUint8Array"

    .globaltype TYPE_ID_ARRAY_ETHEREUM_VALUE, i32
    .globl TYPE_ID_ARRAY_ETHEREUM_VALUE
    TYPE_ID_ARRAY_ETHEREUM_VALUE:
    .int32 15
    .export_name TYPE_ID_ARRAY_ETHEREUM_VALUE, "TypeId.ArrayEthereumValue"

    .globaltype TYPE_ID_ARRAY_STORE_VALUE, i32
    .globl TYPE_ID_ARRAY_STORE_VALUE
    TYPE_ID_ARRAY_STORE_VALUE:
    .int32 16
    .export_name TYPE_ID_ARRAY_STORE_VALUE, "TypeId.ArrayStoreValue"

    .globaltype TYPE_ID_ARRAY_JSON_VALUE, i32
    .globl TYPE_ID_ARRAY_JSON_VALUE
    TYPE_ID_ARRAY_JSON_VALUE:
    .int32 17
    .export_name TYPE_ID_ARRAY_JSON_VALUE, "TypeId.ArrayJsonValue"

    .globaltype TYPE_ID_ARRAY_STRING, i32
    .globl TYPE_ID_ARRAY_STRING
    TYPE_ID_ARRAY_STRING:
    .int32 18
    .export_name TYPE_ID_ARRAY_STRING, "TypeId.ArrayString"

    .globaltype TYPE_ID_ARRAY_EVENT_PARAM, i32
    .globl TYPE_ID_ARRAY_EVENT_PARAM
    TYPE_ID_ARRAY_EVENT_PARAM:
    .int32 19
    .export_name TYPE_ID_ARRAY_EVENT_PARAM, "TypeId.ArrayEventParam"

    .globaltype TYPE_ID_ARRAY_TYPED_MAP_ENTRY_STRING_JSON_VALUE, i32
    .globl TYPE_ID_ARRAY_TYPED_MAP_ENTRY_STRING_JSON_VALUE
    TYPE_ID_ARRAY_TYPED_MAP_ENTRY_STRING_JSON_VALUE:
    .int32 20
    .export_name TYPE_ID_ARRAY_TYPED_MAP_ENTRY_STRING_JSON_VALUE, "TypeId.ArrayTypedMapEntryStringJsonValue"

    .globaltype TYPE_ID_ARRAY_TYPED_MAP_ENTRY_STRING_STORE_VALUE, i32
    .globl TYPE_ID_ARRAY_TYPED_MAP_ENTRY_STRING_STORE_VALUE
    TYPE_ID_ARRAY_TYPED_MAP_ENTRY_STRING_STORE_VALUE:
    .int32 21
    .export_name TYPE_ID_ARRAY_TYPED_MAP_ENTRY_STRING_STORE_VALUE, "TypeId.ArrayTypedMapEntryStringStoreValue"

    .globaltype TYPE_ID_SMART_CONTRACT_CALL, i32
    .globl TYPE_ID_SMART_CONTRACT_CALL
    TYPE_ID_SMART_CONTRACT_CALL:
    .int32 22
    .export_name TYPE_ID_SMART_CONTRACT_CALL, "TypeId.SmartContractCall"

    .globaltype TYPE_ID_EVENT_PARAM, i32
    .globl TYPE_ID_EVENT_PARAM
    TYPE_ID_EVENT_PARAM:
    .int32 23
    .export_name TYPE_ID_EVENT_PARAM, "TypeId.EventParam"

    .globaltype TYPE_ID_ETHEREUM_TRANSACTION, i32
    .globl TYPE_ID_ETHEREUM_TRANSACTION
    TYPE_ID_ETHEREUM_TRANSACTION:
    .int32 24
    .export_name TYPE_ID_ETHEREUM_TRANSACTION, "TypeId.EthereumTransaction"

    .globaltype TYPE_ID_ETHEREUM_BLOCK, i32
    .globl TYPE_ID_ETHEREUM_BLOCK
    TYPE_ID_ETHEREUM_BLOCK:
    .int32 25
    .export_name TYPE_ID_ETHEREUM_BLOCK, "TypeId.EthereumBlock"

    .globaltype TYPE_ID_ETHEREUM_CALL, i32
    .globl TYPE_ID_ETHEREUM_CALL
    TYPE_ID_ETHEREUM_CALL:
    .int32 26
    .export_name TYPE_ID_ETHEREUM_CALL, "TypeId.EthereumCall"

    .globaltype TYPE_ID_WRAPPED_TYPED_MAP_STRING_JSON_VALUE, i32
    .globl TYPE_ID_WRAPPED_TYPED_MAP_STRING_JSON_VALUE
    TYPE_ID_WRAPPED_TYPED_MAP_STRING_JSON_VALUE:
    .int32 27
    .export_name TYPE_ID_WRAPPED_TYPED_MAP_STRING_JSON_VALUE, "TypeId.WrappedTypedMapStringJsonValue"

    .globaltype TYPE_ID_WRAPPED_BOOL, i32
    .globl TYPE_ID_WRAPPED_BOOL
    TYPE_ID_WRAPPED_BOOL:
    .int32 28
    .export_name TYPE_ID_WRAPPED_BOOL, "TypeId.WrappedBool"

    .globaltype TYPE_ID_WRAPPED_JSON_VALUE, i32
    .globl TYPE_ID_WRAPPED_JSON_VALUE
    TYPE_ID_WRAPPED_JSON_VALUE:
    .int32 29
    .export_name TYPE_ID_WRAPPED_JSON_VALUE, "TypeId.WrappedJsonValue"

    .globaltype TYPE_ID_ETHEREUM_VALUE, i32
    .globl TYPE_ID_ETHEREUM_VALUE
    TYPE_ID_ETHEREUM_VALUE:
    .int32 30
    .export_name TYPE_ID_ETHEREUM_VALUE, "TypeId.EthereumValue"

    .globaltype TYPE_ID_STORE_VALUE, i32
    .globl TYPE_ID_STORE_VALUE
    TYPE_ID_STORE_VALUE:
    .int32 31
    .export_name TYPE_ID_STORE_VALUE, "TypeId.StoreValue"

    .globaltype TYPE_ID_JSON_VALUE, i32
    .globl TYPE_ID_JSON_VALUE
    TYPE_ID_JSON_VALUE:
    .int32 32
    .export_name TYPE_ID_JSON_VALUE, "TypeId.JsonValue"

    .globaltype TYPE_ID_ETHEREUM_EVENT, i32
    .globl TYPE_ID_ETHEREUM_EVENT
    TYPE_ID_ETHEREUM_EVENT:
    .int32 33
    .export_name TYPE_ID_ETHEREUM_EVENT, "TypeId.EthereumEvent"

    .globaltype TYPE_ID_TYPED_MAP_ENTRY_STRING_STORE_VALUE, i32
    .globl TYPE_ID_TYPED_MAP_ENTRY_STRING_STORE_VALUE
    TYPE_ID_TYPED_MAP_ENTRY_STRING_STORE_VALUE:
    .int32 34
    .export_name TYPE_ID_TYPED_MAP_ENTRY_STRING_STORE_VALUE, "TypeId.TypedMapEntryStringStoreValue"

    .globaltype TYPE_ID_TYPED_MAP_ENTRY_STRING_JSON_VALUE, i32
    .globl TYPE_ID_TYPED_MAP_ENTRY_STRING_JSON_VALUE
    TYPE_ID_TYPED_MAP_ENTRY_STRING_JSON_VALUE:
    .int32 35
    .export_name TYPE_ID_TYPED_MAP_ENTRY_STRING_JSON_VALUE, "TypeId.TypedMapEntryStringJsonValue"

    .globaltype TYPE_ID_TYPED_MAP_STRING_STORE_VALUE, i32
    .globl TYPE_ID_TYPED_MAP_STRING_STORE_VALUE
    TYPE_ID_TYPED_MAP_STRING_STORE_VALUE:
    .int32 36
    .export_name TYPE_ID_TYPED_MAP_STRING_STORE_VALUE, "TypeId.TypedMapStringStoreValue"

    .globaltype TYPE_ID_TYPED_MAP_STRING_JSON_VALUE, i32
    .globl TYPE_ID_TYPED_MAP_STRING_JSON_VALUE
    TYPE_ID_TYPED_MAP_STRING_JSON_VALUE:
    .int32 37
    .export_name TYPE_ID_TYPED_MAP_STRING_JSON_VALUE, "TypeId.TypedMapStringJsonValue"

    .globaltype TYPE_ID_TYPED_MAP_STRING_TYPED_MAP_STRING_JSON_VALUE, i32
    .globl TYPE_ID_TYPED_MAP_STRING_TYPED_MAP_STRING_JSON_VALUE
    TYPE_ID_TYPED_MAP_STRING_TYPED_MAP_STRING_JSON_VALUE:
    .int32 38
    .export_name TYPE_ID_TYPED_MAP_STRING_TYPED_MAP_STRING_JSON_VALUE, "TypeId.TypedMapStringTypedMapStringJsonValue"

    .globaltype TYPE_ID_RESULT_TYPED_MAP_STRING_JSON_VALUE_BOOL, i32
    .globl TYPE_ID_RESULT_TYPED_MAP_STRING_JSON_VALUE_BOOL
    TYPE_ID_RESULT_TYPED_MAP_STRING_JSON_VALUE_BOOL:
    .int32 39
    .export_name TYPE_ID_RESULT_TYPED_MAP_STRING_JSON_VALUE_BOOL, "TypeId.ResultTypedMapStringJsonValueBool"

    .globaltype TYPE_ID_RESULT_JSON_VALUE_BOOL, i32
    .globl TYPE_ID_RESULT_JSON_VALUE_BOOL
    TYPE_ID_RESULT_JSON_VALUE_BOOL:
    .int32 40
    .export_name TYPE_ID_RESULT_JSON_VALUE_BOOL, "TypeId.ResultJsonValueBool"

    .globaltype TYPE_ID_ARRAY_U8, i32
    .globl TYPE_ID_ARRAY_U8
    TYPE_ID_ARRAY_U8:
    .int32 41
    .export_name TYPE_ID_ARRAY_U8, "TypeId.ArrayU8"

    .globaltype TYPE_ID_ARRAY_U16, i32
    .globl TYPE_ID_ARRAY_U16
    TYPE_ID_ARRAY_U16:
    .int32 42
    .export_name TYPE_ID_ARRAY_U16, "TypeId.ArrayU16"

    .globaltype TYPE_ID_ARRAY_U32, i32
    .globl TYPE_ID_ARRAY_U32
    TYPE_ID_ARRAY_U32:
    .int32 43
    .export_name TYPE_ID_ARRAY_U32, "TypeId.ArrayU32"

    .globaltype TYPE_ID_ARRAY_U64, i32
    .globl TYPE_ID_ARRAY_U64
    TYPE_ID_ARRAY_U64:
    .int32 44
    .export_name TYPE_ID_ARRAY_U64, "TypeId.ArrayU64"

    .globaltype TYPE_ID_ARRAY_I8, i32
    .globl TYPE_ID_ARRAY_I8
    TYPE_ID_ARRAY_I8:
    .int32 45
    .export_name TYPE_ID_ARRAY_I8, "TypeId.ArrayI8"

    .globaltype TYPE_ID_ARRAY_I16, i32
    .globl TYPE_ID_ARRAY_I16
    TYPE_ID_ARRAY_I16:
    .int32 46
    .export_name TYPE_ID_ARRAY_I16, "TypeId.ArrayI16"

    .globaltype TYPE_ID_ARRAY_I32, i32
    .globl TYPE_ID_ARRAY_I32
    TYPE_ID_ARRAY_I32:
    .int32 47
    .export_name TYPE_ID_ARRAY_I32, "TypeId.ArrayI32"

    .globaltype TYPE_ID_ARRAY_I64, i32
    .globl TYPE_ID_ARRAY_I64
    TYPE_ID_ARRAY_I64:
    .int32 48
    .export_name TYPE_ID_ARRAY_I64, "TypeId.ArrayI64"

    .globaltype TYPE_ID_ARRAY_F32, i32
    .globl TYPE_ID_ARRAY_F32
    TYPE_ID_ARRAY_F32:
    .int32 49
    .export_name TYPE_ID_ARRAY_F32, "TypeId.ArrayF32"

    .globaltype TYPE_ID_ARRAY_F64, i32
    .globl TYPE_ID_ARRAY_F64
    TYPE_ID_ARRAY_F64:
    .int32 50
    .export_name TYPE_ID_ARRAY_F64, "TypeId.ArrayF64"

    .globaltype TYPE_ID_ARRAY_BIG_DECIMAL, i32
    .globl TYPE_ID_ARRAY_BIG_DECIMAL
    TYPE_ID_ARRAY_BIG_DECIMAL:
    .int32 51
    .export_name TYPE_ID_ARRAY_BIG_DECIMAL, "TypeId.ArrayBigDecimal"
    "#
);
