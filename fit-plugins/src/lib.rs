pub mod manager;
pub mod nested;
pub mod traits;
pub mod zip_plugin;

pub use manager::PluginManager;
pub use nested::{NestedArchiveExplorer, NestedNode};
pub use traits::{ArchiveDetector, ArchiveEntry, ArchiveReader};
pub use zip_plugin::ZipDetector;
