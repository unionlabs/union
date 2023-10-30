use alloc::fmt;
use core::convert::TryInto;
use core::fmt::Debug;
use core::marker::PhantomData;
use core::str;

use crate::endian::{BigEndian as BE, U32Bytes};
use crate::pod::Pod;
use crate::read::util::StringTable;
use crate::{bytes_of, xcoff, Object, ObjectSection, SectionKind};

use crate::read::{
    self, Bytes, Error, ObjectSymbol, ObjectSymbolTable, ReadError, ReadRef, Result, SectionIndex,
    SymbolFlags, SymbolIndex, SymbolKind, SymbolScope, SymbolSection,
};

use super::{FileHeader, XcoffFile};

/// A table of symbol entries in an XCOFF file.
///
/// Also includes the string table used for the symbol names.
#[derive(Debug)]
pub struct SymbolTable<'data, Xcoff, R = &'data [u8]>
where
    Xcoff: FileHeader,
    R: ReadRef<'data>,
{
    symbols: &'data [xcoff::SymbolBytes],
    strings: StringTable<'data, R>,
    header: PhantomData<Xcoff>,
}

impl<'data, Xcoff, R> Default for SymbolTable<'data, Xcoff, R>
where
    Xcoff: FileHeader,
    R: ReadRef<'data>,
{
    fn default() -> Self {
        Self {
            symbols: &[],
            strings: StringTable::default(),
            header: PhantomData,
        }
    }
}

impl<'data, Xcoff, R> SymbolTable<'data, Xcoff, R>
where
    Xcoff: FileHeader,
    R: ReadRef<'data>,
{
    /// Parse the symbol table.
    pub fn parse(header: Xcoff, data: R) -> Result<Self> {
        let mut offset = header.f_symptr().into();
        let (symbols, strings) = if offset != 0 {
            let symbols = data
                .read_slice(&mut offset, header.f_nsyms() as usize)
                .read_error("Invalid XCOFF symbol table offset or size")?;

            // Parse the string table.
            // Note: don't update data when reading length; the length includes itself.
            let length = data
                .read_at::<U32Bytes<_>>(offset)
                .read_error("Missing XCOFF string table")?
                .get(BE);
            let str_end = offset
                .checked_add(length as u64)
                .read_error("Invalid XCOFF string table length")?;
            let strings = StringTable::new(data, offset, str_end);

            (symbols, strings)
        } else {
            (&[][..], StringTable::default())
        };

        Ok(SymbolTable {
            symbols,
            strings,
            header: PhantomData,
        })
    }

    /// Return the symbol entry at the given index and offset.
    pub fn get<T: Pod>(&self, index: usize, offset: usize) -> Result<&'data T> {
        let entry = index
            .checked_add(offset)
            .and_then(|x| self.symbols.get(x))
            .read_error("Invalid XCOFF symbol index")?;
        let bytes = bytes_of(entry);
        Bytes(bytes).read().read_error("Invalid XCOFF symbol data")
    }

    /// Return the symbol at the given index.
    pub fn symbol(&self, index: usize) -> Result<&'data Xcoff::Symbol> {
        self.get::<Xcoff::Symbol>(index, 0)
    }

    /// Return the file auxiliary symbol.
    pub fn aux_file(&self, index: usize) -> Result<&'data Xcoff::FileAux> {
        debug_assert!(self.symbol(index)?.has_aux_file());
        let aux_file = self.get::<Xcoff::FileAux>(index, 1)?;
        if let Some(aux_type) = aux_file.x_auxtype() {
            if aux_type != xcoff::AUX_FILE {
                return Err(Error("Invalid index for file auxiliary symbol."));
            }
        }
        Ok(aux_file)
    }

    /// Return the csect auxiliary symbol.
    pub fn aux_csect(&self, index: usize, offset: usize) -> Result<&'data Xcoff::CsectAux> {
        debug_assert!(self.symbol(index)?.has_aux_csect());
        let aux_csect = self.get::<Xcoff::CsectAux>(index, offset)?;
        if let Some(aux_type) = aux_csect.x_auxtype() {
            if aux_type != xcoff::AUX_CSECT {
                return Err(Error("Invalid index/offset for csect auxiliary symbol."));
            }
        }
        Ok(aux_csect)
    }

    /// Return true if the symbol table is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    /// The number of symbol table entries.
    ///
    /// This includes auxiliary symbol table entries.
    #[inline]
    pub fn len(&self) -> usize {
        self.symbols.len()
    }
}

