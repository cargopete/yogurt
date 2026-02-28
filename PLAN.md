# yogurt Development Plan

> Rust toolchain for The Graph subgraphs — write mappings in Rust, compile to AS-compatible WASM.

## Project Status

**Current Phase:** Phase 3 — CLI and Developer Experience
**Started:** 2026-02-24
**Last Updated:** 2026-02-24
**Phase 1 Complete:** PoC compiles to 16KB WASM, passes 21 binary compatibility tests
**Phase 2 Complete:** Schema arrays, nullable fields, contract call encoding

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

- [ ] Many `TODO` comments in runtime code need implementation
- [ ] Host function stubs for native target (testing) need proper mock implementations

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

## Phase 3: CLI and Developer Experience

**Goal:** Complete `yogurt` CLI with init/codegen/build/deploy workflow.

### CLI Commands (`yogurt-cli`)

- [x] `yogurt init` — interactive project scaffolding
- [x] `yogurt codegen` — generate Rust from schema/ABIs
- [x] `yogurt build` — compile to WASM
- [x] `yogurt test` — run handler tests (stub)
- [x] `yogurt deploy` — deploy subgraph (stub)
- [x] `yogurt validate` — check WASM exports

### Build Pipeline

- [x] Cargo build integration for `wasm32-unknown-unknown`
- [x] `wasm-opt` integration (optional optimisation)
- [x] WASM export validation
- [x] Codegen freshness checking (SHA-256 hash of manifest, schema, ABIs)
- [x] `__rtti_base` export for AssemblyScript compatibility
- [ ] Custom WASM section stripping/modification

### Deployment

- [x] IPFS upload integration
- [ ] Subgraph Studio deployment
- [x] Self-hosted graph-node deployment
- [ ] Decentralized network deployment

---

## Phase 4: Testing and Ecosystem

**Goal:** Production-ready testing framework and documentation.

### Testing Framework

- [x] `MockContext` for native testing (basic)
- [x] Mock block/transaction/receipt helpers
- [ ] In-memory mock store with proper entity serialization
- [ ] Mock ethereum.call responses
- [ ] Event construction helpers
- [ ] Assertion helpers for entity state
- [ ] WASM test runner (optional high-fidelity mode)

### Documentation

- [ ] Getting started guide
- [ ] Migration guide from AssemblyScript
- [ ] API reference
- [ ] Example subgraphs:
  - [ ] ERC-20 token tracker
  - [ ] ERC-721 NFT indexer
  - [ ] Uniswap V2 clone

---

## Phase 5: Advanced Features (Ongoing)

- [ ] Data source templates (`dataSource.create`)
- [ ] File data sources (IPFS-triggered handlers)
- [ ] Block handlers
- [ ] Call handlers
- [x] `BigInt` and `BigDecimal` with Rust operator overloading (`Add`, `Sub`, etc.)
- [ ] IPFS integration (`ipfs.cat`, `ipfs.map`)
- [ ] JSON parsing utilities
- [ ] `yogurt dev` — file watching with auto-rebuild
- [ ] `yogurt inspect` — WASM export inspection/debugging
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

## References

- [graph-node runtime/wasm source](https://github.com/graphprotocol/graph-node/tree/master/runtime/wasm)
- [AssemblyScript memory layout](https://www.assemblyscript.org/runtime.html)
- [@graphprotocol/graph-ts](https://github.com/graphprotocol/graph-tooling/tree/main/packages/ts)
- [alloy-rs](https://github.com/alloy-rs/alloy)
