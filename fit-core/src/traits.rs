use crate::error::FitResult;
use std::io::{Read, Write};

pub trait Transform: Send + Sync {
    fn name(&self) -> &'static str;
    fn transform(&self, input: &[u8]) -> FitResult<Vec<u8>>;
    fn inverse(&self, input: &[u8]) -> FitResult<Vec<u8>>;
}

pub trait Compressor: Send + Sync {
    fn name(&self) -> &'static str;
    fn compress(&self, input: &[u8]) -> FitResult<Vec<u8>>;
    fn decompress(&self, input: &[u8]) -> FitResult<Vec<u8>>;
}

pub trait StreamingCompressor: Send + Sync {
    fn compress_stream(&self, reader: &mut dyn Read, writer: &mut dyn Write) -> FitResult<u64>;
    fn decompress_stream(&self, reader: &mut dyn Read, writer: &mut dyn Write) -> FitResult<u64>;
}
