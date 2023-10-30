/* Copyright 2019 Mozilla Foundation
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
    BinaryReader, Range, Result, SectionIteratorLimited, SectionReader, SectionWithLimitedItems,
};

#[derive(Debug, Copy, Clone)]
pub struct ProducersFieldValue<'a> {
    pub name: &'a str,
    pub version: &'a str,
}

pub struct ProducersFieldValuesReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

impl<'a> ProducersFieldValuesReader<'a> {
    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn original_position(&self) -> usize {
        self.reader.original_position()
    }

    fn skip(reader: &mut BinaryReader, values_count: u32) -> Result<()> {
        for _ in 0..values_count {
            reader.skip_string()?;
            reader.skip_string()?;
        }
        Ok(())
    }

    pub fn read<'b>(&mut self) -> Result<ProducersFieldValue<'b>>
    where
        'a: 'b,
    {
        let name = self.reader.read_string()?;
        let version = self.reader.read_string()?;
        Ok(ProducersFieldValue { name, version })
    }
}

impl<'a> IntoIterator for ProducersFieldValuesReader<'a> {
    type Item = Result<ProducersFieldValue<'a>>;
    type IntoIter = ProducersFieldValuesIterator<'a>;
    fn into_iter(self) -> Self::IntoIter {
        let count = self.count;
        ProducersFieldValuesIterator {
            reader: self,
            left: count,
            err: false,
        }
    }
}

pub struct ProducersFieldValuesIterator<'a> {
    reader: ProducersFieldValuesReader<'a>,
    left: u32,
    err: bool,
}

impl<'a> Iterator for ProducersFieldValuesIterator<'a> {
    type Item = Result<ProducersFieldValue<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.err || self.left == 0 {
            return None;
        }
        let result = self.reader.read();
        self.err = result.is_err();
        self.left -= 1;
        Some(result)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.reader.get_count() as usize;
        (count, Some(count))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ProducersField<'a> {
    pub name: &'a str,
    values_count: u32,
    values_data: &'a [u8],
    values_offset: usize,
}

impl<'a> ProducersField<'a> {
    pub fn get_producer_field_values_reader<'b>(&self) -> Result<ProducersFieldValuesReader<'b>>
    where
        'a: 'b,
    {
        Ok(ProducersFieldValuesReader {
            reader: BinaryReader::new_with_offset(self.values_data, self.values_offset),
            count: self.values_count,
        })
    }
}

pub struct ProducersSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

impl<'a> ProducersSectionReader<'a> {
    /// Creates reader for the producers section.
    ///
    /// # Examples
    /// ```
    /// # let data: &[u8] = &[0x01, 0x08, 0x6c, 0x61, 0x6e, 0x67, 0x75, 0x61, 0x67, 0x65,
    /// #     0x02, 0x03, 0x77, 0x61, 0x74, 0x01, 0x31, 0x01, 0x43, 0x03, 0x39, 0x2e, 0x30];
    /// use wasmparser::{ProducersSectionReader, ProducersFieldValue, Result};
    /// let mut reader = ProducersSectionReader::new(data, 0).expect("producers reader");
    /// let field = reader.read().expect("producers field");
    /// assert!(field.name == "language");
    /// let mut values_reader = field.get_producer_field_values_reader().expect("values reader");
    /// let value = values_reader.into_iter().collect::<Result<Vec<ProducersFieldValue>>>().expect("values");
    /// assert!(value.len() == 2);
    /// assert!(value[0].name == "wat" && value[0].version == "1");
    /// assert!(value[1].name == "C" && value[1].version == "9.0");
    /// ```
    pub fn new(data: &'a [u8], offset: usize) -> Result<ProducersSectionReader<'a>> {
        let mut reader = BinaryReader::new_with_offset(data, offset);
        let count = reader.read_var_u32()?;
        Ok(ProducersSectionReader { reader, count })
    }

    pub fn original_position(&self) -> usize {
        self.reader.original_position()
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn read<'b>(&mut self) -> Result<ProducersField<'b>>
    where
        'a: 'b,
    {
        let name = self.reader.read_string()?;
        let values_count = self.reader.read_var_u32()?;
        let values_start = self.reader.position;
        ProducersFieldValuesReader::skip(&mut self.reader, values_count)?;
        let values_end = self.reader.position;
        Ok(ProducersField {
            name,
            values_count,
            values_data: &self.reader.buffer[values_start..values_end],
            values_offset: self.reader.original_offset + values_start,
        })
    }
}

impl<'a> SectionReader for ProducersSectionReader<'a> {
    type Item = ProducersField<'a>;
    fn read(&mut self) -> Result<Self::Item> {
        ProducersSectionReader::read(self)
    }
    fn eof(&self) -> bool {
        self.reader.eof()
    }
    fn original_position(&self) -> usize {
        ProducersSectionReader::original_position(self)
    }
    fn range(&self) -> Range {
        self.reader.range()
    }
}

impl<'a> SectionWithLimitedItems for ProducersSectionReader<'a> {
    fn get_count(&self) -> u32 {
        ProducersSectionReader::get_count(self)
    }
}

impl<'a> IntoIterator for ProducersSectionReader<'a> {
    type Item = Result<ProducersField<'a>>;
    type IntoIter = SectionIteratorLimited<ProducersSectionReader<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        SectionIteratorLimited::new(self)
    }
}
