/* Copyright 2018 Mozilla Foundation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use crate::{
    BinaryReader, GlobalType, MemoryType, Result, SectionIteratorLimited, SectionReader,
    SectionWithLimitedItems, TableType, TagType,
};
use std::ops::Range;

/// Represents a reference to a type definition in a WebAssembly module.
#[derive(Debug, Clone, Copy)]
pub enum TypeRef {
    /// The type is a function.
    ///
    /// The value is an index into the type section.
    Func(u32),
    /// The type is a table.
    Table(TableType),
    /// The type is a memory.
    Memory(MemoryType),
    /// The type is a global.
    Global(GlobalType),
    /// The type is a tag.
    ///
    /// This variant is only used for the exception handling proposal.
    ///
    /// The value is an index in the types index space.
    Tag(TagType),
}

/// Represents an import in a WebAssembly module.
#[derive(Debug, Copy, Clone)]
pub struct Import<'a> {
    /// The module being imported from.
    pub module: &'a str,
    /// The name of the imported item.
    pub name: &'a str,
    /// The type of the imported item.
    pub ty: TypeRef,
}

/// A reader for the import section of a WebAssembly module.
#[derive(Clone)]
pub struct ImportSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

impl<'a> ImportSectionReader<'a> {
    /// Constructs a new `ImportSectionReader` for the given data and offset.
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
    /// use wasmparser::ImportSectionReader;
    /// let data: &[u8] = &[0x01, 0x01, 0x41, 0x01, 0x66, 0x00, 0x00];
    /// let mut reader = ImportSectionReader::new(data, 0).unwrap();
    /// for _ in 0..reader.get_count() {
    ///     let import = reader.read().expect("import");
    ///     println!("Import: {:?}", import);
    /// }
    /// ```
    pub fn read(&mut self) -> Result<Import<'a>> {
        self.reader.read_import()
    }
}

impl<'a> SectionReader for ImportSectionReader<'a> {
    type Item = Import<'a>;

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

impl<'a> SectionWithLimitedItems for ImportSectionReader<'a> {
    fn get_count(&self) -> u32 {
        Self::get_count(self)
    }
}

impl<'a> IntoIterator for ImportSectionReader<'a> {
    type Item = Result<Import<'a>>;
    type IntoIter = SectionIteratorLimited<Self>;

    fn into_iter(self) -> Self::IntoIter {
        SectionIteratorLimited::new(self)
    }
}
