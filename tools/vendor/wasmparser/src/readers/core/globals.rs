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
    BinaryReader, ConstExpr, GlobalType, Result, SectionIteratorLimited, SectionReader,
    SectionWithLimitedItems,
};
use std::ops::Range;

/// Represents a core WebAssembly global.
#[derive(Debug, Copy, Clone)]
pub struct Global<'a> {
    /// The global's type.
    pub ty: GlobalType,
    /// The global's initialization expression.
    pub init_expr: ConstExpr<'a>,
}

/// A reader for the global section of a WebAssembly module.
#[derive(Clone)]
pub struct GlobalSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

impl<'a> GlobalSectionReader<'a> {
    /// Constructs a new `GlobalSectionReader` for the given data and offset.
    pub fn new(data: &'a [u8], offset: usize) -> Result<GlobalSectionReader<'a>> {
        let mut reader = BinaryReader::new_with_offset(data, offset);
        let count = reader.read_var_u32()?;
        Ok(GlobalSectionReader { reader, count })
    }

    /// Gets the original position of the section reader.
    pub fn original_position(&self) -> usize {
        self.reader.original_position()
    }

    /// Gets the count of items in the section.
    pub fn get_count(&self) -> u32 {
        self.count
    }

    /// Reads content of the global section.
    ///
    /// # Examples
    /// ```
    /// use wasmparser::GlobalSectionReader;
    /// # let data: &[u8] = &[0x01, 0x7F, 0x01, 0x41, 0x90, 0x88, 0x04, 0x0B];
    /// let mut global_reader = GlobalSectionReader::new(data, 0).unwrap();
    /// for _ in 0..global_reader.get_count() {
    ///     let global = global_reader.read().expect("global");
    ///     println!("Global: {:?}", global);
    ///     let mut init_expr_reader = global.init_expr.get_binary_reader();
    ///     let op = init_expr_reader.read_operator().expect("op");
    ///     println!("Init const: {:?}", op);
    /// }
    /// ```
    pub fn read<'b>(&mut self) -> Result<Global<'b>>
    where
        'a: 'b,
    {
        let ty = self.reader.read_global_type()?;
        let init_expr = self.reader.read_const_expr()?;
        Ok(Global { ty, init_expr })
    }
}

impl<'a> SectionReader for GlobalSectionReader<'a> {
    type Item = Global<'a>;
    fn read(&mut self) -> Result<Self::Item> {
        GlobalSectionReader::read(self)
    }
    fn eof(&self) -> bool {
        self.reader.eof()
    }
    fn original_position(&self) -> usize {
        GlobalSectionReader::original_position(self)
    }
    fn range(&self) -> Range<usize> {
        self.reader.range()
    }
}

impl<'a> SectionWithLimitedItems for GlobalSectionReader<'a> {
    fn get_count(&self) -> u32 {
        GlobalSectionReader::get_count(self)
    }
}

impl<'a> IntoIterator for GlobalSectionReader<'a> {
    type Item = Result<Global<'a>>;
    type IntoIter = SectionIteratorLimited<GlobalSectionReader<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        SectionIteratorLimited::new(self)
    }
}
