# FIT — Extreme Lossless Compression & Universal Archive Platform

> **Tagline**: *Push the Limits of Lossless Compression.*

`FIT` is a modern, secure, and extremely fast universal archive platform and custom lossless compression engine designed as a next-generation alternative to ZIP, 7-Zip, and WinRAR.

---

## Key Features

- **100% Lossless Guarantee**: Every operation is strictly verified (`SHA-256(original) == SHA-256(decompressed)`).
- **Compression Tournament Architecture**: Automatically runs multi-stage compression pipelines (LZ77, Context Predictors, BWT+MTF, Delta, Huffman, Range Coding) and selects the smallest valid representation.
- **Solid Archives & Deduplication**: FastCDC content-defined chunking and solid block packing across multi-file archives.
- **Authenticated Encryption**: Memory-hard key derivation via Argon2id with ChaCha20-Poly1305 payload & metadata encryption.
- **Reed-Solomon Error Correction**: Parity recovery records for corruption repair and archive integrity testing (`fit test`, `fit repair`).
- **Universal Archive Plugin System**: Extensible architecture supporting ZIP, TAR, GZIP, BZIP2, XZ, Zstd, and 7Z archives.
- **Nested Container Explorer**: Recursively inspects nested archives up to 32 levels deep without full extraction.
- **Cross-Platform**: Real working CLI (`fit`) and Tauri 2 desktop GUI written in Rust, TypeScript, React, and Tailwind CSS.

---

## Quick Start

### CLI Usage

```bash
# Compress directory into archive.fit
fit compress ./my_data -o archive.fit --level extreme

# Extract archive
fit extract archive.fit -o ./output

# Test integrity
fit test archive.fit

# Benchmark dataset
fit benchmark ./dataset.json
```

### Building from Source

```bash
# Build CLI
cargo build --release --bin fit

# Run GUI
cd fit-gui && npm install && cargo tauri dev
```
