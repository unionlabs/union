use crate::{BinaryReader, Result, SectionReader};
use std::ops::Range;

/// Represents the start function in a WebAssembly component.
#[derive(Debug, Clone)]
pub struct ComponentStartFunction {
    /// The index to the start function.
    pub func_index: u32,
    /// The start function arguments.
    ///
    /// The arguments are specified by value index.
    pub arguments: Box<[u32]>,
    /// The number of expected results for the start function.
    pub results: u32,
}

/// A reader for the start section of a WebAssembly component.
#[derive(Clone)]
pub struct ComponentStartSectionReader<'a>(BinaryReader<'a>);

impl<'a> ComponentStartSectionReader<'a> {
    /// Constructs a new `ComponentStartSectionReader` for the given data and offset.
    pub fn new(data: &'a [u8], offset: usize) -> Result<Self> {
        Ok(Self(BinaryReader::new_with_offset(data, offset)))
    }

    /// Gets the original position of the section reader.
    pub fn original_position(&self) -> usize {
        self.0.original_position()
    }

    /// Reads the start function from the section.
    ///
    /// # Examples
    /// ```
    /// use wasmparser::ComponentStartSectionReader;
    ///
    /// # let data: &[u8] = &[0x00, 0x03, 0x01, 0x02, 0x03, 0x01];
    /// let mut reader = ComponentStartSectionReader::new(data, 0).unwrap();
    /// let start = reader.read().expect("start");
    /// println!("Start: {:?}", start);
    /// ```
    pub fn read(&mut self) -> Result<ComponentStartFunction> {
        self.0.read_component_start()
    }
}

impl<'a> SectionReader for ComponentStartSectionReader<'a> {
    type Item = ComponentStartFunction;

    fn read(&mut self) -> Result<Self::Item> {
        Self::read(self)
    }

    fn eof(&self) -> bool {
        self.0.eof()
    }

    fn original_position(&self) -> usize {
        Self::original_position(self)
    }

    fn range(&self) -> Range<usize> {
        self.0.range()
    }
}
