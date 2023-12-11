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
    BinaryReader, Range, RelocType, Result, SectionCode, SectionIteratorLimited, SectionReader,
    SectionWithLimitedItems,
};

#[derive(Debug, Copy, Clone)]
pub struct Reloc {
    pub ty: RelocType,
    pub offset: u32,
    pub index: u32,
    pub addend: Option<u32>,
}

pub struct RelocSectionReader<'a> {
    reader: BinaryReader<'a>,
    section_code: SectionCode<'a>,
    count: u32,
}

impl<'a> RelocSectionReader<'a> {
    pub fn new(data: &'a [u8], offset: usize) -> Result<RelocSectionReader<'a>> {
        let mut reader = BinaryReader::new_with_offset(data, offset);

        let section_id_position = reader.position;
        let section_id = reader.read_var_u7()?;
        let section_code = reader.read_section_code(section_id, section_id_position)?;

        let count = reader.read_var_u32()?;
        Ok(RelocSectionReader {
            reader,
            section_code,
            count,
        })
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn get_section_code<'b>(&self) -> SectionCode<'b>
    where
        'a: 'b,
    {
        self.section_code
    }

    pub fn original_position(&self) -> usize {
        self.reader.original_position()
    }

    pub fn read(&mut self) -> Result<Reloc> {
        let ty = self.reader.read_reloc_type()?;
        let offset = self.reader.read_var_u32()?;
        let index = self.reader.read_var_u32()?;
        let addend = match ty {
            RelocType::FunctionIndexLEB
            | RelocType::TableIndexSLEB
            | RelocType::TableIndexI32
            | RelocType::TypeIndexLEB
            | RelocType::GlobalIndexLEB => None,
            RelocType::GlobalAddrLEB | RelocType::GlobalAddrSLEB | RelocType::GlobalAddrI32 => {
                Some(self.reader.read_var_u32()?)
            }
        };
        Ok(Reloc {
            ty,
            offset,
            index,
            addend,
        })
    }
}

impl<'a> SectionReader for RelocSectionReader<'a> {
    type Item = Reloc;
    fn read(&mut self) -> Result<Self::Item> {
        RelocSectionReader::read(self)
    }
    fn eof(&self) -> bool {
        self.reader.eof()
    }
    fn original_position(&self) -> usize {
        RelocSectionReader::original_position(self)
    }
    fn range(&self) -> Range {
        self.reader.range()
    }
}

impl<'a> SectionWithLimitedItems for RelocSectionReader<'a> {
    fn get_count(&self) -> u32 {
        RelocSectionReader::get_count(self)
    }
}

impl<'a> IntoIterator for RelocSectionReader<'a> {
    type Item = Result<Reloc>;
    type IntoIter = SectionIteratorLimited<RelocSectionReader<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        SectionIteratorLimited::new(self)
    }
}
