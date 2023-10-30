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
    BinaryReader, BinaryReaderError, ExternalKind, GlobalType, LinkingType, MemoryType, NameType,
    Naming, Operator, Range, RelocType, Result, SectionCode, TableType, TagType, Type,
};

pub use self::alias_section::*;
pub use self::code_section::*;
pub use self::data_section::*;
pub use self::element_section::*;
pub use self::export_section::*;
pub use self::function_section::*;
pub use self::global_section::*;
pub use self::import_section::*;
pub use self::init_expr::*;
pub use self::instance_section::*;
pub use self::linking_section::*;
pub use self::memory_section::*;
pub use self::module_section::*;
pub use self::name_section::*;
pub use self::operators::*;
pub use self::producers_section::*;
pub use self::reloc_section::*;
pub use self::section_reader::*;
pub use self::table_section::*;
pub use self::tag_section::*;
pub use self::type_section::*;

mod alias_section;
mod code_section;
mod data_section;
mod element_section;
mod export_section;
mod function_section;
mod global_section;
mod import_section;
mod init_expr;
mod instance_section;
mod linking_section;
mod memory_section;
mod module_section;
mod name_section;
mod operators;
mod producers_section;
mod reloc_section;
mod section_reader;
mod table_section;
mod tag_section;
mod type_section;
