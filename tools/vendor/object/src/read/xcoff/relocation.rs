use alloc::fmt;
use core::fmt::Debug;
use core::slice;

use crate::pod::Pod;
use crate::{xcoff, BigEndian as BE, Relocation};

use crate::read::{ReadRef, RelocationEncoding, RelocationKind, RelocationTarget, SymbolIndex};

use super::{FileHeader, SectionHeader, XcoffFile};

/// An iterator over the relocations in a `XcoffSection32`.
pub type XcoffRelocationIterator32<'data, 'file, R = &'data [u8]> =
    XcoffRelocationIterator<'data, 'file, xcoff::FileHeader32, R>;
/// An iterator over the relocations in a `XcoffSection64`.
pub type XcoffRelocationIterator64<'data, 'file, R = &'data [u8]> =
    XcoffRelocationIterator<'data, 'file, xcoff::FileHeader64, R>;

/// An iterator over the relocations in a `XcoffSection`.
pub struct XcoffRelocationIterator<'data, 'file, Xcoff, R = &'data [u8]>
where
    'data: 'file,
    Xcoff: FileHeader,
    R: ReadRef<'data>,
{
    #[allow(unused)]
    pub(super) file: &'file XcoffFile<'data, Xcoff, R>,
    pub(super) relocations:
        slice::Iter<'data, <<Xcoff as FileHeader>::SectionHeader as SectionHeader>::Rel>,
}

impl<'data, 'file, Xcoff, R> Iterator for XcoffRelocationIterator<'data, 'file, Xcoff, R>
where
    Xcoff: FileHeader,
    R: ReadRef<'data>,
{
    type Item = (u64, Relocation);

    fn next(&mut self) -> Option<Self::Item> {
        self.relocations.next().map(|relocation| {
            let encoding = RelocationEncoding::Generic;
            let (kind, addend) = match relocation.r_rtype() {
                xcoff::R_POS
                | xcoff::R_RL
                | xcoff::R_RLA
                | xcoff::R_BA
                | xcoff::R_RBA
                | xcoff::R_TLS => (RelocationKind::Absolute, 0),
                xcoff::R_REL | xcoff::R_BR | xcoff::R_RBR => (RelocationKind::Relative, -4),
                xcoff::R_TOC | xcoff::R_TOCL | xcoff::R_TOCU => (RelocationKind::Got, 0),
                r_type => (RelocationKind::Xcoff(r_type), 0),
            };
            let size = (relocation.r_rsize() & 0x3F) + 1;
            let target = RelocationTarget::Symbol(SymbolIndex(relocation.r_symndx() as usize));
            (
                relocation.r_vaddr().into(),
                Relocation {
                    kind,
                    encoding,
                    size,
                    target,
                    addend,
                    implicit_addend: true,
                },
            )
        })
    }
}

impl<'data, 'file, Xcoff, R> fmt::Debug for XcoffRelocationIterator<'data, 'file, Xcoff, R>
where
    Xcoff: FileHeader,
    R: ReadRef<'data>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("XcoffRelocationIterator").finish()
    }
}

/// A trait for generic access to `Rel32` and `Rel64`.
#[allow(missing_docs)]
pub trait Rel: Debug + Pod {
    type Word: Into<u64>;
    fn r_vaddr(&self) -> Self::Word;
    fn r_symndx(&self) -> u32;
    fn r_rsize(&self) -> u8;
    fn r_rtype(&self) -> u8;
}

impl Rel for xcoff::Rel32 {
    type Word = u32;

    fn r_vaddr(&self) -> Self::Word {
        self.r_vaddr.get(BE)
    }

    fn r_symndx(&self) -> u32 {
        self.r_symndx.get(BE)
    }

    fn r_rsize(&self) -> u8 {
        self.r_rsize
    }

    fn r_rtype(&self) -> u8 {
        self.r_rtype
    }
}

impl Rel for xcoff::Rel64 {
    type Word = u64;

    fn r_vaddr(&self) -> Self::Word {
        self.r_vaddr.get(BE)
    }

    fn r_symndx(&self) -> u32 {
        self.r_symndx.get(BE)
    }

    fn r_rsize(&self) -> u8 {
        self.r_rsize
    }

    fn r_rtype(&self) -> u8 {
        self.r_rtype
    }
}
