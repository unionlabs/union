use crate::{BinaryReader, Result, SectionIteratorLimited, SectionReader, SectionWithLimitedItems};
use std::ops::Range;

/// Represents the kind of an external items of a WebAssembly component.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComponentExternalKind {
    /// The external kind is a core module.
    Module,
    /// The external kind is a function.
    Func,
    /// The external kind is a value.
    Value,
    /// The external kind is a type.
    Type,
    /// The external kind is an instance.
    Instance,
    /// The external kind is a component.
    Component,
}

/// Represents an export in a WebAssembly component.
#[derive(Debug, Clone)]
pub struct ComponentExport<'a> {
    /// The name of the exported item.
    pub name: &'a str,
    /// The optional URL of the exported item.
    pub url: &'a str,
    /// The kind of the export.
    pub kind: ComponentExternalKind,
    /// The index of the exported item.
    pub index: u32,
}

/// A reader for the export section of a WebAssembly component.
#[derive(Clone)]
pub struct ComponentExportSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

impl<'a> ComponentExportSectionReader<'a> {
    /// Constructs a new `ComponentExportSectionReader` for the given data and offset.
    pub fn new(data: &'a [u8], offset: usize) -> Result<Self> {
        let mut reader = BinaryReader::new_with_offset(data, offset);
        let count = reader.read_var_u32()?;
        Ok(Self { reader, count })
    }

    /// Gets the original position of the section reader.
    pub fn original_position(&self) -> usize {
        self.reader.original_position()
    }

    /// Gets the count of items in the section.
    pub fn get_count(&self) -> u32 {
        self.count
    }

    /// Reads content of the export section.
    ///
    /// # Examples
    /// ```
    /// use wasmparser::ComponentExportSectionReader;
    ///
    /// # let data: &[u8] = &[0x01, 0x03, b'f', b'o', b'o', 0x00, 0x01, 0x00];
    /// let mut reader = ComponentExportSectionReader::new(data, 0).unwrap();
    /// for _ in 0..reader.get_count() {
    ///     let export = reader.read().expect("export");
    ///     println!("Export: {:?}", export);
    /// }
    /// ```
    pub fn read(&mut self) -> Result<ComponentExport<'a>> {
        self.reader.read_component_export()
    }
}

impl<'a> SectionReader for ComponentExportSectionReader<'a> {
    type Item = ComponentExport<'a>;

    fn read(&mut self) -> Result<Self::Item> {
        Self::read(self)
    }

    fn eof(&self) -> bool {
        self.reader.eof()
    }

    fn original_position(&self) -> usize {
        Self::original_position(self)
    }

    fn range(&self) -> Range<usize> {
        self.reader.range()
    }
}

impl<'a> SectionWithLimitedItems for ComponentExportSectionReader<'a> {
    fn get_count(&self) -> u32 {
        Self::get_count(self)
    }
}

impl<'a> IntoIterator for ComponentExportSectionReader<'a> {
    type Item = Result<ComponentExport<'a>>;
    type IntoIter = SectionIteratorLimited<Self>;

    fn into_iter(self) -> Self::IntoIter {
        SectionIteratorLimited::new(self)
    }
}
