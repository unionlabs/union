use crate::{
    BinaryReader, ComponentValType, Result, SectionIteratorLimited, SectionReader,
    SectionWithLimitedItems,
};
use std::ops::Range;

/// Represents the type bounds for imports and exports.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TypeBounds {
    /// The type is bounded by equality.
    Eq,
}

/// Represents a reference to a component type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComponentTypeRef {
    /// The reference is to a core module type.
    ///
    /// The index is expected to be core type index to a core module type.
    Module(u32),
    /// The reference is to a function type.
    ///
    /// The index is expected to be a type index to a function type.
    Func(u32),
    /// The reference is to a value type.
    Value(ComponentValType),
    /// The reference is to a bounded type.
    ///
    /// The index is expected to be a type index.
    Type(TypeBounds, u32),
    /// The reference is to an instance type.
    ///
    /// The index is a type index to an instance type.
    Instance(u32),
    /// The reference is to a component type.
    ///
    /// The index is a type index to a component type.
    Component(u32),
}

/// Represents an import in a WebAssembly component
#[derive(Debug, Copy, Clone)]
pub struct ComponentImport<'a> {
    /// The name of the imported item.
    pub name: &'a str,
    /// The optional URL of the imported item.
    pub url: &'a str,
    /// The type reference for the import.
    pub ty: ComponentTypeRef,
}

/// A reader for the import section of a WebAssembly component.
#[derive(Clone)]
pub struct ComponentImportSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

impl<'a> ComponentImportSectionReader<'a> {
    /// Constructs a new `ComponentImportSectionReader` for the given data and offset.
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

    /// Reads content of the import section.
    ///
    /// # Examples
    /// ```
    /// use wasmparser::ComponentImportSectionReader;
    /// let data: &[u8] = &[0x01, 0x01, 0x41, 0x00, 0x01, 0x66, 0x00, 0x00];
    /// let mut reader = ComponentImportSectionReader::new(data, 0).unwrap();
    /// for _ in 0..reader.get_count() {
    ///     let import = reader.read().expect("import");
    ///     println!("Import: {:?}", import);
    /// }
    /// ```
    pub fn read(&mut self) -> Result<ComponentImport<'a>> {
        self.reader.read_component_import()
    }
}

impl<'a> SectionReader for ComponentImportSectionReader<'a> {
    type Item = ComponentImport<'a>;

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

impl<'a> SectionWithLimitedItems for ComponentImportSectionReader<'a> {
    fn get_count(&self) -> u32 {
        Self::get_count(self)
    }
}

impl<'a> IntoIterator for ComponentImportSectionReader<'a> {
    type Item = Result<ComponentImport<'a>>;
    type IntoIter = SectionIteratorLimited<Self>;

    fn into_iter(self) -> Self::IntoIter {
        SectionIteratorLimited::new(self)
    }
}
