use fit_compression::CompressionEngine;
use fit_core::{CompressionConfig, FileMetadata, FitError, FitResult, ProgressCallback, CompressionPhase, ProgressReport};

use fit_crypto::CryptoEngine;
use fit_format::{ArchiveEntryHeader, ArchiveHeader, ChunkHeader, HeaderFlags};
use fit_recovery::RecoveryEngine;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Seek, Write};
use std::path::{Path, PathBuf};

pub struct FitArchiveBuilder {
    config: CompressionConfig,
}

impl FitArchiveBuilder {
    pub fn new(config: CompressionConfig) -> Self {
        Self { config }
    }

    pub fn create_archive<P: AsRef<Path>, W: Write + Seek>(
        &self,
        inputs: &[P],
        writer: &mut W,
        progress: Option<ProgressCallback>,
    ) -> FitResult<u64> {
        let mut file_entries: Vec<(PathBuf, FileMetadata, Vec<u8>)> = Vec::new();
        let mut total_bytes = 0u64;

        if let Some(ref cb) = progress {
            cb(ProgressReport {
                phase: CompressionPhase::Analyzing,
                current_file: "Collecting files".into(),
                bytes_processed: 0,
                total_bytes: 0,
                current_compressed_bytes: 0,
                percent_complete: 0.0,
                current_speed_bytes_sec: 0.0,
                selected_strategy: "Analyzing inputs".into(),
            });
        }

        for input_path in inputs {
            let path = input_path.as_ref();
            if path.is_file() {
                let data = fs::read(path)?;
                let metadata = Self::collect_file_metadata(path, path.file_name().unwrap().as_ref())?;
                total_bytes += data.len() as u64;
                file_entries.push((path.to_path_buf(), metadata, data));
            } else if path.is_dir() {
                Self::collect_dir_recursive(path, path, &mut file_entries, &mut total_bytes)?;
            }
        }

        let engine = CompressionEngine::new(self.config.clone());
        let mut chunk_headers = Vec::new();
        let mut compressed_payloads = Vec::new();
        let mut entry_headers = Vec::new();

        let mut processed_bytes = 0u64;
        let mut total_compressed = 0u64;

        for (idx, (src_path, metadata, raw_data)) in file_entries.into_iter().enumerate() {
            if let Some(ref cb) = progress {
                cb(ProgressReport {
                    phase: CompressionPhase::Compressing,
                    current_file: src_path.to_string_lossy().to_string(),
                    bytes_processed: processed_bytes,
                    total_bytes,
                    current_compressed_bytes: total_compressed,
                    percent_complete: if total_bytes > 0 {
                        (processed_bytes as f32 / total_bytes as f32) * 100.0
                    } else {
                        100.0
                    },
                    current_speed_bytes_sec: 0.0,
                    selected_strategy: "Tournament candidate selection".into(),
                });
            }

            let mut sha_hasher = Sha256::new();
            sha_hasher.update(&raw_data);
            let sha256_hash: [u8; 32] = sha_hasher.finalize().into();

            let (method, compressed_bytes) = engine.compress_buffer(&raw_data)?;
            let crc = crc32fast::hash(&compressed_bytes);
            let xxh = xxhash_rust::xxh64::xxh64(&compressed_bytes, 0);

            let chunk_id = idx as u32;
            let chunk = ChunkHeader {
                id: chunk_id,
                original_size: raw_data.len() as u32,
                compressed_size: compressed_bytes.len() as u32,
                method,
                crc32: crc,
                xxhash64: xxh,
            };

            processed_bytes += raw_data.len() as u64;
            total_compressed += compressed_bytes.len() as u64;

            chunk_headers.push(chunk);
            compressed_payloads.push(compressed_bytes);

            entry_headers.push(ArchiveEntryHeader {
                id: chunk_id,
                metadata,
                sha256: sha256_hash,
                chunk_indices: vec![chunk_id],
            });
        }

        // Encryption handling
        let is_encrypted = self.config.encryption_password.is_some();
        let (salt, nonce_payload, nonce_metadata, enc_key) = if let Some(ref pwd) = self.config.encryption_password {
            let s = CryptoEngine::generate_salt();
            let np = CryptoEngine::generate_nonce();
            let nm = CryptoEngine::generate_nonce();
            let k = CryptoEngine::derive_key(pwd, &s)?;
            (Some(s), Some(np), Some(nm), Some(k))
        } else {
            (None, None, None, None)
        };

        // Combine payloads
        let mut raw_archive_data = Vec::new();
        for (header, payload) in chunk_headers.iter().zip(compressed_payloads.iter()) {
            let header_bytes = bincode::serialize(header)
                .map_err(|e| FitError::Serialization(e.to_string()))?;
            raw_archive_data.extend_from_slice(&(header_bytes.len() as u32).to_be_bytes());
            raw_archive_data.extend_from_slice(&header_bytes);
            raw_archive_data.extend_from_slice(payload);
        }

        let metadata_bytes = bincode::serialize(&entry_headers)
            .map_err(|e| FitError::Serialization(e.to_string()))?;

        let (final_payload, final_metadata) = if is_encrypted {
            let key = enc_key.unwrap();
            let enc_data = CryptoEngine::encrypt_payload(&raw_archive_data, &key, &nonce_payload.unwrap())?;
            let enc_meta = CryptoEngine::encrypt_payload(&metadata_bytes, &key, &nonce_metadata.unwrap())?;
            (enc_data, enc_meta)
        } else {
            (raw_archive_data, metadata_bytes)
        };

        // Parity / Recovery Records
        let has_recovery = self.config.recovery_percent > 0;
        let parity_bytes = if has_recovery {
            if let Some(ref cb) = progress {
                cb(ProgressReport {
                    phase: CompressionPhase::GeneratingRecovery,
                    current_file: "Parity records".into(),
                    bytes_processed: total_bytes,
                    total_bytes,
                    current_compressed_bytes: total_compressed,
                    percent_complete: 95.0,
                    current_speed_bytes_sec: 0.0,
                    selected_strategy: "Reed-Solomon ReedSolomon Erasure Coding".into(),
                });
            }
            let (parity, _, _) = RecoveryEngine::generate_parity(&final_payload, self.config.recovery_percent)?;
            bincode::serialize(&parity).map_err(|e| FitError::Serialization(e.to_string()))?
        } else {
            Vec::new()
        };

        let flags = HeaderFlags {
            is_solid: self.config.solid == fit_core::SolidMode::Solid,
            is_encrypted,
            has_deduplication: self.config.deduplication,
            has_recovery,
            is_multi_volume: false,
        };

        let mut archive_header = ArchiveHeader::new(flags, entry_headers.len() as u32, chunk_headers.len() as u32);

        // Write to stream
        // Header first (will be rewritten with updated offsets)
        archive_header.write_to(writer)?;
        if is_encrypted {
            // Write salt (16 bytes), payload nonce (12 bytes), metadata nonce (12 bytes)
            writer.write_all(&salt.unwrap())?;
            writer.write_all(&nonce_payload.unwrap())?;
            writer.write_all(&nonce_metadata.unwrap())?;
        }

        writer.write_all(&final_payload)?;
        archive_header.metadata_offset = writer.stream_position()?;
        writer.write_all(&(final_metadata.len() as u32).to_be_bytes())?;
        writer.write_all(&final_metadata)?;

        archive_header.recovery_offset = writer.stream_position()?;
        if has_recovery {
            writer.write_all(&(parity_bytes.len() as u32).to_be_bytes())?;
            writer.write_all(&parity_bytes)?;
        }

        // Rewrite updated header with offsets
        writer.seek(std::io::SeekFrom::Start(0))?;
        archive_header.write_to(writer)?;

        if let Some(ref cb) = progress {
            cb(ProgressReport {
                phase: CompressionPhase::Complete,
                current_file: "Done".into(),
                bytes_processed: total_bytes,
                total_bytes,
                current_compressed_bytes: total_compressed,
                percent_complete: 100.0,
                current_speed_bytes_sec: 0.0,
                selected_strategy: "Archive write complete".into(),
            });
        }

        Ok(writer.stream_position()?)
    }

