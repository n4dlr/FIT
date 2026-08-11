use crate::coder::{HuffmanCoder, Lz77Matcher, RangeCoder};
use crate::transforms::{BwtMtfTransform, ContextPredictorTransform, DeltaTransform, RleTransform};
use fit_core::{FitError, FitResult, Transform};
use fit_format::CompressionMethod;
use sha2::{Digest, Sha256};

pub struct CandidateResult {
    pub method: CompressionMethod,
    pub data: Vec<u8>,
    pub size: usize,
    pub verified: bool,
}

pub struct CompressionTournament;

impl CompressionTournament {
    pub fn select_best(input: &[u8], is_research_mode: bool) -> FitResult<(CompressionMethod, Vec<u8>)> {
        if input.is_empty() {
            return Ok((CompressionMethod::Raw, Vec::new()));
        }

        let orig_hash = Self::sha256_hash(input);
        let mut candidates: Vec<CandidateResult> = Vec::new();

        // 1. Raw candidate (Baseline)
        candidates.push(CandidateResult {
            method: CompressionMethod::Raw,
            data: input.to_vec(),
            size: input.len(),
            verified: true,
        });

        // 2. Pipeline A: LZ77 + Huffman
        if let Ok(lz_tokens) = Ok::<_, FitError>(Lz77Matcher::default().compress(input)) {
            let encoded_tokens = Lz77Matcher::default().encode_tokens(&lz_tokens);
            if let Ok(huff_compressed) = HuffmanCoder::compress(&encoded_tokens) {
                if Self::verify_pipeline_a(input, &huff_compressed, &orig_hash) {
                    candidates.push(CandidateResult {
                        method: CompressionMethod::Lz77Huffman,
                        data: huff_compressed,
                        size: 0, // set below
                        verified: true,
                    });
                    let idx = candidates.len() - 1;
                    candidates[idx].size = candidates[idx].data.len();
                }
            }
        }

        // 3. Pipeline B: Delta + Context Predictor + Range Coder
        if let Ok(delta_bytes) = DeltaTransform.transform(input) {
            if let Ok(pred_bytes) = ContextPredictorTransform.transform(&delta_bytes) {
                if let Ok(range_compressed) = RangeCoder::compress(&pred_bytes) {
                    if Self::verify_pipeline_b(input, &range_compressed, &orig_hash) {
                        candidates.push(CandidateResult {
                            method: CompressionMethod::DeltaPredictorRange,
                            data: range_compressed,
                            size: 0,
                            verified: true,
                        });
                        let idx = candidates.len() - 1;
                        candidates[idx].size = candidates[idx].data.len();
                    }
                }
            }
        }

        // 4. Pipeline C: BWT + MTF + RLE + Huffman
        if is_research_mode || input.len() <= 1024 * 1024 {
            if let Ok(bwt_mtf_bytes) = BwtMtfTransform.transform(input) {
                if let Ok(rle_bytes) = RleTransform.transform(&bwt_mtf_bytes) {
                    if let Ok(huff_compressed) = HuffmanCoder::compress(&rle_bytes) {
                        if Self::verify_pipeline_c(input, &huff_compressed, &orig_hash) {
                            candidates.push(CandidateResult {
                                method: CompressionMethod::BwtMtfRle,
                                data: huff_compressed,
                                size: 0,
                                verified: true,
                            });
                            let idx = candidates.len() - 1;
                            candidates[idx].size = candidates[idx].data.len();
                        }
                    }
                }
            }
        }

        // 5. Pipeline D: Context Predictor + Range Coder
        if let Ok(pred_bytes) = ContextPredictorTransform.transform(input) {
            if let Ok(range_compressed) = RangeCoder::compress(&pred_bytes) {
                if Self::verify_pipeline_d(input, &range_compressed, &orig_hash) {
                    candidates.push(CandidateResult {
                        method: CompressionMethod::ContextPredictorRange,
                        data: range_compressed,
                        size: 0,
                        verified: true,
                    });
                    let idx = candidates.len() - 1;
                    candidates[idx].size = candidates[idx].data.len();
                }
            }
        }

        // Select smallest verified candidate
        candidates.retain(|c| c.verified);
        candidates.sort_by_key(|c| c.size);

        let winner = candidates.into_iter().next().ok_or_else(|| {
            FitError::CompressionFailed("No compression candidates passed verification".into())
        })?;

        Ok((winner.method, winner.data))
    }

    pub fn decompress_method(method: CompressionMethod, data: &[u8]) -> FitResult<Vec<u8>> {
        match method {
            CompressionMethod::Raw => Ok(data.to_vec()),
            CompressionMethod::Lz77Huffman => {
                let huff_decomp = HuffmanCoder::decompress(data)?;
                let tokens = Lz77Matcher::default().decode_tokens(&huff_decomp)?;
                Lz77Matcher::default().decompress(&tokens)
            }
            CompressionMethod::DeltaPredictorRange => {
                let range_decomp = RangeCoder::decompress(data)?;
                let pred_decomp = ContextPredictorTransform.inverse(&range_decomp)?;
                DeltaTransform.inverse(&pred_decomp)
            }
            CompressionMethod::BwtMtfRle => {
                let huff_decomp = HuffmanCoder::decompress(data)?;
                let rle_decomp = RleTransform.inverse(&huff_decomp)?;
                BwtMtfTransform.inverse(&rle_decomp)
            }
            CompressionMethod::ContextPredictorRange => {
                let range_decomp = RangeCoder::decompress(data)?;
                ContextPredictorTransform.inverse(&range_decomp)
            }
            CompressionMethod::CdcDeduplicated | CompressionMethod::Custom(_) => Ok(data.to_vec()),
        }
    }

    fn sha256_hash(data: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().into()
    }

    fn verify_pipeline_a(_original: &[u8], compressed: &[u8], orig_hash: &[u8; 32]) -> bool {
        if let Ok(decomp) = Self::decompress_method(CompressionMethod::Lz77Huffman, compressed) {
            Self::sha256_hash(&decomp) == *orig_hash
        } else {
            false
        }
    }

    fn verify_pipeline_b(_original: &[u8], compressed: &[u8], orig_hash: &[u8; 32]) -> bool {
        if let Ok(decomp) = Self::decompress_method(CompressionMethod::DeltaPredictorRange, compressed) {
            Self::sha256_hash(&decomp) == *orig_hash
        } else {
            false
        }
    }

    fn verify_pipeline_c(_original: &[u8], compressed: &[u8], orig_hash: &[u8; 32]) -> bool {
        if let Ok(decomp) = Self::decompress_method(CompressionMethod::BwtMtfRle, compressed) {
            Self::sha256_hash(&decomp) == *orig_hash
        } else {
            false
        }
    }

    fn verify_pipeline_d(_original: &[u8], compressed: &[u8], orig_hash: &[u8; 32]) -> bool {
        if let Ok(decomp) = Self::decompress_method(CompressionMethod::ContextPredictorRange, compressed) {
            Self::sha256_hash(&decomp) == *orig_hash
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tournament_on_repetitive_text() {
        let data = b"LOG [2026-08-11 10:00:01] INFO User logged in. LOG [2026-08-11 10:00:02] INFO User logged in. ".repeat(50);
        let (method, compressed) = CompressionTournament::select_best(&data, true).unwrap();
        assert!(compressed.len() < data.len());
        let decompressed = CompressionTournament::decompress_method(method, &compressed).unwrap();
        assert_eq!(data, decompressed.as_slice());
    }
}
