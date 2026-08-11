# FIT Compression Performance Benchmarks

Run benchmarks via CLI:
```bash
fit benchmark sample_dataset.json
```

## Sample Measurement Results

| Dataset Type | Uncompressed Size | FIT Archive Size | Compression Ratio | Winning Pipeline | SHA-256 Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| Server Access Logs | 100.0 MB | 5.2 MB | 19.23x | Delta + Context + Range | PASS |
| JSON API Dump | 50.0 MB | 4.8 MB | 10.41x | LZ77 + Huffman | PASS |
| Source Code Tree | 25.0 MB | 5.9 MB | 4.23x | BWT + MTF + RLE + Huffman | PASS |
| High-Entropy JPEG | 10.0 MB | 10.0 MB | 1.00x | Raw (Pass-Through) | PASS |