/// A symbol table of an `XcoffFile32`.
pub type XcoffSymbolTable32<'data, 'file, R = &'data [u8]> =
    XcoffSymbolTable<'data, 'file, xcoff::FileHeader32, R>;
/// A symbol table of an `XcoffFile64`.
pub type XcoffSymbolTable64<'data, 'file, R = &'data [u8]> =
    XcoffSymbolTable<'data, 'file, xcoff::FileHeader64, R>;

/// A symbol table of an `XcoffFile`.
#[derive(Debug, Clone, Copy)]
pub struct XcoffSymbolTable<'data, 'file, Xcoff, R = &'data [u8]>
where
    'data: 'file,
    Xcoff: FileHeader,
    R: ReadRef<'data>,
{
    pub(crate) file: &'file XcoffFile<'data, Xcoff, R>,
    pub(super) symbols: &'file SymbolTable<'data, Xcoff, R>,
}

impl<'data, 'file, Xcoff: FileHeader, R: ReadRef<'data>> read::private::Sealed
    for XcoffSymbolTable<'data, 'file, Xcoff, R>
{
}

impl<'data, 'file, Xcoff: FileHeader, R: ReadRef<'data>> ObjectSymbolTable<'data>
    for XcoffSymbolTable<'data, 'file, Xcoff, R>
{
    type Symbol = XcoffSymbol<'data, 'file, Xcoff, R>;
    type SymbolIterator = XcoffSymbolIterator<'data, 'file, Xcoff, R>;

    fn symbols(&self) -> Self::SymbolIterator {
        XcoffSymbolIterator {
            file: self.file,
            symbols: self.symbols,
            index: 0,
        }
    }

    fn symbol_by_index(&self, index: SymbolIndex) -> read::Result<Self::Symbol> {
        let symbol = self.symbols.symbol(index.0)?;
        Ok(XcoffSymbol {
            file: self.file,
            symbols: self.symbols,
            index,
            symbol,
        })
    }
}

/// An iterator over the symbols of an `XcoffFile32`.
pub type XcoffSymbolIterator32<'data, 'file, R = &'data [u8]> =
    XcoffSymbolIterator<'data, 'file, xcoff::FileHeader32, R>;
/// An iterator over the symbols of an `XcoffFile64`.
pub type XcoffSymbolIterator64<'data, 'file, R = &'data [u8]> =
    XcoffSymbolIterator<'data, 'file, xcoff::FileHeader64, R>;

/// An iterator over the symbols of an `XcoffFile`.
pub struct XcoffSymbolIterator<'data, 'file, Xcoff, R = &'data [u8]>
where
    'data: 'file,
    Xcoff: FileHeader,
    R: ReadRef<'data>,
{
    pub(crate) file: &'file XcoffFile<'data, Xcoff, R>,
    pub(super) symbols: &'file SymbolTable<'data, Xcoff, R>,
    pub(super) index: usize,
}

impl<'data, 'file, Xcoff: FileHeader, R: ReadRef<'data>> fmt::Debug
    for XcoffSymbolIterator<'data, 'file, Xcoff, R>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("XcoffSymbolIterator").finish()
    }
}

impl<'data, 'file, Xcoff: FileHeader, R: ReadRef<'data>> Iterator
    for XcoffSymbolIterator<'data, 'file, Xcoff, R>
{
    type Item = XcoffSymbol<'data, 'file, Xcoff, R>;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        let symbol = self.symbols.symbol(index).ok()?;
        // TODO: skip over the auxiliary symbols for now.
        self.index += 1 + symbol.n_numaux() as usize;
        Some(XcoffSymbol {
            file: self.file,
            symbols: self.symbols,
            index: SymbolIndex(index),
            symbol,
        })
    }
}

