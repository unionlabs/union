use crate::{
    BinaryReader, BinaryReaderError, ExternalKind, Range, Result, SectionIteratorLimited,
    SectionReader, SectionWithLimitedItems,
};

#[derive(Clone)]
pub struct AliasSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

#[derive(Clone, Debug)]
pub enum Alias<'a> {
    OuterType {
        relative_depth: u32,
        index: u32,
    },
    OuterModule {
        relative_depth: u32,
        index: u32,
    },
    InstanceExport {
        instance: u32,
        kind: ExternalKind,
        export: &'a str,
    },
}

impl<'a> AliasSectionReader<'a> {
    pub fn new(data: &'a [u8], offset: usize) -> Result<AliasSectionReader<'a>> {
        let mut reader = BinaryReader::new_with_offset(data, offset);
        let count = reader.read_var_u32()?;
        Ok(AliasSectionReader { reader, count })
    }

    pub fn original_position(&self) -> usize {
        self.reader.original_position()
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn read(&mut self) -> Result<Alias<'a>> {
        Ok(match self.reader.read_u8()? {
            0x00 => Alias::InstanceExport {
                instance: self.reader.read_var_u32()?,
                kind: self.reader.read_external_kind()?,
                export: self.reader.read_string()?,
            },
            0x01 => {
                let relative_depth = self.reader.read_var_u32()?;
                match self.reader.read_external_kind()? {
                    ExternalKind::Type => Alias::OuterType {
                        relative_depth,
                        index: self.reader.read_var_u32()?,
                    },
                    ExternalKind::Module => Alias::OuterModule {
                        relative_depth,
                        index: self.reader.read_var_u32()?,
                    },
                    _ => {
                        return Err(BinaryReaderError::new(
                            "invalid external kind in alias",
                            self.original_position() - 1,
                        ))
                    }
                }
            }
            _ => {
                return Err(BinaryReaderError::new(
                    "invalid byte in alias",
                    self.original_position() - 1,
                ))
            }
        })
    }
}

impl<'a> SectionReader for AliasSectionReader<'a> {
    type Item = Alias<'a>;

    fn read(&mut self) -> Result<Self::Item> {
        AliasSectionReader::read(self)
    }
    fn eof(&self) -> bool {
        self.reader.eof()
    }
    fn original_position(&self) -> usize {
        AliasSectionReader::original_position(self)
    }
    fn range(&self) -> Range {
        self.reader.range()
    }
}

impl<'a> SectionWithLimitedItems for AliasSectionReader<'a> {
    fn get_count(&self) -> u32 {
        AliasSectionReader::get_count(self)
    }
}

impl<'a> IntoIterator for AliasSectionReader<'a> {
    type Item = Result<Alias<'a>>;
    type IntoIter = SectionIteratorLimited<AliasSectionReader<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        SectionIteratorLimited::new(self)
    }
}
