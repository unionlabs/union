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

use super::{BinaryReader, OperatorsReader};

#[derive(Debug, Copy, Clone)]
pub struct InitExpr<'a> {
    offset: usize,
    data: &'a [u8],
}

impl<'a> InitExpr<'a> {
    pub fn new(data: &[u8], offset: usize) -> InitExpr {
        InitExpr { offset, data }
    }

    pub fn get_binary_reader<'b>(&self) -> BinaryReader<'b>
    where
        'a: 'b,
    {
        BinaryReader::new_with_offset(self.data, self.offset)
    }

    pub fn get_operators_reader<'b>(&self) -> OperatorsReader<'b>
    where
        'a: 'b,
    {
        OperatorsReader::new(self.data, self.offset)
    }
}
