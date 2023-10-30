use crate::{
    BinaryReader, BinaryReaderError, Range, Result, SectionIteratorLimited, SectionReader,
    SectionWithLimitedItems,
};

pub struct ModuleCodeSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

#[derive(Debug)]
pub struct ModuleCode<'a> {
    reader: BinaryReader<'a>,
}

impl<'a> ModuleCodeSectionReader<'a> {
    pub fn new(data: &'a [u8], offset: usize) -> Result<ModuleCodeSectionReader<'a>> {
        let mut reader = BinaryReader::new_with_offset(data, offset);
        let count = reader.read_var_u32()?;
        Ok(ModuleCodeSectionReader { reader, count })
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

    pub fn read(&mut self) -> Result<ModuleCode<'a>> {
        let size = self.reader.read_var_u32()? as usize;
        let module_start = self.reader.position;
        let module_end = module_start + size;
        self.verify_module_end(module_end)?;
        self.reader.skip_to(module_end);
        Ok(ModuleCode {
            reader: BinaryReader::new_with_offset(
                &self.reader.buffer[module_start..module_end],
                self.reader.original_offset + module_start,
            ),
        })
    }
}

impl<'a> SectionReader for ModuleCodeSectionReader<'a> {
    type Item = ModuleCode<'a>;

    fn read(&mut self) -> Result<Self::Item> {
        ModuleCodeSectionReader::read(self)
    }
    fn eof(&self) -> bool {
        self.reader.eof()
    }
    fn original_position(&self) -> usize {
        ModuleCodeSectionReader::original_position(self)
    }
    fn range(&self) -> Range {
        self.reader.range()
    }
}

impl<'a> SectionWithLimitedItems for ModuleCodeSectionReader<'a> {
    fn get_count(&self) -> u32 {
        ModuleCodeSectionReader::get_count(self)
    }
}

impl<'a> IntoIterator for ModuleCodeSectionReader<'a> {
    type Item = Result<ModuleCode<'a>>;
    type IntoIter = SectionIteratorLimited<ModuleCodeSectionReader<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        SectionIteratorLimited::new(self)
    }
}

impl<'a> ModuleCode<'a> {
    pub fn raw_bytes(&self) -> (usize, &'a [u8]) {
        (self.reader.original_position(), self.reader.buffer)
    }

    pub fn original_position(&self) -> usize {
        self.reader.original_position()
    }
}
