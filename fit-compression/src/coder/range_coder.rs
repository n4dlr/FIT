use fit_core::{FitError, FitResult};

/// Production-grade Range Coder using Schindler's carry-propagating algorithm.
///
/// Header framing:
///   - u32 original_len
///   - u32[256] byte frequencies (scaled so sum <= 65535)
///   - bitstream payload
pub struct RangeCoder;

const TOP: u32 = 1 << 24; // 0x0100_0000

impl RangeCoder {
    pub fn compress(input: &[u8]) -> FitResult<Vec<u8>> {
        let orig_len = input.len() as u32;
        if orig_len == 0 {
            // Header for empty file: 4 bytes orig_len (0) + 256 * 4 bytes freq (0)
            return Ok(vec![0u8; 4 + 256 * 4]);
        }

        // Compute raw symbol counts
        let mut counts = [0u32; 256];
        for &b in input {
            counts[b as usize] += 1;
        }

        // Scale frequencies so total sum <= 65535
        let total_raw = input.len() as u64;
        let target_total: u64 = 65535;
        let mut freq = [0u32; 256];
        for i in 0..256 {
            if counts[i] > 0 {
                let f = ((counts[i] as u64 * target_total) / total_raw).max(1) as u32;
                freq[i] = f;
            }
        }

        // Compute cumulative frequencies
        let mut cum = [0u32; 257];
        for i in 0..256 {
            cum[i + 1] = cum[i] + freq[i];
        }
        let total = cum[256];

        // Encoder state
        let mut low: u64 = 0;
        let mut range: u32 = 0xFFFF_FFFF;
        let mut cache: u8 = 0;
        let mut carry_count: u32 = 0;
        let mut first_byte = true;

        let mut payload = Vec::with_capacity(input.len());

        for &b in input {
            let s = b as usize;
            let r = range / total;
            low += (r as u64) * (cum[s] as u64);
            range = r * freq[s];
            if range == 0 {
                range = 1;
            }

            while range < TOP {
                let carry = (low >> 32) as u8;
                let top_byte = ((low >> 24) & 0xFF) as u8;

                if carry != 0 || top_byte != 0xFF {
                    if !first_byte {
                        payload.push(cache.wrapping_add(carry));
                        for _ in 0..carry_count {
                            payload.push(if carry != 0 { 0x00 } else { 0xFF });
                        }
                    } else {
                        first_byte = false;
                    }
                    cache = top_byte;
                    carry_count = 0;
                } else {
                    carry_count += 1;
                }

                low = (low & 0x00FF_FFFF) << 8;
                range <<= 8;
            }
        }

        // Flush remaining bits
        let carry = (low >> 32) as u8;
        let top_byte = ((low >> 24) & 0xFF) as u8;
        if !first_byte {
            payload.push(cache.wrapping_add(carry));
            for _ in 0..carry_count {
                payload.push(if carry != 0 { 0x00 } else { 0xFF });
            }
        }
        payload.push(top_byte);
        payload.push(((low >> 16) & 0xFF) as u8);
        payload.push(((low >> 8) & 0xFF) as u8);
        payload.push((low & 0xFF) as u8);

        // Build header + payload
        let mut out = Vec::with_capacity(4 + 256 * 4 + payload.len());
        out.extend_from_slice(&orig_len.to_be_bytes());
        for &f in &freq {
            out.extend_from_slice(&f.to_be_bytes());
        }
        out.extend_from_slice(&payload);

        Ok(out)
    }

    pub fn decompress(input: &[u8]) -> FitResult<Vec<u8>> {
        let hdr_size = 4 + 256 * 4;
        if input.len() < hdr_size {
            return Err(FitError::DecompressionFailed("RangeCoder: header too short".into()));
        }

        let orig_len = u32::from_be_bytes([input[0], input[1], input[2], input[3]]) as usize;
        if orig_len == 0 {
            return Ok(Vec::new());
        }

        let mut freq = [0u32; 256];
        for i in 0..256 {
            let off = 4 + i * 4;
            freq[i] = u32::from_be_bytes([input[off], input[off + 1], input[off + 2], input[off + 3]]);
        }

        let mut cum = [0u32; 257];
        for i in 0..256 {
            cum[i + 1] = cum[i] + freq[i];
        }
        let total = cum[256];
        if total == 0 {
            return Ok(Vec::new());
        }

        let payload = &input[hdr_size..];
        let mut pi = 0usize;

        let get_byte = |payload: &[u8], pi: &mut usize| -> u8 {
            if *pi < payload.len() {
                let b = payload[*pi];
                *pi += 1;
                b
            } else {
                0
            }
        };

        // Initialize code buffer with first 4 bytes of payload
        let mut code: u32 = 0;
        for _ in 0..4 {
            code = (code << 8) | (get_byte(payload, &mut pi) as u32);
        }

        let mut low: u32 = 0;
        let mut range: u32 = 0xFFFF_FFFF;
        let mut output = Vec::with_capacity(orig_len);

        for _ in 0..orig_len {
            let r = range / total;
            let count = (code.wrapping_sub(low)) / r;

            // Binary search symbol `s` such that cum[s] <= count < cum[s+1]
            let mut lo_idx = 0usize;
            let mut hi_idx = 256usize;
            while lo_idx + 1 < hi_idx {
                let mid = (lo_idx + hi_idx) / 2;
                if cum[mid] <= count {
                    lo_idx = mid;
                } else {
                    hi_idx = mid;
                }
            }
            let s = lo_idx;

            low = low.wrapping_add(r * cum[s]);
            range = r * freq[s];
            if range == 0 {
                range = 1;
            }

            while range < TOP {
                code = (code << 8) | (get_byte(payload, &mut pi) as u32);
                low <<= 8;
                range <<= 8;
            }

            output.push(s as u8);
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_coder_roundtrip() {
        let data = b"Range Coder lossless test AABBCCDD 112233.";
        let c = RangeCoder::compress(data).unwrap();
        let d = RangeCoder::decompress(&c).unwrap();
        assert_eq!(data.to_vec(), d, "roundtrip mismatch");
    }

    #[test]
    fn test_range_coder_all_same() {
        let data = vec![77u8; 500];
        let c = RangeCoder::compress(&data).unwrap();
        let d = RangeCoder::decompress(&c).unwrap();
        assert_eq!(data, d);
    }

    #[test]
    fn test_range_coder_empty() {
        let data: Vec<u8> = vec![];
        let c = RangeCoder::compress(&data).unwrap();
        let d = RangeCoder::decompress(&c).unwrap();
        assert_eq!(data, d);
    }
}
