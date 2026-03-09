# CLI Commands

Complete reference for the `yogurt` command-line interface.

## yogurt init

Create a new subgraph project.

```bash
yogurt init [name]
```

**Arguments:**
- `name` — Project directory name (optional, prompts if not provided)

**Example:**
```bash
yogurt init my-subgraph
cd my-subgraph
```

Creates a project with:
- `Cargo.toml`
- `subgraph.yaml`
- `schema.graphql`
- `src/lib.rs`
- `abis/` directory

## yogurt codegen

Generate Rust types from schema and ABIs.

```bash
yogurt codegen [options]
```

**Options:**
- `--manifest <path>` — Path to subgraph.yaml (default: `./subgraph.yaml`)

**Example:**
```bash
yogurt codegen
yogurt codegen --manifest ./path/to/subgraph.yaml
```

Generates:
- `src/generated/schema.rs` — Entity structs
- `src/generated/abi.rs` — Event and contract types
- `src/generated/templates.rs` — Data source templates
- `src/generated/mod.rs` — Module exports

## yogurt build

Compile the subgraph to WASM.

```bash
yogurt build [options]
```

**Options:**
- `--release` — Build with optimizations
- `--output <path>` — Output path (default: `./build/subgraph.wasm`)
- `--manifest <path>` — Path to subgraph.yaml
- `--no-optimize` — Skip wasm-opt optimization

**Examples:**
```bash
yogurt build                    # Debug build
yogurt build --release          # Release build
yogurt build --output ./out.wasm
```

## yogurt validate

Check WASM binary for graph-node compatibility.

```bash
yogurt validate [path]
```

**Arguments:**
- `path` — WASM file to validate (default: `./build/subgraph.wasm`)

**Example:**
```bash
yogurt validate
yogurt validate ./my-output/subgraph.wasm
```

Checks:
- Required exports exist
- Handler signatures are correct
- No invalid imports
- File size is reasonable

## yogurt deploy

Deploy subgraph to graph-node or Subgraph Studio.

```bash
yogurt deploy <name> [options]
```

**Arguments:**
- `name` — Subgraph name (e.g., `myaccount/my-subgraph`)

**Options:**
- `--node <url>` — Graph node URL (default: `http://localhost:8020`)
- `--ipfs <url>` — IPFS URL (default: `http://localhost:5001`)
- `--version <label>` — Version label
- `--studio` — Deploy to Subgraph Studio
- `--manifest <path>` — Path to subgraph.yaml

**Examples:**
```bash
# Local deployment
yogurt deploy myaccount/my-subgraph

# Custom endpoints
yogurt deploy myaccount/my-subgraph \
  --node http://graph.example.com:8020 \
  --ipfs http://ipfs.example.com:5001

# Studio deployment
yogurt deploy my-subgraph --studio --version 0.0.1
```

## yogurt auth

Store Subgraph Studio authentication.

```bash
yogurt auth <token>
```

**Arguments:**
- `token` — Deploy key from Studio dashboard

**Example:**
```bash
yogurt auth abc123def456...
```

Saves to `~/.config/yogurt/auth.json`.

## yogurt dev

Watch mode with auto-rebuild.

```bash
yogurt dev [options]
```

**Options:**
- `--manifest <path>` — Path to subgraph.yaml
- `--release` — Use release builds

**Example:**
```bash
yogurt dev
yogurt dev --release
```

Watches:
- `src/**/*.rs`
- `schema.graphql`
- `subgraph.yaml`
- `abis/**/*.json`

Press `Ctrl+C` to stop.

## yogurt inspect

Analyze a WASM module.

```bash
yogurt inspect [path]
```

**Arguments:**
- `path` — WASM file to inspect (default: `./build/subgraph.wasm`)

**Example:**
```bash
yogurt inspect
yogurt inspect ./build/subgraph.wasm
```

Shows:
- File size
- Memory configuration
- Exported functions (handlers, runtime)
- Imported host functions
- Compatibility assessment

## yogurt test

Run native tests.

```bash
yogurt test [options]
```

**Options:**
- `--release` — Run in release mode
- `--` — Pass additional arguments to cargo test

**Examples:**
```bash
yogurt test
yogurt test -- --nocapture
yogurt test -- test_handle_transfer
```

Equivalent to:
```bash
cargo test --target <native-target>
```

## Global Options

These work with all commands:

- `--help` — Show help message
- `--version` — Show version
- `--quiet` — Suppress output
- `--verbose` — Verbose output

## Exit Codes

- `0` — Success
- `1` — Error (build failed, validation failed, etc.)

## Environment Variables

- `GRAPH_AUTH_TOKEN` — Studio deploy key (alternative to `yogurt auth`)
- `ETHEREUM_RPC` — Default Ethereum RPC URL for local testing scripts
