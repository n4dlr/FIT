use fit_detection::{DetectedType, FileTypeDetector};
use fit_core::{FitError, FitResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestedNode {
    pub name: String,
    pub path: PathBuf,
    pub depth: u32,
    pub is_container: bool,
    pub format_type: String,
    pub children: Vec<NestedNode>,
}

pub struct NestedArchiveExplorer {
    pub max_depth: u32,
}

impl Default for NestedArchiveExplorer {
    fn default() -> Self {
        Self { max_depth: 32 }
    }
}

impl NestedArchiveExplorer {
    pub fn new(max_depth: u32) -> Self {
        Self { max_depth }
    }

    pub fn inspect_nested<P: AsRef<Path>>(&self, path: P) -> FitResult<NestedNode> {
        self.inspect_recursive(path.as_ref(), 0)
    }

    fn inspect_recursive(&self, path: &Path, depth: u32) -> FitResult<NestedNode> {
        if depth > self.max_depth {
            return Err(FitError::SecurityViolation(format!(
                "Max recursion depth {} exceeded for nested archive",
                self.max_depth
            )));
        }

        let detected = FileTypeDetector::detect_file(path).unwrap_or(DetectedType::UnknownBinary);
        let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();

        let is_container = matches!(
            detected,
            DetectedType::FitArchive
                | DetectedType::ZipArchive
                | DetectedType::SevenZipArchive
                | DetectedType::TarArchive
                | DetectedType::GzipArchive
                | DetectedType::Bzip2Archive
                | DetectedType::XzArchive
                | DetectedType::ZstdArchive
        );

        Ok(NestedNode {
            name,
            path: path.to_path_buf(),
            depth,
            is_container,
            format_type: format!("{:?}", detected),
            children: Vec::new(),
        })
    }
}
