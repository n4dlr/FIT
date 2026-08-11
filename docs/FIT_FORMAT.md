# FIT Binary Format Specification v1

Magic Signature: `0x46 0x49 0x54 0x01` (`FIT\x01`)

```
+-------------------------------------------------------------+
| Header (32 Bytes)                                           |
| - Magic: "FIT\x01" (4 bytes)                                |
| - Version: u16                                              |
| - Flags: u32 (Solid, Encrypted, Deduplicated, Recovery)     |
| - Timestamp: u64                                            |
| - Entry Count: u32                                          |
| - Chunk Count: u32                                          |
| - Metadata Offset: u64                                      |
| - Recovery Offset: u64                                      |
+-------------------------------------------------------------+
| Encrypted Header Salt (16 Bytes, Optional)                  |
| Encrypted Header Nonce (12 Bytes, Optional)                 |
+-------------------------------------------------------------+
| Compressed Data Payload Blocks                              |
| [Chunk 0 Header + Encrypted/Raw Compressed Bytes]           |
| [Chunk 1 Header + Encrypted/Raw Compressed Bytes]           |
| ...                                                         |
+-------------------------------------------------------------+
| Metadata Table Offset                                       |
| - Length (u32)                                              |
| - Bincode ArchiveEntryHeader Array                          |
+-------------------------------------------------------------+
| Recovery Records Offset                                     |
| - Length (u32)                                              |
| - Reed-Solomon Parity Shards Data                           |
+-------------------------------------------------------------+
```

## Forward & Backward Compatibility
- Unknown bit flags in the Header field must be ignored by older readers unless an unsupported feature flag prevents safe extraction.
- Serialized metadata structs utilize extensible `bincode` framing.
