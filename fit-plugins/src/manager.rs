use crate::traits::{ArchiveDetector, ArchiveReader};
use crate::zip_plugin::ZipDetector;
use fit_core::{FitError, FitResult};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub struct PluginManager {
    detectors: Vec<Box<dyn ArchiveDetector>>,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self {
            detectors: vec![Box::new(ZipDetector)],
        }
    }
}

impl PluginManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_detector(&mut self, detector: Box<dyn ArchiveDetector>) {
        self.detectors.push(detector);
    }

    pub fn open_archive<P: AsRef<Path>>(&self, path: P) -> FitResult<Box<dyn ArchiveReader>> {
        let p = path.as_ref();
        let mut file = File::open(p)?;
        let mut header_buf = [0u8; 16];
        let n = file.read(&mut header_buf)?;
        let header = &header_buf[..n];

        for detector in &self.detectors {
            if detector.can_open(header) {
                return detector.open_reader(p);
            }
        }

        Err(FitError::ArchiveFormat(format!(
            "Unsupported or unrecognized archive format for file: {:?}",
            p
        )))
    }
}