/// A symbol of an `XcoffFile32`.
pub type XcoffSymbol32<'data, 'file, R = &'data [u8]> =
    XcoffSymbol<'data, 'file, xcoff::FileHeader32, R>;
/// A symbol of an `XcoffFile64`.
pub type XcoffSymbol64<'data, 'file, R = &'data [u8]> =
    XcoffSymbol<'data, 'file, xcoff::FileHeader64, R>;

/// A symbol of an `XcoffFile`.
#[derive(Debug, Clone, Copy)]
pub struct XcoffSymbol<'data, 'file, Xcoff, R = &'data [u8]>
where
    'data: 'file,
    Xcoff: FileHeader,
    R: ReadRef<'data>,
{
    pub(crate) file: &'file XcoffFile<'data, Xcoff, R>,
    pub(super) symbols: &'file SymbolTable<'data, Xcoff, R>,
    pub(super) index: SymbolIndex,
    pub(super) symbol: &'data Xcoff::Symbol,
}

impl<'data, 'file, Xcoff: FileHeader, R: ReadRef<'data>> read::private::Sealed
    for XcoffSymbol<'data, 'file, Xcoff, R>
{
}

impl<'data, 'file, Xcoff: FileHeader, R: ReadRef<'data>> ObjectSymbol<'data>
    for XcoffSymbol<'data, 'file, Xcoff, R>
{
    #[inline]
    fn index(&self) -> SymbolIndex {
        self.index
    }

    fn name_bytes(&self) -> Result<&'data [u8]> {
        self.symbol.name(self.symbols.strings)
    }

    fn name(&self) -> Result<&'data str> {
        let name = self.name_bytes()?;
        str::from_utf8(name)
            .ok()
            .read_error("Non UTF-8 XCOFF symbol name")
    }

    #[inline]
    fn address(&self) -> u64 {
        match self.symbol.n_sclass() {
            // Relocatable address.
            xcoff::C_EXT
            | xcoff::C_WEAKEXT
            | xcoff::C_HIDEXT
            | xcoff::C_FCN
            | xcoff::C_BLOCK
            | xcoff::C_STAT => self.symbol.n_value().into(),
            _ => 0,
        }
    }

    #[inline]
    fn size(&self) -> u64 {
        if self.symbol.has_aux_csect() {
            // XCOFF32 must have the csect auxiliary entry as the last auxiliary entry.
            // XCOFF64 doesn't require this, but conventionally does.
            if let Ok(aux_csect) = self
                .file
                .symbols
                .aux_csect(self.index.0, self.symbol.n_numaux() as usize)
            {
                let sym_type = aux_csect.sym_type() & 0x07;
                if sym_type == xcoff::XTY_SD || sym_type == xcoff::XTY_CM {
                    aux_csect.x_scnlen()
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            0
        }
    }

    fn kind(&self) -> SymbolKind {
        match self.symbol.n_sclass() {
            xcoff::C_FILE => SymbolKind::File,
            xcoff::C_NULL => SymbolKind::Null,
            _ => self
                .file
                .section_by_index(SectionIndex((self.symbol.n_scnum() - 1) as usize))
                .map(|section| match section.kind() {
                    SectionKind::Data | SectionKind::UninitializedData => SymbolKind::Data,
                    SectionKind::UninitializedTls | SectionKind::Tls => SymbolKind::Tls,
                    SectionKind::Text => SymbolKind::Text,
                    _ => SymbolKind::Unknown,
                })
                .unwrap_or(SymbolKind::Unknown),
        }
    }

    fn section(&self) -> SymbolSection {
        match self.symbol.n_scnum() {
            xcoff::N_ABS => SymbolSection::Absolute,
            xcoff::N_UNDEF => SymbolSection::Undefined,
            xcoff::N_DEBUG => SymbolSection::None,
            index if index > 0 => SymbolSection::Section(SectionIndex(index as usize)),
            _ => SymbolSection::Unknown,
        }
    }

    #[inline]
    fn is_undefined(&self) -> bool {
        self.symbol.is_undefined()
    }

    /// Return true if the symbol is a definition of a function or data object.
    #[inline]
    fn is_definition(&self) -> bool {
        if self.symbol.has_aux_csect() {
            if let Ok(aux_csect) = self
                .symbols
                .aux_csect(self.index.0, self.symbol.n_numaux() as usize)
            {
                let smclas = aux_csect.x_smclas();
                self.symbol.n_scnum() != xcoff::N_UNDEF
                    && (smclas == xcoff::XMC_PR
                        || smclas == xcoff::XMC_RW
                        || smclas == xcoff::XMC_RO)
            } else {
                false
            }
        } else {
            false
        }
    }

    #[inline]
    fn is_common(&self) -> bool {
        self.symbol.n_sclass() == xcoff::C_EXT && self.symbol.n_scnum() == xcoff::N_UNDEF
    }

    #[inline]
    fn is_weak(&self) -> bool {
        self.symbol.n_sclass() == xcoff::C_WEAKEXT
    }

    fn scope(&self) -> SymbolScope {
        if self.symbol.n_scnum() == xcoff::N_UNDEF {
            SymbolScope::Unknown
        } else {
            match self.symbol.n_sclass() {
                xcoff::C_EXT | xcoff::C_WEAKEXT | xcoff::C_HIDEXT => {
                    let visibility = self.symbol.n_type() & xcoff::SYM_V_MASK;
                    if visibility == xcoff::SYM_V_HIDDEN {
                        SymbolScope::Linkage
                    } else {
                        SymbolScope::Dynamic
                    }
                }
                _ => SymbolScope::Compilation,
            }
        }
    }

    #[inline]
    fn is_global(&self) -> bool {
        match self.symbol.n_sclass() {
            xcoff::C_EXT | xcoff::C_WEAKEXT => true,
            _ => false,
        }
    }

    #[inline]
    fn is_local(&self) -> bool {
        !self.is_global()
    }

    #[inline]
    fn flags(&self) -> SymbolFlags<SectionIndex> {
        SymbolFlags::None
    }
}

/// A trait for generic access to `Symbol32` and `Symbol64`.
#[allow(missing_docs)]
pub trait Symbol: Debug + Pod {
    type Word: Into<u64>;

    fn n_value(&self) -> Self::Word;
    fn n_scnum(&self) -> i16;
    fn n_type(&self) -> u16;
    fn n_sclass(&self) -> u8;
    fn n_numaux(&self) -> u8;

    fn name<'data, R: ReadRef<'data>>(
        &'data self,
        strings: StringTable<'data, R>,
    ) -> Result<&'data [u8]>;

    /// Return true if the symbol is undefined.
    #[inline]
    fn is_undefined(&self) -> bool {
        let n_sclass = self.n_sclass();
        (n_sclass == xcoff::C_EXT || n_sclass == xcoff::C_WEAKEXT)
            && self.n_scnum() == xcoff::N_UNDEF
    }

    /// Return true if the symbol has file auxiliary entry.
    fn has_aux_file(&self) -> bool {
        self.n_numaux() > 0 && self.n_sclass() == xcoff::C_FILE
    }

    /// Return true if the symbol has csect auxiliary entry.
    ///
    /// A csect auxiliary entry is required for each symbol table entry that has
    /// a storage class value of C_EXT, C_WEAKEXT, or C_HIDEXT.
    fn has_aux_csect(&self) -> bool {
        let sclass = self.n_sclass();
        self.n_numaux() > 0
            && (sclass == xcoff::C_EXT || sclass == xcoff::C_WEAKEXT || sclass == xcoff::C_HIDEXT)
    }
}

impl Symbol for xcoff::Symbol64 {
    type Word = u64;

    fn n_value(&self) -> Self::Word {
        self.n_value.get(BE)
    }

    fn n_scnum(&self) -> i16 {
        self.n_scnum.get(BE)
    }

    fn n_type(&self) -> u16 {
        self.n_type.get(BE)
    }

    fn n_sclass(&self) -> u8 {
        self.n_sclass
    }

    fn n_numaux(&self) -> u8 {
        self.n_numaux
    }

    /// Parse the symbol name for XCOFF64.
    fn name<'data, R: ReadRef<'data>>(
        &'data self,
        strings: StringTable<'data, R>,
    ) -> Result<&'data [u8]> {
        strings
            .get(self.n_offset.get(BE))
            .read_error("Invalid XCOFF symbol name offset")
    }
}