    fn collect_file_metadata(path: &Path, rel_path: &Path) -> FitResult<FileMetadata> {
        let meta = fs::metadata(path)?;
        let modified = meta
            .modified()
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let created = meta
            .created()
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok(FileMetadata {
            relative_path: rel_path.to_path_buf(),
            is_dir: meta.is_dir(),
            size: meta.len(),
            created_at: created,
            modified_at: modified,
            mode: 0o644,
            is_symlink: meta.file_type().is_symlink(),
            symlink_target: None,
        })
    }

    fn collect_dir_recursive(
        base_dir: &Path,
        current_dir: &Path,
        entries: &mut Vec<(PathBuf, FileMetadata, Vec<u8>)>,
        total_bytes: &mut u64,
    ) -> FitResult<()> {
        for entry in fs::read_dir(current_dir)? {
            let entry = entry?;
            let path = entry.path();
            let rel = path.strip_prefix(base_dir).unwrap_or(&path);
            if path.is_file() {
                let data = fs::read(&path)?;
                let meta = Self::collect_file_metadata(&path, rel)?;
                *total_bytes += data.len() as u64;
                entries.push((path, meta, data));
            } else if path.is_dir() {
                Self::collect_dir_recursive(base_dir, &path, entries, total_bytes)?;
            }
        }
        Ok(())
    }
}
