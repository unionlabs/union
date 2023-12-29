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
    BinaryReader, Result, SectionIteratorLimited, SectionReader, SectionWithLimitedItems, TableType,
};
use std::ops::Range;

/// A reader for the table section of a WebAssembly module.
#[derive(Clone)]
pub struct TableSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

impl<'a> TableSectionReader<'a> {
    /// Constructs a new `TableSectionReader` for the given data and offset.
    pub fn new(data: &'a [u8], offset: usize) -> Result<TableSectionReader<'a>> {
        let mut reader = BinaryReader::new_with_offset(data, offset);
        let count = reader.read_var_u32()?;
        Ok(TableSectionReader { reader, count })
    }

    /// Gets the original position of the section reader.
    pub fn original_position(&self) -> usize {
        self.reader.original_position()
    }

    /// Gets the count of items in the section.
    pub fn get_count(&self) -> u32 {
        self.count
    }

    /// Reads content of the table section.
    ///
    /// # Examples
    /// ```
    /// use wasmparser::TableSectionReader;
    ///
    /// # let data: &[u8] = &[0x01, 0x70, 0x01, 0x01, 0x01];
    /// let mut table_reader = TableSectionReader::new(data, 0).unwrap();
    /// for _ in 0..table_reader.get_count() {
    ///     let table = table_reader.read().expect("table");
    ///     println!("Table: {:?}", table);
    /// }
    /// ```
    pub fn read(&mut self) -> Result<TableType> {
        self.reader.read_table_type()
    }
}

impl<'a> SectionReader for TableSectionReader<'a> {
    type Item = TableType;
    fn read(&mut self) -> Result<Self::Item> {
        TableSectionReader::read(self)
    }
    fn eof(&self) -> bool {
        self.reader.eof()
    }
    fn original_position(&self) -> usize {
        TableSectionReader::original_position(self)
    }
    fn range(&self) -> Range<usize> {
        self.reader.range()
    }
}

impl<'a> SectionWithLimitedItems for TableSectionReader<'a> {
    fn get_count(&self) -> u32 {
        TableSectionReader::get_count(self)
    }
}

impl<'a> IntoIterator for TableSectionReader<'a> {
    type Item = Result<TableType>;
    type IntoIter = SectionIteratorLimited<TableSectionReader<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        SectionIteratorLimited::new(self)
    }
}
