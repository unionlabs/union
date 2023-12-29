use crate::{
    BinaryReader, BinaryReaderError, ExternalKind, Range, Result, SectionIteratorLimited,
    SectionReader, SectionWithLimitedItems,
};

#[derive(Clone)]
pub struct InstanceSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

impl<'a> InstanceSectionReader<'a> {
    pub fn new(data: &'a [u8], offset: usize) -> Result<InstanceSectionReader<'a>> {
        let mut reader = BinaryReader::new_with_offset(data, offset);
        let count = reader.read_var_u32()?;
        Ok(InstanceSectionReader { reader, count })
    }

    pub fn original_position(&self) -> usize {
        self.reader.original_position()
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn read(&mut self) -> Result<Instance<'a>> {
        let instance = Instance::new(
            &self.reader.buffer[self.reader.position..],
            self.original_position(),
        )?;

        // FIXME(#188): should probably figure out a different API for
        // wasmparser such that we don't have to read instances to skip them
        // here.
        let mut args = instance.args()?;
        for _ in 0..args.get_count() {
            args.read()?;
        }
        self.reader = args.reader;
        Ok(instance)
    }
}

impl<'a> SectionReader for InstanceSectionReader<'a> {
    type Item = Instance<'a>;

    fn read(&mut self) -> Result<Self::Item> {
        InstanceSectionReader::read(self)
    }
    fn eof(&self) -> bool {
        self.reader.eof()
    }
    fn original_position(&self) -> usize {
        InstanceSectionReader::original_position(self)
    }
    fn range(&self) -> Range {
        self.reader.range()
    }
}

impl<'a> SectionWithLimitedItems for InstanceSectionReader<'a> {
    fn get_count(&self) -> u32 {
        InstanceSectionReader::get_count(self)
    }
}

impl<'a> IntoIterator for InstanceSectionReader<'a> {
    type Item = Result<Instance<'a>>;
    type IntoIter = SectionIteratorLimited<InstanceSectionReader<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        SectionIteratorLimited::new(self)
    }
}

#[derive(Clone)]
pub struct Instance<'a> {
    reader: BinaryReader<'a>,
    module: u32,
}

impl<'a> Instance<'a> {
    pub fn new(data: &'a [u8], offset: usize) -> Result<Instance<'a>> {
        let mut reader = BinaryReader::new_with_offset(data, offset);
        if reader.read_u8()? != 0 {
            return Err(BinaryReaderError::new(
                "instantiate instruction not found",
                offset,
            ));
        }
        let module = reader.read_var_u32()?;
        Ok(Instance { module, reader })
    }

    pub fn original_position(&self) -> usize {
        self.reader.original_position()
    }

    pub fn module(&self) -> u32 {
        self.module
    }

    pub fn args(&self) -> Result<InstanceArgsReader<'a>> {
        let mut reader = self.reader.clone();
        let count = reader.read_var_u32()?;
        Ok(InstanceArgsReader {
            count,
            remaining: count,
            reader,
        })
    }
}

#[derive(Clone)]
pub struct InstanceArgsReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
    remaining: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct InstanceArg<'a> {
    pub name: &'a str,
    pub kind: ExternalKind,
    pub index: u32,
}

impl<'a> InstanceArgsReader<'a> {
    pub fn original_position(&self) -> usize {
        self.reader.original_position()
    }

    pub fn read(&mut self) -> Result<InstanceArg<'a>> {
        self.remaining -= 1;
        Ok(InstanceArg {
            name: self.reader.read_string()?,
            kind: self.reader.read_external_kind()?,
            index: self.reader.read_var_u32()?,
        })
    }
}

impl<'a> SectionReader for InstanceArgsReader<'a> {
    type Item = InstanceArg<'a>;

    fn read(&mut self) -> Result<Self::Item> {
        InstanceArgsReader::read(self)
    }
    fn eof(&self) -> bool {
        self.remaining == 0
    }
    fn original_position(&self) -> usize {
        InstanceArgsReader::original_position(self)
    }
    fn range(&self) -> Range {
        self.reader.range()
    }
}

impl<'a> SectionWithLimitedItems for InstanceArgsReader<'a> {
    fn get_count(&self) -> u32 {
        self.count
    }
}

impl<'a> IntoIterator for InstanceArgsReader<'a> {
    type Item = Result<InstanceArg<'a>>;
    type IntoIter = SectionIteratorLimited<InstanceArgsReader<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        SectionIteratorLimited::new(self)
    }
}
