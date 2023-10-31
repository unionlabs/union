use crate::read::{Architecture, Error, ReadError, ReadRef, Result};
use crate::{macho, BigEndian, Pod};

pub use macho::{FatArch32, FatArch64, FatHeader};

impl FatHeader {
    /// Attempt to parse a fat header.
    ///
    /// Does not validate the magic value.
    pub fn parse<'data, R: ReadRef<'data>>(file: R) -> Result<&'data FatHeader> {
        file.read_at::<FatHeader>(0)
            .read_error("Invalid fat header size or alignment")
    }

    /// Attempt to parse a fat header and 32-bit fat arches.
    pub fn parse_arch32<'data, R: ReadRef<'data>>(file: R) -> Result<&'data [FatArch32]> {
        let mut offset = 0;
        let header = file
            .read::<FatHeader>(&mut offset)
            .read_error("Invalid fat header size or alignment")?;
        if header.magic.get(BigEndian) != macho::FAT_MAGIC {
            return Err(Error("Invalid 32-bit fat magic"));
        }
        file.read_slice::<FatArch32>(&mut offset, header.nfat_arch.get(BigEndian) as usize)
            .read_error("Invalid nfat_arch")
    }

    /// Attempt to parse a fat header and 64-bit fat arches.
    pub fn parse_arch64<'data, R: ReadRef<'data>>(file: R) -> Result<&'data [FatArch64]> {
        let mut offset = 0;
        let header = file
            .read::<FatHeader>(&mut offset)
            .read_error("Invalid fat header size or alignment")?;
        if header.magic.get(BigEndian) != macho::FAT_MAGIC_64 {
            return Err(Error("Invalid 64-bit fat magic"));
        }
        file.read_slice::<FatArch64>(&mut offset, header.nfat_arch.get(BigEndian) as usize)
            .read_error("Invalid nfat_arch")
    }
}

/// A trait for generic access to `FatArch32` and `FatArch64`.
#[allow(missing_docs)]
pub trait FatArch: Pod {
    type Word: Into<u64>;

    fn cputype(&self) -> u32;
    fn cpusubtype(&self) -> u32;
    fn offset(&self) -> Self::Word;
    fn size(&self) -> Self::Word;
    fn align(&self) -> u32;

    fn architecture(&self) -> Architecture {
        match self.cputype() {
            macho::CPU_TYPE_ARM => Architecture::Arm,
            macho::CPU_TYPE_ARM64 => Architecture::Aarch64,
            macho::CPU_TYPE_X86 => Architecture::I386,
            macho::CPU_TYPE_X86_64 => Architecture::X86_64,
            macho::CPU_TYPE_MIPS => Architecture::Mips,
            _ => Architecture::Unknown,
        }
    }

    fn file_range(&self) -> (u64, u64) {
        (self.offset().into(), self.size().into())
    }

    fn data<'data, R: ReadRef<'data>>(&self, file: R) -> Result<&'data [u8]> {
        file.read_bytes_at(self.offset().into(), self.size().into())
            .read_error("Invalid fat arch offset or size")
    }
}

impl FatArch for FatArch32 {
    type Word = u32;

    fn cputype(&self) -> u32 {
        self.cputype.get(BigEndian)
    }

    fn cpusubtype(&self) -> u32 {
        self.cpusubtype.get(BigEndian)
    }

    fn offset(&self) -> Self::Word {
        self.offset.get(BigEndian)
    }

    fn size(&self) -> Self::Word {
        self.size.get(BigEndian)
    }

    fn align(&self) -> u32 {
        self.align.get(BigEndian)
    }
}

impl FatArch for FatArch64 {
    type Word = u64;

    fn cputype(&self) -> u32 {
        self.cputype.get(BigEndian)
    }

    fn cpusubtype(&self) -> u32 {
        self.cpusubtype.get(BigEndian)
    }

    fn offset(&self) -> Self::Word {
        self.offset.get(BigEndian)
    }

    fn size(&self) -> Self::Word {
        self.size.get(BigEndian)
    }

    fn align(&self) -> u32 {
        self.align.get(BigEndian)
    }
}
