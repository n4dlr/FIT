use crate::security::SecurityValidator;
use fit_compression::CompressionEngine;
use fit_core::{CompressionConfig, CompressionPhase, FitError, FitResult, ProgressCallback, ProgressReport};
use fit_crypto::CryptoEngine;
use fit_format::{ArchiveEntryHeader, ArchiveHeader, ChunkHeader};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Serialized header size in bytes:
///  magic(4) + version(2) + flags(4) + timestamp(8) + entry_count(4) +
///  chunk_count(4) + metadata_offset(8) + recovery_offset(8) = 42 bytes
const HEADER_SIZE: u64 = 42;

/// When encrypted: salt(16) + nonce_payload(12) + nonce_metadata(12) = 40 bytes
const ENC_PREFIX_SIZE: u64 = 40;

pub struct FitArchiveReader;

impl FitArchiveReader {
    /// Read the fixed-size header region and the encrypted prefix (if any),
    /// returning `(header, salt, nonce_payload, nonce_metadata, data_start_offset)`.
    fn read_header_region<R: Read + Seek>(
        reader: &mut R,
    ) -> FitResult<(ArchiveHeader, [u8; 16], [u8; 12], [u8; 12], u64)> {
        reader.seek(SeekFrom::Start(0))?;
        let header = ArchiveHeader::read_from(reader)?;

        let mut salt = [0u8; 16];
        let mut nonce_payload = [0u8; 12];
        let mut nonce_metadata = [0u8; 12];

        if header.flags.is_encrypted {
            reader.seek(SeekFrom::Start(HEADER_SIZE))?;
            reader.read_exact(&mut salt)?;
            reader.read_exact(&mut nonce_payload)?;
            reader.read_exact(&mut nonce_metadata)?;
        }

        let data_start = if header.flags.is_encrypted {
            HEADER_SIZE + ENC_PREFIX_SIZE
        } else {
            HEADER_SIZE
        };

        Ok((header, salt, nonce_payload, nonce_metadata, data_start))
    }

    pub fn list_entries<R: Read + Seek>(
        reader: &mut R,
        password: Option<&str>,
    ) -> FitResult<(ArchiveHeader, Vec<ArchiveEntryHeader>)> {
        let (header, salt, _nonce_payload, nonce_metadata, _data_start) =
            Self::read_header_region(reader)?;

        // Jump to metadata section
        reader.seek(SeekFrom::Start(header.metadata_offset))?;
        let mut len_bytes = [0u8; 4];
        reader.read_exact(&mut len_bytes)?;
        let meta_len = u32::from_be_bytes(len_bytes) as usize;

        let mut raw_meta = vec![0u8; meta_len];
        reader.read_exact(&mut raw_meta)?;

        let meta_bytes = if header.flags.is_encrypted {
            let pwd = password.ok_or(FitError::InvalidPassword)?;
            let key = CryptoEngine::derive_key(pwd, &salt)?;
            CryptoEngine::decrypt_payload(&raw_meta, &key, &nonce_metadata)?
        } else {
            raw_meta
        };

        let entries: Vec<ArchiveEntryHeader> = bincode::deserialize(&meta_bytes)
            .map_err(|e| FitError::Serialization(e.to_string()))?;

        Ok((header, entries))
    }

    pub fn extract_all<R: Read + Seek, P: AsRef<Path>>(
        reader: &mut R,
        output_dir: P,
        password: Option<&str>,
        progress: Option<ProgressCallback>,
    ) -> FitResult<u64> {
        let out_path = output_dir.as_ref();
        fs::create_dir_all(out_path)?;

        // First parse header + entry table (requires re-seeking in list_entries)
        let (header, entries) = Self::list_entries(reader, password)?;

        // Now read encryption region again to get the payload nonce
        let (_, salt, nonce_payload, _, data_start) = Self::read_header_region(reader)?;

        // Read the entire compressed payload block
        let payload_len = header.metadata_offset.saturating_sub(data_start) as usize;
        reader.seek(SeekFrom::Start(data_start))?;
        let mut raw_payload = vec![0u8; payload_len];
        reader.read_exact(&mut raw_payload)?;

        let payload_bytes = if header.flags.is_encrypted {
            let pwd = password.ok_or(FitError::InvalidPassword)?;
            let key = CryptoEngine::derive_key(pwd, &salt)?;
            CryptoEngine::decrypt_payload(&raw_payload, &key, &nonce_payload)?
        } else {
            raw_payload
        };

        let engine = CompressionEngine::new(CompressionConfig::default());
        let mut extracted_bytes = 0u64;
        let mut p_cursor = 0usize;

        for entry in &entries {
            if p_cursor + 4 > payload_bytes.len() {
                break;
            }

            let chunk_hdr_len = u32::from_be_bytes([
                payload_bytes[p_cursor],
                payload_bytes[p_cursor + 1],
                payload_bytes[p_cursor + 2],
                payload_bytes[p_cursor + 3],
            ]) as usize;
            p_cursor += 4;

            if p_cursor + chunk_hdr_len > payload_bytes.len() {
                return Err(FitError::ArchiveFormat("Chunk header truncated".into()));
            }
            let chunk_hdr_bytes = &payload_bytes[p_cursor..p_cursor + chunk_hdr_len];
            p_cursor += chunk_hdr_len;

            let chunk_hdr: ChunkHeader = bincode::deserialize(chunk_hdr_bytes)
                .map_err(|e| FitError::Serialization(e.to_string()))?;

            let comp_end = p_cursor + chunk_hdr.compressed_size as usize;
            if comp_end > payload_bytes.len() {
                return Err(FitError::ArchiveFormat("Chunk payload truncated".into()));
            }
            let comp_bytes = &payload_bytes[p_cursor..comp_end];
            p_cursor = comp_end;

            let decomp_data = engine.decompress_buffer(chunk_hdr.method, comp_bytes)?;

            // Integrity verification — strict SHA-256 byte-level match
            let mut hasher = Sha256::new();
            hasher.update(&decomp_data);
            let decomp_sha256: [u8; 32] = hasher.finalize().into();

            if decomp_sha256 != entry.sha256 {
                return Err(FitError::HashMismatch {
                    expected: hex::encode(entry.sha256),
                    actual: hex::encode(decomp_sha256),
                });
            }

            // Security: prevent path traversal
            let dest_file_path =
                SecurityValidator::sanitize_path(out_path, &entry.metadata.relative_path)?;
            if let Some(parent) = dest_file_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(&dest_file_path, &decomp_data)?;
            extracted_bytes += decomp_data.len() as u64;

            if let Some(ref cb) = progress {
                cb(ProgressReport {
                    phase: CompressionPhase::Complete,
                    current_file: dest_file_path.to_string_lossy().to_string(),
                    bytes_processed: extracted_bytes,
                    total_bytes: extracted_bytes,
                    current_compressed_bytes: p_cursor as u64,
                    percent_complete: 100.0,
                    current_speed_bytes_sec: 0.0,
                    selected_strategy: format!("Extracted {:?}", chunk_hdr.method),
                });
            }
        }

        Ok(extracted_bytes)
    }

    pub fn test_archive<R: Read + Seek>(
        reader: &mut R,
        password: Option<&str>,
    ) -> FitResult<bool> {
        match Self::list_entries(reader, password) {
            Ok((_, entries)) => Ok(!entries.is_empty()),
            Err(_) => Ok(false),
        }
    }
}
