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
    BinaryReader, BinaryReaderError, ConstExpr, ExternalKind, Result, SectionIteratorLimited,
    SectionReader, SectionWithLimitedItems, ValType,
};
use std::ops::Range;

/// Represents a core WebAssembly element segment.
#[derive(Clone)]
pub struct Element<'a> {
    /// The kind of the element segment.
    pub kind: ElementKind<'a>,
    /// The initial elements of the element segment.
    pub items: ElementItems<'a>,
    /// The type of the elements.
    pub ty: ValType,
    /// The range of the the element segment.
    pub range: Range<usize>,
}

/// The kind of element segment.
#[derive(Clone)]
pub enum ElementKind<'a> {
    /// The element segment is passive.
    Passive,
    /// The element segment is active.
    Active {
        /// The index of the table being initialized.
        table_index: u32,
        /// The initial expression of the element segment.
        offset_expr: ConstExpr<'a>,
    },
    /// The element segment is declared.
    Declared,
}

/// Represents the items of an element segment.
#[derive(Debug, Copy, Clone)]
pub struct ElementItems<'a> {
    exprs: bool,
    offset: usize,
    data: &'a [u8],
}

/// Represents an individual item of an element segment.
#[derive(Debug)]
pub enum ElementItem<'a> {
    /// The item is a function index.
    Func(u32),
    /// The item is an initialization expression.
    Expr(ConstExpr<'a>),
}

impl<'a> ElementItems<'a> {
    /// Gets an items reader for the items in an element segment.
    pub fn get_items_reader<'b>(&self) -> Result<ElementItemsReader<'b>>
    where
        'a: 'b,
    {
        ElementItemsReader::new(self.data, self.offset, self.exprs)
    }
}

/// A reader for element items in an element segment.
pub struct ElementItemsReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
    exprs: bool,
}

impl<'a> ElementItemsReader<'a> {
    /// Constructs a new `ElementItemsReader` for the given data and offset.
    pub fn new(data: &[u8], offset: usize, exprs: bool) -> Result<ElementItemsReader> {
        let mut reader = BinaryReader::new_with_offset(data, offset);
        let count = reader.read_var_u32()?;
        Ok(ElementItemsReader {
            reader,
            count,
            exprs,
        })
    }

    /// Gets the original position of the reader.
    pub fn original_position(&self) -> usize {
        self.reader.original_position()
    }

    /// Gets the count of element items in the segment.
    pub fn get_count(&self) -> u32 {
        self.count
    }

    /// Whether or not initialization expressions are used.
    pub fn uses_exprs(&self) -> bool {
        self.exprs
    }

    /// Reads an element item from the segment.
    pub fn read(&mut self) -> Result<ElementItem<'a>> {
        if self.exprs {
            let expr = self.reader.read_const_expr()?;
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

/// An iterator over element items in an element segment.
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

/// A reader for the element section of a WebAssembly module.
#[derive(Clone)]
pub struct ElementSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

impl<'a> ElementSectionReader<'a> {
    /// Constructs a new `ElementSectionReader` for the given data and offset.
    pub fn new(data: &'a [u8], offset: usize) -> Result<ElementSectionReader<'a>> {
        let mut reader = BinaryReader::new_with_offset(data, offset);
        let count = reader.read_var_u32()?;
        Ok(ElementSectionReader { reader, count })
    }

    /// Gets the original position of the section reader.
    pub fn original_position(&self) -> usize {
        self.reader.original_position()
    }

    /// Gets the count of items in the section.
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
    ///     if let ElementKind::Active { offset_expr, .. } = element.kind {
    ///         let mut offset_expr_reader = offset_expr.get_binary_reader();
    ///         let op = offset_expr_reader.read_operator().expect("op");
    ///         println!("offset expression: {:?}", op);
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
        // The current handling of the flags is largely specified in the `bulk-memory` proposal,
        // which at the time this commend is written has been merged to the main specification
        // draft.
        //
        // Notably, this proposal allows multiple different encodings of the table index 0. `00`
        // and `02 00` are both valid ways to specify the 0-th table. However it also makes
        // another encoding of the 0-th memory `80 00` no longer valid.
        //
        // We, however maintain this support by parsing `flags` as a LEB128 integer. In that case,
        // `80 00` encoding is parsed out as `0` and is therefore assigned a `tableidx` 0, even
        // though the current specification draft does not allow for this.
        //
        // See also https://github.com/WebAssembly/spec/issues/1439
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
            let offset_expr = {
                let expr_offset = self.reader.position;
                self.reader.skip_const_expr()?;
                let data = &self.reader.buffer[expr_offset..self.reader.position];
                ConstExpr::new(data, self.reader.original_offset + expr_offset)
            };
            ElementKind::Active {
                table_index,
                offset_expr,
            }
        };
        let exprs = flags & 0b100 != 0;
        let ty = if flags & 0b011 != 0 {
            if exprs {
                self.reader.read_val_type()?
            } else {
                match self.reader.read_external_kind()? {
                    ExternalKind::Func => ValType::FuncRef,
                    _ => {
                        return Err(BinaryReaderError::new(
                            "only the function external type is supported in elem segment",
                            self.reader.original_position() - 1,
                        ));
                    }
                }
            }
        } else {
            ValType::FuncRef
        };
        let data_start = self.reader.position;
        let items_count = self.reader.read_var_u32()?;
        if exprs {
            for _ in 0..items_count {
                self.reader.skip_const_expr()?;
            }
        } else {
            for _ in 0..items_count {
                self.reader.read_var_u32()?;
            }
        }
        let data_end = self.reader.position;
        let items = ElementItems {
            offset: self.reader.original_offset + data_start,
            data: &self.reader.buffer[data_start..data_end],
            exprs,
        };

        let elem_end = self.reader.original_position();
        let range = elem_start..elem_end;

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
    fn range(&self) -> Range<usize> {
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