impl Symbol for xcoff::Symbol32 {
    type Word = u32;

    fn n_value(&self) -> Self::Word {
        self.n_value.get(BE)
    }

    fn n_scnum(&self) -> i16 {
        self.n_scnum.get(BE)
    }

    fn n_type(&self) -> u16 {
        self.n_type.get(BE)
    }

    fn n_sclass(&self) -> u8 {
        self.n_sclass
    }

    fn n_numaux(&self) -> u8 {
        self.n_numaux
    }

    /// Parse the symbol name for XCOFF32.
    fn name<'data, R: ReadRef<'data>>(
        &'data self,
        strings: StringTable<'data, R>,
    ) -> Result<&'data [u8]> {
        if self.n_name[0] == 0 {
            // If the name starts with 0 then the last 4 bytes are a string table offset.
            let offset = u32::from_be_bytes(self.n_name[4..8].try_into().unwrap());
            strings
                .get(offset)
                .read_error("Invalid XCOFF symbol name offset")
        } else {
            // The name is inline and padded with nulls.
            Ok(match memchr::memchr(b'\0', &self.n_name) {
                Some(end) => &self.n_name[..end],
                None => &self.n_name,
            })
        }
    }
}

/// A trait for generic access to `FileAux32` and `FileAux64`.
#[allow(missing_docs)]
pub trait FileAux: Debug + Pod {
    fn x_fname(&self) -> &[u8; 8];
    fn x_ftype(&self) -> u8;
    fn x_auxtype(&self) -> Option<u8>;
}

