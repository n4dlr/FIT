use crate::traits::{ArchiveDetector, ArchiveEntry, ArchiveReader};
use fit_core::{FileMetadata, FitError, FitResult};
use std::fs::File;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

pub struct ZipEntryImpl {
    pub rel_path: PathBuf,
    pub is_directory: bool,
    pub size: u64,
}

impl ArchiveEntry for ZipEntryImpl {
    fn path(&self) -> PathBuf {
        self.rel_path.clone()
    }
    fn metadata(&self) -> FileMetadata {
        FileMetadata {
            relative_path: self.rel_path.clone(),
            is_dir: self.is_directory,
            size: self.size,
            created_at: 0,
            modified_at: 0,
            mode: 0o644,
            is_symlink: false,
            symlink_target: None,
        }
    }
    fn is_dir(&self) -> bool {
        self.is_directory
    }
    fn uncompressed_size(&self) -> u64 {
        self.size
    }
}

pub struct ZipReaderImpl {
    path: PathBuf,
}

impl ArchiveReader for ZipReaderImpl {
    fn entries(&self) -> FitResult<Vec<Box<dyn ArchiveEntry>>> {
        let file = File::open(&self.path)?;
        let mut zip = ZipArchive::new(file)
            .map_err(|e| FitError::ArchiveFormat(format!("Failed to open ZIP: {}", e)))?;

        let mut results: Vec<Box<dyn ArchiveEntry>> = Vec::new();
        for i in 0..zip.len() {
            let file_entry = zip
                .by_index(i)
                .map_err(|e| FitError::ArchiveFormat(format!("ZIP entry read error: {}", e)))?;
            results.push(Box::new(ZipEntryImpl {
                rel_path: PathBuf::from(file_entry.name()),
                is_directory: file_entry.is_dir(),
                size: file_entry.size(),
            }));
        }
        Ok(results)
    }

    fn extract_entry(&self, entry_path: &Path, target_dir: &Path) -> FitResult<u64> {
        let file = File::open(&self.path)?;
        let mut zip = ZipArchive::new(file)
            .map_err(|e| FitError::ArchiveFormat(format!("Failed to open ZIP: {}", e)))?;

        let entry_str = entry_path.to_string_lossy();
        let mut zip_file = zip
            .by_name(&entry_str)
            .map_err(|e| FitError::ArchiveFormat(format!("ZIP entry missing: {}", e)))?;

        let out_dest = target_dir.join(entry_path);
        if let Some(p) = out_dest.parent() {
            std::fs::create_dir_all(p)?;
        }
        let mut out_file = File::create(&out_dest)?;
        let written = std::io::copy(&mut zip_file, &mut out_file)?;
        Ok(written)
    }

    fn extract_all(&self, target_dir: &Path) -> FitResult<u64> {
        let file = File::open(&self.path)?;
        let mut zip = ZipArchive::new(file)
            .map_err(|e| FitError::ArchiveFormat(format!("Failed to open ZIP: {}", e)))?;

        zip.extract(target_dir)
            .map_err(|e| FitError::ArchiveFormat(format!("ZIP extract error: {}", e)))?;
        Ok(0)
    }
}

pub struct ZipDetector;

impl ArchiveDetector for ZipDetector {
    fn name(&self) -> &'static str {
        "ZIP"
    }

    fn can_open(&self, header_bytes: &[u8]) -> bool {
        header_bytes.starts_with(&[0x50, 0x4B, 0x03, 0x04]) || header_bytes.starts_with(&[0x50, 0x4B, 0x05, 0x06])
    }

    fn open_reader(&self, archive_path: &Path) -> FitResult<Box<dyn ArchiveReader>> {
        Ok(Box::new(ZipReaderImpl {
            path: archive_path.to_path_buf(),
        }))
    }
}
