# Building FIT Platform Across Operating Systems

## Requirements

- **Rust**: 1.75+ (`rustc` & `cargo`)
- **Node.js**: 18+ & `npm`
- **GCC / Clang** toolchain

## Building Cargo Workspace

```bash
# Check all workspace crates
cargo check --workspace

# Run full test suite
cargo test --workspace

# Build CLI binary in release mode
cargo build --release -p fit-cli
```

## Running Tauri GUI Desktop App

```bash
cd fit-gui
npm install
cargo tauri dev
```
