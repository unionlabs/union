use crate::{
    BinaryReader, ComponentExternalKind, ExternalKind, Result, SectionIteratorLimited,
    SectionReader, SectionWithLimitedItems,
};
use std::ops::Range;

/// Represents the kind of an outer alias in a WebAssembly component.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComponentOuterAliasKind {
    /// The alias is to a core module.
    CoreModule,
    /// The alias is to a core type.
    CoreType,
    /// The alias is to a component type.
    Type,
    /// The alias is to a component.
    Component,
}

/// Represents an alias in a WebAssembly component.
#[derive(Debug, Clone)]
pub enum ComponentAlias<'a> {
    /// The alias is to an export of a component instance.
    InstanceExport {
        /// The alias kind.
        kind: ComponentExternalKind,
        /// The instance index.
        instance_index: u32,
        /// The export name.
        name: &'a str,
    },
    /// The alias is to an export of a module instance.
    CoreInstanceExport {
        /// The alias kind.
        kind: ExternalKind,
        /// The instance index.
        instance_index: u32,
        /// The export name.
        name: &'a str,
    },
    /// The alias is to an outer item.
    Outer {
        /// The alias kind.
        kind: ComponentOuterAliasKind,
        /// The outward count, starting at zero for the current component.
        count: u32,
        /// The index of the item within the outer component.
        index: u32,
    },
}

/// A reader for the alias section of a WebAssembly component.
#[derive(Clone)]
pub struct ComponentAliasSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

impl<'a> ComponentAliasSectionReader<'a> {
    /// Constructs a new `ComponentAliasSectionReader` for the given data and offset.
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

    /// Reads content of the alias section.
    ///
    /// # Examples
    /// ```
    /// use wasmparser::ComponentAliasSectionReader;
    /// let data: &[u8] = &[0x01, 0x01, 0x00, 0x00, 0x03, b'f', b'o', b'o'];
    /// let mut reader = ComponentAliasSectionReader::new(data, 0).unwrap();
    /// for _ in 0..reader.get_count() {
    ///     let alias = reader.read().expect("alias");
    ///     println!("Alias: {:?}", alias);
    /// }
    /// ```
    pub fn read(&mut self) -> Result<ComponentAlias<'a>> {
        self.reader.read_component_alias()
    }
}

impl<'a> SectionReader for ComponentAliasSectionReader<'a> {
    type Item = ComponentAlias<'a>;

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

impl<'a> SectionWithLimitedItems for ComponentAliasSectionReader<'a> {
    fn get_count(&self) -> u32 {
        Self::get_count(self)
    }
}

impl<'a> IntoIterator for ComponentAliasSectionReader<'a> {
    type Item = Result<ComponentAlias<'a>>;
    type IntoIter = SectionIteratorLimited<Self>;

    fn into_iter(self) -> Self::IntoIter {
        SectionIteratorLimited::new(self)
    }
}
