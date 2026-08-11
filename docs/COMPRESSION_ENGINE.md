# FIT Extreme Compression Engine & Tournament Architecture

The FIT compression engine dynamically optimizes compression ratio, CPU time, and data losslessness using a multi-pass **Compression Tournament**.

## Pipelines & Transforms

1. **Delta Encoding (`DeltaTransform`)**: Replaces absolute byte values with sequential residuals (`b[i] - b[i-1]`). Highly effective on numerical datasets, CSVs, and audio/sensor logs.
2. **Run-Length Encoding (`RleTransform`)**: Packs repeating byte runs into `(count, byte)` pairs.
3. **Burrows-Wheeler Transform + MTF (`BwtMtfTransform`)**: Reorders data into clusters of identical characters followed by Move-To-Front indexing.
4. **Context Predictors (`ContextPredictorTransform`)**: Tracks previous byte contexts to predict incoming bytes, storing only the prediction residual.
5. **LZ77 Match Finder (`Lz77Matcher`)**: Sliding-window dictionary matcher generating literal/match tokens.
6. **Entropy Coders**:
   - `HuffmanCoder`: Canonical Huffman variable-length bit stream encoder.
   - `RangeCoder`: Arithmetic range coder for precise fractional-bit entropy reduction.

## Tournament Decision Procedure

For each chunk:
1. Run candidates A, B, C, D, E in parallel via `rayon`.
2. Decompress each candidate back to uncompressed state.
3. Calculate SHA-256 digest: `SHA-256(original) == SHA-256(decompressed)`.
4. Discard any candidate failing hash verification or expanding beyond raw size.
5. Store winning representation.
