use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionLevel {
    Fast,
    Balanced,
    High,
    Ultra,
    Extreme,
    Research,
}

impl Default for CompressionLevel {
    fn default() -> Self {
        CompressionLevel::Balanced
    }
}

impl fmt::Display for CompressionLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompressionLevel::Fast => write!(f, "Fast"),
            CompressionLevel::Balanced => write!(f, "Balanced"),
            CompressionLevel::High => write!(f, "High"),
            CompressionLevel::Ultra => write!(f, "Ultra"),
            CompressionLevel::Extreme => write!(f, "Extreme"),
            CompressionLevel::Research => write!(f, "Research"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SolidMode {
    NonSolid,
    Solid,
    Auto,
}

impl Default for SolidMode {
    fn default() -> Self {
        SolidMode::Auto
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    pub level: CompressionLevel,
    pub solid: SolidMode,
    pub deduplication: bool,
    pub encryption_password: Option<String>,
    pub recovery_percent: u8,
    pub threads: usize,
    pub block_size: usize,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            level: CompressionLevel::Balanced,
            solid: SolidMode::Auto,
            deduplication: true,
            encryption_password: None,
            recovery_percent: 5,
            threads: rayon::current_num_threads(),
            block_size: 4 * 1024 * 1024, // 4MB chunks default
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub relative_path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub created_at: u64,
    pub modified_at: u64,
    pub mode: u32,
    pub is_symlink: bool,
    pub symlink_target: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionStats {
    pub original_size: u64,
    pub compressed_size: u64,
    pub ratio: f64,
    pub space_saved_percent: f64,
    pub duration_ms: u64,
    pub algorithm_used: String,
    pub sha256_verified: bool,
}
