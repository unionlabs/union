use crate::{
    BinaryReader, BinaryReaderError, Range, Result, SectionIteratorLimited, SectionReader,
    SectionWithLimitedItems,
};

pub struct ModuleSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

#[derive(Debug)]
pub struct NestedModule<'a> {
    reader: BinaryReader<'a>,
}

impl<'a> ModuleSectionReader<'a> {
    pub fn new(data: &'a [u8], offset: usize) -> Result<ModuleSectionReader<'a>> {
        let mut reader = BinaryReader::new_with_offset(data, offset);
        let count = reader.read_var_u32()?;
        Ok(ModuleSectionReader { reader, count })
    }

    pub fn original_position(&self) -> usize {
        self.reader.original_position()
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    fn verify_module_end(&self, end: usize) -> Result<()> {
        if self.reader.buffer.len() < end {
            return Err(BinaryReaderError::new(
                "module body extends past end of the module code section",
                self.reader.original_offset + self.reader.buffer.len(),
            ));
        }
        Ok(())
    }

    pub fn read(&mut self) -> Result<NestedModule<'a>> {
        let size = self.reader.read_var_u32()? as usize;
        let module_start = self.reader.position;
        let module_end = module_start + size;
        self.verify_module_end(module_end)?;
        self.reader.skip_to(module_end);
        Ok(NestedModule {
            reader: BinaryReader::new_with_offset(
                &self.reader.buffer[module_start..module_end],
                self.reader.original_offset + module_start,
            ),
        })
    }
}

impl<'a> SectionReader for ModuleSectionReader<'a> {
    type Item = NestedModule<'a>;

    fn read(&mut self) -> Result<Self::Item> {
        ModuleSectionReader::read(self)
    }
    fn eof(&self) -> bool {
        self.reader.eof()
    }
    fn original_position(&self) -> usize {
        ModuleSectionReader::original_position(self)
    }
    fn range(&self) -> Range {
        self.reader.range()
    }
}

impl<'a> SectionWithLimitedItems for ModuleSectionReader<'a> {
    fn get_count(&self) -> u32 {
        ModuleSectionReader::get_count(self)
    }
}

impl<'a> IntoIterator for ModuleSectionReader<'a> {
    type Item = Result<NestedModule<'a>>;
    type IntoIter = SectionIteratorLimited<ModuleSectionReader<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        SectionIteratorLimited::new(self)
    }
}

impl<'a> NestedModule<'a> {
    pub fn raw_bytes(&self) -> (usize, &'a [u8]) {
        (self.reader.original_position(), self.reader.buffer)
    }

    pub fn original_position(&self) -> usize {
        self.reader.original_position()
    }
}
