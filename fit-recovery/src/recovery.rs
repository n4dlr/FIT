use fit_core::{FitError, FitResult};
use reed_solomon_erasure::galois_8::ReedSolomon;

pub struct RecoveryEngine;

impl RecoveryEngine {
    pub fn generate_parity(data: &[u8], parity_percent: u8) -> FitResult<(Vec<Vec<u8>>, usize, usize)> {
        if data.is_empty() {
            return Ok((Vec::new(), 0, 0));
        }

        let data_shards = 10;
        let parity_shards = ((data_shards * parity_percent as usize) / 100).max(1);

        let r = ReedSolomon::new(data_shards, parity_shards)
            .map_err(|e| FitError::RecoveryFailed(format!("ReedSolomon init failed: {:?}", e)))?;

        let shard_len = (data.len() + data_shards - 1) / data_shards;
        let mut shards: Vec<Vec<u8>> = vec![vec![0u8; shard_len]; data_shards + parity_shards];

        for (i, chunk) in data.chunks(shard_len).enumerate() {
            shards[i][..chunk.len()].copy_from_slice(chunk);
        }

        r.encode(&mut shards)
            .map_err(|e| FitError::RecoveryFailed(format!("ReedSolomon encoding failed: {:?}", e)))?;

        let parity_data = shards[data_shards..].to_vec();
        Ok((parity_data, data_shards, parity_shards))
    }

    pub fn reconstruct_data(
        mut data_shards: Vec<Option<Vec<u8>>>,
        mut parity_shards: Vec<Option<Vec<u8>>>,
        original_len: usize,
    ) -> FitResult<Vec<u8>> {
        let data_count = data_shards.len();
        let parity_count = parity_shards.len();

        let r = ReedSolomon::new(data_count, parity_count)
            .map_err(|e| FitError::RecoveryFailed(format!("ReedSolomon init failed: {:?}", e)))?;

        let mut all_shards: Vec<Option<Vec<u8>>> = Vec::with_capacity(data_count + parity_count);
        all_shards.append(&mut data_shards);
        all_shards.append(&mut parity_shards);

        r.reconstruct(&mut all_shards)
            .map_err(|e| FitError::RecoveryFailed(format!("ReedSolomon reconstruction failed: {:?}", e)))?;

        let mut reconstructed = Vec::with_capacity(original_len);
        for i in 0..data_count {
            if let Some(ref shard) = all_shards[i] {
                reconstructed.extend_from_slice(shard);
            }
        }
        reconstructed.truncate(original_len);
        Ok(reconstructed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_roundtrip_with_corruption() {
        let original_data = b"FIT Archive Recovery System with Reed-Solomon Erasure Coding test dataset.";
        let (parity, data_shards_cnt, _parity_shards_cnt) =
            RecoveryEngine::generate_parity(original_data, 20).unwrap();

        let shard_len = (original_data.len() + data_shards_cnt - 1) / data_shards_cnt;
        let mut data_shards: Vec<Option<Vec<u8>>> = Vec::new();

        for (i, chunk) in original_data.chunks(shard_len).enumerate() {
            let mut buf = vec![0u8; shard_len];
            buf[..chunk.len()].copy_from_slice(chunk);
            if i == 2 {
                // Simulate lost/corrupted shard 2!
                data_shards.push(None);
            } else {
                data_shards.push(Some(buf));
            }
        }

        let parity_shards: Vec<Option<Vec<u8>>> = parity.into_iter().map(Some).collect();

        let restored =
            RecoveryEngine::reconstruct_data(data_shards, parity_shards, original_data.len()).unwrap();
        assert_eq!(original_data.to_vec(), restored);
    }
}
