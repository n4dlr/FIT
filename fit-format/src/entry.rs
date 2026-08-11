use fit_core::FileMetadata;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveEntryHeader {
    pub id: u32,
    pub metadata: FileMetadata,
    pub sha256: [u8; 32],
    pub chunk_indices: Vec<u32>,
}
