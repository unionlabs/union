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

use super::{BinaryReader, BinaryReaderError, Operator, Result};

#[derive(Clone)]
pub struct OperatorsReader<'a> {
    pub(crate) reader: BinaryReader<'a>,
}

impl<'a> OperatorsReader<'a> {
    pub(crate) fn new<'b>(data: &'a [u8], offset: usize) -> OperatorsReader<'b>
    where
        'a: 'b,
    {
        OperatorsReader {
            reader: BinaryReader::new_with_offset(data, offset),
        }
    }

    pub fn eof(&self) -> bool {
        self.reader.eof()
    }

    pub fn original_position(&self) -> usize {
        self.reader.original_position()
    }

    pub fn allow_memarg64(&mut self, allow: bool) {
        self.reader.allow_memarg64(allow);
    }

    pub fn ensure_end(&self) -> Result<()> {
        if self.eof() {
            return Ok(());
        }
        Err(BinaryReaderError::new(
            "Unexpected data at the end of operators",
            self.reader.original_position(),
        ))
    }

    pub fn read<'b>(&mut self) -> Result<Operator<'b>>
    where
        'a: 'b,
    {
        self.reader.read_operator()
    }

    pub fn into_iter_with_offsets<'b>(self) -> OperatorsIteratorWithOffsets<'b>
    where
        'a: 'b,
    {
        OperatorsIteratorWithOffsets {
            reader: self,
            err: false,
        }
    }

    pub fn read_with_offset<'b>(&mut self) -> Result<(Operator<'b>, usize)>
    where
        'a: 'b,
    {
        let pos = self.reader.original_position();
        Ok((self.read()?, pos))
    }

    pub fn get_binary_reader(&self) -> BinaryReader<'a> {
        self.reader.clone()
    }
}

impl<'a> IntoIterator for OperatorsReader<'a> {
    type Item = Result<Operator<'a>>;
    type IntoIter = OperatorsIterator<'a>;

    /// Reads content of the code section.
    ///
    /// # Examples
    /// ```
    /// use wasmparser::{Operator, CodeSectionReader, Result};
    /// # let data: &[u8] = &[
    /// #     0x01, 0x03, 0x00, 0x01, 0x0b];
    /// let mut code_reader = CodeSectionReader::new(data, 0).unwrap();
    /// for _ in 0..code_reader.get_count() {
    ///     let body = code_reader.read().expect("function body");
    ///     let mut op_reader = body.get_operators_reader().expect("op reader");
    ///     let ops = op_reader.into_iter().collect::<Result<Vec<Operator>>>().expect("ops");
    ///     assert!(
    ///         if let [Operator::Nop, Operator::End] = ops.as_slice() { true } else { false },
    ///         "found {:?}",
    ///         ops
    ///     );
    /// }
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        OperatorsIterator {
            reader: self,
            err: false,
        }
    }
}

pub struct OperatorsIterator<'a> {
    reader: OperatorsReader<'a>,
    err: bool,
}

impl<'a> Iterator for OperatorsIterator<'a> {
    type Item = Result<Operator<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.err || self.reader.eof() {
            return None;
        }
        let result = self.reader.read();
        self.err = result.is_err();
        Some(result)
    }
}

pub struct OperatorsIteratorWithOffsets<'a> {
    reader: OperatorsReader<'a>,
    err: bool,
}

impl<'a> Iterator for OperatorsIteratorWithOffsets<'a> {
    type Item = Result<(Operator<'a>, usize)>;

    /// Reads content of the code section with offsets.
    ///
    /// # Examples
    /// ```
    /// use wasmparser::{Operator, CodeSectionReader, Result};
    /// # let data: &[u8] = &[
    /// #     0x01, 0x03, 0x00, /* offset = 23 */ 0x01, 0x0b];
    /// let mut code_reader = CodeSectionReader::new(data, 20).unwrap();
    /// for _ in 0..code_reader.get_count() {
    ///     let body = code_reader.read().expect("function body");
    ///     let mut op_reader = body.get_operators_reader().expect("op reader");
    ///     let ops = op_reader.into_iter_with_offsets().collect::<Result<Vec<(Operator, usize)>>>().expect("ops");
    ///     assert!(
    ///         if let [(Operator::Nop, 23), (Operator::End, 24)] = ops.as_slice() { true } else { false },
    ///         "found {:?}",
    ///         ops
    ///     );
    /// }
    /// ```
    fn next(&mut self) -> Option<Self::Item> {
        if self.err || self.reader.eof() {
            return None;
        }
        let result = self.reader.read_with_offset();
        self.err = result.is_err();
        Some(result)
    }
}
