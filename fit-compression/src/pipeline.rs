use crate::tournament::CompressionTournament;
use fit_core::{CompressionConfig, CompressionLevel, FitResult};
use fit_format::CompressionMethod;
use rayon::prelude::*;

pub struct CompressionEngine {
    config: CompressionConfig,
}

impl CompressionEngine {
    pub fn new(config: CompressionConfig) -> Self {
        Self { config }
    }

    pub fn compress_buffer(&self, input: &[u8]) -> FitResult<(CompressionMethod, Vec<u8>)> {
        let is_research = self.config.level == CompressionLevel::Research
            || self.config.level == CompressionLevel::Extreme;
        CompressionTournament::select_best(input, is_research)
    }

    pub fn decompress_buffer(&self, method: CompressionMethod, data: &[u8]) -> FitResult<Vec<u8>> {
        CompressionTournament::decompress_method(method, data)
    }

    pub fn compress_chunks_parallel(&self, chunks: &[Vec<u8>]) -> Vec<FitResult<(CompressionMethod, Vec<u8>)>> {
        let is_research = self.config.level == CompressionLevel::Research
            || self.config.level == CompressionLevel::Extreme;

        chunks
            .par_iter()
            .map(|chunk| CompressionTournament::select_best(chunk, is_research))
            .collect()
    }
}
