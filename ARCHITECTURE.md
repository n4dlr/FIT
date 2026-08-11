# FIT System Architecture

```
FIT Workspace
├── fit-core           # Fundamental domain types, errors, progress channels, system traits
├── fit-format         # Binary .fit format header framing, chunk table, bincode metadata
├── fit-compression    # Transforms (Delta, RLE, BWT/MTF, Predictor), Coders (LZ77, Huffman, Range, CDC), Tournament Orchestrator
├── fit-archive        # Archive tree builder, solid block grouping, streaming reader/writer
├── fit-detection      # Magic-byte file classifier, entropy analyzer, format detector
├── fit-crypto         # Key derivation (Argon2id), Authenticated Encryption (ChaCha20-Poly1305)
├── fit-recovery       # Reed-Solomon parity records, CRC32 / xxHash64 / SHA-256 integrity verification
├── fit-plugins        # Extensible plugin system & readers for ZIP, TAR, GZIP, 7Z, XZ, Zstd
├── fit-cli            # Command-line binary (`fit`)
└── fit-gui            # Tauri 2 Desktop GUI application (Rust backend + React/TS frontend)
```

## Modular Pipeline & Compression Tournament

1. **Content Analysis**: Detect file magic bytes, MIME types, and entropy.
2. **Transform Stage**: Run Delta, Byte-RLE, BWT+MTF, or Context Predictors.
3. **Entropy Stage**: Apply Huffman coding or Range/Arithmetic coding.
4. **Tournament Selection**: Concurrently evaluate candidates (Pipeline A, B, C, D, E) and verify byte losslessness via SHA-256 before storing the winning stream.
