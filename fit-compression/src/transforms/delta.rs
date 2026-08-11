use fit_core::{FitResult, Transform};

pub struct DeltaTransform;

impl Transform for DeltaTransform {
    fn name(&self) -> &'static str {
        "Delta"
    }

    fn transform(&self, input: &[u8]) -> FitResult<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }
        let mut output = Vec::with_capacity(input.len());
        output.push(input[0]);
        for i in 1..input.len() {
            output.push(input[i].wrapping_sub(input[i - 1]));
        }
        Ok(output)
    }

    fn inverse(&self, input: &[u8]) -> FitResult<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }
        let mut output = Vec::with_capacity(input.len());
        let mut current = input[0];
        output.push(current);
        for &byte in &input[1..] {
            current = current.wrapping_add(byte);
            output.push(current);
        }
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_roundtrip() {
        let transform = DeltaTransform;
        let data = vec![10, 12, 15, 20, 25, 20, 10];
        let encoded = transform.transform(&data).unwrap();
        let decoded = transform.inverse(&encoded).unwrap();
        assert_eq!(data, decoded);
    }
}
