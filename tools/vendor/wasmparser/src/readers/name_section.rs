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
    BinaryReader, BinaryReaderError, NameType, Naming, Range, Result, SectionIterator,
    SectionReader,
};

#[derive(Debug, Copy, Clone)]
pub struct SingleName<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> SingleName<'a> {
    pub fn get_name<'b>(&self) -> Result<&'b str>
    where
        'a: 'b,
    {
        let mut reader = BinaryReader::new_with_offset(self.data, self.offset);
        reader.read_string()
    }

    pub fn original_position(&self) -> usize {
        self.offset
    }
}

pub struct NamingReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

impl<'a> NamingReader<'a> {
    fn new(data: &'a [u8], offset: usize) -> Result<NamingReader<'a>> {
        let mut reader = BinaryReader::new_with_offset(data, offset);
        let count = reader.read_var_u32()?;
        Ok(NamingReader { reader, count })
    }

    fn skip(reader: &mut BinaryReader) -> Result<()> {
        let count = reader.read_var_u32()?;
        for _ in 0..count {
            reader.skip_var_32()?;
            reader.skip_string()?;
        }
        Ok(())
    }

    pub fn original_position(&self) -> usize {
        self.reader.original_position()
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn read<'b>(&mut self) -> Result<Naming<'b>>
    where
        'a: 'b,
    {
        let index = self.reader.read_var_u32()?;
        let name = self.reader.read_string()?;
        Ok(Naming { index, name })
    }
}

#[derive(Debug, Copy, Clone)]
pub struct NameMap<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> NameMap<'a> {
    pub fn get_map<'b>(&self) -> Result<NamingReader<'b>>
    where
        'a: 'b,
    {
        NamingReader::new(self.data, self.offset)
    }

    pub fn original_position(&self) -> usize {
        self.offset
    }
}

#[derive(Debug, Copy, Clone)]
pub struct IndirectNaming<'a> {
    pub indirect_index: u32,
    data: &'a [u8],
    offset: usize,
}

impl<'a> IndirectNaming<'a> {
    pub fn get_map<'b>(&self) -> Result<NamingReader<'b>>
    where
        'a: 'b,
    {
        NamingReader::new(self.data, self.offset)
    }

    pub fn original_position(&self) -> usize {
        self.offset
    }
}

pub struct IndirectNamingReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

impl<'a> IndirectNamingReader<'a> {
    fn new(data: &'a [u8], offset: usize) -> Result<IndirectNamingReader<'a>> {
        let mut reader = BinaryReader::new_with_offset(data, offset);
        let count = reader.read_var_u32()?;
        Ok(IndirectNamingReader { reader, count })
    }

    pub fn get_indirect_count(&self) -> u32 {
        self.count
    }

    pub fn original_position(&self) -> usize {
        self.reader.original_position()
    }

    pub fn read<'b>(&mut self) -> Result<IndirectNaming<'b>>
    where
        'a: 'b,
    {
        let index = self.reader.read_var_u32()?;
        let start = self.reader.position;
        NamingReader::skip(&mut self.reader)?;
        let end = self.reader.position;
        Ok(IndirectNaming {
            indirect_index: index,
            data: &self.reader.buffer[start..end],
            offset: self.reader.original_offset + start,
        })
    }
}

#[derive(Debug, Copy, Clone)]
pub struct IndirectNameMap<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> IndirectNameMap<'a> {
    pub fn get_indirect_map<'b>(&self) -> Result<IndirectNamingReader<'b>>
    where
        'a: 'b,
    {
        IndirectNamingReader::new(self.data, self.offset)
    }

    pub fn original_position(&self) -> usize {
        self.offset
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Name<'a> {
    Module(SingleName<'a>),
    Function(NameMap<'a>),
    Local(IndirectNameMap<'a>),
    Label(IndirectNameMap<'a>),
    Type(NameMap<'a>),
    Table(NameMap<'a>),
    Memory(NameMap<'a>),
    Global(NameMap<'a>),
    Element(NameMap<'a>),
    Data(NameMap<'a>),
    /// An unknown [name subsection](https://webassembly.github.io/spec/core/appendix/custom.html#subsections).
    Unknown {
        /// The identifier for this subsection.
        ty: u32,
        /// The contents of this subsection.
        data: &'a [u8],
        /// The range of bytes, relative to the start of the original data
        /// stream, that the contents of this subsection reside in.
        range: Range,
    },
}

pub struct NameSectionReader<'a> {
    reader: BinaryReader<'a>,
}

impl<'a> NameSectionReader<'a> {
    pub fn new(data: &'a [u8], offset: usize) -> Result<NameSectionReader<'a>> {
        Ok(NameSectionReader {
            reader: BinaryReader::new_with_offset(data, offset),
        })
    }

    fn verify_section_end(&self, end: usize) -> Result<()> {
        if self.reader.buffer.len() < end {
            return Err(BinaryReaderError::new(
                "Name entry extends past end of the code section",
                self.reader.original_offset + self.reader.buffer.len(),
            ));
        }
        Ok(())
    }

    pub fn eof(&self) -> bool {
        self.reader.eof()
    }

    pub fn original_position(&self) -> usize {
        self.reader.original_position()
    }

    pub fn read<'b>(&mut self) -> Result<Name<'b>>
    where
        'a: 'b,
    {
        let ty = self.reader.read_name_type()?;
        let payload_len = self.reader.read_var_u32()? as usize;
        let payload_start = self.reader.position;
        let payload_end = payload_start + payload_len;
        self.verify_section_end(payload_end)?;
        let offset = self.reader.original_offset + payload_start;
        let data = &self.reader.buffer[payload_start..payload_end];
        self.reader.skip_to(payload_end);
        Ok(match ty {
            NameType::Module => Name::Module(SingleName { data, offset }),
            NameType::Function => Name::Function(NameMap { data, offset }),
            NameType::Local => Name::Local(IndirectNameMap { data, offset }),
            NameType::Label => Name::Label(IndirectNameMap { data, offset }),
            NameType::Type => Name::Type(NameMap { data, offset }),
            NameType::Table => Name::Table(NameMap { data, offset }),
            NameType::Memory => Name::Memory(NameMap { data, offset }),
            NameType::Global => Name::Global(NameMap { data, offset }),
            NameType::Element => Name::Element(NameMap { data, offset }),
            NameType::Data => Name::Data(NameMap { data, offset }),
            NameType::Unknown(ty) => Name::Unknown {
                ty,
                data,
                range: Range::new(offset, offset + payload_len),
            },
        })
    }
}

impl<'a> SectionReader for NameSectionReader<'a> {
    type Item = Name<'a>;
    fn read(&mut self) -> Result<Self::Item> {
        NameSectionReader::read(self)
    }
    fn eof(&self) -> bool {
        NameSectionReader::eof(self)
    }
    fn original_position(&self) -> usize {
        NameSectionReader::original_position(self)
    }
    fn range(&self) -> Range {
        self.reader.range()
    }
}

impl<'a> IntoIterator for NameSectionReader<'a> {
    type Item = Result<Name<'a>>;
    type IntoIter = SectionIterator<NameSectionReader<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        SectionIterator::new(self)
    }
}
