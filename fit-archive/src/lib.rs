pub mod builder;
pub mod reader;
pub mod security;

pub use builder::FitArchiveBuilder;
pub use reader::FitArchiveReader;
pub use security::SecurityValidator;

#[cfg(test)]
mod integration_tests {
    use super::*;
    use fit_core::{CompressionConfig, CompressionLevel, SolidMode};
    use sha2::{Digest, Sha256};
    use std::io::Cursor;
    use tempfile::tempdir;

    #[test]
    fn test_end_to_end_archive_roundtrip() {
        let temp_dir = tempdir().unwrap();
        let input_file_path = temp_dir.path().join("sample_log.txt");

        let original_content = b"LOG [2026-08-11 12:00:00] INFO FIT Archive System initialized successfully.\n".repeat(100);
        std::fs::write(&input_file_path, &original_content).unwrap();

        let mut sha = Sha256::new();
        sha.update(&original_content);
        let orig_hash: [u8; 32] = sha.finalize().into();

        let config = CompressionConfig {
            level: CompressionLevel::Balanced,
            solid: SolidMode::Auto,
            deduplication: true,
            encryption_password: Some("Pass123!".into()),
            recovery_percent: 10,
            threads: 4,
            block_size: 1024 * 1024,
        };

        let builder = FitArchiveBuilder::new(config);
        let mut archive_buffer = Cursor::new(Vec::new());

        let written = builder
            .create_archive(&[input_file_path.clone()], &mut archive_buffer, None)
            .unwrap();
        assert!(written > 0);

        // Decompress and verify
        archive_buffer.set_position(0);
        let extract_dir = temp_dir.path().join("extracted");
        let extracted_bytes =
            FitArchiveReader::extract_all(&mut archive_buffer, &extract_dir, Some("Pass123!"), None).unwrap();

        assert_eq!(extracted_bytes as usize, original_content.len());

        let extracted_file = extract_dir.join("sample_log.txt");
        let decompressed_content = std::fs::read(&extracted_file).unwrap();

        let mut decomp_sha = Sha256::new();
        decomp_sha.update(&decompressed_content);
        let decomp_hash: [u8; 32] = decomp_sha.finalize().into();

        assert_eq!(orig_hash, decomp_hash);
        assert_eq!(original_content, decompressed_content.as_slice());
    }
}
