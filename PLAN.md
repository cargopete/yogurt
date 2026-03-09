# yogurt Development Plan

> Rust toolchain for The Graph subgraphs — write mappings in Rust, compile to AS-compatible WASM.

## Project Status

**Current Phase:** Phase 4 — Testing and Ecosystem
**Started:** 2026-02-24
**Last Updated:** 2026-03-09
**Phase 1 Complete:** PoC compiles to 16KB WASM, passes 21 binary compatibility tests
**Phase 2 Complete:** Schema arrays, nullable fields, contract call encoding
**Phase 3 Complete:** Full CLI with dev/inspect, Studio deployment, testing framework

---

## Architecture Overview

```
yogurt/
├── crates/
│   ├── yogurt-cli/          # Binary — the `yogurt` command
│   ├── yogurt-runtime/      # Library — Rust equivalent of graph-ts
│   ├── yogurt-codegen/      # Library — schema/ABI → Rust code generator
│   └── yogurt-macros/       # Proc-macro — #[handler] and other attribute macros
├── templates/               # Project scaffolding templates
└── tests/
    ├── integration/         # End-to-end build + deploy tests
    └── compat/              # Binary compatibility tests against AS output
```

---

## Phase 1: Foundation ✅ Complete

**Goal:** Compile a trivial Rust subgraph handler to WASM that graph-node executes successfully.

### Core Runtime (`yogurt-runtime`)

- [x] Project scaffold and workspace setup
- [x] Bump allocator with 20-byte AS-compatible headers
- [x] `AscPtr<T>` type for WASM memory pointers
- [x] UTF-16LE string encoding (`str_to_asc`, `asc_to_string`)
- [x] Byte array encoding (`bytes_to_asc`, `asc_to_bytes`)
- [x] Host function imports (store, ethereum, typeConversion, bigInt, etc.)
- [x] Core types: `Address`, `Bytes`, `BigInt`, `BigDecimal`, `EntityData`, `Value`
- [x] `Entity` trait definition
- [x] Store operations: `store::get`, `store::set`, `store::remove`
- [x] Ethereum types: `Block`, `Transaction`, `Event<P>`, `Token`
- [x] Logging: `log::info`, `log::error`, etc.
- [x] Panic handler and AS runtime exports (`__new`, `__pin`, `__unpin`, `__collect`, `abort`)
- [x] **Entity serialization**: TypedMap/TypedMapEntry/AscEnum layouts for AS memory
- [x] **BigInt/BigDecimal arithmetic**: Full operator overloading via host functions
- [x] **FromAscPtr trait**: Event deserialization from AS memory layout

### Proof of Concept: ERC-20 Transfer Subgraph

- [x] Created `tests/integration/erc20-transfer/` with Transfer event handler
- [x] Compiles to 16KB release WASM (well under 100KB target)
- [x] Passes `yogurt validate` with all required exports
- [x] Exports: memory, __new, __pin, __unpin, __collect, abort, handleTransfer

### Binary Compatibility Test Suite

- [x] 21 tests in `crates/yogurt-cli/tests/binary_compatibility.rs`
- [x] Required export validation (memory, __new, __pin, __unpin, __collect, abort)
- [x] Function signature validation (correct parameter/return types)
- [x] Handler signature validation (i32 param, void return)
- [x] Import validation (only graph-node compatible modules)
- [x] Memory layout validation (single memory, exported as "memory")
- [x] WASM structure validation (no start function, under 100KB)

### Technical Debt / Known Issues

- [x] `TODO` comments in runtime code — all addressed
- [x] Host function stubs for native target (testing) — proper mock implementations added

---

## Phase 2: Code Generation ✅ Complete

**Goal:** `yogurt codegen` produces type-safe Rust from schema + ABIs.

### Schema Codegen (`yogurt-codegen`)

- [x] GraphQL schema parser (using `graphql-parser`)
- [x] Entity struct generation with getters/setters
- [x] `Entity` trait implementation generation
- [x] `@entity` directive handling
- [x] `@derivedFrom` directive handling (skip derived fields)
- [x] Array field types (Vec<T> for all inner types)
- [x] Nullable field handling (Option<T> for non-required fields, unset_* methods)
- [x] Relation fields (entity references stored as ID strings)
- [x] `@entity(immutable: true)` support (skips setter generation)

### ABI Codegen

- [x] ABI JSON parser (using `alloy-json-abi`)
- [x] Event struct generation with typed parameters
- [x] Contract binding generation for view/pure functions
- [x] Event deserialization from AS memory layout (`FromAscPtr`)
- [x] Contract call encoding/decoding (Token serialization to AS memory)
- [x] Return value extraction from Token array
- [x] Tuple parameter support (recursive component handling)
- [x] Fixed-size array support (`[T; N]` Rust arrays)

### Proc Macros (`yogurt-macros`)

