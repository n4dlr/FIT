use fit_core::{FitError, FitResult};
use std::path::{Component, Path, PathBuf};

pub struct SecurityValidator;

impl SecurityValidator {
    pub fn sanitize_path(base_dir: &Path, relative_path: &Path) -> FitResult<PathBuf> {
        let mut clean_rel = PathBuf::new();
        for component in relative_path.components() {
            match component {
                Component::Normal(part) => clean_rel.push(part),
                Component::ParentDir => {
                    return Err(FitError::SecurityViolation(format!(
                        "Path traversal attempt detected in path: {:?}",
                        relative_path
                    )));
                }
                Component::RootDir | Component::Prefix(_) => {
                    return Err(FitError::SecurityViolation(format!(
                        "Absolute path extraction prohibited: {:?}",
                        relative_path
                    )));
                }
                Component::CurDir => {}
            }
        }

        let full_destination = base_dir.join(&clean_rel);
        if !full_destination.starts_with(base_dir) {
            return Err(FitError::SecurityViolation(format!(
                "Extracted path {:?} escapes destination directory {:?}",
                full_destination, base_dir
            )));
        }

        Ok(full_destination)
    }

    pub fn validate_bomb_limits(
        current_depth: u32,
        max_depth: u32,
        total_extracted_bytes: u64,
        max_bytes: u64,
    ) -> FitResult<()> {
        if current_depth > max_depth {
            return Err(FitError::SecurityViolation(format!(
                "Archive recursion depth {} exceeds maximum limit {}",
                current_depth, max_depth
            )));
        }
        if total_extracted_bytes > max_bytes {
            return Err(FitError::SecurityViolation(format!(
                "Extracted size {} exceeds maximum permitted limit {}",
                total_extracted_bytes, max_bytes
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_traversal_prevention() {
        let base = Path::new("/tmp/output");
        let evil = Path::new("../../etc/passwd");
        assert!(SecurityValidator::sanitize_path(base, evil).is_err());

        let good = Path::new("subfolder/file.txt");
        assert!(SecurityValidator::sanitize_path(base, good).is_ok());
    }
}
