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
    BinaryReader, BinaryReaderError, Range, Result, SectionIteratorLimited, SectionReader,
    SectionWithLimitedItems, TypeDef,
};

#[derive(Clone)]
pub struct TypeSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

impl<'a> TypeSectionReader<'a> {
    pub fn new(data: &'a [u8], offset: usize) -> Result<TypeSectionReader<'a>> {
        let mut reader = BinaryReader::new_with_offset(data, offset);
        let count = reader.read_var_u32()?;
        Ok(TypeSectionReader { reader, count })
    }

    pub fn original_position(&self) -> usize {
        self.reader.original_position()
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    /// Reads content of the type section.
    ///
    /// # Examples
    /// ```
    /// use wasmparser::TypeSectionReader;
    /// # let data: &[u8] = &[0x01, 0x60, 0x00, 0x00];
    /// let mut type_reader = TypeSectionReader::new(data, 0).unwrap();
    /// for _ in 0..type_reader.get_count() {
    ///     let ty = type_reader.read().expect("type");
    ///     println!("Type {:?}", ty);
    /// }
    /// ```
    pub fn read(&mut self) -> Result<TypeDef<'a>> {
        Ok(match self.reader.read_u8()? {
            0x60 => TypeDef::Func(self.reader.read_func_type()?),
            0x61 => TypeDef::Module(self.reader.read_module_type()?),
            0x62 => TypeDef::Instance(self.reader.read_instance_type()?),
            _ => {
                return Err(BinaryReaderError::new(
                    "invalid leading byte in type definition",
                    self.original_position() - 1,
                ))
            }
        })
    }
}

impl<'a> SectionReader for TypeSectionReader<'a> {
    type Item = TypeDef<'a>;
    fn read(&mut self) -> Result<Self::Item> {
        TypeSectionReader::read(self)
    }
    fn eof(&self) -> bool {
        self.reader.eof()
    }
    fn original_position(&self) -> usize {
        TypeSectionReader::original_position(self)
    }
    fn range(&self) -> Range {
        self.reader.range()
    }
}

impl<'a> SectionWithLimitedItems for TypeSectionReader<'a> {
    fn get_count(&self) -> u32 {
        TypeSectionReader::get_count(self)
    }
}

impl<'a> IntoIterator for TypeSectionReader<'a> {
    type Item = Result<TypeDef<'a>>;
    type IntoIter = SectionIteratorLimited<TypeSectionReader<'a>>;

    /// Implements iterator over the type section.
    ///
    /// # Examples
    /// ```
    /// use wasmparser::TypeSectionReader;
    /// # let data: &[u8] = &[0x01, 0x60, 0x00, 0x00];
    /// let mut type_reader = TypeSectionReader::new(data, 0).unwrap();
    /// for ty in type_reader {
    ///     println!("Type {:?}", ty);
    /// }
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        SectionIteratorLimited::new(self)
    }
}
