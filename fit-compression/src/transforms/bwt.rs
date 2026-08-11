use fit_core::{FitError, FitResult, Transform};

pub struct BwtMtfTransform;

impl BwtMtfTransform {
    pub fn bwt(input: &[u8]) -> (Vec<u8>, usize) {
        if input.is_empty() {
            return (Vec::new(), 0);
        }
        let n = input.len();
        let mut double_input = Vec::with_capacity(n * 2);
        double_input.extend_from_slice(input);
        double_input.extend_from_slice(input);

        let mut sa: Vec<usize> = (0..n).collect();
        sa.sort_by(|&a, &b| {
            let shift_a = &double_input[a..a + n];
            let shift_b = &double_input[b..b + n];
            shift_a.cmp(shift_b)
        });

        let mut primary_index = 0;
        let mut bwt_bytes = Vec::with_capacity(n);
        for (idx, &sa_val) in sa.iter().enumerate() {
            if sa_val == 0 {
                primary_index = idx;
                bwt_bytes.push(input[n - 1]);
            } else {
                bwt_bytes.push(input[sa_val - 1]);
            }
        }
        (bwt_bytes, primary_index)
    }

    pub fn ibwt(bwt_bytes: &[u8], primary_index: usize) -> FitResult<Vec<u8>> {
        if bwt_bytes.is_empty() {
            return Ok(Vec::new());
        }
        let n = bwt_bytes.len();
        if primary_index >= n {
            return Err(FitError::DecompressionFailed("Invalid BWT primary index".into()));
        }

        let mut counts = [0usize; 256];
        for &b in bwt_bytes {
            counts[b as usize] += 1;
        }

        let mut cumsum = [0usize; 256];
        let mut sum = 0;
        for i in 0..256 {
            cumsum[i] = sum;
            sum += counts[i];
        }

        let mut next = vec![0usize; n];
        for (i, &b) in bwt_bytes.iter().enumerate() {
            let byte_idx = b as usize;
            next[i] = cumsum[byte_idx];
            cumsum[byte_idx] += 1;
        }

        let mut output = Vec::with_capacity(n);
        let mut curr = primary_index;
        for _ in 0..n {
            output.push(bwt_bytes[curr]);
            curr = next[curr];
        }
        output.reverse();
        Ok(output)
    }

    pub fn mtf(input: &[u8]) -> Vec<u8> {
        let mut alphabet: Vec<u8> = (0..=255).collect();
        let mut output = Vec::with_capacity(input.len());
        for &byte in input {
            let pos = alphabet.iter().position(|&b| b == byte).unwrap_or(0);
            output.push(pos as u8);
            let val = alphabet.remove(pos);
            alphabet.insert(0, val);
        }
        output
    }

    pub fn imtf(input: &[u8]) -> Vec<u8> {
        let mut alphabet: Vec<u8> = (0..=255).collect();
        let mut output = Vec::with_capacity(input.len());
        for &pos in input {
            let idx = pos as usize;
            if idx < alphabet.len() {
                let val = alphabet.remove(idx);
                output.push(val);
                alphabet.insert(0, val);
            }
        }
        output
    }
}

impl Transform for BwtMtfTransform {
    fn name(&self) -> &'static str {
        "BWT+MTF"
    }

    fn transform(&self, input: &[u8]) -> FitResult<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }
        let (bwt_bytes, p_idx) = Self::bwt(input);
        let mtf_bytes = Self::mtf(&bwt_bytes);
        let mut output = Vec::with_capacity(4 + mtf_bytes.len());
        output.extend_from_slice(&(p_idx as u32).to_be_bytes());
        output.extend_from_slice(&mtf_bytes);
        Ok(output)
    }

    fn inverse(&self, input: &[u8]) -> FitResult<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }
        if input.len() < 4 {
            return Err(FitError::DecompressionFailed("Invalid BWT stream length".into()));
        }
        let p_idx = u32::from_be_bytes([input[0], input[1], input[2], input[3]]) as usize;
        let mtf_bytes = &input[4..];
        let bwt_bytes = Self::imtf(mtf_bytes);
        Self::ibwt(&bwt_bytes, p_idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bwt_mtf_roundtrip() {
        let transform = BwtMtfTransform;
        let data = b"banana_apple_banana_pineapple";
        let encoded = transform.transform(data).unwrap();
        let decoded = transform.inverse(&encoded).unwrap();
        assert_eq!(data.to_vec(), decoded);
    }
}
