use fit_core::{FitError, FitResult};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Eq, PartialEq)]
struct Node {
    freq: usize,
    symbol: Option<u8>,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.freq.cmp(&self.freq)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct HuffmanCoder;

impl HuffmanCoder {
    pub fn compress(input: &[u8]) -> FitResult<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }

        let mut freqs = [0usize; 256];
        for &b in input {
            freqs[b as usize] += 1;
        }

        let mut heap = BinaryHeap::new();
        for (symbol, &freq) in freqs.iter().enumerate() {
            if freq > 0 {
                heap.push(Node {
                    freq,
                    symbol: Some(symbol as u8),
                    left: None,
                    right: None,
                });
            }
        }

        if heap.len() == 1 {
            let single = heap.pop().unwrap();
            let mut out = Vec::new();
            out.extend_from_slice(&(input.len() as u32).to_be_bytes());
            out.push(single.symbol.unwrap());
            return Ok(out);
        }

        while heap.len() > 1 {
            let n1 = heap.pop().unwrap();
            let n2 = heap.pop().unwrap();
            let parent = Node {
                freq: n1.freq + n2.freq,
                symbol: None,
                left: Some(Box::new(n1)),
                right: Some(Box::new(n2)),
            };
            heap.push(parent);
        }

        let root = heap.pop().unwrap();
        let mut codes = vec![(0u32, 0u8); 256];
        Self::build_codes(&root, 0, 0, &mut codes);

        let mut bit_writer = BitWriter::new();
        for &b in input {
            let (code, bits) = codes[b as usize];
            bit_writer.write_bits(code, bits);
        }

        let mut out = Vec::new();
        out.extend_from_slice(&(input.len() as u32).to_be_bytes());
        for &f in &freqs {
            out.extend_from_slice(&(f as u32).to_be_bytes());
        }
        out.extend_from_slice(&bit_writer.finish());

        Ok(out)
    }

    fn build_codes(node: &Node, current_code: u32, depth: u8, codes: &mut [(u32, u8)]) {
        if let Some(symbol) = node.symbol {
            codes[symbol as usize] = (current_code, depth.max(1));
            return;
        }
        if let Some(ref left) = node.left {
            Self::build_codes(left, current_code << 1, depth + 1, codes);
        }
        if let Some(ref right) = node.right {
            Self::build_codes(right, (current_code << 1) | 1, depth + 1, codes);
        }
    }

    pub fn decompress(input: &[u8]) -> FitResult<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }
        if input.len() < 5 {
            return Err(FitError::DecompressionFailed("Invalid Huffman stream size".into()));
        }

        let orig_len = u32::from_be_bytes([input[0], input[1], input[2], input[3]]) as usize;
        if orig_len == 0 {
            return Ok(Vec::new());
        }

        if input.len() == 5 {
            let sym = input[4];
            return Ok(vec![sym; orig_len]);
        }

        if input.len() < 4 + 256 * 4 {
            return Err(FitError::DecompressionFailed("Huffman table truncated".into()));
        }

        let mut freqs = [0usize; 256];
        let mut idx = 4;
        for i in 0..256 {
            freqs[i] = u32::from_be_bytes([input[idx], input[idx + 1], input[idx + 2], input[idx + 3]]) as usize;
            idx += 4;
        }

        let mut heap = BinaryHeap::new();
        for (symbol, &freq) in freqs.iter().enumerate() {
            if freq > 0 {
                heap.push(Node {
                    freq,
                    symbol: Some(symbol as u8),
                    left: None,
                    right: None,
                });
            }
        }

        while heap.len() > 1 {
            let n1 = heap.pop().unwrap();
            let n2 = heap.pop().unwrap();
            let parent = Node {
                freq: n1.freq + n2.freq,
                symbol: None,
                left: Some(Box::new(n1)),
                right: Some(Box::new(n2)),
            };
            heap.push(parent);
        }

        let root = heap.pop().unwrap();
        let payload = &input[idx..];
        let mut bit_reader = BitReader::new(payload);
        let mut output = Vec::with_capacity(orig_len);

        while output.len() < orig_len {
            let mut curr = &root;
            while curr.symbol.is_none() {
                let bit = bit_reader.read_bit().ok_or_else(|| {
                    FitError::DecompressionFailed("Unexpected EOF in Huffman bitstream".into())
                })?;
                if bit {
                    curr = curr.right.as_ref().unwrap();
                } else {
                    curr = curr.left.as_ref().unwrap();
                }
            }
            output.push(curr.symbol.unwrap());
        }

        Ok(output)
    }
}

struct BitWriter {
    bytes: Vec<u8>,
    current_byte: u8,
    num_bits: u8,
}

impl BitWriter {
    fn new() -> Self {
        Self {
            bytes: Vec::new(),
            current_byte: 0,
            num_bits: 0,
        }
    }

    fn write_bits(&mut self, code: u32, bits: u8) {
        for i in (0..bits).rev() {
            let bit = ((code >> i) & 1) as u8;
            self.current_byte = (self.current_byte << 1) | bit;
            self.num_bits += 1;
            if self.num_bits == 8 {
                self.bytes.push(self.current_byte);
                self.current_byte = 0;
                self.num_bits = 0;
            }
        }
    }

    fn finish(mut self) -> Vec<u8> {
        if self.num_bits > 0 {
            self.bytes.push(self.current_byte << (8 - self.num_bits));
        }
        self.bytes
    }
}

struct BitReader<'a> {
    bytes: &'a [u8],
    byte_idx: usize,
    bit_idx: u8,
}

impl<'a> BitReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            byte_idx: 0,
            bit_idx: 0,
        }
    }

    fn read_bit(&mut self) -> Option<bool> {
        if self.byte_idx >= self.bytes.len() {
            return None;
        }
        let byte = self.bytes[self.byte_idx];
        let bit = (byte >> (7 - self.bit_idx)) & 1;
        self.bit_idx += 1;
        if self.bit_idx == 8 {
            self.bit_idx = 0;
            self.byte_idx += 1;
        }
        Some(bit != 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_huffman_roundtrip() {
        let input = b"Hello, World! Huffman entropy coding testing 1234567890.";
        let compressed = HuffmanCoder::compress(input).unwrap();
        let decompressed = HuffmanCoder::decompress(&compressed).unwrap();
        assert_eq!(input.to_vec(), decompressed);
    }
}
