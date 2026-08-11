use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use fit_core::{FitError, FitResult};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

pub const FIT_MAGIC: &[u8; 4] = b"FIT\x01";
pub const CURRENT_FORMAT_VERSION: u16 = 1;
pub const ARCHIVE_HEADER_SIZE: u64 = 42;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeaderFlags {
    pub is_solid: bool,
    pub is_encrypted: bool,
    pub has_deduplication: bool,
    pub has_recovery: bool,
    pub is_multi_volume: bool,
}

impl HeaderFlags {
    pub fn to_bits(&self) -> u32 {
        let mut bits = 0u32;
        if self.is_solid {
            bits |= 1 << 0;
        }
        if self.is_encrypted {
            bits |= 1 << 1;
        }
        if self.has_deduplication {
            bits |= 1 << 2;
        }
        if self.has_recovery {
            bits |= 1 << 3;
        }
        if self.is_multi_volume {
            bits |= 1 << 4;
        }
        bits
    }

    pub fn from_bits(bits: u32) -> Self {
        Self {
            is_solid: (bits & (1 << 0)) != 0,
            is_encrypted: (bits & (1 << 1)) != 0,
            has_deduplication: (bits & (1 << 2)) != 0,
            has_recovery: (bits & (1 << 3)) != 0,
            is_multi_volume: (bits & (1 << 4)) != 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveHeader {
    pub version: u16,
    pub flags: HeaderFlags,
    pub created_timestamp: u64,
    pub entry_count: u32,
    pub chunk_count: u32,
    pub metadata_offset: u64,
    pub recovery_offset: u64,
}

impl ArchiveHeader {
    pub fn new(flags: HeaderFlags, entry_count: u32, chunk_count: u32) -> Self {
        let created_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            version: CURRENT_FORMAT_VERSION,
            flags,
            created_timestamp,
            entry_count,
            chunk_count,
            metadata_offset: 0,
            recovery_offset: 0,
        }
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> FitResult<()> {
        writer.write_all(FIT_MAGIC)?;
        writer.write_u16::<BigEndian>(self.version)?;
        writer.write_u32::<BigEndian>(self.flags.to_bits())?;
        writer.write_u64::<BigEndian>(self.created_timestamp)?;
        writer.write_u32::<BigEndian>(self.entry_count)?;
        writer.write_u32::<BigEndian>(self.chunk_count)?;
        writer.write_u64::<BigEndian>(self.metadata_offset)?;
        writer.write_u64::<BigEndian>(self.recovery_offset)?;
        Ok(())
    }

    pub fn read_from<R: Read>(reader: &mut R) -> FitResult<Self> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if &magic != FIT_MAGIC {
            return Err(FitError::InvalidMagic {
                expected: FIT_MAGIC.to_vec(),
                actual: magic.to_vec(),
            });
        }

        let version = reader.read_u16::<BigEndian>()?;
        if version > CURRENT_FORMAT_VERSION {
            return Err(FitError::UnsupportedVersion(version));
        }

        let flags_bits = reader.read_u32::<BigEndian>()?;
        let flags = HeaderFlags::from_bits(flags_bits);
        let created_timestamp = reader.read_u64::<BigEndian>()?;
        let entry_count = reader.read_u32::<BigEndian>()?;
        let chunk_count = reader.read_u32::<BigEndian>()?;
        let metadata_offset = reader.read_u64::<BigEndian>()?;
        let recovery_offset = reader.read_u64::<BigEndian>()?;

        Ok(Self {
            version,
            flags,
            created_timestamp,
            entry_count,
            chunk_count,
            metadata_offset,
            recovery_offset,
        })
    }
}
