use alloc::fmt;
use core::slice;

use crate::endian::LittleEndian as LE;
use crate::pe;
use crate::read::{
    ReadRef, Relocation, RelocationEncoding, RelocationKind, RelocationTarget, SymbolIndex,
};

use super::CoffFile;

/// An iterator over the relocations in a `CoffSection`.
pub struct CoffRelocationIterator<'data, 'file, R: ReadRef<'data> = &'data [u8]> {
    pub(super) file: &'file CoffFile<'data, R>,
    pub(super) iter: slice::Iter<'data, pe::ImageRelocation>,
}

impl<'data, 'file, R: ReadRef<'data>> Iterator for CoffRelocationIterator<'data, 'file, R> {
    type Item = (u64, Relocation);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|relocation| {
            let (kind, size, addend) = match self.file.header.machine.get(LE) {
                pe::IMAGE_FILE_MACHINE_ARMNT => match relocation.typ.get(LE) {
                    pe::IMAGE_REL_ARM_ADDR32 => (RelocationKind::Absolute, 32, 0),
                    pe::IMAGE_REL_ARM_ADDR32NB => (RelocationKind::ImageOffset, 32, 0),
                    pe::IMAGE_REL_ARM_REL32 => (RelocationKind::Relative, 32, -4),
                    pe::IMAGE_REL_ARM_SECTION => (RelocationKind::SectionIndex, 16, 0),
                    pe::IMAGE_REL_ARM_SECREL => (RelocationKind::SectionOffset, 32, 0),
                    typ => (RelocationKind::Coff(typ), 0, 0),
                },
                pe::IMAGE_FILE_MACHINE_ARM64 => match relocation.typ.get(LE) {
                    pe::IMAGE_REL_ARM64_ADDR32 => (RelocationKind::Absolute, 32, 0),
                    pe::IMAGE_REL_ARM64_ADDR32NB => (RelocationKind::ImageOffset, 32, 0),
                    pe::IMAGE_REL_ARM64_SECREL => (RelocationKind::SectionOffset, 32, 0),
                    pe::IMAGE_REL_ARM64_SECTION => (RelocationKind::SectionIndex, 16, 0),
                    pe::IMAGE_REL_ARM64_ADDR64 => (RelocationKind::Absolute, 64, 0),
                    pe::IMAGE_REL_ARM64_REL32 => (RelocationKind::Relative, 32, -4),
                    typ => (RelocationKind::Coff(typ), 0, 0),
                },
                pe::IMAGE_FILE_MACHINE_I386 => match relocation.typ.get(LE) {
                    pe::IMAGE_REL_I386_DIR16 => (RelocationKind::Absolute, 16, 0),
                    pe::IMAGE_REL_I386_REL16 => (RelocationKind::Relative, 16, 0),
                    pe::IMAGE_REL_I386_DIR32 => (RelocationKind::Absolute, 32, 0),
                    pe::IMAGE_REL_I386_DIR32NB => (RelocationKind::ImageOffset, 32, 0),
                    pe::IMAGE_REL_I386_SECTION => (RelocationKind::SectionIndex, 16, 0),
                    pe::IMAGE_REL_I386_SECREL => (RelocationKind::SectionOffset, 32, 0),
                    pe::IMAGE_REL_I386_SECREL7 => (RelocationKind::SectionOffset, 7, 0),
                    pe::IMAGE_REL_I386_REL32 => (RelocationKind::Relative, 32, -4),
                    typ => (RelocationKind::Coff(typ), 0, 0),
                },
                pe::IMAGE_FILE_MACHINE_AMD64 => match relocation.typ.get(LE) {
                    pe::IMAGE_REL_AMD64_ADDR64 => (RelocationKind::Absolute, 64, 0),
                    pe::IMAGE_REL_AMD64_ADDR32 => (RelocationKind::Absolute, 32, 0),
                    pe::IMAGE_REL_AMD64_ADDR32NB => (RelocationKind::ImageOffset, 32, 0),
                    pe::IMAGE_REL_AMD64_REL32 => (RelocationKind::Relative, 32, -4),
                    pe::IMAGE_REL_AMD64_REL32_1 => (RelocationKind::Relative, 32, -5),
                    pe::IMAGE_REL_AMD64_REL32_2 => (RelocationKind::Relative, 32, -6),
                    pe::IMAGE_REL_AMD64_REL32_3 => (RelocationKind::Relative, 32, -7),
                    pe::IMAGE_REL_AMD64_REL32_4 => (RelocationKind::Relative, 32, -8),
                    pe::IMAGE_REL_AMD64_REL32_5 => (RelocationKind::Relative, 32, -9),
                    pe::IMAGE_REL_AMD64_SECTION => (RelocationKind::SectionIndex, 16, 0),
                    pe::IMAGE_REL_AMD64_SECREL => (RelocationKind::SectionOffset, 32, 0),
                    pe::IMAGE_REL_AMD64_SECREL7 => (RelocationKind::SectionOffset, 7, 0),
                    typ => (RelocationKind::Coff(typ), 0, 0),
                },
                _ => (RelocationKind::Coff(relocation.typ.get(LE)), 0, 0),
            };
            let target = RelocationTarget::Symbol(SymbolIndex(
                relocation.symbol_table_index.get(LE) as usize,
            ));
            (
                u64::from(relocation.virtual_address.get(LE)),
                Relocation {
                    kind,
                    encoding: RelocationEncoding::Generic,
                    size,
                    target,
                    addend,
                    implicit_addend: true,
                },
            )
        })
    }
}

impl<'data, 'file, R: ReadRef<'data>> fmt::Debug for CoffRelocationIterator<'data, 'file, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CoffRelocationIterator").finish()
    }
}
