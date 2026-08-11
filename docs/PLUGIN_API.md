# Universal Archive Plugin API

`fit-plugins` provides abstract traits to add support for third-party archive formats and custom compression algorithms without modifying `fit-core`.

```rust
pub trait ArchiveDetector: Send + Sync {
    fn name(&self) -> &'static str;
    fn can_open(&self, header_bytes: &[u8]) -> bool;
    fn open_reader(&self, archive_path: &Path) -> FitResult<Box<dyn ArchiveReader>>;
}

pub trait ArchiveReader: Send + Sync {
    fn entries(&self) -> FitResult<Vec<Box<dyn ArchiveEntry>>>;
    fn extract_entry(&self, entry_path: &Path, target_dir: &Path) -> FitResult<u64>;
    fn extract_all(&self, target_dir: &Path) -> FitResult<u64>;
}
```
