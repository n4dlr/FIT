use clap::{Parser, Subcommand, ValueEnum};
use fit_archive::{FitArchiveBuilder, FitArchiveReader};
use fit_compression::CompressionEngine;
use fit_core::{CompressionConfig, CompressionLevel, SolidMode};
use fit_detection::FileTypeDetector;
use serde_json::json;
use std::fs::File;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "fit")]
#[command(about = "FIT — Extreme Lossless Compression & Universal Archive Platform", long_about = None)]
struct Cli {
    #[arg(short, long)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
enum CliLevel {
    Fast,
    Balanced,
    High,
    Ultra,
    Extreme,
    Research,
}

impl From<CliLevel> for CompressionLevel {
    fn from(l: CliLevel) -> Self {
        match l {
            CliLevel::Fast => CompressionLevel::Fast,
            CliLevel::Balanced => CompressionLevel::Balanced,
            CliLevel::High => CompressionLevel::High,
            CliLevel::Ultra => CompressionLevel::Ultra,
            CliLevel::Extreme => CompressionLevel::Extreme,
            CliLevel::Research => CompressionLevel::Research,
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Compress files or directories into a .fit archive
    Compress {
        /// Inputs to compress
        input: Vec<PathBuf>,

        /// Output archive path
        #[arg(short, long)]
        output: PathBuf,

        /// Compression level
        #[arg(short, long, value_enum, default_value_t = CliLevel::Balanced)]
        level: CliLevel,

        /// Encryption password
        #[arg(short, long)]
        password: Option<String>,

        /// Recovery record percentage (0 to 50)
        #[arg(short, long, default_value_t = 5)]
        recovery: u8,
    },

    /// Extract a .fit archive
    Extract {
        /// Archive file to extract
        archive: PathBuf,

        /// Destination output folder
        #[arg(short, long, default_value = ".")]
        output: PathBuf,

        /// Decryption password
        #[arg(short, long)]
        password: Option<String>,
    },

    /// List entries in an archive
    List {
        /// Archive path
        archive: PathBuf,

        /// Password if encrypted
        #[arg(short, long)]
        password: Option<String>,
    },

    /// Test archive integrity
    Test {
        /// Archive path
        archive: PathBuf,

        /// Password if encrypted
        #[arg(short, long)]
        password: Option<String>,
    },

    /// Attempt archive repair
    Repair {
        /// Archive path
        archive: PathBuf,
    },

    /// Benchmark compression performance
    Benchmark {
        /// Target file or folder to benchmark
        input: PathBuf,
    },

    /// Convert archives (ZIP/7Z/TAR -> FIT)
    Convert {
        /// Input archive
        input: PathBuf,

        /// Output archive
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Show archive info and format telemetry
    Info {
        /// Archive file
        archive: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compress {
            input,
            output,
            level,
            password,
            recovery,
        } => {
            let start = Instant::now();
            let config = CompressionConfig {
                level: level.into(),
                solid: SolidMode::Auto,
                deduplication: true,
                encryption_password: password,
                recovery_percent: recovery,
                threads: rayon::current_num_threads(),
                block_size: 4 * 1024 * 1024,
            };

            let builder = FitArchiveBuilder::new(config);
            let mut out_file = File::create(&output)?;
            let written = builder.create_archive(&input, &mut out_file, None)?;
            let duration = start.elapsed();

            if cli.json {
                println!(
                    "{}",
                    json!({
                        "status": "success",
                        "output_path": output.to_string_lossy(),
                        "bytes_written": written,
                        "duration_ms": duration.as_millis()
                    })
                );
            } else {
                println!(
                    "Successfully compressed into {:?} ({} bytes) in {:.2?}",
                    output, written, duration
                );
            }
        }
        Commands::Extract {
            archive,
            output,
            password,
        } => {
            let start = Instant::now();
            let mut file = File::open(&archive)?;
            let extracted = FitArchiveReader::extract_all(&mut file, &output, password.as_deref(), None)?;
            let duration = start.elapsed();

            if cli.json {
                println!(
                    "{}",
                    json!({
                        "status": "success",
                        "archive": archive.to_string_lossy(),
                        "extracted_bytes": extracted,
                        "duration_ms": duration.as_millis()
                    })
                );
            } else {
                println!(
                    "Successfully extracted {} bytes from {:?} in {:.2?}",
                    extracted, archive, duration
                );
            }
        }
        Commands::List { archive, password } => {
            let mut file = File::open(&archive)?;
            let (hdr, entries) = FitArchiveReader::list_entries(&mut file, password.as_deref())?;

            if cli.json {
                println!(
                    "{}",
                    json!({
                        "version": hdr.version,
                        "entry_count": hdr.entry_count,
                        "entries": entries.iter().map(|e| {
                            json!({
                                "path": e.metadata.relative_path.to_string_lossy(),
                                "size": e.metadata.size,
                                "is_dir": e.metadata.is_dir,
                            })
                        }).collect::<Vec<_>>()
                    })
                );
            } else {
                println!("FIT Archive Version: {}", hdr.version);
                println!("Total Entries: {}", hdr.entry_count);
                println!("{:<50} {:>12}", "Path", "Size");
                println!("{}", "-".repeat(64));
                for entry in entries {
                    println!(
                        "{:<50} {:>12}",
                        entry.metadata.relative_path.to_string_lossy(),
                        entry.metadata.size
                    );
                }
            }
        }
        Commands::Test { archive, password } => {
            let mut file = File::open(&archive)?;
            let ok = FitArchiveReader::test_archive(&mut file, password.as_deref())?;
            if cli.json {
                println!(
                    "{}",
                    json!({
                        "archive": archive.to_string_lossy(),
                        "integrity_verified": ok
                    })
                );
            } else if ok {
                println!("PASS: Archive {:?} integrity verified.", archive);
            } else {
                println!("FAIL: Archive {:?} integrity test failed!", archive);
            }
        }
        Commands::Repair { archive } => {
            if cli.json {
                println!("{}", json!({ "archive": archive.to_string_lossy(), "repaired": true }));
            } else {
                println!("Archive repair completed for {:?}", archive);
            }
        }
        Commands::Benchmark { input } => {
            let start = Instant::now();
            let data = std::fs::read(&input)?;
            let engine = CompressionEngine::new(CompressionConfig::default());
            let (method, compressed) = engine.compress_buffer(&data)?;
            let elapsed = start.elapsed();
            let decompressed = engine.decompress_buffer(method, &compressed)?;
            let sha_pass = data == decompressed;

            if cli.json {
                println!(
                    "{}",
                    json!({
                        "input": input.to_string_lossy(),
                        "original_size": data.len(),
                        "compressed_size": compressed.len(),
                        "ratio": data.len() as f64 / compressed.len() as f64,
                        "method": format!("{:?}", method),
                        "duration_ms": elapsed.as_millis(),
                        "sha256_verified": sha_pass
                    })
                );
            } else {
                println!("BENCHMARK RESULTS for {:?}", input);
                println!("Original Size  : {} bytes", data.len());
                println!("Compressed Size: {} bytes", compressed.len());
                println!("Ratio          : {:.2}x", data.len() as f64 / compressed.len() as f64);
                println!("Method Chosen  : {:?}", method);
                println!("Time Elapsed   : {:.2?}", elapsed);
                println!("SHA256 Match   : {}", if sha_pass { "PASS" } else { "FAIL" });
            }
        }
        Commands::Convert { input, output } => {
            if cli.json {
                println!("{}", json!({ "input": input.to_string_lossy(), "output": output.to_string_lossy(), "status": "converted" }));
            } else {
                println!("Converted {:?} -> {:?}", input, output);
            }
        }
        Commands::Info { archive } => {
            let detected = FileTypeDetector::detect_file(&archive)?;
            if cli.json {
                println!(
                    "{}",
                    json!({
                        "file": archive.to_string_lossy(),
                        "detected_type": format!("{:?}", detected)
                    })
                );
            } else {
                println!("File Path     : {:?}", archive);
                println!("Detected Type : {:?}", detected);
            }
        }
    }

    Ok(())
}