- [x] `#[handler]` attribute macro (basic structure)
- [x] `FromAscPtr` trait and implementations for all core types
- [x] Handler name customization (`#[handler(name = "...")]`)
- [x] Single-parameter design validated (graph-node passes one struct pointer)

---

## Phase 3: CLI and Developer Experience ✅ Complete

**Goal:** Complete `yogurt` CLI with init/codegen/build/deploy workflow.

### CLI Commands (`yogurt-cli`)

- [x] `yogurt init` — interactive project scaffolding
- [x] `yogurt codegen` — generate Rust from schema/ABIs
- [x] `yogurt build` — compile to WASM
- [x] `yogurt test` — run handler tests
- [x] `yogurt deploy` — deploy to local graph-node or Subgraph Studio
- [x] `yogurt validate` — check WASM exports
- [x] `yogurt dev` — file watching with auto-rebuild
- [x] `yogurt inspect` — WASM module analysis (imports, exports, memory)
- [x] `yogurt auth` — store Subgraph Studio deploy key

### Build Pipeline

- [x] Cargo build integration for `wasm32-unknown-unknown`
- [x] `wasm-opt` integration (optional optimisation)
- [x] WASM export validation
- [x] Codegen freshness checking (SHA-256 hash of manifest, schema, ABIs)
- [x] `__rtti_base` export for AssemblyScript compatibility

### Deployment

- [x] IPFS upload integration
- [x] Subgraph Studio deployment (with publish to decentralized network via web UI)
- [x] Self-hosted graph-node deployment

---

## Phase 4: Testing and Ecosystem

**Goal:** Production-ready testing framework and documentation.

### Testing Framework ✅ Complete

- [x] Native BigInt/BigDecimal (using `num-bigint` for native builds)
- [x] Thread-local mock store with proper entity serialization
- [x] `EventBuilder` for constructing test events
- [x] `CallBuilder` for constructing test call handlers
- [x] `BlockBuilder` for constructing test blocks
- [x] Mock data source (`mock_data_source_address`, `mock_data_source_network`, `mock_data_source_context`)
- [x] Mock IPFS (`mock_ipfs_cat`, `clear_ipfs_mocks`)
- [x] Native crypto (`keccak256` using sha3 crate)
- [x] JSON parsing (`json::from_bytes`, `json::from_string`, `JsonValue` API)
- [x] Assertion helpers (`assert_entity_exists`, `assert_entity_not_exists`, `entity_count`)
- [x] Type conversion utilities (`Bytes::from_hex_string`, `Address::from_string`, `BigInt::from_string`)
- [ ] WASM test runner (optional high-fidelity mode)

### Documentation

- [ ] Getting started guide
- [ ] Migration guide from AssemblyScript
- [ ] API reference
- [ ] Example subgraphs:
  - [x] ERC-20 token tracker
  - [ ] ERC-721 NFT indexer
  - [x] Uniswap V2 clone

---

## Phase 5: Advanced Features (Ongoing)

