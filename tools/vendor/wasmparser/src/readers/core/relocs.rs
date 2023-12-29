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

use crate::{BinaryReader, Result, SectionIteratorLimited, SectionReader, SectionWithLimitedItems};
use std::ops::Range;

/// Represents a relocation type.
#[derive(Debug, Copy, Clone)]
#[allow(missing_docs)]
pub enum RelocType {
    FunctionIndexLEB,
    TableIndexSLEB,
    TableIndexI32,
    GlobalAddrLEB,
    GlobalAddrSLEB,
    GlobalAddrI32,
    TypeIndexLEB,
    GlobalIndexLEB,
}

/// Represents known custom section kinds.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CustomSectionKind {
    /// The custom section is not known.
    Unknown,
    /// The name custom section.
    Name,
    /// The producers custom section.
    Producers,
    /// The source mapping URL custom section.
    SourceMappingURL,
    /// The reloc custom section.
    Reloc,
    /// The linking custom section.
    Linking,
}

/// Section code as defined [here].
///
/// [here]: https://webassembly.github.io/spec/core/binary/modules.html#sections
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SectionCode<'a> {
    /// The custom section.
    Custom {
        /// The name of the custom section.
        name: &'a str,
        /// The kind of the custom section.
        kind: CustomSectionKind,
    },
    /// The type section.
    Type,
    /// The import section.
    Import,
    /// The function section.
    Function,
    /// The table section.
    Table,
    /// The memory section.
    Memory,
    /// The global section.
    Global,
    /// The export section.
    Export,
    /// The start section.
    Start,
    /// The element section.
    Element,
    /// The code section.
    Code,
    /// The data section.
    Data,
    /// The passive data count section.
    DataCount,
    /// The tag section.
    Tag,
}

/// Represents a relocation entry.
#[derive(Debug, Copy, Clone)]
pub struct Reloc {
    /// The relocation type.
    pub ty: RelocType,
    /// The relocation offset.
    pub offset: u32,
    /// The relocation index.
    pub index: u32,
    /// The relocation addend.
    pub addend: Option<u32>,
}

/// A reader for the relocations custom section of a WebAssembly module.
pub struct RelocSectionReader<'a> {
    reader: BinaryReader<'a>,
    section_code: SectionCode<'a>,
    count: u32,
}

impl<'a> RelocSectionReader<'a> {
    /// Constructs a new `RelocSectionReader` for the given data and offset.
    pub fn new(data: &'a [u8], offset: usize) -> Result<RelocSectionReader<'a>> {
        let mut reader = BinaryReader::new_with_offset(data, offset);

        let section_id_position = reader.position;
        let section_id = reader.read_u7()?;
        let section_code = reader.read_section_code(section_id, section_id_position)?;

        let count = reader.read_var_u32()?;
        Ok(RelocSectionReader {
            reader,
            section_code,
            count,
        })
    }

    /// Gets a count of items in the section.
    pub fn get_count(&self) -> u32 {
        self.count
    }

    /// Gets the section code from the section.
    pub fn get_section_code<'b>(&self) -> SectionCode<'b>
    where
        'a: 'b,
    {
        self.section_code
    }

    /// Gets the original position of the reader.
    pub fn original_position(&self) -> usize {
        self.reader.original_position()
    }

    /// Reads an item from the reader.
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
    fn range(&self) -> Range<usize> {
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
