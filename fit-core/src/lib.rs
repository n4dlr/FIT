pub mod error;
pub mod progress;
pub mod traits;
pub mod types;

pub use error::{FitError, FitResult};
pub use progress::{CompressionPhase, ProgressCallback, ProgressReport};
pub use traits::{Compressor, StreamingCompressor, Transform};
pub use types::{CompressionConfig, CompressionLevel, CompressionStats, FileMetadata, SolidMode};
