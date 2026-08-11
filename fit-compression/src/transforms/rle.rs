use fit_core::{FitError, FitResult, Transform};

pub struct RleTransform;

impl Transform for RleTransform {
    fn name(&self) -> &'static str {
        "RLE"
    }

    fn transform(&self, input: &[u8]) -> FitResult<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }
        let mut output = Vec::with_capacity(input.len());
        let mut i = 0;
        while i < input.len() {
            let byte = input[i];
            let mut count = 1u8;
            while i + (count as usize) < input.len() && count < 255 && input[i + (count as usize)] == byte {
                count += 1;
            }
            output.push(count);
            output.push(byte);
            i += count as usize;
        }
        Ok(output)
    }

    fn inverse(&self, input: &[u8]) -> FitResult<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }
        if input.len() % 2 != 0 {
            return Err(FitError::DecompressionFailed("Corrupted RLE data stream".into()));
        }
        let mut output = Vec::new();
        for chunk in input.chunks_exact(2) {
            let count = chunk[0] as usize;
            let byte = chunk[1];
            output.extend(std::iter::repeat(byte).take(count));
        }
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rle_roundtrip() {
        let transform = RleTransform;
        let data = b"AAAAABBBCCCDDDDDDD";
        let encoded = transform.transform(data).unwrap();
        let decoded = transform.inverse(&encoded).unwrap();
        assert_eq!(data.to_vec(), decoded);
    }
}