impl FileAux for xcoff::FileAux64 {
    fn x_fname(&self) -> &[u8; 8] {
        &self.x_fname
    }

    fn x_ftype(&self) -> u8 {
        self.x_ftype
    }

    fn x_auxtype(&self) -> Option<u8> {
        Some(self.x_auxtype)
    }
}

impl FileAux for xcoff::FileAux32 {
    fn x_fname(&self) -> &[u8; 8] {
        &self.x_fname
    }

    fn x_ftype(&self) -> u8 {
        self.x_ftype
    }

    fn x_auxtype(&self) -> Option<u8> {
        None
    }
}

/// A trait for generic access to `CsectAux32` and `CsectAux64`.
#[allow(missing_docs)]
pub trait CsectAux: Debug + Pod {
    fn x_scnlen(&self) -> u64;
    fn x_parmhash(&self) -> u32;
    fn x_snhash(&self) -> u16;
    fn x_smtyp(&self) -> u8;
    fn x_smclas(&self) -> u8;
    fn x_auxtype(&self) -> Option<u8>;

    fn sym_type(&self) -> u8 {
        self.x_smtyp() & 0x07
    }
}

impl CsectAux for xcoff::CsectAux64 {
    fn x_scnlen(&self) -> u64 {
        self.x_scnlen_lo.get(BE) as u64 | ((self.x_scnlen_hi.get(BE) as u64) << 32)
    }

    fn x_parmhash(&self) -> u32 {
        self.x_parmhash.get(BE)
    }

    fn x_snhash(&self) -> u16 {
        self.x_snhash.get(BE)
    }

    fn x_smtyp(&self) -> u8 {
        self.x_smtyp
    }

    fn x_smclas(&self) -> u8 {
        self.x_smclas
    }

    fn x_auxtype(&self) -> Option<u8> {
        Some(self.x_auxtype)
    }
}

impl CsectAux for xcoff::CsectAux32 {
    fn x_scnlen(&self) -> u64 {
        self.x_scnlen.get(BE) as u64
    }

    fn x_parmhash(&self) -> u32 {
        self.x_parmhash.get(BE)
    }

    fn x_snhash(&self) -> u16 {
        self.x_snhash.get(BE)
    }

    fn x_smtyp(&self) -> u8 {
        self.x_smtyp
    }

    fn x_smclas(&self) -> u8 {
        self.x_smclas
    }

    fn x_auxtype(&self) -> Option<u8> {
        None
    }
}
