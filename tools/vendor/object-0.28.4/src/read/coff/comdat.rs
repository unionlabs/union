use core::str;

use crate::endian::LittleEndian as LE;
use crate::pe;
use crate::read::{
    self, ComdatKind, ObjectComdat, ReadError, ReadRef, Result, SectionIndex, SymbolIndex,
};

use super::CoffFile;

/// An iterator over the COMDAT section groups of a `CoffFile`.
#[derive(Debug)]
pub struct CoffComdatIterator<'data, 'file, R: ReadRef<'data> = &'data [u8]> {
    pub(super) file: &'file CoffFile<'data, R>,
    pub(super) index: usize,
}

impl<'data, 'file, R: ReadRef<'data>> Iterator for CoffComdatIterator<'data, 'file, R> {
    type Item = CoffComdat<'data, 'file, R>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let index = self.index;
            let symbol = self.file.common.symbols.symbol(index).ok()?;
            self.index += 1 + symbol.number_of_aux_symbols as usize;
            if let Some(comdat) = CoffComdat::parse(self.file, symbol, index) {
                return Some(comdat);
            }
        }
    }
}

/// A COMDAT section group of a `CoffFile`.
#[derive(Debug)]
pub struct CoffComdat<'data, 'file, R: ReadRef<'data> = &'data [u8]> {
    file: &'file CoffFile<'data, R>,
    symbol_index: SymbolIndex,
    symbol: &'data pe::ImageSymbol,
    selection: u8,
}

impl<'data, 'file, R: ReadRef<'data>> CoffComdat<'data, 'file, R> {
    fn parse(
        file: &'file CoffFile<'data, R>,
        section_symbol: &'data pe::ImageSymbol,
        index: usize,
    ) -> Option<CoffComdat<'data, 'file, R>> {
        // Must be a section symbol.
        if !section_symbol.has_aux_section() {
            return None;
        }

        // Auxiliary record must have a non-associative selection.
        let aux = file.common.symbols.aux_section(index).ok()?;
        let selection = aux.selection;
        if selection == 0 || selection == pe::IMAGE_COMDAT_SELECT_ASSOCIATIVE {
            return None;
        }

        // Find the COMDAT symbol.
        let mut symbol_index = index;
        let mut symbol = section_symbol;
        let section_number = section_symbol.section_number.get(LE);
        loop {
            symbol_index += 1 + symbol.number_of_aux_symbols as usize;
            symbol = file.common.symbols.symbol(symbol_index).ok()?;
            if section_number == symbol.section_number.get(LE) {
                break;
            }
        }

        Some(CoffComdat {
            file,
            symbol_index: SymbolIndex(symbol_index),
            symbol,
            selection,
        })
    }
}

impl<'data, 'file, R: ReadRef<'data>> read::private::Sealed for CoffComdat<'data, 'file, R> {}

impl<'data, 'file, R: ReadRef<'data>> ObjectComdat<'data> for CoffComdat<'data, 'file, R> {
    type SectionIterator = CoffComdatSectionIterator<'data, 'file, R>;

    #[inline]
    fn kind(&self) -> ComdatKind {
        match self.selection {
            pe::IMAGE_COMDAT_SELECT_NODUPLICATES => ComdatKind::NoDuplicates,
            pe::IMAGE_COMDAT_SELECT_ANY => ComdatKind::Any,
            pe::IMAGE_COMDAT_SELECT_SAME_SIZE => ComdatKind::SameSize,
            pe::IMAGE_COMDAT_SELECT_EXACT_MATCH => ComdatKind::ExactMatch,
            pe::IMAGE_COMDAT_SELECT_LARGEST => ComdatKind::Largest,
            pe::IMAGE_COMDAT_SELECT_NEWEST => ComdatKind::Newest,
            _ => ComdatKind::Unknown,
        }
    }

    #[inline]
    fn symbol(&self) -> SymbolIndex {
        self.symbol_index
    }

    #[inline]
    fn name_bytes(&self) -> Result<&[u8]> {
        // Find the name of first symbol referring to the section.
        self.symbol.name(self.file.common.symbols.strings())
    }

    #[inline]
    fn name(&self) -> Result<&str> {
        let bytes = self.name_bytes()?;
        str::from_utf8(bytes)
            .ok()
            .read_error("Non UTF-8 COFF COMDAT name")
    }

    #[inline]
    fn sections(&self) -> Self::SectionIterator {
        CoffComdatSectionIterator {
            file: self.file,
            section_number: self.symbol.section_number.get(LE),
            index: 0,
        }
    }
}

/// An iterator over the sections in a COMDAT section group of a `CoffFile`.
#[derive(Debug)]
pub struct CoffComdatSectionIterator<'data, 'file, R: ReadRef<'data> = &'data [u8]> {
    file: &'file CoffFile<'data, R>,
    section_number: u16,
    index: usize,
}

impl<'data, 'file, R: ReadRef<'data>> Iterator for CoffComdatSectionIterator<'data, 'file, R> {
    type Item = SectionIndex;

    fn next(&mut self) -> Option<Self::Item> {
        // Find associated COMDAT symbols.
        // TODO: it seems gcc doesn't use associated symbols for this
        loop {
            let index = self.index;
            let symbol = self.file.common.symbols.symbol(index).ok()?;
            self.index += 1 + symbol.number_of_aux_symbols as usize;

            // Must be a section symbol.
            if !symbol.has_aux_section() {
                continue;
            }

            let section_number = symbol.section_number.get(LE);

            let aux = self.file.common.symbols.aux_section(index).ok()?;
            if aux.selection == pe::IMAGE_COMDAT_SELECT_ASSOCIATIVE {
                // TODO: use high_number for bigobj
                if aux.number.get(LE) == self.section_number {
                    return Some(SectionIndex(section_number as usize));
                }
            } else if aux.selection != 0 {
                if section_number == self.section_number {
                    return Some(SectionIndex(section_number as usize));
                }
            }
        }
    }
}
