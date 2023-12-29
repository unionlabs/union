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

use crate::{BinaryReader, Result, SectionIteratorLimited, SectionReader, SectionWithLimitedItems};
use std::ops::Range;

/// External types as defined [here].
///
/// [here]: https://webassembly.github.io/spec/core/syntax/types.html#external-types
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ExternalKind {
    /// The external kind is a function.
    Func,
    /// The external kind if a table.
    Table,
    /// The external kind is a memory.
    Memory,
    /// The external kind is a global.
    Global,
    /// The external kind is a tag.
    Tag,
}

/// Represents an export in a WebAssembly module.
#[derive(Debug, Copy, Clone)]
pub struct Export<'a> {
    /// The name of the exported item.
    pub name: &'a str,
    /// The kind of the export.
    pub kind: ExternalKind,
    /// The index of the exported item.
    pub index: u32,
}

/// A reader for the export section of a WebAssembly module.
#[derive(Clone)]
pub struct ExportSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

impl<'a> ExportSectionReader<'a> {
    /// Constructs a new `ExportSectionReader` for the given data and offset.
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
    /// use wasmparser::ExportSectionReader;
    ///
    /// # let data: &[u8] = &[0x01, 0x01, 0x65, 0x00, 0x00];
    /// let mut reader = ExportSectionReader::new(data, 0).unwrap();
    /// for _ in 0..reader.get_count() {
    ///     let export = reader.read().expect("export");
    ///     println!("Export: {:?}", export);
    /// }
    /// ```
    pub fn read(&mut self) -> Result<Export<'a>> {
        self.reader.read_export()
    }
}

impl<'a> SectionReader for ExportSectionReader<'a> {
    type Item = Export<'a>;

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

impl<'a> SectionWithLimitedItems for ExportSectionReader<'a> {
    fn get_count(&self) -> u32 {
        Self::get_count(self)
    }
}

impl<'a> IntoIterator for ExportSectionReader<'a> {
    type Item = Result<Export<'a>>;
    type IntoIter = SectionIteratorLimited<Self>;

    fn into_iter(self) -> Self::IntoIter {
        SectionIteratorLimited::new(self)
    }
}
