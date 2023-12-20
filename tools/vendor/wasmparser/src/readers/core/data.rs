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
    BinaryReader, BinaryReaderError, ConstExpr, Result, SectionIteratorLimited, SectionReader,
    SectionWithLimitedItems,
};
use std::ops::Range;

/// Represents a data segment in a core WebAssembly module.
#[derive(Debug, Clone)]
pub struct Data<'a> {
    /// The kind of data segment.
    pub kind: DataKind<'a>,
    /// The data of the data segment.
    pub data: &'a [u8],
    /// The range of the data segment.
    pub range: Range<usize>,
}

/// The kind of data segment.
#[derive(Debug, Copy, Clone)]
pub enum DataKind<'a> {
    /// The data segment is passive.
    Passive,
    /// The data segment is active.
    Active {
        /// The memory index for the data segment.
        memory_index: u32,
        /// The initialization expression for the data segment.
        offset_expr: ConstExpr<'a>,
    },
}

/// A reader for the data section of a WebAssembly module.
#[derive(Clone)]
pub struct DataSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

impl<'a> DataSectionReader<'a> {
    /// Constructs a new `DataSectionReader` for the given data and offset.
    pub fn new(data: &'a [u8], offset: usize) -> Result<DataSectionReader<'a>> {
        let mut reader = BinaryReader::new_with_offset(data, offset);
        let count = reader.read_var_u32()?;
        Ok(DataSectionReader { reader, count })
    }

    /// Gets the original position of the section reader.
    pub fn original_position(&self) -> usize {
        self.reader.original_position()
    }

    /// Gets the count of items in the section.
    pub fn get_count(&self) -> u32 {
        self.count
    }

    fn verify_data_end(&self, end: usize) -> Result<()> {
        if self.reader.buffer.len() < end {
            return Err(BinaryReaderError::new(
                "unexpected end of section or function: data segment extends past end of the data section",
                self.reader.original_offset + self.reader.buffer.len(),
            ));
        }
        Ok(())
    }

    /// Reads content of the data section.
    ///
    /// # Examples
    /// ```
    /// use wasmparser::{DataSectionReader, DataKind};
    /// # let data: &[u8] = &[
    /// #     0x01, 0x00, 0x41, 0x80, 0x08, 0x0b, 0x04, 0x00, 0x00, 0x00, 0x00];
    /// let mut data_reader = DataSectionReader::new(data, 0).unwrap();
    /// for _ in 0..data_reader.get_count() {
    ///     let data = data_reader.read().expect("data");
    ///     println!("Data: {:?}", data);
    ///     if let DataKind::Active { offset_expr, .. } = data.kind {
    ///         let mut offset_expr_reader = offset_expr.get_binary_reader();
    ///         let op = offset_expr_reader.read_operator().expect("op");
    ///         println!("offset expression: {:?}", op);
    ///     }
    /// }
    /// ```
    pub fn read<'b>(&mut self) -> Result<Data<'b>>
    where
        'a: 'b,
    {
        let segment_start = self.reader.original_position();

        // The current handling of the flags is largely specified in the `bulk-memory` proposal,
        // which at the time this commend is written has been merged to the main specification
        // draft.
        //
        // Notably, this proposal allows multiple different encodings of the memory index 0. `00`
        // and `02 00` are both valid ways to specify the 0-th memory. However it also makes
        // another encoding of the 0-th memory `80 00` no longer valid.
        //
        // We, however maintain this by parsing `flags` as a LEB128 integer. In that case, `80 00`
        // encoding is parsed out as `0` and is therefore assigned a `memidx` 0, even though the
        // current specification draft does not allow for this.
        //
        // See also https://github.com/WebAssembly/spec/issues/1439
        let flags = self.reader.read_var_u32()?;
        let kind = match flags {
            1 => DataKind::Passive,
            0 | 2 => {
                let memory_index = if flags == 0 {
                    0
                } else {
                    self.reader.read_var_u32()?
                };
                let offset_expr = {
                    let expr_offset = self.reader.position;
                    self.reader.skip_const_expr()?;
                    let data = &self.reader.buffer[expr_offset..self.reader.position];
                    ConstExpr::new(data, self.reader.original_offset + expr_offset)
                };
                DataKind::Active {
                    memory_index,
                    offset_expr,
                }
            }
            _ => {
                return Err(BinaryReaderError::new(
                    "invalid flags byte in data segment",
                    self.reader.original_position() - 1,
                ));
            }
        };

        let data_len = self.reader.read_var_u32()? as usize;
        let data_end = self.reader.position + data_len;
        self.verify_data_end(data_end)?;
        let data = &self.reader.buffer[self.reader.position..data_end];
        self.reader.skip_to(data_end);

        let segment_end = self.reader.original_position();
        let range = segment_start..segment_end;

        Ok(Data { kind, data, range })
    }
}

impl<'a> SectionReader for DataSectionReader<'a> {
    type Item = Data<'a>;
    fn read(&mut self) -> Result<Self::Item> {
        DataSectionReader::read(self)
    }
    fn eof(&self) -> bool {
        self.reader.eof()
    }
    fn original_position(&self) -> usize {
        DataSectionReader::original_position(self)
    }
    fn range(&self) -> Range<usize> {
        self.reader.range()
    }
}

impl<'a> SectionWithLimitedItems for DataSectionReader<'a> {
    fn get_count(&self) -> u32 {
        DataSectionReader::get_count(self)
    }
}

impl<'a> IntoIterator for DataSectionReader<'a> {
    type Item = Result<Data<'a>>;
    type IntoIter = SectionIteratorLimited<DataSectionReader<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        SectionIteratorLimited::new(self)
    }
}
