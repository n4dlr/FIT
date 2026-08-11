use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionPhase {
    Analyzing,
    Deduplicating,
    RunningTournament,
    Compressing,
    GeneratingRecovery,
    VerifyingIntegrity,
    Complete,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressReport {
    pub phase: CompressionPhase,
    pub current_file: String,
    pub bytes_processed: u64,
    pub total_bytes: u64,
    pub current_compressed_bytes: u64,
    pub percent_complete: f32,
    pub current_speed_bytes_sec: f64,
    pub selected_strategy: String,
}

pub type ProgressCallback = Box<dyn Fn(ProgressReport) + Send + Sync>;
