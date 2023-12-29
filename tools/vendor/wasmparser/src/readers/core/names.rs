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
    BinaryReader, BinaryReaderError, Result, SectionIterator, SectionIteratorLimited,
    SectionReader, SectionWithLimitedItems,
};
use std::ops::Range;

/// Represents a name for an index from the names section.
#[derive(Debug, Copy, Clone)]
pub struct Naming<'a> {
    /// The index being named.
    pub index: u32,
    /// The name for the index.
    pub name: &'a str,
}

/// Represents a name map from the names custom section.
#[derive(Debug, Clone)]
pub struct NameMap<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

impl<'a> NameMap<'a> {
    /// Creates a new "name map" which parses the input data.
    ///
    /// This is intended to parse the `namemap` production in the name section
    /// appending of the wasm spec.
    pub fn new(data: &'a [u8], offset: usize) -> Result<NameMap<'a>> {
        let mut reader = BinaryReader::new_with_offset(data, offset);
        let count = reader.read_var_u32()?;
        Ok(NameMap { reader, count })
    }
}

impl<'a> SectionReader for NameMap<'a> {
    type Item = Naming<'a>;

    fn read(&mut self) -> Result<Naming<'a>> {
        let index = self.reader.read_var_u32()?;
        let name = self.reader.read_string()?;
        Ok(Naming { index, name })
    }

    fn eof(&self) -> bool {
        self.reader.eof()
    }

    fn original_position(&self) -> usize {
        self.reader.original_position()
    }

    fn range(&self) -> Range<usize> {
        self.reader.range()
    }
}

impl SectionWithLimitedItems for NameMap<'_> {
    fn get_count(&self) -> u32 {
        self.count
    }
}

impl<'a> IntoIterator for NameMap<'a> {
    type Item = Result<Naming<'a>>;
    type IntoIter = SectionIteratorLimited<Self>;

    fn into_iter(self) -> Self::IntoIter {
        SectionIteratorLimited::new(self)
    }
}

/// Represents an indirect name in the names custom section.
#[derive(Debug, Clone)]
pub struct IndirectNaming<'a> {
    /// The indirect index of the name.
    pub index: u32,
    /// The map of names within the `index` prior.
    pub names: NameMap<'a>,
}

/// Represents a reader for indirect names from the names custom section.
#[derive(Clone)]
pub struct IndirectNameMap<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

impl<'a> IndirectNameMap<'a> {
    fn new(data: &'a [u8], offset: usize) -> Result<IndirectNameMap<'a>> {
        let mut reader = BinaryReader::new_with_offset(data, offset);
        let count = reader.read_var_u32()?;
        Ok(IndirectNameMap { reader, count })
    }
}

impl<'a> SectionReader for IndirectNameMap<'a> {
    type Item = IndirectNaming<'a>;

    fn read(&mut self) -> Result<IndirectNaming<'a>> {
        let index = self.reader.read_var_u32()?;
        let start = self.reader.position;

        // Skip the `NameMap` manually here.
        //
        // FIXME(#188) shouldn't need to skip here
        let count = self.reader.read_var_u32()?;
        for _ in 0..count {
            self.reader.read_var_u32()?;
            self.reader.skip_string()?;
        }

        let end = self.reader.position;
        Ok(IndirectNaming {
            index,
            names: NameMap::new(
                &self.reader.buffer[start..end],
                self.reader.original_offset + start,
            )?,
        })
    }

    fn eof(&self) -> bool {
        self.reader.eof()
    }

    fn original_position(&self) -> usize {
        self.reader.original_position()
    }

    fn range(&self) -> Range<usize> {
        self.reader.range()
    }
}

impl SectionWithLimitedItems for IndirectNameMap<'_> {
    fn get_count(&self) -> u32 {
        self.count
    }
}

impl<'a> IntoIterator for IndirectNameMap<'a> {
    type Item = Result<IndirectNaming<'a>>;
    type IntoIter = SectionIteratorLimited<Self>;

    fn into_iter(self) -> Self::IntoIter {
        SectionIteratorLimited::new(self)
    }
}

