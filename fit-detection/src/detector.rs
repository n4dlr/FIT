use fit_core::FitResult;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetectedType {
    FitArchive,
    ZipArchive,
    SevenZipArchive,
    TarArchive,
    GzipArchive,
    Bzip2Archive,
    XzArchive,
    ZstdArchive,
    IsoImage,
    JpegImage,
    PngImage,
    Mp4Video,
    PdfDocument,
    StructuredText(String), // JSON, XML, CSV, Logs, Source Code
    HighEntropyBinary,
    UnknownBinary,
}

pub struct FileTypeDetector;

impl FileTypeDetector {
    pub fn detect_bytes(sample: &[u8]) -> DetectedType {
        if sample.is_empty() {
            return DetectedType::UnknownBinary;
        }

        // Magic Byte Signatures
        if sample.starts_with(b"FIT\x01") {
            return DetectedType::FitArchive;
        }
        if sample.starts_with(&[0x50, 0x4B, 0x03, 0x04]) || sample.starts_with(&[0x50, 0x4B, 0x05, 0x06]) {
            return DetectedType::ZipArchive;
        }
        if sample.starts_with(&[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]) {
            return DetectedType::SevenZipArchive;
        }
        if sample.starts_with(&[0x1F, 0x8B]) {
            return DetectedType::GzipArchive;
        }
        if sample.starts_with(b"BZh") {
            return DetectedType::Bzip2Archive;
        }
        if sample.starts_with(&[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00]) {
            return DetectedType::XzArchive;
        }
        if sample.starts_with(&[0x28, 0xB5, 0x2F, 0xFD]) {
            return DetectedType::ZstdArchive;
        }
        if sample.len() > 262 && &sample[257..262] == b"ustar" {
            return DetectedType::TarArchive;
        }
        if sample.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return DetectedType::JpegImage;
        }
        if sample.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
            return DetectedType::PngImage;
        }
        if sample.len() >= 12 && &sample[4..8] == b"ftyp" {
            return DetectedType::Mp4Video;
        }
        if sample.starts_with(b"%PDF-") {
            return DetectedType::PdfDocument;
        }

        // Text & Structure heuristics
        if std::str::from_utf8(sample).is_ok() {
            let s = std::str::from_utf8(sample).unwrap().trim();
            if s.starts_with('{') || s.starts_with('[') {
                return DetectedType::StructuredText("JSON".into());
            }
            if s.starts_with('<') {
                return DetectedType::StructuredText("XML/HTML".into());
            }
            return DetectedType::StructuredText("Text/SourceCode/Logs".into());
        }

        // Entropy analysis
        let entropy = Self::calculate_entropy(sample);
        if entropy > 7.5 {
            DetectedType::HighEntropyBinary
        } else {
            DetectedType::UnknownBinary
        }
    }

    pub fn detect_file<P: AsRef<Path>>(path: P) -> FitResult<DetectedType> {
        let mut file = File::open(path)?;
        let mut buffer = vec![0u8; 8192];
        let bytes_read = file.read(&mut buffer)?;
        buffer.truncate(bytes_read);
        Ok(Self::detect_bytes(&buffer))
    }

    pub fn calculate_entropy(data: &[u8]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }
        let mut counts = [0usize; 256];
        for &byte in data {
            counts[byte as usize] += 1;
        }
        let len_f = data.len() as f64;
        let mut entropy = 0.0;
        for &count in &counts {
            if count > 0 {
                let p = count as f64 / len_f;
                entropy -= p * p.log2();
            }
        }
        entropy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_detection() {
        assert_eq!(FileTypeDetector::detect_bytes(b"FIT\x01\x00\x01"), DetectedType::FitArchive);
        assert_eq!(FileTypeDetector::detect_bytes(&[0x50, 0x4B, 0x03, 0x04]), DetectedType::ZipArchive);
        assert_eq!(FileTypeDetector::detect_bytes(&[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]), DetectedType::SevenZipArchive);
    }

    #[test]
    fn test_entropy_calculation() {
        let low_entropy = vec![0u8; 1000];
        assert_eq!(FileTypeDetector::calculate_entropy(&low_entropy), 0.0);

        let high_entropy: Vec<u8> = (0..256).map(|i| i as u8).cycle().take(1024).collect();
        assert!((FileTypeDetector::calculate_entropy(&high_entropy) - 8.0).abs() < 0.01);
    }
}
