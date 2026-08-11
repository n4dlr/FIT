use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionMethod {
    Raw,
    Lz77Huffman,
    DeltaPredictorRange,
    BwtMtfRle,
    ContextPredictorRange,
    CdcDeduplicated,
    Custom(u16),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkHeader {
    pub id: u32,
    pub original_size: u32,
    pub compressed_size: u32,
    pub method: CompressionMethod,
    pub crc32: u32,
    pub xxhash64: u64,
}
