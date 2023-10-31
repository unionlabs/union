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
    BinaryReader, BinaryReaderError, InitExpr, Range, Result, SectionIteratorLimited,
    SectionReader, SectionWithLimitedItems, Type,
};
use crate::ExternalKind;

#[derive(Clone)]
pub struct Element<'a> {
    pub kind: ElementKind<'a>,
    pub items: ElementItems<'a>,
    pub ty: Type,
    pub range: Range,
}

#[derive(Clone)]
pub enum ElementKind<'a> {
    Passive,
    Active {
        table_index: u32,
        init_expr: InitExpr<'a>,
    },
    Declared,
}

#[derive(Debug, Copy, Clone)]
pub struct ElementItems<'a> {
    exprs: bool,
    offset: usize,
    data: &'a [u8],
}

#[derive(Debug)]
pub enum ElementItem<'a> {
    Func(u32),
    Expr(InitExpr<'a>),
}

impl<'a> ElementItems<'a> {
    pub fn get_items_reader<'b>(&self) -> Result<ElementItemsReader<'b>>
    where
        'a: 'b,
    {
        ElementItemsReader::new(self.data, self.offset, self.exprs)
    }
}

pub struct ElementItemsReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
    exprs: bool,
}

impl<'a> ElementItemsReader<'a> {
    pub fn new(data: &[u8], offset: usize, exprs: bool) -> Result<ElementItemsReader> {
        let mut reader = BinaryReader::new_with_offset(data, offset);
        let count = reader.read_var_u32()?;
        Ok(ElementItemsReader {
            reader,
            count,
            exprs,
        })
    }

    pub fn original_position(&self) -> usize {
        self.reader.original_position()
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn uses_exprs(&self) -> bool {
        self.exprs
    }

    pub fn read(&mut self) -> Result<ElementItem<'a>> {
        if self.exprs {
            let expr = self.reader.read_init_expr()?;
            Ok(ElementItem::Expr(expr))
        } else {
            let idx = self.reader.read_var_u32()?;
            Ok(ElementItem::Func(idx))
        }
    }
}

impl<'a> IntoIterator for ElementItemsReader<'a> {
    type Item = Result<ElementItem<'a>>;
    type IntoIter = ElementItemsIterator<'a>;
    fn into_iter(self) -> Self::IntoIter {
        let count = self.count;
        ElementItemsIterator {
            reader: self,
            left: count,
            err: false,
        }
    }
}

pub struct ElementItemsIterator<'a> {
    reader: ElementItemsReader<'a>,
    left: u32,
    err: bool,
}

impl<'a> Iterator for ElementItemsIterator<'a> {
    type Item = Result<ElementItem<'a>>;
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

#[derive(Clone)]
pub struct ElementSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

impl<'a> ElementSectionReader<'a> {
    pub fn new(data: &'a [u8], offset: usize) -> Result<ElementSectionReader<'a>> {
        let mut reader = BinaryReader::new_with_offset(data, offset);
        let count = reader.read_var_u32()?;
        Ok(ElementSectionReader { reader, count })
    }

    pub fn original_position(&self) -> usize {
        self.reader.original_position()
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    /// Reads content of the element section.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # let data: &[u8] = &[];
    /// use wasmparser::{ElementSectionReader, ElementKind};
    /// let mut element_reader = ElementSectionReader::new(data, 0).unwrap();
    /// for _ in 0..element_reader.get_count() {
    ///     let element = element_reader.read().expect("element");
    ///     if let ElementKind::Active { init_expr, .. } = element.kind {
    ///         let mut init_expr_reader = init_expr.get_binary_reader();
    ///         let op = init_expr_reader.read_operator().expect("op");
    ///         println!("Init const: {:?}", op);
    ///     }
    ///     let mut items_reader = element.items.get_items_reader().expect("items reader");
    ///     for _ in 0..items_reader.get_count() {
    ///         let item = items_reader.read().expect("item");
    ///         println!("  Item: {:?}", item);
    ///     }
    /// }
    /// ```
    pub fn read<'b>(&mut self) -> Result<Element<'b>>
    where
        'a: 'b,
    {
        let elem_start = self.reader.original_position();

        let flags = self.reader.read_var_u32()?;
        if (flags & !0b111) != 0 {
            return Err(BinaryReaderError::new(
                "invalid flags byte in element segment",
                self.reader.original_position() - 1,
            ));
        }
        let kind = if flags & 0b001 != 0 {
            if flags & 0b010 != 0 {
                ElementKind::Declared
            } else {
                ElementKind::Passive
            }
        } else {
            let table_index = if flags & 0b010 == 0 {
                0
            } else {
                self.reader.read_var_u32()?
            };
            let init_expr = {
                let expr_offset = self.reader.position;
                self.reader.skip_init_expr()?;
                let data = &self.reader.buffer[expr_offset..self.reader.position];
                InitExpr::new(data, self.reader.original_offset + expr_offset)
            };
            ElementKind::Active {
                table_index,
                init_expr,
            }
        };
        let exprs = flags & 0b100 != 0;
        let ty = if flags & 0b011 != 0 {
            if exprs {
                self.reader.read_type()?
            } else {
                match self.reader.read_external_kind()? {
                    ExternalKind::Function => Type::FuncRef,
                    _ => {
                        return Err(BinaryReaderError::new(
                            "only the function external type is supported in elem segment",
                            self.reader.original_position() - 1,
                        ));
                    }
                }
            }
        } else {
            Type::FuncRef
        };
        let data_start = self.reader.position;
        let items_count = self.reader.read_var_u32()?;
        if exprs {
            for _ in 0..items_count {
                self.reader.skip_init_expr()?;
            }
        } else {
            for _ in 0..items_count {
                self.reader.skip_var_32()?;
            }
        }
        let data_end = self.reader.position;
        let items = ElementItems {
            offset: self.reader.original_offset + data_start,
            data: &self.reader.buffer[data_start..data_end],
            exprs,
        };

        let elem_end = self.reader.original_position();
        let range = Range::new(elem_start, elem_end);

        Ok(Element {
            kind,
            items,
            ty,
            range,
        })
    }
}

impl<'a> SectionReader for ElementSectionReader<'a> {
    type Item = Element<'a>;
    fn read(&mut self) -> Result<Self::Item> {
        ElementSectionReader::read(self)
    }
    fn eof(&self) -> bool {
        self.reader.eof()
    }
    fn original_position(&self) -> usize {
        ElementSectionReader::original_position(self)
    }
    fn range(&self) -> Range {
        self.reader.range()
    }
}

impl<'a> SectionWithLimitedItems for ElementSectionReader<'a> {
    fn get_count(&self) -> u32 {
        ElementSectionReader::get_count(self)
    }
}

impl<'a> IntoIterator for ElementSectionReader<'a> {
    type Item = Result<Element<'a>>;
    type IntoIter = SectionIteratorLimited<ElementSectionReader<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        SectionIteratorLimited::new(self)
    }
}
