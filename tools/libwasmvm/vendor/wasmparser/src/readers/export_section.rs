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
    BinaryReader, ExternalKind, Range, Result, SectionIteratorLimited, SectionReader,
    SectionWithLimitedItems,
};

#[derive(Debug, Copy, Clone)]
pub struct Export<'a> {
    pub field: &'a str,
    pub kind: ExternalKind,
    pub index: u32,
}

#[derive(Clone)]
pub struct ExportSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

impl<'a> ExportSectionReader<'a> {
    pub fn new(data: &'a [u8], offset: usize) -> Result<ExportSectionReader<'a>> {
        let mut reader = BinaryReader::new_with_offset(data, offset);
        let count = reader.read_var_u32()?;
        Ok(ExportSectionReader { reader, count })
    }

    pub fn original_position(&self) -> usize {
        self.reader.original_position()
    }

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
    /// let mut export_reader = ExportSectionReader::new(data, 0).unwrap();
    /// for _ in 0..export_reader.get_count() {
    ///     let export = export_reader.read().expect("export");
    ///     println!("Export: {:?}", export);
    /// }
    /// ```
    pub fn read<'b>(&mut self) -> Result<Export<'b>>
    where
        'a: 'b,
    {
        let field = self.reader.read_string()?;
        let kind = self.reader.read_external_kind()?;
        let index = self.reader.read_var_u32()?;
        Ok(Export { field, kind, index })
    }
}

impl<'a> SectionReader for ExportSectionReader<'a> {
    type Item = Export<'a>;
    fn read(&mut self) -> Result<Self::Item> {
        ExportSectionReader::read(self)
    }
    fn eof(&self) -> bool {
        self.reader.eof()
    }
    fn original_position(&self) -> usize {
        ExportSectionReader::original_position(self)
    }
    fn range(&self) -> Range {
        self.reader.range()
    }
}

impl<'a> SectionWithLimitedItems for ExportSectionReader<'a> {
    fn get_count(&self) -> u32 {
        ExportSectionReader::get_count(self)
    }
}

impl<'a> IntoIterator for ExportSectionReader<'a> {
    type Item = Result<Export<'a>>;
    type IntoIter = SectionIteratorLimited<ExportSectionReader<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        SectionIteratorLimited::new(self)
    }
}
