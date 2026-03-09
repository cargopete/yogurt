# Building

Compile your subgraph to WASM for deployment.

## Basic Build

```bash
yogurt build
```

This compiles to `build/subgraph.wasm` (debug mode).

## Release Build

For production, use release mode with optimizations:

```bash
yogurt build --release
```

Release builds are smaller and faster.

## Build Output

```
build/
└── subgraph.wasm    # The compiled WASM module
```

The WASM file is what gets deployed to graph-node.

## Build Options

### Output Path

```bash
yogurt build --output ./my-output/subgraph.wasm
```

### Manifest Path

```bash
yogurt build --manifest ./path/to/subgraph.yaml
```

### Skip wasm-opt

```bash
yogurt build --release --no-optimize
```

By default, release builds run `wasm-opt` for additional optimization. Use `--no-optimize` to skip this step (useful if wasm-opt isn't installed).

## Validation

After building, validate the WASM:

```bash
yogurt validate
```

Or validate a specific file:

```bash
yogurt validate build/subgraph.wasm
```

Validation checks:
- Required exports exist (`memory`, `__new`, `__pin`, `__unpin`, `__collect`, `abort`)
- Handler functions are exported with correct signatures
- No invalid imports
- File size is reasonable

## Watch Mode

Auto-rebuild on file changes:

```bash
yogurt dev
```

Watches:
- `src/**/*.rs` — Rust source files
- `schema.graphql` — Entity schema
- `subgraph.yaml` — Manifest
- `abis/**/*.json` — Contract ABIs

Press `Ctrl+C` to stop.

## Inspect WASM

Analyze the compiled module:

```bash
yogurt inspect build/subgraph.wasm
```

Shows:
- File size
- Memory configuration
- Exported functions
- Imported host functions
- Compatibility status

## Codegen Before Build

If schema or ABIs changed, regenerate types first:

```bash
yogurt codegen
yogurt build --release
```

Or use the combined workflow:

```bash
yogurt codegen && yogurt build --release
```

## Build Errors

### Missing wasm32 target

```
error: target `wasm32-unknown-unknown` not found
```

Fix:
```bash
rustup target add wasm32-unknown-unknown
```

### Codegen out of date

```
warning: generated code may be stale
```

Fix:
```bash
yogurt codegen
```

### wasm-opt not found

```
warning: wasm-opt not found, skipping optimization
```

This is just a warning. Install wasm-opt for smaller builds:

```bash
# macOS
brew install binaryen

# Ubuntu
apt install binaryen
```

## Typical Workflow

```bash
# Development
yogurt dev                    # Watch mode

# Production deployment
yogurt codegen               # Regenerate types
yogurt build --release       # Optimized build
yogurt validate              # Check exports
yogurt deploy ...            # Deploy
```

## CI/CD

Example GitHub Actions step:

```yaml
- name: Build subgraph
  run: |
    rustup target add wasm32-unknown-unknown
    cargo install --path crates/yogurt-cli
    yogurt codegen
    yogurt build --release
    yogurt validate
```
