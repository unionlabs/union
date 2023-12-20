use crate::{
    BinaryReader, ComponentExport, ComponentExternalKind, Export, Result, SectionIteratorLimited,
    SectionReader, SectionWithLimitedItems,
};
use std::ops::Range;

/// Represents the kind of an instantiation argument for a core instance.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InstantiationArgKind {
    /// The instantiation argument is a core instance.
    Instance,
}

/// Represents an argument to instantiating a WebAssembly module.
#[derive(Debug, Clone)]
pub struct InstantiationArg<'a> {
    /// The name of the module argument.
    pub name: &'a str,
    /// The kind of the module argument.
    pub kind: InstantiationArgKind,
    /// The index of the argument item.
    pub index: u32,
}

/// Represents an instance of a WebAssembly module.
#[derive(Debug, Clone)]
pub enum Instance<'a> {
    /// The instance is from instantiating a WebAssembly module.
    Instantiate {
        /// The module index.
        module_index: u32,
        /// The module's instantiation arguments.
        args: Box<[InstantiationArg<'a>]>,
    },
    /// The instance is a from exporting local items.
    FromExports(Box<[Export<'a>]>),
}

/// A reader for the core instance section of a WebAssembly component.
#[derive(Clone)]
pub struct InstanceSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

impl<'a> InstanceSectionReader<'a> {
    /// Constructs a new `InstanceSectionReader` for the given data and offset.
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

    /// Reads content of the instance section.
    ///
    /// # Examples
    /// ```
    /// use wasmparser::InstanceSectionReader;
    /// # let data: &[u8] = &[0x01, 0x00, 0x00, 0x01, 0x03, b'f', b'o', b'o', 0x12, 0x00];
    /// let mut reader = InstanceSectionReader::new(data, 0).unwrap();
    /// for _ in 0..reader.get_count() {
    ///     let instance = reader.read().expect("instance");
    ///     println!("Instance: {:?}", instance);
    /// }
    /// ```
    pub fn read(&mut self) -> Result<Instance<'a>> {
        self.reader.read_instance()
    }
}

impl<'a> SectionReader for InstanceSectionReader<'a> {
    type Item = Instance<'a>;

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

impl<'a> SectionWithLimitedItems for InstanceSectionReader<'a> {
    fn get_count(&self) -> u32 {
        Self::get_count(self)
    }
}

impl<'a> IntoIterator for InstanceSectionReader<'a> {
    type Item = Result<Instance<'a>>;
    type IntoIter = SectionIteratorLimited<Self>;

    /// Implements iterator over the instance section.
    ///
    /// # Examples
    ///
    /// ```
    /// use wasmparser::InstanceSectionReader;
    /// # let data: &[u8] = &[0x01, 0x00, 0x00, 0x01, 0x03, b'f', b'o', b'o', 0x12, 0x00];
    /// let mut reader = InstanceSectionReader::new(data, 0).unwrap();
    /// for inst in reader {
    ///     println!("Instance {:?}", inst.expect("instance"));
    /// }
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        SectionIteratorLimited::new(self)
    }
}

/// Represents an argument to instantiating a WebAssembly component.
#[derive(Debug, Clone)]
pub struct ComponentInstantiationArg<'a> {
    /// The name of the component argument.
    pub name: &'a str,
    /// The kind of the component argument.
    pub kind: ComponentExternalKind,
    /// The index of the argument item.
    pub index: u32,
}

/// Represents an instance in a WebAssembly component.
#[derive(Debug, Clone)]
pub enum ComponentInstance<'a> {
    /// The instance is from instantiating a WebAssembly component.
    Instantiate {
        /// The component index.
        component_index: u32,
        /// The component's instantiation arguments.
        args: Box<[ComponentInstantiationArg<'a>]>,
    },
    /// The instance is a from exporting local items.
    FromExports(Box<[ComponentExport<'a>]>),
}

/// A reader for the component instance section of a WebAssembly component.
#[derive(Clone)]
pub struct ComponentInstanceSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

impl<'a> ComponentInstanceSectionReader<'a> {
    /// Constructs a new `ComponentInstanceSectionReader` for the given data and offset.
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

    /// Reads content of the instance section.
    ///
    /// # Examples
    /// ```
    /// use wasmparser::ComponentInstanceSectionReader;
    /// # let data: &[u8] = &[0x01, 0x00, 0x00, 0x01, 0x03, b'f', b'o', b'o', 0x01, 0x00];
    /// let mut reader = ComponentInstanceSectionReader::new(data, 0).unwrap();
    /// for _ in 0..reader.get_count() {
    ///     let instance = reader.read().expect("instance");
    ///     println!("Instance: {:?}", instance);
    /// }
    /// ```
    pub fn read(&mut self) -> Result<ComponentInstance<'a>> {
        self.reader.read_component_instance()
    }
}

impl<'a> SectionReader for ComponentInstanceSectionReader<'a> {
    type Item = ComponentInstance<'a>;

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

impl<'a> SectionWithLimitedItems for ComponentInstanceSectionReader<'a> {
    fn get_count(&self) -> u32 {
        Self::get_count(self)
    }
}

impl<'a> IntoIterator for ComponentInstanceSectionReader<'a> {
    type Item = Result<ComponentInstance<'a>>;
    type IntoIter = SectionIteratorLimited<Self>;

    /// Implements iterator over the instance section.
    ///
    /// # Examples
    ///
    /// ```
    /// use wasmparser::ComponentInstanceSectionReader;
    /// # let data: &[u8] = &[0x01, 0x00, 0x00, 0x01, 0x03, b'f', b'o', b'o', 0x01, 0x00];
    /// let mut reader = ComponentInstanceSectionReader::new(data, 0).unwrap();
    /// for inst in reader {
    ///     println!("Instance {:?}", inst.expect("instance"));
    /// }
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        SectionIteratorLimited::new(self)
    }
}
