# Installation

## Prerequisites

You'll need:

- **Rust 1.80+** with the `wasm32-unknown-unknown` target
- **wasm-opt** (optional) for optimized release builds

### Install Rust

If you don't have Rust installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Add WASM Target

```bash
rustup target add wasm32-unknown-unknown
```

### Install wasm-opt (Optional)

wasm-opt optimizes your WASM output for smaller binaries. Install via your package manager:

```bash
# macOS
brew install binaryen

# Ubuntu/Debian
apt install binaryen

# Or via npm
npm install -g binaryen
```

## Install yogurt

Install the yogurt CLI from source:

```bash
git clone https://github.com/example/yogurt.git
cd yogurt
cargo install --path crates/yogurt-cli
```

Or, if published to crates.io:

```bash
cargo install yogurt-cli
```

## Verify Installation

```bash
yogurt --version
```

You should see the version number. You're ready to create your first subgraph!

## Next Steps

- [Quick Start](./quick-start.md) — Create your first Rust subgraph
- [Project Structure](./project-structure.md) — Understand the layout
