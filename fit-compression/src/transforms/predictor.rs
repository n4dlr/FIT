use fit_core::{FitResult, Transform};

pub struct ContextPredictorTransform;

impl Transform for ContextPredictorTransform {
    fn name(&self) -> &'static str {
        "ContextPredictor"
    }

    fn transform(&self, input: &[u8]) -> FitResult<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }
        let mut table = vec![0u8; 256];
        let mut output = Vec::with_capacity(input.len());

        let mut prev = 0u8;
        for &actual in input {
            let pred = table[prev as usize];
            let diff = actual.wrapping_sub(pred);
            output.push(diff);
            table[prev as usize] = actual;
            prev = actual;
        }
        Ok(output)
    }

    fn inverse(&self, input: &[u8]) -> FitResult<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }
        let mut table = vec![0u8; 256];
        let mut output = Vec::with_capacity(input.len());

        let mut prev = 0u8;
        for &diff in input {
            let pred = table[prev as usize];
            let actual = pred.wrapping_add(diff);
            output.push(actual);
            table[prev as usize] = actual;
            prev = actual;
        }
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_predictor_roundtrip() {
        let transform = ContextPredictorTransform;
        let data = b"The quick brown fox jumps over the lazy dog repeatedly and repeatedly.";
        let encoded = transform.transform(data).unwrap();
        let decoded = transform.inverse(&encoded).unwrap();
        assert_eq!(data.to_vec(), decoded);
    }
}
