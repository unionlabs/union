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

use super::{
    BinaryReader, BinaryReaderError, InitExpr, Range, Result, SectionIteratorLimited,
    SectionReader, SectionWithLimitedItems,
};

#[derive(Debug, Copy, Clone)]
pub struct Data<'a> {
    pub kind: DataKind<'a>,
    pub data: &'a [u8],
    pub range: Range,
}

#[derive(Debug, Copy, Clone)]
pub enum DataKind<'a> {
    Passive,
    Active {
        memory_index: u32,
        init_expr: InitExpr<'a>,
    },
}

#[derive(Clone)]
pub struct DataSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
    forbid_bulk_memory: bool,
}

impl<'a> DataSectionReader<'a> {
    pub fn new(data: &'a [u8], offset: usize) -> Result<DataSectionReader<'a>> {
        let mut reader = BinaryReader::new_with_offset(data, offset);
        let count = reader.read_var_u32()?;
        Ok(DataSectionReader {
            reader,
            count,
            forbid_bulk_memory: false,
        })
    }

    pub fn original_position(&self) -> usize {
        self.reader.original_position()
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn forbid_bulk_memory(&mut self, forbid: bool) {
        self.forbid_bulk_memory = forbid;
    }

    fn verify_data_end(&self, end: usize) -> Result<()> {
        if self.reader.buffer.len() < end {
            return Err(BinaryReaderError::new(
                "Data segment extends past end of the data section",
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
    ///     if let DataKind::Active { init_expr, .. } = data.kind {
    ///         let mut init_expr_reader = init_expr.get_binary_reader();
    ///         let op = init_expr_reader.read_operator().expect("op");
    ///         println!("Init const: {:?}", op);
    ///     }
    /// }
    /// ```
    pub fn read<'b>(&mut self) -> Result<Data<'b>>
    where
        'a: 'b,
    {
        let segment_start = self.reader.original_position();

        let flags = self.reader.read_var_u32()?;
        let kind = if !self.forbid_bulk_memory && flags == 1 {
            DataKind::Passive
        } else {
            let memory_index = match flags {
                0 => 0,
                _ if self.forbid_bulk_memory => flags,
                2 => self.reader.read_var_u32()?,
                _ => {
                    return Err(BinaryReaderError::new(
                        "invalid flags byte in data segment",
                        self.reader.original_position() - 1,
                    ));
                }
            };
            let init_expr = {
                let expr_offset = self.reader.position;
                self.reader.skip_init_expr()?;
                let data = &self.reader.buffer[expr_offset..self.reader.position];
                InitExpr::new(data, self.reader.original_offset + expr_offset)
            };
            DataKind::Active {
                memory_index,
                init_expr,
            }
        };
        let data_len = self.reader.read_var_u32()? as usize;
        let data_end = self.reader.position + data_len;
        self.verify_data_end(data_end)?;
        let data = &self.reader.buffer[self.reader.position..data_end];
        self.reader.skip_to(data_end);

        let segment_end = self.reader.original_position();
        let range = Range::new(segment_start, segment_end);

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
    fn range(&self) -> Range {
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