/// Represents a name read from the names custom section.
#[derive(Clone)]
pub enum Name<'a> {
    /// The name is for the module.
    Module {
        /// The specified name.
        name: &'a str,
        /// The byte range that `name` occupies in the original binary.
        name_range: Range<usize>,
    },
    /// The name is for the functions.
    Function(NameMap<'a>),
    /// The name is for the function locals.
    Local(IndirectNameMap<'a>),
    /// The name is for the function labels.
    Label(IndirectNameMap<'a>),
    /// The name is for the types.
    Type(NameMap<'a>),
    /// The name is for the tables.
    Table(NameMap<'a>),
    /// The name is for the memories.
    Memory(NameMap<'a>),
    /// The name is for the globals.
    Global(NameMap<'a>),
    /// The name is for the element segments.
    Element(NameMap<'a>),
    /// The name is for the data segments.
    Data(NameMap<'a>),
    /// An unknown [name subsection](https://webassembly.github.io/spec/core/appendix/custom.html#subsections).
    Unknown {
        /// The identifier for this subsection.
        ty: u8,
        /// The contents of this subsection.
        data: &'a [u8],
        /// The range of bytes, relative to the start of the original data
        /// stream, that the contents of this subsection reside in.
        range: Range<usize>,
    },
}

/// A reader for the name custom section of a WebAssembly module.
pub struct NameSectionReader<'a> {
    reader: BinaryReader<'a>,
}

impl<'a> NameSectionReader<'a> {
    /// Constructs a new `NameSectionReader` from the given data and offset.
    pub fn new(data: &'a [u8], offset: usize) -> Result<NameSectionReader<'a>> {
        Ok(NameSectionReader {
            reader: BinaryReader::new_with_offset(data, offset),
        })
    }

    fn verify_section_end(&self, end: usize) -> Result<()> {
        if self.reader.buffer.len() < end {
            return Err(BinaryReaderError::new(
                "name entry extends past end of the code section",
                self.reader.original_offset + self.reader.buffer.len(),
            ));
        }
        Ok(())
    }

    /// Determines if the reader is at the end of the section.
    pub fn eof(&self) -> bool {
        self.reader.eof()
    }

    /// Gets the original position of the section reader.
    pub fn original_position(&self) -> usize {
        self.reader.original_position()
    }

    /// Reads a name from the section.
    pub fn read<'b>(&mut self) -> Result<Name<'b>>
    where
        'a: 'b,
    {
        let subsection_id = self.reader.read_u7()?;
        let payload_len = self.reader.read_var_u32()? as usize;
        let payload_start = self.reader.position;
        let payload_end = payload_start + payload_len;
        self.verify_section_end(payload_end)?;
        let offset = self.reader.original_offset + payload_start;
        let data = &self.reader.buffer[payload_start..payload_end];
        self.reader.skip_to(payload_end);

        Ok(match subsection_id {
            0 => {
                let mut reader = BinaryReader::new_with_offset(data, offset);
                let name = reader.read_string()?;
                if !reader.eof() {
                    return Err(BinaryReaderError::new(
                        "trailing data at the end of a name",
                        reader.original_position(),
                    ));
                }
                Name::Module {
                    name,
                    name_range: offset..offset + reader.position,
                }
            }
            1 => Name::Function(NameMap::new(data, offset)?),
            2 => Name::Local(IndirectNameMap::new(data, offset)?),
            3 => Name::Label(IndirectNameMap::new(data, offset)?),
            4 => Name::Type(NameMap::new(data, offset)?),
            5 => Name::Table(NameMap::new(data, offset)?),
            6 => Name::Memory(NameMap::new(data, offset)?),
            7 => Name::Global(NameMap::new(data, offset)?),
            8 => Name::Element(NameMap::new(data, offset)?),
            9 => Name::Data(NameMap::new(data, offset)?),
            ty => Name::Unknown {
                ty,
                data,
                range: offset..offset + payload_len,
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
    fn range(&self) -> Range<usize> {
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
