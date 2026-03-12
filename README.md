# yogurt

**Status: Discontinued Proof of Concept**

yogurt was an experimental attempt to write The Graph subgraph handlers in Rust instead of AssemblyScript, compiling to WASM modules that would be binary-compatible with graph-node's existing runtime.

## What We Learned

After 66 deployment iterations and extensive debugging, we concluded that **graph-node is too tightly coupled to AssemblyScript's memory model** for a pure Rust toolchain to be practical without modifying graph-node itself.

### What Worked

- Entity **serialization** — we successfully wrote entities to the store (v0.1.57 proved this with hardcoded values)
- WASM exports — `__new`, `__pin`, `__unpin`, `__collect` satisfied the AS runtime contract
- TypeId globals — injected via WASM post-processing with correct constant values
- AscEnum headers — 16-byte structures matching graph-node's expectations
- TypedArray wrappers — Uint8Array for Bytes/BigInt fields

### What Didn't Work

- Event **deserialization** — reading TypedArray/Uint8Array pointers from graph-node's AS-formatted memory remained intractable
- The fundamental issue: graph-node passes event parameters as AssemblyScript objects with specific memory layouts (managed object headers, rtId/rtSize at negative offsets, TypedArray wrappers). Replicating this from Rust proved fragile and ultimately unsuccessful.

### The Core Problem

Graph-node assumes AssemblyScript's memory layout throughout its codebase. Key incompatibilities included:

1. **20-byte managed object headers** with rtId at offset -8, rtSize at offset -4
2. **UTF-16LE strings** with specific array layouts
3. **AscEnum structures** with precise padding requirements
4. **TypedArray wrappers** for Bytes/BigInt that require runtime introspection
5. **Memory sanitization** that catches pointer arithmetic before our guards could run

We could match the *output* format (entities saved correctly), but couldn't reliably parse the *input* format (event parameters) from graph-node's memory.

## Future Direction

The right approach would be to modify graph-node to accept Rust-compiled WASM directly, rather than trying to emulate AssemblyScript at the binary level. This would involve:

1. A new WASM ABI for Rust subgraphs (simpler structs, no AS runtime requirements)
2. Graph-node changes to detect and handle Rust WASM modules differently
3. Potentially a compile-time flag or manifest field to indicate the subgraph's source language

If you're interested in pursuing this direction, the investigation notes in the git history document every compatibility issue we encountered.

## What Was Built

Despite not reaching production viability, yogurt included:

- `yogurt init` — project scaffolding
- `yogurt codegen` — generate Rust types from schema/ABIs
- `yogurt build` — compile to WASM with wasm-opt and TypeId injection
- `yogurt deploy` — deploy to local graph-node or Subgraph Studio
- Full entity builder pattern, ID macros, testing framework
- 90% of graph-ts API surface area

The runtime library (`yogurt-runtime`) and codegen (`yogurt-codegen`) are architecturally sound; the incompatibility lies at the graph-node interface boundary.

## Licence

MIT OR Apache-2.0
