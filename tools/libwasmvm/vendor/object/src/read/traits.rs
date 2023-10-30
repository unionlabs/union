use alloc::borrow::Cow;
use alloc::vec::Vec;

use crate::read::{
    self, Architecture, CodeView, ComdatKind, CompressedData, CompressedFileRange, Export,
    FileFlags, Import, ObjectKind, ObjectMap, Relocation, Result, SectionFlags, SectionIndex,
    SectionKind, SegmentFlags, SymbolFlags, SymbolIndex, SymbolKind, SymbolMap, SymbolMapName,
    SymbolScope, SymbolSection,
};
use crate::Endianness;

/// An object file.
pub trait Object<'data: 'file, 'file>: read::private::Sealed {
    /// A segment in the object file.
    type Segment: ObjectSegment<'data>;

    /// An iterator over the segments in the object file.
    type SegmentIterator: Iterator<Item = Self::Segment>;

    /// A section in the object file.
    type Section: ObjectSection<'data>;

    /// An iterator over the sections in the object file.
    type SectionIterator: Iterator<Item = Self::Section>;

    /// A COMDAT section group in the object file.
    type Comdat: ObjectComdat<'data>;

    /// An iterator over the COMDAT section groups in the object file.
    type ComdatIterator: Iterator<Item = Self::Comdat>;

    /// A symbol in the object file.
    type Symbol: ObjectSymbol<'data>;

    /// An iterator over symbols in the object file.
    type SymbolIterator: Iterator<Item = Self::Symbol>;

    /// A symbol table in the object file.
    type SymbolTable: ObjectSymbolTable<
        'data,
        Symbol = Self::Symbol,
        SymbolIterator = Self::SymbolIterator,
    >;

    /// An iterator over dynamic relocations in the file.
    ///
    /// The first field in the item tuple is the address
    /// that the relocation applies to.
    type DynamicRelocationIterator: Iterator<Item = (u64, Relocation)>;

    /// Get the architecture type of the file.
    fn architecture(&self) -> Architecture;

    /// Get the endianness of the file.
    #[inline]
    fn endianness(&self) -> Endianness {
        if self.is_little_endian() {
            Endianness::Little
        } else {
            Endianness::Big
        }
    }

    /// Return true if the file is little endian, false if it is big endian.
    fn is_little_endian(&self) -> bool;

    /// Return true if the file can contain 64-bit addresses.
    fn is_64(&self) -> bool;

    /// Return the kind of this object.
    fn kind(&self) -> ObjectKind;

    /// Get an iterator over the segments in the file.
    fn segments(&'file self) -> Self::SegmentIterator;

    /// Get the section named `section_name`, if such a section exists.
    ///
    /// If `section_name` starts with a '.' then it is treated as a system section name,
    /// and is compared using the conventions specific to the object file format. This
    /// includes:
    /// - if ".debug_str_offsets" is requested for a Mach-O object file, then the actual
    /// section name that is searched for is "__debug_str_offs".
    /// - if ".debug_info" is requested for an ELF object file, then
    /// ".zdebug_info" may be returned (and similarly for other debug sections).
    ///
    /// For some object files, multiple segments may contain sections with the same
    /// name. In this case, the first matching section will be used.
    ///
    /// This method skips over sections with invalid names.
    fn section_by_name(&'file self, section_name: &str) -> Option<Self::Section> {
        self.section_by_name_bytes(section_name.as_bytes())
    }

    /// Like [`Self::section_by_name`], but allows names that are not UTF-8.
    fn section_by_name_bytes(&'file self, section_name: &[u8]) -> Option<Self::Section>;

    /// Get the section at the given index.
    ///
    /// The meaning of the index depends on the object file.
    ///
    /// For some object files, this requires iterating through all sections.
    ///
    /// Returns an error if the index is invalid.
    fn section_by_index(&'file self, index: SectionIndex) -> Result<Self::Section>;

    /// Get an iterator over the sections in the file.
    fn sections(&'file self) -> Self::SectionIterator;

    /// Get an iterator over the COMDAT section groups in the file.
    fn comdats(&'file self) -> Self::ComdatIterator;

    /// Get the symbol table, if any.
    fn symbol_table(&'file self) -> Option<Self::SymbolTable>;

    /// Get the debugging symbol at the given index.
    ///
    /// The meaning of the index depends on the object file.
    ///
    /// Returns an error if the index is invalid.
    fn symbol_by_index(&'file self, index: SymbolIndex) -> Result<Self::Symbol>;

    /// Get an iterator over the debugging symbols in the file.
    ///
    /// This may skip over symbols that are malformed or unsupported.
    ///
    /// For Mach-O files, this does not include STAB entries.
    fn symbols(&'file self) -> Self::SymbolIterator;

    /// Get the dynamic linking symbol table, if any.
    ///
    /// Only ELF has a separate dynamic linking symbol table.
    fn dynamic_symbol_table(&'file self) -> Option<Self::SymbolTable>;

    /// Get an iterator over the dynamic linking symbols in the file.
    ///
    /// This may skip over symbols that are malformed or unsupported.
    ///
    /// Only ELF has separate dynamic linking symbols.
    /// Other file formats will return an empty iterator.
    fn dynamic_symbols(&'file self) -> Self::SymbolIterator;

    /// Get the dynamic relocations for this file.
    ///
    /// Symbol indices in these relocations refer to the dynamic symbol table.
    ///
    /// Only ELF has dynamic relocations.
    fn dynamic_relocations(&'file self) -> Option<Self::DynamicRelocationIterator>;

    /// Construct a map from addresses to symbol names.
    ///
    /// The map will only contain defined text and data symbols.
    /// The dynamic symbol table will only be used if there are no debugging symbols.
    fn symbol_map(&'file self) -> SymbolMap<SymbolMapName<'data>> {
        let mut symbols = Vec::new();
        if let Some(table) = self.symbol_table().or_else(|| self.dynamic_symbol_table()) {
            for symbol in table.symbols() {
                if !symbol.is_definition() {
                    continue;
                }
                if let Ok(name) = symbol.name() {
                    symbols.push(SymbolMapName::new(symbol.address(), name));
                }
            }
        }
        SymbolMap::new(symbols)
    }

    /// Construct a map from addresses to symbol names and object file names.
    ///
    /// This is derived from Mach-O STAB entries.
    fn object_map(&'file self) -> ObjectMap<'data> {
        ObjectMap::default()
    }

    /// Get the imported symbols.
    fn imports(&self) -> Result<Vec<Import<'data>>>;

    /// Get the exported symbols that expose both a name and an address.
    ///
    /// Some file formats may provide other kinds of symbols, that can be retrieved using
    /// the lower-level API.
    fn exports(&self) -> Result<Vec<Export<'data>>>;

    /// Return true if the file contains debug information sections, false if not.
    fn has_debug_symbols(&self) -> bool;

    /// The UUID from a Mach-O `LC_UUID` load command.
    #[inline]
    fn mach_uuid(&self) -> Result<Option<[u8; 16]>> {
        Ok(None)
    }

    /// The build ID from an ELF `NT_GNU_BUILD_ID` note.
    #[inline]
    fn build_id(&self) -> Result<Option<&'data [u8]>> {
        Ok(None)
    }

    /// The filename and CRC from a `.gnu_debuglink` section.
    #[inline]
    fn gnu_debuglink(&self) -> Result<Option<(&'data [u8], u32)>> {
        Ok(None)
    }

    /// The filename and build ID from a `.gnu_debugaltlink` section.
    #[inline]
    fn gnu_debugaltlink(&self) -> Result<Option<(&'data [u8], &'data [u8])>> {
        Ok(None)
    }

    /// The filename and GUID from the PE CodeView section
    #[inline]
    fn pdb_info(&self) -> Result<Option<CodeView>> {
        Ok(None)
    }

    /// Get the base address used for relative virtual addresses.
    ///
    /// Currently this is only non-zero for PE.
    fn relative_address_base(&'file self) -> u64;

    /// Get the virtual address of the entry point of the binary
    fn entry(&'file self) -> u64;

    /// File flags that are specific to each file format.
    fn flags(&self) -> FileFlags;
}

/// A loadable segment defined in an object file.
///
/// For ELF, this is a program header with type `PT_LOAD`.
/// For Mach-O, this is a load command with type `LC_SEGMENT` or `LC_SEGMENT_64`.
pub trait ObjectSegment<'data>: read::private::Sealed {
    /// Returns the virtual address of the segment.
    fn address(&self) -> u64;

    /// Returns the size of the segment in memory.
    fn size(&self) -> u64;

    /// Returns the alignment of the segment in memory.
    fn align(&self) -> u64;

    /// Returns the offset and size of the segment in the file.
    fn file_range(&self) -> (u64, u64);

    /// Returns a reference to the file contents of the segment.
    ///
    /// The length of this data may be different from the size of the
    /// segment in memory.
    fn data(&self) -> Result<&'data [u8]>;

    /// Return the segment data in the given range.
    ///
    /// Returns `Ok(None)` if the segment does not contain the given range.
    fn data_range(&self, address: u64, size: u64) -> Result<Option<&'data [u8]>>;

    /// Returns the name of the segment.
    fn name_bytes(&self) -> Result<Option<&[u8]>>;

    /// Returns the name of the segment.
    ///
    /// Returns an error if the name is not UTF-8.
    fn name(&self) -> Result<Option<&str>>;

    /// Return the flags of segment.
    fn flags(&self) -> SegmentFlags;
}

/// A section defined in an object file.
pub trait ObjectSection<'data>: read::private::Sealed {
    /// An iterator over the relocations for a section.
    ///
    /// The first field in the item tuple is the section offset
    /// that the relocation applies to.
    type RelocationIterator: Iterator<Item = (u64, Relocation)>;

    /// Returns the section index.
    fn index(&self) -> SectionIndex;

    /// Returns the address of the section.
    fn address(&self) -> u64;

    /// Returns the size of the section in memory.
    fn size(&self) -> u64;

    /// Returns the alignment of the section in memory.
    fn align(&self) -> u64;

    /// Returns offset and size of on-disk segment (if any).
    fn file_range(&self) -> Option<(u64, u64)>;

    /// Returns the raw contents of the section.
    ///
    /// The length of this data may be different from the size of the
    /// section in memory.
    ///
    /// This does not do any decompression.
    fn data(&self) -> Result<&'data [u8]>;

    /// Return the raw contents of the section data in the given range.
    ///
    /// This does not do any decompression.
    ///
    /// Returns `Ok(None)` if the section does not contain the given range.
    fn data_range(&self, address: u64, size: u64) -> Result<Option<&'data [u8]>>;

    /// Returns the potentially compressed file range of the section,
    /// along with information about the compression.
    fn compressed_file_range(&self) -> Result<CompressedFileRange>;

    /// Returns the potentially compressed contents of the section,
    /// along with information about the compression.
    fn compressed_data(&self) -> Result<CompressedData<'data>>;

    /// Returns the uncompressed contents of the section.
    ///
    /// The length of this data may be different from the size of the
    /// section in memory.
    ///
    /// If no compression is detected, then returns the data unchanged.
    /// Returns `Err` if decompression fails.
    fn uncompressed_data(&self) -> Result<Cow<'data, [u8]>> {
        self.compressed_data()?.decompress()
    }

    /// Returns the name of the section.
    fn name_bytes(&self) -> Result<&[u8]>;

    /// Returns the name of the section.
    ///
    /// Returns an error if the name is not UTF-8.
    fn name(&self) -> Result<&str>;

    /// Returns the name of the segment for this section.
    fn segment_name_bytes(&self) -> Result<Option<&[u8]>>;

    /// Returns the name of the segment for this section.
    ///
    /// Returns an error if the name is not UTF-8.
    fn segment_name(&self) -> Result<Option<&str>>;

    /// Return the kind of this section.
    fn kind(&self) -> SectionKind;

    /// Get the relocations for this section.
    fn relocations(&self) -> Self::RelocationIterator;

    /// Section flags that are specific to each file format.
    fn flags(&self) -> SectionFlags;
}

/// A COMDAT section group defined in an object file.
pub trait ObjectComdat<'data>: read::private::Sealed {
    /// An iterator over the sections in the object file.
    type SectionIterator: Iterator<Item = SectionIndex>;

    /// Returns the COMDAT selection kind.
    fn kind(&self) -> ComdatKind;

    /// Returns the index of the symbol used for the name of COMDAT section group.
    fn symbol(&self) -> SymbolIndex;

    /// Returns the name of the COMDAT section group.
    fn name_bytes(&self) -> Result<&[u8]>;

    /// Returns the name of the COMDAT section group.
    ///
    /// Returns an error if the name is not UTF-8.
    fn name(&self) -> Result<&str>;

    /// Get the sections in this section group.
    fn sections(&self) -> Self::SectionIterator;
}

/// A symbol table.
pub trait ObjectSymbolTable<'data>: read::private::Sealed {
    /// A symbol table entry.
    type Symbol: ObjectSymbol<'data>;

    /// An iterator over the symbols in a symbol table.
    type SymbolIterator: Iterator<Item = Self::Symbol>;

    /// Get an iterator over the symbols in the table.
    ///
    /// This may skip over symbols that are malformed or unsupported.
    fn symbols(&self) -> Self::SymbolIterator;

    /// Get the symbol at the given index.
    ///
    /// The meaning of the index depends on the object file.
    ///
    /// Returns an error if the index is invalid.
    fn symbol_by_index(&self, index: SymbolIndex) -> Result<Self::Symbol>;
}

/// A symbol table entry.
pub trait ObjectSymbol<'data>: read::private::Sealed {
    /// The index of the symbol.
    fn index(&self) -> SymbolIndex;

    /// The name of the symbol.
    fn name_bytes(&self) -> Result<&'data [u8]>;

    /// The name of the symbol.
    ///
    /// Returns an error if the name is not UTF-8.
    fn name(&self) -> Result<&'data str>;

    /// The address of the symbol. May be zero if the address is unknown.
    fn address(&self) -> u64;

    /// The size of the symbol. May be zero if the size is unknown.
    fn size(&self) -> u64;

    /// Return the kind of this symbol.
    fn kind(&self) -> SymbolKind;

    /// Returns the section where the symbol is defined.
    fn section(&self) -> SymbolSection;

    /// Returns the section index for the section containing this symbol.
    ///
    /// May return `None` if the symbol is not defined in a section.
    fn section_index(&self) -> Option<SectionIndex> {
        self.section().index()
    }

    /// Return true if the symbol is undefined.
    fn is_undefined(&self) -> bool;

    /// Return true if the symbol is a definition of a function or data object
    /// that has a known address.
    fn is_definition(&self) -> bool;

    /// Return true if the symbol is common data.
    ///
    /// Note: does not check for `SymbolSection::Section` with `SectionKind::Common`.
    fn is_common(&self) -> bool;

    /// Return true if the symbol is weak.
    fn is_weak(&self) -> bool;

    /// Returns the symbol scope.
    fn scope(&self) -> SymbolScope;

    /// Return true if the symbol visible outside of the compilation unit.
    ///
    /// This treats `SymbolScope::Unknown` as global.
    fn is_global(&self) -> bool;

    /// Return true if the symbol is only visible within the compilation unit.
    fn is_local(&self) -> bool;

    /// Symbol flags that are specific to each file format.
    fn flags(&self) -> SymbolFlags<SectionIndex>;
}

/// An iterator for files that don't have dynamic relocations.
#[derive(Debug)]
pub struct NoDynamicRelocationIterator;

impl Iterator for NoDynamicRelocationIterator {
    type Item = (u64, Relocation);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
