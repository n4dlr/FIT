use fit_core::{FitError, FitResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LzToken {
    Literal(u8),
    Match { distance: u16, length: u16 },
}

pub struct Lz77Matcher {
    pub window_size: usize,
    pub max_match_len: usize,
    pub min_match_len: usize,
}

impl Default for Lz77Matcher {
    fn default() -> Self {
        Self {
            window_size: 32768,
            max_match_len: 258,
            min_match_len: 3,
        }
    }
}

impl Lz77Matcher {
    pub fn compress(&self, input: &[u8]) -> Vec<LzToken> {
        let mut tokens = Vec::with_capacity(input.len());
        let mut pos = 0;

        while pos < input.len() {
            let window_start = pos.saturating_sub(self.window_size);
            let window = &input[window_start..pos];

            let mut best_len = 0;
            let mut best_dist = 0;

            let max_possible = (input.len() - pos).min(self.max_match_len);
            if max_possible >= self.min_match_len {
                for offset in 1..=window.len() {
                    let match_start = pos - offset;
                    let mut match_len = 0;
                    while match_len < max_possible && input[match_start + match_len] == input[pos + match_len] {
                        match_len += 1;
                    }
                    if match_len > best_len {
                        best_len = match_len;
                        best_dist = offset;
                        if best_len == max_possible {
                            break;
                        }
                    }
                }
            }

            if best_len >= self.min_match_len {
                tokens.push(LzToken::Match {
                    distance: best_dist as u16,
                    length: best_len as u16,
                });
                pos += best_len;
            } else {
                tokens.push(LzToken::Literal(input[pos]));
                pos += 1;
            }
        }
        tokens
    }

    pub fn decompress(&self, tokens: &[LzToken]) -> FitResult<Vec<u8>> {
        let mut output = Vec::new();
        for token in tokens {
            match token {
                LzToken::Literal(byte) => output.push(*byte),
                LzToken::Match { distance, length } => {
                    let dist = *distance as usize;
                    let len = *length as usize;
                    if dist > output.len() {
                        return Err(FitError::DecompressionFailed("LZ77 match distance out of bounds".into()));
                    }
                    let start = output.len() - dist;
                    for i in 0..len {
                        let byte = output[start + i];
                        output.push(byte);
                    }
                }
            }
        }
        Ok(output)
    }

    pub fn encode_tokens(&self, tokens: &[LzToken]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for token in tokens {
            match token {
                LzToken::Literal(b) => {
                    bytes.push(0u8);
                    bytes.push(*b);
                }
                LzToken::Match { distance, length } => {
                    bytes.push(1u8);
                    bytes.extend_from_slice(&distance.to_be_bytes());
                    bytes.extend_from_slice(&length.to_be_bytes());
                }
            }
        }
        bytes
    }

    pub fn decode_tokens(&self, bytes: &[u8]) -> FitResult<Vec<LzToken>> {
        let mut tokens = Vec::new();
        let mut i = 0;
        while i < bytes.len() {
            let tag = bytes[i];
            i += 1;
            if tag == 0 {
                if i >= bytes.len() {
                    return Err(FitError::DecompressionFailed("Unexpected EOF in LZ token literals".into()));
                }
                tokens.push(LzToken::Literal(bytes[i]));
                i += 1;
            } else if tag == 1 {
                if i + 4 > bytes.len() {
                    return Err(FitError::DecompressionFailed("Unexpected EOF in LZ match tokens".into()));
                }
                let dist = u16::from_be_bytes([bytes[i], bytes[i + 1]]);
                let len = u16::from_be_bytes([bytes[i + 2], bytes[i + 3]]);
                tokens.push(LzToken::Match {
                    distance: dist,
                    length: len,
                });
                i += 4;
            } else {
                return Err(FitError::DecompressionFailed(format!("Invalid LZ token tag: {}", tag)));
            }
        }
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lz77_roundtrip() {
        let matcher = Lz77Matcher::default();
        let input = b"abcde_abcde_abcde_12345_abcde_12345";
        let tokens = matcher.compress(input);
        let bytes = matcher.encode_tokens(&tokens);
        let decoded_tokens = matcher.decode_tokens(&bytes).unwrap();
        let output = matcher.decompress(&decoded_tokens).unwrap();
        assert_eq!(input.to_vec(), output);
    }
}
