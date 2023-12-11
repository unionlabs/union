use crate::{BinaryReader, BinaryReaderError, NameMap, Result, SectionIterator, SectionReader};
use std::ops::Range;

/// Represents a name read from the names custom section.
#[derive(Clone)]
#[allow(missing_docs)]
pub enum ComponentName<'a> {
    Component {
        name: &'a str,
        name_range: Range<usize>,
    },
    CoreFuncs(NameMap<'a>),
    CoreGlobals(NameMap<'a>),
    CoreMemories(NameMap<'a>),
    CoreTables(NameMap<'a>),
    CoreModules(NameMap<'a>),
    CoreInstances(NameMap<'a>),
    CoreTypes(NameMap<'a>),
    Types(NameMap<'a>),
    Instances(NameMap<'a>),
    Components(NameMap<'a>),
    Funcs(NameMap<'a>),
    Values(NameMap<'a>),

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
pub struct ComponentNameSectionReader<'a> {
    reader: BinaryReader<'a>,
}

impl<'a> ComponentNameSectionReader<'a> {
    /// Constructs a new `ComponentNameSectionReader` from the given data and offset.
    pub fn new(data: &'a [u8], offset: usize) -> Result<ComponentNameSectionReader<'a>> {
        Ok(ComponentNameSectionReader {
            reader: BinaryReader::new_with_offset(data, offset),
        })
    }

    fn verify_section_end(&self, end: usize) -> Result<()> {
        if self.reader.buffer.len() < end {
            return Err(BinaryReaderError::new(
                "component name entry extends past end of the code section",
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
    pub fn read(&mut self) -> Result<ComponentName<'a>> {
        let subsection_id = self.reader.read_u7()?;
        let payload_len = self.reader.read_var_u32()? as usize;
        let payload_start = self.reader.position;
        let payload_end = payload_start + payload_len;
        self.verify_section_end(payload_end)?;
        let offset = self.reader.original_offset + payload_start;
        let data = &self.reader.buffer[payload_start..payload_end];
        self.reader.skip_to(payload_end);
        let mut reader = BinaryReader::new_with_offset(data, offset);

        Ok(match subsection_id {
            0 => {
                let name = reader.read_string()?;
                if !reader.eof() {
                    return Err(BinaryReaderError::new(
                        "trailing data at the end of a name",
                        reader.original_position(),
                    ));
                }
                ComponentName::Component {
                    name,
                    name_range: offset..offset + reader.position,
                }
            }
            1 => {
                let ctor: fn(NameMap<'a>) -> ComponentName<'a> = match reader.read_u8()? {
                    0x00 => match reader.read_u8()? {
                        0x00 => ComponentName::CoreFuncs,
                        0x01 => ComponentName::CoreTables,
                        0x02 => ComponentName::CoreMemories,
                        0x03 => ComponentName::CoreGlobals,
                        0x10 => ComponentName::CoreTypes,
                        0x11 => ComponentName::CoreModules,
                        0x12 => ComponentName::CoreInstances,
                        _ => {
                            return Ok(ComponentName::Unknown {
                                ty: 1,
                                data,
                                range: offset..offset + payload_len,
                            });
                        }
                    },
                    0x01 => ComponentName::Funcs,
                    0x02 => ComponentName::Values,
                    0x03 => ComponentName::Types,
                    0x04 => ComponentName::Components,
                    0x05 => ComponentName::Instances,
                    _ => {
                        return Ok(ComponentName::Unknown {
                            ty: 1,
                            data,
                            range: offset..offset + payload_len,
                        });
                    }
                };
                ctor(NameMap::new(
                    reader.remaining_buffer(),
                    reader.original_position(),
                )?)
            }
            ty => ComponentName::Unknown {
                ty,
                data,
                range: offset..offset + payload_len,
            },
        })
    }
}

impl<'a> SectionReader for ComponentNameSectionReader<'a> {
    type Item = ComponentName<'a>;
    fn read(&mut self) -> Result<Self::Item> {
        ComponentNameSectionReader::read(self)
    }
    fn eof(&self) -> bool {
        ComponentNameSectionReader::eof(self)
    }
    fn original_position(&self) -> usize {
        ComponentNameSectionReader::original_position(self)
    }
    fn range(&self) -> Range<usize> {
        self.reader.range()
    }
}

impl<'a> IntoIterator for ComponentNameSectionReader<'a> {
    type Item = Result<ComponentName<'a>>;
    type IntoIter = SectionIterator<ComponentNameSectionReader<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        SectionIterator::new(self)
    }
}
