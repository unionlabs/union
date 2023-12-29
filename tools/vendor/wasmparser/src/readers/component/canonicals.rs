use crate::{BinaryReader, Result, SectionIteratorLimited, SectionReader, SectionWithLimitedItems};
use std::ops::Range;

/// Represents options for component functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanonicalOption {
    /// The string types in the function signature are UTF-8 encoded.
    UTF8,
    /// The string types in the function signature are UTF-16 encoded.
    UTF16,
    /// The string types in the function signature are compact UTF-16 encoded.
    CompactUTF16,
    /// The memory to use if the lifting or lowering of a function requires memory access.
    ///
    /// The value is an index to a core memory.
    Memory(u32),
    /// The realloc function to use if the lifting or lowering of a function requires memory
    /// allocation.
    ///
    /// The value is an index to a core function of type `(func (param i32 i32 i32 i32) (result i32))`.
    Realloc(u32),
    /// The post-return function to use if the lifting of a function requires
    /// cleanup after the function returns.
    PostReturn(u32),
}

/// Represents a canonical function in a WebAssembly component.
#[derive(Debug, Clone)]
pub enum CanonicalFunction {
    /// The function lifts a core WebAssembly function to the canonical ABI.
    Lift {
        /// The index of the core WebAssembly function to lift.
        core_func_index: u32,
        /// The index of the lifted function's type.
        type_index: u32,
        /// The canonical options for the function.
        options: Box<[CanonicalOption]>,
    },
    /// The function lowers a canonical ABI function to a core WebAssembly function.
    Lower {
        /// The index of the function to lower.
        func_index: u32,
        /// The canonical options for the function.
        options: Box<[CanonicalOption]>,
    },
}

/// A reader for the canonical section of a WebAssembly component.
#[derive(Clone)]
pub struct ComponentCanonicalSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

impl<'a> ComponentCanonicalSectionReader<'a> {
    /// Constructs a new `ComponentFunctionSectionReader` for the given data and offset.
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

    /// Reads function type index from the function section.
    ///
    /// # Examples
    ///
    /// ```
    /// use wasmparser::ComponentCanonicalSectionReader;
    /// # let data: &[u8] = &[0x01, 0x00, 0x00, 0x00, 0x01, 0x03, 0x00, 0x00];
    /// let mut reader = ComponentCanonicalSectionReader::new(data, 0).unwrap();
    /// for _ in 0..reader.get_count() {
    ///     let func = reader.read().expect("func");
    ///     println!("Function: {:?}", func);
    /// }
    /// ```
    pub fn read(&mut self) -> Result<CanonicalFunction> {
        self.reader.read_canonical_func()
    }
}

impl<'a> SectionReader for ComponentCanonicalSectionReader<'a> {
    type Item = CanonicalFunction;

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

impl<'a> SectionWithLimitedItems for ComponentCanonicalSectionReader<'a> {
    fn get_count(&self) -> u32 {
        Self::get_count(self)
    }
}

impl<'a> IntoIterator for ComponentCanonicalSectionReader<'a> {
    type Item = Result<CanonicalFunction>;
    type IntoIter = SectionIteratorLimited<Self>;

    fn into_iter(self) -> Self::IntoIter {
        SectionIteratorLimited::new(self)
    }
}
