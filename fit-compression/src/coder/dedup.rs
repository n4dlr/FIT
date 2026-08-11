use sha2::{Digest, Sha256};
use std::collections::HashMap;

pub struct ChunkRef {
    pub hash: [u8; 32],
    pub offset: u64,
    pub length: u32,
}

pub struct FastCdcChunker {
    pub min_size: usize,
    pub avg_size: usize,
    pub max_size: usize,
}

impl Default for FastCdcChunker {
    fn default() -> Self {
        Self {
            min_size: 4096,
            avg_size: 16384,
            max_size: 65536,
        }
    }
}

impl FastCdcChunker {
    pub fn chunk_data(&self, data: &[u8]) -> Vec<(usize, usize)> {
        let mut chunks = Vec::new();
        if data.is_empty() {
            return chunks;
        }

        let mut start = 0;
        let mask = (self.avg_size - 1) as u64;

        while start < data.len() {
            let mut end = (start + self.min_size).min(data.len());
            let max = (start + self.max_size).min(data.len());

            let mut hash: u64 = 0;
            while end < max {
                let byte = data[end] as u64;
                hash = (hash << 1).wrapping_add(byte.wrapping_mul(0x45d9f3b));
                if (hash & mask) == 0 {
                    end += 1;
                    break;
                }
                end += 1;
            }

            chunks.push((start, end - start));
            start = end;
        }
        chunks
    }
}

pub struct DeduplicationIndex {
    chunk_map: HashMap<[u8; 32], Vec<u8>>,
}

impl Default for DeduplicationIndex {
    fn default() -> Self {
        Self {
            chunk_map: HashMap::new(),
        }
    }
}

impl DeduplicationIndex {
    pub fn insert_or_get(&mut self, chunk: &[u8]) -> ([u8; 32], bool) {
        let mut hasher = Sha256::new();
        hasher.update(chunk);
        let hash: [u8; 32] = hasher.finalize().into();

        if self.chunk_map.contains_key(&hash) {
            (hash, true) // Already stored!
        } else {
            self.chunk_map.insert(hash, chunk.to_vec());
            (hash, false) // New chunk
        }
    }

    pub fn get_chunk(&self, hash: &[u8; 32]) -> Option<&Vec<u8>> {
        self.chunk_map.get(hash)
    }
}
