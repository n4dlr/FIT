use fit_core::{FileMetadata, FitResult};
use std::path::{Path, PathBuf};

pub trait ArchiveEntry: Send + Sync {
    fn path(&self) -> PathBuf;
    fn metadata(&self) -> FileMetadata;
    fn is_dir(&self) -> bool;
    fn uncompressed_size(&self) -> u64;
}

pub trait ArchiveReader: Send + Sync {
    fn entries(&self) -> FitResult<Vec<Box<dyn ArchiveEntry>>>;
    fn extract_entry(&self, entry_path: &Path, target_dir: &Path) -> FitResult<u64>;
    fn extract_all(&self, target_dir: &Path) -> FitResult<u64>;
}

pub trait ArchiveDetector: Send + Sync {
    fn name(&self) -> &'static str;
    fn can_open(&self, header_bytes: &[u8]) -> bool;
    fn open_reader(&self, archive_path: &Path) -> FitResult<Box<dyn ArchiveReader>>;
}