- [x] Data source templates (`dataSource.create`)
- [x] File data sources (IPFS-triggered handlers)
- [x] Block handlers
- [x] Call handlers
- [x] `BigInt` and `BigDecimal` with Rust operator overloading (`Add`, `Sub`, etc.)
- [x] `BigInt` and `BigDecimal` comparison operators (`<`, `>`, `<=`, `>=`, `lt()`, `gt()`, `le()`, `ge()`)
- [x] `BigInt::abs()` — absolute value
- [x] `BigInt::sqrt()` — integer square root (Newton's method)
- [x] `BigDecimal::truncate()` — truncate to N decimal places
- [x] `Bytes::concat()` and `Bytes::concat_i32()` — byte array concatenation
- [x] IPFS integration (`ipfs.cat`, `ipfs.map` for streaming)
- [x] ENS reverse lookup (`ens::name_by_hash`)
- [x] `store::get_in_block` — load entities modified in current block
- [x] JSON parsing utilities (`json::from_bytes`, `JsonValue` API)
- [x] `yogurt dev` — file watching with auto-rebuild
- [x] `yogurt inspect` — WASM export inspection/debugging
- [x] `day_id!` / `hour_id!` macros — time-based entity IDs for analytics subgraphs
- [x] `format_units` / `parse_units` — token amount formatting (wei ↔ ETH conversion)
- [x] `Address::is_zero()` — check for zero address
- [x] `BigInt::safe_div` / `BigDecimal::safe_div` — safe division (returns zero on divide-by-zero)
- [ ] Immutable entity optimizations

---

## Technical Notes

### AssemblyScript Memory Layout

Every AS object has a 20-byte header at negative offsets:

```
Ptr - 20: mmInfo   (u32) — Memory manager metadata
Ptr - 16: gcInfo   (u32) — GC tracking (unused)
Ptr - 12: gcInfo2  (u32) — Additional GC metadata (unused)
Ptr -  8: rtId     (u32) — Runtime type class ID
Ptr -  4: rtSize   (u32) — Payload byte length
Ptr     : [payload bytes...]
```

Hardcoded type IDs:
- `Object = 0`
- `ArrayBuffer = 1`
- `String = 2`

### Required WASM Exports

| Export | Signature | Purpose |
|--------|-----------|---------|
| `memory` | Memory | Linear memory |
| `__new` | `(size: i32, classId: i32) → i32` | Allocator |
| `__pin` | `(ptr: i32) → i32` | GC pinning (no-op) |
| `__unpin` | `(ptr: i32)` | GC unpinning (no-op) |
| `__collect` | `()` | GC collection (no-op) |
| `abort` | `(msg, file, line, col: i32)` | Panic handler |
| Handler functions | `(eventPtr: i32)` | Event handlers |

### String Encoding

All strings are UTF-16LE encoded. Conversion:

```rust
// Rust &str → AS String
fn str_to_asc(s: &str) -> AscPtr<AscString> {
    let utf16: Vec<u16> = s.encode_utf16().collect();
    // ... allocate and write
}

// AS String → Rust String
fn asc_to_string(ptr: AscPtr<AscString>) -> String {
    // Read rtSize, decode UTF-16LE
    String::from_utf16_lossy(&units)
}
```

---

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-02-24 | Use `graphql-parser` for schema parsing | Mature, well-tested, handles full GraphQL spec |
| 2026-02-24 | Use `alloy-json-abi` for ABI parsing | Part of alloy-rs ecosystem, actively maintained |
| 2026-02-24 | Target Rust 2024 edition | Latest stable, better defaults |
| 2026-02-24 | Bump allocator (no GC) | Handler lifetime is short, memory never freed |

---

## Roadmap / Next Steps

**Top Priority — Documentation:**
1. **Documentation site** — Getting started, migration guide, API reference
2. **ERC-721 NFT example** — Complete example set

**Future Work:**
- WASM test runner (high-fidelity mode running handlers in actual WASM)
- Immutable entity optimizations

**Recently Completed (2026-03-09) — Developer Experience:**
- ✅ `log_id!` macro — generate unique event IDs from tx hash + log index
- ✅ `call_id!` macro — generate unique call IDs from tx hash
- ✅ `block_id!` macro — generate unique block IDs from block number
- ✅ `day_id!` / `hour_id!` macros — time-based entity IDs for analytics
- ✅ `format_units` / `parse_units` — token amount formatting (wei ↔ ETH style)
- ✅ `Address::is_zero()` — check for zero address
- ✅ `BigInt::safe_div` / `BigDecimal::safe_div` — division that returns zero on divide-by-zero
- ✅ `From<Address> for Bytes`, `From<[u8; 20]>`, `From<[u8; 32]>` — automatic coercion
- ✅ Entity builder pattern via codegen — fluent `Entity::builder().field().save()`
- ✅ Entity helper methods: `load_or_create`, `update`, `upsert`, `exists`
- ✅ Fixed CLI codegen to output relative to manifest, not CWD
- ✅ Updated example subgraphs to showcase all DX improvements

**Previously Completed (2026-03-09):**
- ✅ BigInt/BigDecimal comparison operators (`<`, `>`, `<=`, `>=` and `lt()`, `gt()`, `le()`, `ge()`)
- ✅ BigInt `abs()`, `sqrt()` and BigDecimal `truncate()` methods
- ✅ Bytes `concat()` and `concat_i32()` methods
- ✅ ENS reverse lookup (`ens::name_by_hash`)
- ✅ IPFS streaming (`ipfs::map` for callback-based processing)
- ✅ `store::get_in_block` / `loadInBlock` for block-scoped entity loading
- ✅ Native testing framework with `EventBuilder`, `CallBuilder`, `BlockBuilder`
- ✅ Mock store, mock data source, mock IPFS
- ✅ Native crypto (keccak256)
- ✅ JSON parsing utilities
- ✅ `yogurt dev` (file watching with auto-rebuild)
- ✅ `yogurt inspect` (WASM module analysis)
- ✅ Type conversion utilities (hex parsing, BigInt from string/bytes)

**Previously Completed:**
- ✅ Subgraph Studio deployment (`yogurt auth` + `yogurt deploy --studio`)
- ✅ Uniswap V2 example subgraph (`tests/integration/uniswap-v2/`)
- ✅ File data sources and data source templates
- ✅ Block handlers
- ✅ Call handlers

---

## References

- [graph-node runtime/wasm source](https://github.com/graphprotocol/graph-node/tree/master/runtime/wasm)
- [AssemblyScript memory layout](https://www.assemblyscript.org/runtime.html)
- [@graphprotocol/graph-ts](https://github.com/graphprotocol/graph-tooling/tree/main/packages/ts)
- [alloy-rs](https://github.com/alloy-rs/alloy)
