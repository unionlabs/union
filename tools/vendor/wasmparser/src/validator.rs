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

use crate::limits::*;
use crate::operators_validator::OperatorValidator;
use crate::WasmModuleResources;
use crate::{Alias, ExternalKind, Import, ImportSectionEntryType};
use crate::{BinaryReaderError, GlobalType, MemoryType, Range, Result, TableType, TagType, Type};
use crate::{DataKind, ElementItem, ElementKind, InitExpr, Instance, Operator};
use crate::{FuncType, SectionReader, SectionWithLimitedItems};
use crate::{FunctionBody, Parser, Payload};
use std::collections::{HashMap, HashSet};
use std::mem;
use std::sync::Arc;

/// Test whether the given buffer contains a valid WebAssembly module,
/// analogous to [`WebAssembly.validate`][js] in the JS API.
///
/// This functions requires the wasm module is entirely resident in memory and
/// is specified by `bytes`. Additionally this validates the given bytes with
/// the default set of WebAssembly features implemented by `wasmparser`.
///
/// For more fine-tuned control over validation it's recommended to review the
/// documentation of [`Validator`].
///
/// [js]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/validate
pub fn validate(bytes: &[u8]) -> Result<()> {
    Validator::new().validate_all(bytes)
}

#[test]
fn test_validate() {
    assert!(validate(&[0x0, 0x61, 0x73, 0x6d, 0x1, 0x0, 0x0, 0x0]).is_ok());
    assert!(validate(&[0x0, 0x61, 0x73, 0x6d, 0x2, 0x0, 0x0, 0x0]).is_err());
}

mod func;
pub use func::FuncValidator;

/// Validator for a WebAssembly binary module.
///
/// This structure encapsulates state necessary to validate a WebAssembly
/// binary. This implements validation as defined by the [core
/// specification][core]. A `Validator` is designed, like
/// [`Parser`], to accept incremental input over time.
/// Additionally a `Validator` is also designed for parallel validation of
/// functions as they are received.
///
/// It's expected that you'll be using a [`Parser`] in tandem with a
/// `Validator`. As each [`Payload`](crate::Payload) is received from a
/// [`Parser`] you'll pass it into a `Validator` to test the validity of the
/// payload. Note that all payloads received from a [`Parser`] are expected to
/// be passed to a [`Validator`]. For example if you receive
/// [`Payload::TypeSection`](crate::Payload) you'll call
/// [`Validator::type_section`] to validate this.
///
/// The design of [`Validator`] is intended that you'll interleave, in your own
/// application's processing, calls to validation. Each variant, after it's
/// received, will be validated and then your application would proceed as
/// usual. At all times, however, you'll have access to the [`Validator`] and
/// the validation context up to that point. This enables applications to check
/// the types of functions and learn how many globals there are, for example.
///
/// [core]: https://webassembly.github.io/spec/core/valid/index.html
#[derive(Default)]
pub struct Validator {
    /// The current module that we're validating.
    cur: Module,

    /// With module linking this is the list of parent modules to this module,
    /// sorted from oldest to most recent.
    parents: Vec<Module>,

    /// This is the global list of all types shared by this validator which all
    /// modules will reference.
    types: SnapshotList<TypeDef>,

    /// Enabled WebAssembly feature flags, dictating what's valid and what
    /// isn't.
    features: WasmFeatures,

    /// The current byte-level offset in the wasm binary. This is updated to
    /// produce error messages in `create_error`.
    offset: usize,
}

#[derive(Default)]
struct Module {
    /// Internal state that is incrementally built-up for the module being
    /// validate. This houses type information for all wasm items, like
    /// functions. Note that this starts out as a solely owned `Arc<T>` so we can
    /// get mutable access, but after we get to the code section this is never
    /// mutated to we can clone it cheaply and hand it to sub-validators.
    state: arc::MaybeOwned<ModuleState>,

    /// Where we are, order-wise, in the wasm binary.
    order: Order,

    /// The number of data segments we ended up finding in this module, or 0 if
    /// they either weren't present or none were found.
    data_found: u32,

    /// The number of functions we expect to be defined in the code section, or
    /// basically the length of the function section if it was found. The next
    /// index is where we are, in the code section index space, for the next
    /// entry in the code section (used to figure out what type is next for the
    /// function being validated).
    expected_code_bodies: Option<u32>,
    code_section_index: usize,
}

#[derive(Default)]
struct ModuleState {
    /// This is a snapshot of `validator.types` when it is created. This is not
    /// initially filled in but once everything in a module except the code
    /// section has been parsed then this will be filled in.
    ///
    /// Note that this `ModuleState` will be a separately-owned structure living
    /// in each function's validator. This is done to allow parallel validation
    /// of functions while the main module is possibly still being parsed.
    all_types: Option<Arc<SnapshotList<TypeDef>>>,

    types: Vec<usize>, // pointer into `validator.types`
    tables: Vec<TableType>,
    memories: Vec<MemoryType>,
    globals: Vec<GlobalType>,
    num_imported_globals: u32,
    element_types: Vec<Type>,
    data_count: Option<u32>,
    code_type_indexes: Vec<u32>, // pointer into `types` above
    func_types: Vec<usize>,      // pointer into `validator.types`
    tags: Vec<usize>,            // pointer into `validator.types`
    submodules: Vec<usize>,      // pointer into `validator.types`
    instances: Vec<usize>,       // pointer into `validator.types`
    function_references: HashSet<u32>,

    // This is populated when we hit the export section
    exports: NameSet,

    // This is populated as we visit import sections, which might be
    // incrementally in the face of a module-linking-using module.
    imports: NameSet,
}

/// Flags for features that are enabled for validation.
#[derive(Hash, Debug, Copy, Clone)]
pub struct WasmFeatures {
    /// The WebAssembly `mutable-global` proposal (enabled by default)
    pub mutable_global: bool,
    /// The WebAssembly `nontrapping-float-to-int-conversions` proposal (enabled by default)
    pub saturating_float_to_int: bool,
    /// The WebAssembly `sign-extension-ops` proposal (enabled by default)
    pub sign_extension: bool,
    /// The WebAssembly reference types proposal (enabled by default)
    pub reference_types: bool,
    /// The WebAssembly multi-value proposal (enabled by default)
    pub multi_value: bool,
    /// The WebAssembly bulk memory operations proposal (enabled by default)
    pub bulk_memory: bool,
    /// The WebAssembly module linking proposal
    pub module_linking: bool,
    /// The WebAssembly SIMD proposal
    pub simd: bool,
    /// The WebAssembly Relaxed SIMD proposal
    pub relaxed_simd: bool,
    /// The WebAssembly threads proposal
    pub threads: bool,
    /// The WebAssembly tail-call proposal
    pub tail_call: bool,
    /// Whether or not only deterministic instructions are allowed
    pub deterministic_only: bool,
    /// The WebAssembly multi memory proposal
    pub multi_memory: bool,
    /// The WebAssembly exception handling proposal
    pub exceptions: bool,
    /// The WebAssembly memory64 proposal
    pub memory64: bool,
    /// The WebAssembly extended_const proposal
    pub extended_const: bool,
}

impl Default for WasmFeatures {
    fn default() -> WasmFeatures {
        WasmFeatures {
            // off-by-default features
            module_linking: false,
            relaxed_simd: false,
            threads: false,
            tail_call: false,
            multi_memory: false,
            exceptions: false,
            memory64: false,
            extended_const: false,
            deterministic_only: cfg!(feature = "deterministic"),

            // on-by-default features
            mutable_global: true,
            saturating_float_to_int: true,
            sign_extension: true,
            bulk_memory: true,
            multi_value: true,
            reference_types: true,
            simd: true,
        }
    }
}

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Debug)]
enum Order {
    Initial,
    AfterHeader,
    Type,
    Import,
    ModuleLinkingHeader,
    Function,
    Table,
    Memory,
    Tag,
    Global,
    Export,
    Start,
    Element,
    DataCount,
    Code,
    Data,
}

impl Default for Order {
    fn default() -> Order {
        Order::Initial
    }
}

enum TypeDef {
    Func(FuncType),
    Module(ModuleType),
    Instance(InstanceType),
}

impl TypeDef {
    fn unwrap_func(&self) -> &FuncType {
        match self {
            TypeDef::Func(f) => f,
            _ => panic!("not a function type"),
        }
    }

    fn unwrap_module(&self) -> &ModuleType {
        match self {
            TypeDef::Module(f) => f,
            _ => panic!("not a module type"),
        }
    }

    fn unwrap_instance(&self) -> &InstanceType {
        match self {
            TypeDef::Instance(f) => f,
            _ => panic!("not an instance type"),
        }
    }
}

struct ModuleType {
    imports_size: u32,
    exports_size: u32,
    imports: HashMap<String, EntityType>,
    exports: HashMap<String, EntityType>,
}

#[derive(Default)]
struct InstanceType {
    type_size: u32,
    exports: HashMap<String, EntityType>,
}

#[derive(Clone)]
enum EntityType {
    Global(GlobalType),
    Memory(MemoryType),
    Table(TableType),
    Func(usize),     // pointer into `validator.types`
    Module(usize),   // pointer into `validator.types`
    Instance(usize), // pointer into `validator.types`
    Tag(usize),      // pointer into `validator.types`
}

/// Possible return values from [`Validator::payload`].
pub enum ValidPayload<'a> {
    /// The payload validated, no further action need be taken.
    Ok,
    /// The payload validated, but it started a nested module.
    ///
    /// This result indicates that the specified parser should be used instead
    /// of the currently-used parser until this returned one ends.
    Submodule(Parser),
    /// A function was found to be validate.
    Func(FuncValidator<ValidatorResources>, FunctionBody<'a>),
}

impl Validator {
    /// Creates a new [`Validator`] ready to validate a WebAssembly module.
    ///
    /// The new validator will receive payloads parsed from
    /// [`Parser`], and expects the first payload received to be
    /// the version header from the parser.
    pub fn new() -> Validator {
        Validator::default()
    }

    /// Configures the enabled WebAssembly features for this `Validator`.
    pub fn wasm_features(&mut self, features: WasmFeatures) -> &mut Validator {
        self.features = features;
        self
    }

    /// Validates an entire in-memory module with this validator.
    ///
    /// This function will internally create a [`Parser`] to parse the `bytes`
    /// provided. The entire wasm module specified by `bytes` will be parsed and
    /// validated. Parse and validation errors will be returned through
    /// `Err(_)`, and otherwise a successful validation means `Ok(())` is
    /// returned.
    pub fn validate_all(&mut self, bytes: &[u8]) -> Result<()> {
        let mut functions_to_validate = Vec::new();
        for payload in Parser::new(0).parse_all(bytes) {
            if let ValidPayload::Func(a, b) = self.payload(&payload?)? {
                functions_to_validate.push((a, b));
            }
        }

        for (mut validator, body) in functions_to_validate {
            validator.validate(&body)?;
        }
        Ok(())
    }

    /// Convenience function to validate a single [`Payload`].
    ///
    /// This function is intended to be used as a convenience. It will
    /// internally perform any validation necessary to validate the [`Payload`]
    /// provided. The convenience part is that you're likely already going to
    /// be matching on [`Payload`] in your application, at which point it's more
    /// appropriate to call the individual methods on [`Validator`] per-variant
    /// in [`Payload`], such as [`Validator::type_section`].
    ///
    /// This function returns a [`ValidPayload`] variant on success, indicating
    /// one of a few possible actions that need to be taken after a payload is
    /// validated. For example function contents are not validated here, they're
    /// returned through [`ValidPayload`] for validation by the caller.
    pub fn payload<'a>(&mut self, payload: &Payload<'a>) -> Result<ValidPayload<'a>> {
        use crate::Payload::*;
        match payload {
            Version { num, range } => self.version(*num, range)?,
            TypeSection(s) => self.type_section(s)?,
            ImportSection(s) => self.import_section(s)?,
            AliasSection(s) => self.alias_section(s)?,
            InstanceSection(s) => self.instance_section(s)?,
            FunctionSection(s) => self.function_section(s)?,
            TableSection(s) => self.table_section(s)?,
            MemorySection(s) => self.memory_section(s)?,
            TagSection(s) => self.tag_section(s)?,
            GlobalSection(s) => self.global_section(s)?,
            ExportSection(s) => self.export_section(s)?,
            StartSection { func, range } => self.start_section(*func, range)?,
            ElementSection(s) => self.element_section(s)?,
            DataCountSection { count, range } => self.data_count_section(*count, range)?,
            CodeSectionStart {
                count,
                range,
                size: _,
            } => self.code_section_start(*count, range)?,
            CodeSectionEntry(body) => {
                let func_validator = self.code_section_entry()?;
                return Ok(ValidPayload::Func(func_validator, *body));
            }
            ModuleSectionStart {
                count,
                range,
                size: _,
            } => self.module_section_start(*count, range)?,
            DataSection(s) => self.data_section(s)?,
            End => self.end()?,

            CustomSection { .. } => {} // no validation for custom sections
            UnknownSection { id, range, .. } => self.unknown_section(*id, range)?,
            ModuleSectionEntry { parser, .. } => {
                self.module_section_entry();
                return Ok(ValidPayload::Submodule(parser.clone()));
            }
        }
        Ok(ValidPayload::Ok)
    }

    fn create_error<T>(&self, msg: impl Into<String>) -> Result<T> {
        Err(BinaryReaderError::new(msg.into(), self.offset))
    }

    /// Validates [`Payload::Version`](crate::Payload)
    pub fn version(&mut self, num: u32, range: &Range) -> Result<()> {
        self.offset = range.start;
        if self.cur.order != Order::Initial {
            return self.create_error("wasm version header out of order");
        }
        self.cur.order = Order::AfterHeader;
        if num != 1 {
            return self.create_error("bad wasm file version");
        }
        Ok(())
    }

    fn update_order(&mut self, order: Order) -> Result<()> {
        let prev = mem::replace(&mut self.cur.order, order);
        // If the previous section came before this section, then that's always
        // valid.
        if prev < order {
            return Ok(());
        }
        // ... otherwise if this is a repeated section then only the "module
        // linking header" is allows to have repeats
        if prev == self.cur.order && self.cur.order == Order::ModuleLinkingHeader {
            return Ok(());
        }
        self.create_error("section out of order")
    }

    fn header_order(&mut self, order: Order) -> Order {
        if self.features.module_linking {
            Order::ModuleLinkingHeader
        } else {
            order
        }
    }

    fn get_type(&self, idx: u32) -> Result<&TypeDef> {
        match self.cur.state.types.get(idx as usize) {
            Some(t) => Ok(&self.types[*t]),
            None => self.create_error(format!("unknown type {}: type index out of bounds", idx)),
        }
    }

    fn get_table(&self, idx: u32) -> Result<&TableType> {
        match self.cur.state.tables.get(idx as usize) {
            Some(t) => Ok(t),
            None => self.create_error(format!("unknown table {}: table index out of bounds", idx)),
        }
    }

    fn get_memory(&self, idx: u32) -> Result<&MemoryType> {
        match self.cur.state.memories.get(idx as usize) {
            Some(t) => Ok(t),
            None => self.create_error(format!(
                "unknown memory {}: memory index out of bounds",
                idx,
            )),
        }
    }

    fn get_global(&self, idx: u32) -> Result<&GlobalType> {
        match self.cur.state.globals.get(idx as usize) {
            Some(t) => Ok(t),
            None => self.create_error(format!(
                "unknown global {}: global index out of bounds",
                idx,
            )),
        }
    }

    fn get_func_type(&self, func_idx: u32) -> Result<&FuncType> {
        match self.cur.state.func_types.get(func_idx as usize) {
            Some(t) => Ok(self.types[*t].unwrap_func()),
            None => self.create_error(format!(
                "unknown function {}: func index out of bounds",
                func_idx,
            )),
        }
    }

    fn get_module_type(&self, module_idx: u32) -> Result<&ModuleType> {
        match self.cur.state.submodules.get(module_idx as usize) {
            Some(t) => Ok(self.types[*t].unwrap_module()),
            None => self.create_error("unknown module: module index out of bounds"),
        }
    }

    fn get_instance_type(&self, instance_idx: u32) -> Result<&InstanceType> {
        match self.cur.state.instances.get(instance_idx as usize) {
            Some(t) => Ok(self.types[*t].unwrap_instance()),
            None => self.create_error("unknown instance: instance index out of bounds"),
        }
    }

    fn func_type_at(&self, type_index: u32) -> Result<&FuncType> {
        let def = self.get_type(type_index)?;
        match def {
            TypeDef::Func(item) => Ok(item),
            _ => self.create_error("type index is not a function"),
        }
    }

    fn module_type_at(&self, type_index: u32) -> Result<&ModuleType> {
        if !self.features.module_linking {
            return self.create_error("module linking proposal not enabled");
        }
        let ty = self.get_type(type_index)?;
        match ty {
            TypeDef::Module(item) => Ok(item),
            _ => self.create_error("type index is not a module"),
        }
    }

    fn instance_type_at(&self, type_index: u32) -> Result<&InstanceType> {
        if !self.features.module_linking {
            return self.create_error("module linking proposal not enabled");
        }
        let ty = self.get_type(type_index)?;
        match ty {
            TypeDef::Instance(item) => Ok(item),
            _ => self.create_error("type index is not an instance"),
        }
    }

    fn check_max(&self, cur_len: usize, amt_added: u32, max: usize, desc: &str) -> Result<()> {
        let overflow = max
            .checked_sub(cur_len)
            .and_then(|amt| amt.checked_sub(amt_added as usize))
            .is_none();
        if overflow {
            return if max == 1 {
                self.create_error(format!("multiple {}", desc))
            } else {
                self.create_error(format!("{} count is out of bounds", desc))
            };
        }
        Ok(())
    }

    fn section<T>(
        &mut self,
        order: Order,
        section: &T,
        mut validate_item: impl FnMut(&mut Self, T::Item) -> Result<()>,
    ) -> Result<()>
    where
        T: SectionReader + Clone + SectionWithLimitedItems,
    {
        self.offset = section.range().start;
        self.update_order(order)?;

        let mut section = section.clone();
        for _ in 0..section.get_count() {
            self.offset = section.original_position();
            let item = section.read()?;
            validate_item(self, item)?;
        }
        self.offset = section.range().end;
        section.ensure_end()?;
        Ok(())
    }

    /// Validates [`Payload::TypeSection`](crate::Payload)
    pub fn type_section(&mut self, section: &crate::TypeSectionReader<'_>) -> Result<()> {
        let order = self.header_order(Order::Type);
        self.check_max(
            self.cur.state.types.len(),
            section.get_count(),
            MAX_WASM_TYPES,
            "types",
        )?;
        self.section(order, section, |me, item| me.type_def(item))
    }

    fn type_def(&mut self, def: crate::TypeDef<'_>) -> Result<()> {
        let def = match def {
            crate::TypeDef::Func(t) => {
                for ty in t.params.iter().chain(t.returns.iter()) {
                    self.value_type(*ty)?;
                }
                if t.returns.len() > 1 && !self.features.multi_value {
                    return self
                        .create_error("invalid result arity: func type returns multiple values");
                }
                TypeDef::Func(t)
            }
            crate::TypeDef::Module(t) => {
                if !self.features.module_linking {
                    return self.create_error("module linking proposal not enabled");
                }
                let mut imports = NameSet::default();
                for i in t.imports.iter() {
                    let ty = self.import_entry_type(&i.ty)?;
                    imports.push(
                        self.offset,
                        i.module,
                        i.field,
                        ty,
                        &mut self.types,
                        "import",
                    )?;
                }

                let mut exports = NameSet::default();
                for e in t.exports.iter() {
                    let ty = self.import_entry_type(&e.ty)?;
                    exports.push(self.offset, e.name, None, ty, &mut self.types, "export")?;
                }
                combine_type_sizes(self.offset, imports.type_size, exports.type_size)?;
                TypeDef::Module(ModuleType {
                    imports_size: imports.type_size,
                    exports_size: exports.type_size,
                    imports: imports.set,
                    exports: exports.set,
                })
            }
            crate::TypeDef::Instance(t) => {
                if !self.features.module_linking {
                    return self.create_error("module linking proposal not enabled");
                }
                let mut exports = NameSet::default();
                for e in t.exports.iter() {
                    let ty = self.import_entry_type(&e.ty)?;
                    exports.push(self.offset, e.name, None, ty, &mut self.types, "export")?;
                }
                TypeDef::Instance(InstanceType {
                    type_size: exports.type_size,
                    exports: exports.set,
                })
            }
        };
        self.cur.state.assert_mut().types.push(self.types.len());
        self.types.push(def);
        Ok(())
    }

    fn value_type(&self, ty: Type) -> Result<()> {
        match self.features.check_value_type(ty) {
            Ok(()) => Ok(()),
            Err(e) => self.create_error(e),
        }
    }

    fn import_entry_type(&self, import_type: &ImportSectionEntryType) -> Result<EntityType> {
        match import_type {
            ImportSectionEntryType::Function(type_index) => {
                self.func_type_at(*type_index)?;
                Ok(EntityType::Func(self.cur.state.types[*type_index as usize]))
            }
            ImportSectionEntryType::Table(t) => {
                self.table_type(t)?;
                Ok(EntityType::Table(*t))
            }
            ImportSectionEntryType::Memory(t) => {
                self.memory_type(t)?;
                Ok(EntityType::Memory(*t))
            }
            ImportSectionEntryType::Tag(t) => {
                self.tag_type(t)?;
                Ok(EntityType::Tag(self.cur.state.types[t.type_index as usize]))
            }
            ImportSectionEntryType::Global(t) => {
                self.global_type(t)?;
                Ok(EntityType::Global(*t))
            }
            ImportSectionEntryType::Module(type_index) => {
                self.module_type_at(*type_index)?;
                Ok(EntityType::Module(
                    self.cur.state.types[*type_index as usize],
                ))
            }
            ImportSectionEntryType::Instance(type_index) => {
                self.instance_type_at(*type_index)?;
                Ok(EntityType::Instance(
                    self.cur.state.types[*type_index as usize],
                ))
            }
        }
    }

    fn table_type(&self, ty: &TableType) -> Result<()> {
        match ty.element_type {
            Type::FuncRef => {}
            Type::ExternRef => {
                if !self.features.reference_types {
                    return self.create_error("element is not anyfunc");
                }
            }
            _ => return self.create_error("element is not reference type"),
        }
        self.limits(ty.initial, ty.maximum)?;
        if ty.initial > MAX_WASM_TABLE_ENTRIES as u32 {
            return self.create_error("minimum table size is out of bounds");
        }
        Ok(())
    }

    fn memory_type(&self, ty: &MemoryType) -> Result<()> {
        self.limits(ty.initial, ty.maximum)?;
        let (true_maximum, err) = if ty.memory64 {
            if !self.features.memory64 {
                return self.create_error("memory64 must be enabled for 64-bit memories");
            }
            (
                MAX_WASM_MEMORY64_PAGES,
                "memory size must be at most 2**48 pages",
            )
        } else {
            (
                MAX_WASM_MEMORY32_PAGES,
                "memory size must be at most 65536 pages (4GiB)",
            )
        };
        if ty.initial > true_maximum {
            return self.create_error(err);
        }
        if let Some(maximum) = ty.maximum {
            if maximum > true_maximum {
                return self.create_error(err);
            }
        }
        if ty.shared {
            if !self.features.threads {
                return self.create_error("threads must be enabled for shared memories");
            }
            if ty.maximum.is_none() {
                return self.create_error("shared memory must have maximum size");
            }
        }
        Ok(())
    }

    fn tag_type(&self, ty: &TagType) -> Result<()> {
        if !self.features.exceptions {
            return self.create_error("exceptions proposal not enabled");
        }
        let ty = self.func_type_at(ty.type_index)?;
        if ty.returns.len() > 0 {
            return self.create_error("invalid exception type: non-empty tag result type");
        }
        Ok(())
    }

    fn global_type(&self, ty: &GlobalType) -> Result<()> {
        self.value_type(ty.content_type)
    }

    fn limits<T>(&self, initial: T, maximum: Option<T>) -> Result<()>
    where
        T: Into<u64>,
    {
        if let Some(max) = maximum {
            if initial.into() > max.into() {
                return self.create_error("size minimum must not be greater than maximum");
            }
        }
        Ok(())
    }

    /// Validates [`Payload::ImportSection`](crate::Payload)
    pub fn import_section(&mut self, section: &crate::ImportSectionReader<'_>) -> Result<()> {
        let order = self.header_order(Order::Import);
        self.section(order, section, |me, item| me.import(item))?;

        // Clear the list of implicit imports after the import section is
        // finished since later import sections cannot append further to the
        // pseudo-instances defined in this import section.
        self.cur.state.assert_mut().imports.implicit.drain();
        Ok(())
    }

    fn import(&mut self, entry: Import<'_>) -> Result<()> {
        if !self.features.module_linking && entry.field.is_none() {
            return self.create_error("module linking proposal is not enabled");
        }
        let ty = self.import_entry_type(&entry.ty)?;
        let state = self.cur.state.assert_mut();

        // Build up a map of what this module imports, for when this module is a
        // nested module it'll be needed to infer the type signature of the
        // nested module. Note that this does not happen unless the module
        // linking proposal is enabled.
        //
        // This is a breaking change! This will disallow multiple imports that
        // import from the same item twice. We can't turn module linking
        // on-by-default as-is without some sort of recourse for consumers that
        // want to backwards-compatibly parse older modules still. Unclear how
        // to do this.
        if self.features.module_linking {
            let implicit_instance_type = state.imports.push(
                self.offset,
                entry.module,
                entry.field,
                ty,
                &mut self.types,
                "import",
            )?;
            if let Some(idx) = implicit_instance_type {
                state.instances.push(idx);
            }
        }
        let (len, max, desc) = match entry.ty {
            ImportSectionEntryType::Function(type_index) => {
                let ty = state.types[type_index as usize];
                state.func_types.push(ty);
                (state.func_types.len(), MAX_WASM_FUNCTIONS, "funcs")
            }
            ImportSectionEntryType::Table(ty) => {
                state.tables.push(ty);
                (state.tables.len(), self.max_tables(), "tables")
            }
            ImportSectionEntryType::Memory(ty) => {
                state.memories.push(ty);
                (state.memories.len(), self.max_memories(), "memories")
            }
            ImportSectionEntryType::Tag(ty) => {
                let ty = state.types[ty.type_index as usize];
                state.tags.push(ty);
                (state.tags.len(), MAX_WASM_TAGS, "tags")
            }
            ImportSectionEntryType::Global(ty) => {
                if !self.features.mutable_global && ty.mutable {
                    return self.create_error("mutable global support is not enabled");
                }
                state.globals.push(ty);
                state.num_imported_globals += 1;
                (state.globals.len(), MAX_WASM_GLOBALS, "globals")
            }
            ImportSectionEntryType::Instance(type_idx) => {
                let index = state.types[type_idx as usize];
                state.instances.push(index);
                (state.instances.len(), MAX_WASM_INSTANCES, "instances")
            }
            ImportSectionEntryType::Module(type_index) => {
                let index = state.types[type_index as usize];
                state.submodules.push(index);
                (state.submodules.len(), MAX_WASM_MODULES, "modules")
            }
        };
        self.check_max(len, 0, max, desc)?;
        Ok(())
    }

    /// Validates [`Payload::ModuleSectionStart`](crate::Payload)
    pub fn module_section_start(&mut self, count: u32, range: &Range) -> Result<()> {
        if !self.features.module_linking {
            return self.create_error("module linking proposal not enabled");
        }
        self.offset = range.start;
        self.update_order(Order::ModuleLinkingHeader)?;
        self.check_max(
            self.cur.state.submodules.len(),
            count,
            MAX_WASM_MODULES,
            "modules",
        )?;
        Ok(())
    }

    /// Validates [`Payload::ModuleSectionEntry`](crate::Payload).
    ///
    /// Note that this does not actually perform any validation itself. The
    /// `ModuleSectionEntry` payload is associated with a sub-parser for the
    /// sub-module, and it's expected that the events from the [`Parser`]
    /// are fed into this validator.
    pub fn module_section_entry(&mut self) {
        // Start a new module...
        let prev = mem::take(&mut self.cur);
        // ... and record the current module as its parent.
        self.parents.push(prev);
    }

    /// Validates [`Payload::AliasSection`](crate::Payload)
    pub fn alias_section(&mut self, section: &crate::AliasSectionReader<'_>) -> Result<()> {
        if !self.features.module_linking {
            return self.create_error("module linking proposal not enabled");
        }
        self.section(Order::ModuleLinkingHeader, section, |me, a| me.alias(a))
    }

    fn alias(&mut self, alias: Alias) -> Result<()> {
        match alias {
            Alias::InstanceExport {
                instance,
                kind,
                export,
            } => {
                let ty = self.get_instance_type(instance)?;
                let export = match ty.exports.get(export) {
                    Some(e) => e,
                    None => {
                        return self.create_error(format!(
                            "aliased name `{}` does not exist in instance",
                            export
                        ));
                    }
                };
                match (export, kind) {
                    (EntityType::Func(ty), ExternalKind::Function) => {
                        let ty = *ty;
                        self.cur.state.assert_mut().func_types.push(ty);
                    }
                    (EntityType::Table(ty), ExternalKind::Table) => {
                        let ty = *ty;
                        self.cur.state.assert_mut().tables.push(ty);
                    }
                    (EntityType::Memory(ty), ExternalKind::Memory) => {
                        let ty = *ty;
                        self.cur.state.assert_mut().memories.push(ty);
                    }
                    (EntityType::Tag(ty), ExternalKind::Tag) => {
                        let ty = *ty;
                        self.cur.state.assert_mut().tags.push(ty);
                    }
                    (EntityType::Global(ty), ExternalKind::Global) => {
                        let ty = *ty;
                        let state = self.cur.state.assert_mut();
                        state.num_imported_globals += 1;
                        state.globals.push(ty);
                    }
                    (EntityType::Instance(ty), ExternalKind::Instance) => {
                        let ty = *ty;
                        self.cur.state.assert_mut().instances.push(ty);
                    }
                    (EntityType::Module(ty), ExternalKind::Module) => {
                        let ty = *ty;
                        self.cur.state.assert_mut().submodules.push(ty);
                    }
                    _ => return self.create_error("alias kind mismatch with export kind"),
                }
            }
            Alias::OuterType {
                relative_depth,
                index,
            } => {
                let i = self
                    .parents
                    .len()
                    .checked_sub(relative_depth as usize)
                    .and_then(|i| i.checked_sub(1))
                    .ok_or_else(|| {
                        BinaryReaderError::new("relative depth too large", self.offset)
                    })?;
                let ty = match self.parents[i].state.types.get(index as usize) {
                    Some(m) => *m,
                    None => return self.create_error("alias to type not defined in parent yet"),
                };
                self.cur.state.assert_mut().types.push(ty);
            }
            Alias::OuterModule {
                relative_depth,
                index,
            } => {
                let i = self
                    .parents
                    .len()
                    .checked_sub(relative_depth as usize)
                    .and_then(|i| i.checked_sub(1))
                    .ok_or_else(|| {
                        BinaryReaderError::new("relative depth too large", self.offset)
                    })?;
                let module = match self.parents[i].state.submodules.get(index as usize) {
                    Some(m) => *m,
                    None => return self.create_error("alias to module not defined in parent yet"),
                };
                self.cur.state.assert_mut().submodules.push(module);
            }
        }

        Ok(())
    }

    /// Validates [`Payload::InstanceSection`](crate::Payload)
    pub fn instance_section(&mut self, section: &crate::InstanceSectionReader<'_>) -> Result<()> {
        if !self.features.module_linking {
            return self.create_error("module linking proposal not enabled");
        }
        self.check_max(
            self.cur.state.instances.len(),
            section.get_count(),
            MAX_WASM_INSTANCES,
            "instances",
        )?;
        self.section(Order::ModuleLinkingHeader, section, |me, i| me.instance(i))
    }

    fn instance(&mut self, instance: Instance<'_>) -> Result<()> {
        // Fetch the type of the instantiated module so we can typecheck all of
        // the import items.
        let module_idx = instance.module();

        // Build up the set of what we're providing as imports.
        let mut set = NameSet::default();
        for arg in instance.args()? {
            let arg = arg?;
            let ty = self.check_external_kind("instance argument", arg.kind, arg.index)?;
            set.push(self.offset, arg.name, None, ty, &mut self.types, "arg")?;
        }

        // Check our provided `set` to ensure it's a subtype of the expected set
        // of imports from the module's import types.
        let ty = self.get_module_type(module_idx)?;
        self.check_type_sets_match(&set.set, &ty.imports, "import")?;

        // Create a synthetic type declaration for this instance's type and
        // record its type in the global type list. We might not have another
        // `TypeDef::Instance` to point to if the module was locally declared.
        //
        // Note that the predicted size of this type is inflated due to
        // accounting for the imports on the original module, but that should be
        // ok for now since it's only used to limit the size of larger types.
        let instance_ty = InstanceType {
            type_size: ty.exports_size,
            exports: ty.exports.clone(),
        };
        self.cur.state.assert_mut().instances.push(self.types.len());
        self.types.push(TypeDef::Instance(instance_ty));
        Ok(())
    }

    // Note that this function is basically implementing
    // https://webassembly.github.io/spec/core/exec/modules.html#import-matching
    fn check_subtypes(&self, a: &EntityType, b: &EntityType) -> Result<()> {
        macro_rules! limits_match {
            ($a:expr, $b:expr) => {{
                let a = $a;
                let b = $b;
                a.initial >= b.initial
                    && match b.maximum {
                        Some(b_max) => match a.maximum {
                            Some(a_max) => a_max <= b_max,
                            None => false,
                        },
                        None => true,
                    }
            }};
        }
        match a {
            EntityType::Global(a) => {
                let b = match b {
                    EntityType::Global(b) => b,
                    _ => return self.create_error("item type mismatch"),
                };
                if a == b {
                    Ok(())
                } else {
                    self.create_error("global type mismatch")
                }
            }
            EntityType::Table(a) => {
                let b = match b {
                    EntityType::Table(b) => b,
                    _ => return self.create_error("item type mismatch"),
                };
                if a.element_type == b.element_type && limits_match!(a, b) {
                    Ok(())
                } else {
                    self.create_error("table type mismatch")
                }
            }
            EntityType::Func(a) => {
                let b = match b {
                    EntityType::Func(b) => b,
                    _ => return self.create_error("item type mismatch"),
                };
                if self.types[*a].unwrap_func() == self.types[*b].unwrap_func() {
                    Ok(())
                } else {
                    self.create_error("func type mismatch")
                }
            }
            EntityType::Tag(a) => {
                let b = match b {
                    EntityType::Tag(b) => b,
                    _ => return self.create_error("item type mismatch"),
                };
                if self.types[*a].unwrap_func() == self.types[*b].unwrap_func() {
                    Ok(())
                } else {
                    self.create_error("tag type mismatch")
                }
            }
            EntityType::Memory(a) => {
                let b = match b {
                    EntityType::Memory(b) => b,
                    _ => return self.create_error("item type mismatch"),
                };
                if limits_match!(a, b) && a.shared == b.shared && a.memory64 == b.memory64 {
                    Ok(())
                } else {
                    self.create_error("memory type mismatch")
                }
            }
            EntityType::Instance(a) => {
                let b = match b {
                    EntityType::Instance(b) => b,
                    _ => return self.create_error("item type mismatch"),
                };
                let a = self.types[*a].unwrap_instance();
                let b = self.types[*b].unwrap_instance();
                self.check_type_sets_match(&a.exports, &b.exports, "export")?;
                Ok(())
            }
            EntityType::Module(a) => {
                let b = match b {
                    EntityType::Module(b) => b,
                    _ => return self.create_error("item type mismatch"),
                };
                let a = self.types[*a].unwrap_module();
                let b = self.types[*b].unwrap_module();
                // Note the order changing between imports and exports! This is
                // a live demonstration of variance in action.
                self.check_type_sets_match(&b.imports, &a.imports, "import")?;
                self.check_type_sets_match(&a.exports, &b.exports, "export")?;
                Ok(())
            }
        }
    }

    fn check_type_sets_match(
        &self,
        a: &HashMap<String, EntityType>,
        b: &HashMap<String, EntityType>,
        desc: &str,
    ) -> Result<()> {
        for (name, b) in b {
            match a.get(name) {
                Some(a) => self.check_subtypes(a, b)?,
                None => return self.create_error(&format!("no {} named `{}`", desc, name)),
            }
        }
        Ok(())
    }

    /// Validates [`Payload::FunctionSection`](crate::Payload)
    pub fn function_section(&mut self, section: &crate::FunctionSectionReader<'_>) -> Result<()> {
        self.cur.expected_code_bodies = Some(section.get_count());
        self.check_max(
            self.cur.state.func_types.len(),
            section.get_count(),
            MAX_WASM_FUNCTIONS,
            "funcs",
        )?;
        // Assert that each type index is indeed a function type, and otherwise
        // just push it for handling later.
        self.section(Order::Function, section, |me, i| {
            me.func_type_at(i)?;
            let state = me.cur.state.assert_mut();
            state.func_types.push(state.types[i as usize]);
            state.code_type_indexes.push(i);
            Ok(())
        })
    }

    fn max_tables(&self) -> usize {
        if self.features.reference_types || self.features.module_linking {
            MAX_WASM_TABLES
        } else {
            1
        }
    }

    /// Validates [`Payload::TableSection`](crate::Payload)
    pub fn table_section(&mut self, section: &crate::TableSectionReader<'_>) -> Result<()> {
        self.check_max(
            self.cur.state.tables.len(),
            section.get_count(),
            self.max_tables(),
            "tables",
        )?;
        self.section(Order::Table, section, |me, ty| {
            me.table_type(&ty)?;
            me.cur.state.assert_mut().tables.push(ty);
            Ok(())
        })
    }

    fn max_memories(&self) -> usize {
        if self.features.multi_memory {
            MAX_WASM_MEMORIES
        } else {
            1
        }
    }

    pub fn memory_section(&mut self, section: &crate::MemorySectionReader<'_>) -> Result<()> {
        self.check_max(
            self.cur.state.memories.len(),
            section.get_count(),
            self.max_memories(),
            "memories",
        )?;
        self.section(Order::Memory, section, |me, ty| {
            me.memory_type(&ty)?;
            me.cur.state.assert_mut().memories.push(ty);
            Ok(())
        })
    }

    pub fn tag_section(&mut self, section: &crate::TagSectionReader<'_>) -> Result<()> {
        if !self.features.exceptions {
            return self.create_error("exceptions proposal not enabled");
        }
        self.check_max(
            self.cur.state.tags.len(),
            section.get_count(),
            MAX_WASM_TAGS,
            "tags",
        )?;
        self.section(Order::Tag, section, |me, ty| {
            me.tag_type(&ty)?;
            let state = me.cur.state.assert_mut();
            state.tags.push(state.types[ty.type_index as usize]);
            Ok(())
        })
    }

    /// Validates [`Payload::GlobalSection`](crate::Payload)
    pub fn global_section(&mut self, section: &crate::GlobalSectionReader<'_>) -> Result<()> {
        self.check_max(
            self.cur.state.globals.len(),
            section.get_count(),
            MAX_WASM_GLOBALS,
            "globals",
        )?;
        self.section(Order::Global, section, |me, g| {
            me.global_type(&g.ty)?;
            me.init_expr(&g.init_expr, g.ty.content_type)?;
            me.cur.state.assert_mut().globals.push(g.ty);
            Ok(())
        })
    }

    fn init_expr(&mut self, expr: &InitExpr<'_>, expected_ty: Type) -> Result<()> {
        let mut ops = expr.get_operators_reader();
        let mut validator = OperatorValidator::new_init_expr(&self.features, expected_ty);
        let mut uninserted_function_references = false;

        while !ops.eof() {
            let offset = ops.original_position();
            self.offset = offset;
            let op = ops.read()?;
            match &op {
                // These are always valid in const expressions.
                Operator::I32Const { .. }
                | Operator::I64Const { .. }
                | Operator::F32Const { .. }
                | Operator::F64Const { .. }
                | Operator::RefNull { .. }
                | Operator::V128Const { .. }
                | Operator::End => {}

                // These are valid const expressions when the extended-const proposal is enabled.
                Operator::I32Add
                | Operator::I32Sub
                | Operator::I32Mul
                | Operator::I64Add
                | Operator::I64Sub
                | Operator::I64Mul
                    if self.features.extended_const => {}

                // `global.get` is a valid const expression for imported, immutable globals.
                Operator::GlobalGet { global_index } => {
                    let global = self.get_global(*global_index)?;
                    if *global_index >= self.cur.state.num_imported_globals {
                        return self.create_error(
                            "constant expression required: global.get of locally defined global",
                        );
                    }
                    if global.mutable {
                        return self.create_error(
                            "constant expression required: global.get of mutable global",
                        );
                    }
                }

                // Functions in initialization expressions are only valid in
                // element segment initialization expressions and globals. In
                // these contexts we want to record all function references.
                //
                // Initialization expressions can also be found in the data
                // section, however. A `RefFunc` instruction in those situations
                // is always invalid and needs to produce a validation error. In
                // this situation, though, we can no longer modify
                // `self.cur.state` since it's been "snapshot" already for
                // parallel validation of functions.
                //
                // Consequently we test where we are in the module to determine
                // whether we can modify the set of function references. If we
                // cannot modify the function references then this function
                // *should* result in a validation error, but we defer that
                // validation error to happen later. The
                // `uninserted_function_references` boolean here is used to
                // track this and will cause a panic (aka a fuzz bug) if we
                // somehow forget to emit an error somewhere else.
                Operator::RefFunc { function_index } => {
                    if self.cur.order <= Order::Code {
                        self.cur
                            .state
                            .assert_mut()
                            .function_references
                            .insert(*function_index);
                    } else {
                        uninserted_function_references = true;
                    }
                }
                _ => {
                    return self
                        .create_error("constant expression required: invalid init_expr operator")
                }
            }
            validator
                .process_operator(&op, self)
                .map_err(|e| e.set_offset(offset))?;
        }
        validator.finish().map_err(|e| e.set_offset(self.offset))?;

        // See comment in `RefFunc` above for why this is an assert.
        assert!(!uninserted_function_references);

        Ok(())
    }

    /// Validates [`Payload::ExportSection`](crate::Payload)
    pub fn export_section(&mut self, section: &crate::ExportSectionReader<'_>) -> Result<()> {
        self.section(Order::Export, section, |me, e| {
            if let ExternalKind::Type = e.kind {
                return me.create_error("cannot export types");
            }
            let ty = me.check_external_kind("exported", e.kind, e.index)?;
            if !me.features.mutable_global {
                if let EntityType::Global(global_type) = ty {
                    if global_type.mutable {
                        return me.create_error("mutable global support is not enabled");
                    }
                }
            }
            let state = me.cur.state.assert_mut();
            state
                .exports
                .push(me.offset, e.field, None, ty, &mut me.types, "export")?;
            Ok(())
        })
    }

    fn check_external_kind(
        &mut self,
        desc: &str,
        kind: ExternalKind,
        index: u32,
    ) -> Result<EntityType> {
        let check = |ty: &str, total: usize| {
            if index as usize >= total {
                self.create_error(&format!(
                    "unknown {ty} {index}: {desc} {ty} index out of bounds",
                    desc = desc,
                    index = index,
                    ty = ty,
                ))
            } else {
                Ok(())
            }
        };
        Ok(match kind {
            ExternalKind::Function => {
                check("function", self.cur.state.func_types.len())?;
                self.cur
                    .state
                    .assert_mut()
                    .function_references
                    .insert(index);
                EntityType::Func(self.cur.state.func_types[index as usize])
            }
            ExternalKind::Table => {
                check("table", self.cur.state.tables.len())?;
                EntityType::Table(self.cur.state.tables[index as usize])
            }
            ExternalKind::Memory => {
                check("memory", self.cur.state.memories.len())?;
                EntityType::Memory(self.cur.state.memories[index as usize])
            }
            ExternalKind::Global => {
                check("global", self.cur.state.globals.len())?;
                EntityType::Global(self.cur.state.globals[index as usize])
            }
            ExternalKind::Tag => {
                check("tag", self.cur.state.tags.len())?;
                EntityType::Tag(self.cur.state.tags[index as usize])
            }
            ExternalKind::Module => {
                check("module", self.cur.state.submodules.len())?;
                EntityType::Module(self.cur.state.submodules[index as usize])
            }
            ExternalKind::Instance => {
                check("instance", self.cur.state.instances.len())?;
                EntityType::Instance(self.cur.state.instances[index as usize])
            }
            ExternalKind::Type => return self.create_error("cannot export types"),
        })
    }

    /// Validates [`Payload::StartSection`](crate::Payload)
    pub fn start_section(&mut self, func: u32, range: &Range) -> Result<()> {
        self.offset = range.start;
        self.update_order(Order::Start)?;
        let ty = self.get_func_type(func)?;
        if !ty.params.is_empty() || !ty.returns.is_empty() {
            return self.create_error("invalid start function type");
        }
        Ok(())
    }

    /// Validates [`Payload::ElementSection`](crate::Payload)
    pub fn element_section(&mut self, section: &crate::ElementSectionReader<'_>) -> Result<()> {
        self.section(Order::Element, section, |me, e| {
            match e.ty {
                Type::FuncRef => {}
                Type::ExternRef if me.features.reference_types => {}
                Type::ExternRef => {
                    return me
                        .create_error("reference types must be enabled for anyref elem segment");
                }
                _ => return me.create_error("invalid reference type"),
            }
            match e.kind {
                ElementKind::Active {
                    table_index,
                    init_expr,
                } => {
                    let table = me.get_table(table_index)?;
                    if e.ty != table.element_type {
                        return me.create_error("element_type != table type");
                    }
                    me.init_expr(&init_expr, Type::I32)?;
                }
                ElementKind::Passive | ElementKind::Declared => {
                    if !me.features.bulk_memory {
                        return me.create_error("bulk memory must be enabled");
                    }
                }
            }
            let mut items = e.items.get_items_reader()?;
            if items.get_count() > MAX_WASM_TABLE_ENTRIES as u32 {
                return me.create_error("num_elements is out of bounds");
            }
            for _ in 0..items.get_count() {
                me.offset = items.original_position();
                match items.read()? {
                    ElementItem::Expr(expr) => {
                        me.init_expr(&expr, e.ty)?;
                    }
                    ElementItem::Func(f) => {
                        if e.ty != Type::FuncRef {
                            return me
                                .create_error("type mismatch: segment does not have funcref type");
                        }
                        me.get_func_type(f)?;
                        me.cur.state.assert_mut().function_references.insert(f);
                    }
                }
            }

            me.cur.state.assert_mut().element_types.push(e.ty);
            Ok(())
        })
    }

    /// Validates [`Payload::DataCountSection`](crate::Payload)
    pub fn data_count_section(&mut self, count: u32, range: &Range) -> Result<()> {
        self.offset = range.start;
        self.update_order(Order::DataCount)?;
        self.cur.state.assert_mut().data_count = Some(count);
        if count > MAX_WASM_DATA_SEGMENTS as u32 {
            return self.create_error("data count section specifies too many data segments");
        }
        Ok(())
    }

    /// Validates [`Payload::CodeSectionStart`](crate::Payload).
    pub fn code_section_start(&mut self, count: u32, range: &Range) -> Result<()> {
        self.offset = range.start;
        self.update_order(Order::Code)?;
        match self.cur.expected_code_bodies.take() {
            Some(n) if n == count => {}
            Some(_) => {
                return self.create_error("function and code section have inconsistent lengths");
            }
            // empty code sections are allowed even if the function section is
            // missing
            None if count == 0 => {}
            None => return self.create_error("code section without function section"),
        }

        // Prepare our module's view into the global `types` array. This enables
        // parallel function validation to accesss our built-so-far list off
        // types. Note that all the `WasmModuleResources` methods rely on
        // `all_types` being filled in, and this is the point at which they're
        // filled in.
        let types = self.types.commit();
        self.cur.state.assert_mut().all_types = Some(Arc::new(types));

        Ok(())
    }

    /// Validates [`Payload::CodeSectionEntry`](crate::Payload).
    ///
    /// This function will prepare a [`FuncValidator`] which can be used to
    /// validate the function. The function body provided will be parsed only
    /// enough to create the function validation context. After this the
    /// [`OperatorsReader`](crate::readers::OperatorsReader) returned can be used to read the
    /// opcodes of the function as well as feed information into the validator.
    ///
    /// Note that the returned [`FuncValidator`] is "connected" to this
    /// [`Validator`] in that it uses the internal context of this validator for
    /// validating the function. The [`FuncValidator`] can be sent to
    /// another thread, for example, to offload actual processing of functions
    /// elsewhere.
    pub fn code_section_entry(&mut self) -> Result<FuncValidator<ValidatorResources>> {
        let ty = self.cur.state.code_type_indexes[self.cur.code_section_index];
        self.cur.code_section_index += 1;
        let resources = ValidatorResources(self.cur.state.arc().clone());
        Ok(FuncValidator::new(ty, 0, resources, &self.features).unwrap())
    }

    /// Validates [`Payload::DataSection`](crate::Payload).
    pub fn data_section(&mut self, section: &crate::DataSectionReader<'_>) -> Result<()> {
        self.cur.data_found = section.get_count();
        self.check_max(0, section.get_count(), MAX_WASM_DATA_SEGMENTS, "segments")?;
        let mut section = section.clone();
        section.forbid_bulk_memory(!self.features.bulk_memory);
        self.section(Order::Data, &section, |me, d| {
            match d.kind {
                DataKind::Passive => {}
                DataKind::Active {
                    memory_index,
                    init_expr,
                } => {
                    let ty = me.get_memory(memory_index)?.index_type();
                    me.init_expr(&init_expr, ty)?;
                }
            }
            Ok(())
        })
    }

    /// Validates [`Payload::UnknownSection`](crate::Payload).
    ///
    /// Currently always returns an error.
    pub fn unknown_section(&mut self, id: u8, range: &Range) -> Result<()> {
        self.offset = range.start;
        self.create_error(format!("invalid section code: {}", id))
    }

    /// Validates [`Payload::End`](crate::Payload).
    pub fn end(&mut self) -> Result<()> {
        // Ensure that the data count section, if any, was correct.
        if let Some(data_count) = self.cur.state.data_count {
            if data_count != self.cur.data_found {
                return self.create_error("data count section and passive data mismatch");
            }
        }
        // Ensure that the function section, if nonzero, was paired with a code
        // section with the appropriate length.
        if let Some(n) = self.cur.expected_code_bodies.take() {
            if n > 0 {
                return self.create_error("function and code sections have inconsistent lengths");
            }
        }

        // Ensure that the effective type size of this module is of a bounded
        // size. This is primarily here for the module linking proposal, and
        // we'll record this in the module type below if we're part of a nested
        // module.
        combine_type_sizes(
            self.offset,
            self.cur.state.imports.type_size,
            self.cur.state.exports.type_size,
        )?;

        // If we have a parent then we're going to exit this module's context
        // and resume where we left off in the parent. We inject a new type for
        // our module we just validated in the parent's module index space, and
        // then we reset our current state to the parent.
        if let Some(mut parent) = self.parents.pop() {
            let module_type = self.types.len();
            self.types.push(TypeDef::Module(ModuleType {
                imports_size: self.cur.state.imports.type_size,
                exports_size: self.cur.state.exports.type_size,
                imports: self.cur.state.imports.set.clone(),
                exports: self.cur.state.exports.set.clone(),
            }));
            parent.state.assert_mut().submodules.push(module_type);
            self.cur = parent;
        }
        Ok(())
    }
}

impl WasmModuleResources for Validator {
    type FuncType = crate::FuncType;

    fn table_at(&self, at: u32) -> Option<TableType> {
        self.cur.state.table_at(at)
    }

    fn memory_at(&self, at: u32) -> Option<MemoryType> {
        self.cur.state.memory_at(at)
    }

    fn tag_at(&self, at: u32) -> Option<&Self::FuncType> {
        self.cur.state.tag_at(at)
    }

    fn global_at(&self, at: u32) -> Option<GlobalType> {
        self.cur.state.global_at(at)
    }

    fn func_type_at(&self, type_idx: u32) -> Option<&Self::FuncType> {
        self.cur.state.func_type_at(type_idx)
    }

    fn type_of_function(&self, func_idx: u32) -> Option<&Self::FuncType> {
        self.get_func_type(func_idx).ok()
    }

    fn element_type_at(&self, at: u32) -> Option<Type> {
        self.cur.state.element_type_at(at)
    }

    fn element_count(&self) -> u32 {
        self.cur.state.element_count()
    }

    fn data_count(&self) -> u32 {
        self.cur.state.data_count()
    }

    fn is_function_referenced(&self, idx: u32) -> bool {
        self.cur.state.is_function_referenced(idx)
    }
}

fn combine_type_sizes(offset: usize, a: u32, b: u32) -> Result<u32> {
    match a.checked_add(b) {
        Some(sum) if sum < MAX_TYPE_SIZE => Ok(sum),
        _ => Err(BinaryReaderError::new(
            "effective type size too large".to_string(),
            offset,
        )),
    }
}

impl WasmFeatures {
    pub(crate) fn check_value_type(&self, ty: Type) -> Result<(), &'static str> {
        match ty {
            Type::I32 | Type::I64 | Type::F32 | Type::F64 => Ok(()),
            Type::FuncRef | Type::ExternRef => {
                if self.reference_types {
                    Ok(())
                } else {
                    Err("reference types support is not enabled")
                }
            }
            Type::ExnRef => {
                if self.exceptions {
                    Ok(())
                } else {
                    Err("exceptions support is not enabled")
                }
            }
            Type::V128 => {
                if self.simd {
                    Ok(())
                } else {
                    Err("SIMD support is not enabled")
                }
            }
            _ => Err("invalid value type"),
        }
    }
}

impl WasmModuleResources for ModuleState {
    type FuncType = crate::FuncType;

    fn table_at(&self, at: u32) -> Option<TableType> {
        self.tables.get(at as usize).cloned()
    }

    fn memory_at(&self, at: u32) -> Option<MemoryType> {
        self.memories.get(at as usize).cloned()
    }

    fn tag_at(&self, at: u32) -> Option<&Self::FuncType> {
        let types = self.all_types.as_ref().unwrap();
        let i = *self.tags.get(at as usize)?;
        match &types[i] {
            TypeDef::Func(f) => Some(f),
            _ => None,
        }
    }

    fn global_at(&self, at: u32) -> Option<GlobalType> {
        self.globals.get(at as usize).cloned()
    }

    fn func_type_at(&self, at: u32) -> Option<&Self::FuncType> {
        let types = self.all_types.as_ref().unwrap();
        let i = *self.types.get(at as usize)?;
        match &types[i] {
            TypeDef::Func(f) => Some(f),
            _ => None,
        }
    }

    fn type_of_function(&self, at: u32) -> Option<&Self::FuncType> {
        let types = self.all_types.as_ref().unwrap();
        let i = *self.func_types.get(at as usize)?;
        match &types[i] {
            TypeDef::Func(f) => Some(f),
            _ => None,
        }
    }

    fn element_type_at(&self, at: u32) -> Option<Type> {
        self.element_types.get(at as usize).cloned()
    }

    fn element_count(&self) -> u32 {
        self.element_types.len() as u32
    }

    fn data_count(&self) -> u32 {
        self.data_count.unwrap_or(0)
    }

    fn is_function_referenced(&self, idx: u32) -> bool {
        self.function_references.contains(&idx)
    }
}

/// The implementation of [`WasmModuleResources`] used by [`Validator`].
pub struct ValidatorResources(Arc<ModuleState>);

impl WasmModuleResources for ValidatorResources {
    type FuncType = crate::FuncType;

    fn table_at(&self, at: u32) -> Option<TableType> {
        self.0.table_at(at)
    }

    fn memory_at(&self, at: u32) -> Option<MemoryType> {
        self.0.memory_at(at)
    }

    fn tag_at(&self, at: u32) -> Option<&Self::FuncType> {
        self.0.tag_at(at)
    }

    fn global_at(&self, at: u32) -> Option<GlobalType> {
        self.0.global_at(at)
    }

    fn func_type_at(&self, at: u32) -> Option<&Self::FuncType> {
        self.0.func_type_at(at)
    }

    fn type_of_function(&self, at: u32) -> Option<&Self::FuncType> {
        self.0.type_of_function(at)
    }

    fn element_type_at(&self, at: u32) -> Option<Type> {
        self.0.element_type_at(at)
    }

    fn element_count(&self) -> u32 {
        self.0.element_count()
    }

    fn data_count(&self) -> u32 {
        self.0.data_count()
    }

    fn is_function_referenced(&self, idx: u32) -> bool {
        self.0.is_function_referenced(idx)
    }
}

mod arc {
    use std::ops::Deref;
    use std::sync::Arc;

    pub struct MaybeOwned<T> {
        owned: bool,
        arc: Arc<T>,
    }

    impl<T> MaybeOwned<T> {
        fn as_mut(&mut self) -> Option<&mut T> {
            if !self.owned {
                return None;
            }
            debug_assert!(Arc::get_mut(&mut self.arc).is_some());
            Some(unsafe { &mut *(Arc::as_ptr(&self.arc) as *mut T) })
        }

        pub fn assert_mut(&mut self) -> &mut T {
            self.as_mut().unwrap()
        }

        pub fn arc(&mut self) -> &Arc<T> {
            self.owned = false;
            &self.arc
        }
    }

    impl<T: Default> Default for MaybeOwned<T> {
        fn default() -> MaybeOwned<T> {
            MaybeOwned {
                owned: true,
                arc: Arc::default(),
            }
        }
    }

    impl<T> Deref for MaybeOwned<T> {
        type Target = T;

        fn deref(&self) -> &T {
            &self.arc
        }
    }
}

/// This is a helper structure to create a map from names to types.
///
/// The main purpose of this structure is to handle the two-level imports found
/// in wasm modules pre-module-linking. A two-level import is equivalent to a
/// single-level import of an instance, and that mapping happens here.
#[derive(Default)]
struct NameSet {
    set: HashMap<String, EntityType>,
    implicit: HashSet<String>,
    type_size: u32,
}

impl NameSet {
    /// Pushes a new name into this typed set off names, internally handling the
    /// mapping of two-level namespaces into a single-level namespace.
    ///
    /// * `offset` - the binary offset in the original wasm file of where to
    ///   report errors about.
    /// * `module` - the first-level name in the namespace
    /// * `name` - the optional second-level namespace
    /// * `ty` - the type of the item being pushed
    /// * `types` - our global list of types
    /// * `desc` - a human-readable description of the item being pushed, used
    ///   for generating errors.
    ///
    /// Returns an error if the name was a duplicate. Returns `Ok(Some(idx))` if
    /// this push was the first push to define an implicit instance with the
    /// type `idx` into the global list of types. Returns `Ok(None)` otherwise.
    fn push(
        &mut self,
        offset: usize,
        module: &str,
        name: Option<&str>,
        ty: EntityType,
        types: &mut SnapshotList<TypeDef>,
        desc: &str,
    ) -> Result<Option<usize>> {
        self.type_size =
            combine_type_sizes(offset, self.type_size, ty.size(types).saturating_add(1))?;
        let name = match name {
            Some(name) => name,
            // If the `name` is not provided then this is a module-linking style
            // definition with only one level of a name rather than two. This
            // means we can insert the `ty` into the set of imports directly,
            // error-ing out on duplicates.
            None => {
                let prev = self.set.insert(module.to_string(), ty);
                return if prev.is_some() {
                    Err(BinaryReaderError::new(
                        format!("duplicate {} name `{}` already defined", desc, module),
                        offset,
                    ))
                } else {
                    Ok(None)
                };
            }
        };
        match self.set.get(module) {
            // If `module` was previously defined and we implicitly defined it,
            // then we know that it points to an instance type. We update the
            // set of exports of the instance type here since we're adding a new
            // entry. Note that nothing, at the point that we're building this
            // set, should point to this instance type so it should be safe to
            // mutate.
            Some(instance) if self.implicit.contains(module) => {
                let instance = match instance {
                    EntityType::Instance(i) => match &mut types[*i] {
                        TypeDef::Instance(i) => i,
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                };
                let prev = instance.exports.insert(name.to_string(), ty);
                if prev.is_some() {
                    return Err(BinaryReaderError::new(
                        format!(
                            "duplicate {} name `{}::{}` already defined",
                            desc, module, name
                        ),
                        offset,
                    ));
                }
                Ok(None)
            }

            // Otherwise `module` was previously defined, but it *wasn't*
            // implicitly defined through a two level import (rather was
            // explicitly defined with a single-level import), then that's an
            // error.
            Some(_) => {
                return Err(BinaryReaderError::new(
                    format!("cannot define the {} `{}` twice", desc, module),
                    offset,
                ))
            }

            // And finally if `module` wasn't already defined then we go ahead
            // and define it here as a instance type with a single export, our
            // `name`.
            None => {
                let idx = types.len();
                let mut instance = InstanceType::default();
                instance.exports.insert(name.to_string(), ty);
                types.push(TypeDef::Instance(instance));
                assert!(self.implicit.insert(module.to_string()));
                self.set
                    .insert(module.to_string(), EntityType::Instance(idx));
                Ok(Some(idx))
            }
        }
    }
}

impl EntityType {
    fn size(&self, list: &SnapshotList<TypeDef>) -> u32 {
        match self {
            // Note that this function computes the size of the *type*, not the
            // size of the value, so these "leaves" all count as 1
            EntityType::Global(_) | EntityType::Memory(_) | EntityType::Table(_) => 1,

            // These types have recursive sizes so we look up the size in the
            // type tables.
            EntityType::Func(i)
            | EntityType::Module(i)
            | EntityType::Instance(i)
            | EntityType::Tag(i) => match &list[*i] {
                TypeDef::Func(f) => 1 + (f.params.len() + f.returns.len()) as u32,
                TypeDef::Module(m) => m.imports_size + m.exports_size,
                TypeDef::Instance(i) => i.type_size,
            },
        }
    }
}

/// This is a type which mirrors a subset of the `Vec<T>` API, but is intended
/// to be able to be cheaply snapshotted and cloned.
///
/// When each module's code sections start we "commit" the current list of types
/// in the global list of types. This means that the temporary `cur` vec here is
/// pushed onto `snapshots` and wrapped up in an `Arc`. At that point we clone
/// this entire list (which is then O(modules), not O(types in all modules)) and
/// pass out as a context to each function validator.
///
/// Otherwise, though, this type behaves as if it were a large `Vec<T>`, but
/// it's represented by lists of contiguous chunks.
struct SnapshotList<T> {
    // All previous snapshots, the "head" of the list that this type represents.
    // The first entry in this pair is the starting index for all elements
    // contained in the list, and the second element is the list itself. Note
    // the `Arc` wrapper around sub-lists, which makes cloning time for this
    // `SnapshotList` O(snapshots) rather than O(snapshots_total), which for
    // us in this context means the number of modules, not types.
    //
    // Note that this list is sorted least-to-greatest in order of the index for
    // binary searching.
    snapshots: Vec<(usize, Arc<Vec<T>>)>,

    // This is the total length of all lists in the `snapshots` array.
    snapshots_total: usize,

    // The current list of types for the current snapshot that are being built.
    cur: Vec<T>,
}

impl<T> SnapshotList<T> {
    /// Same as `<&[T]>::get`
    fn get(&self, index: usize) -> Option<&T> {
        // Check to see if this index falls on our local list
        if index >= self.snapshots_total {
            return self.cur.get(index - self.snapshots_total);
        }
        // ... and failing that we do a binary search to figure out which bucket
        // it's in. Note the `i-1` in the `Err` case because if we don't find an
        // exact match the type is located in the previous bucket.
        let i = match self.snapshots.binary_search_by_key(&index, |(i, _)| *i) {
            Ok(i) => i,
            Err(i) => i - 1,
        };
        let (len, list) = &self.snapshots[i];
        Some(&list[index - len])
    }

    /// Same as `<&mut [T]>::get_mut`, except only works for indexes into the
    /// current snapshot being built.
    ///
    /// # Panics
    ///
    /// Panics if an index is passed in which falls within the
    /// previously-snapshotted list of types. This should never happen in our
    /// contesxt and the panic is intended to weed out possible bugs in
    /// wasmparser.
    fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.snapshots_total {
            return self.cur.get_mut(index - self.snapshots_total);
        }
        panic!("cannot get a mutable reference in snapshotted part of list")
    }

    /// Same as `Vec::push`
    fn push(&mut self, val: T) {
        self.cur.push(val);
    }

    /// Same as `<[T]>::len`
    fn len(&self) -> usize {
        self.cur.len() + self.snapshots_total
    }

    /// Commits previously pushed types into this snapshot vector, and returns a
    /// clone of this list.
    ///
    /// The returned `SnapshotList` can be used to access all the same types as
    /// this list itself. This list also is not changed (from an external
    /// perspective) and can continue to access all the same types.
    fn commit(&mut self) -> SnapshotList<T> {
        // If the current chunk has new elements, commit them in to an
        // `Arc`-wrapped vector in the snapshots list. Note the `shrink_to_fit`
        // ahead of time to hopefully keep memory usage lower than it would
        // otherwise be.
        let len = self.cur.len();
        if len > 0 {
            self.cur.shrink_to_fit();
            self.snapshots
                .push((self.snapshots_total, Arc::new(mem::take(&mut self.cur))));
            self.snapshots_total += len;
        }
        SnapshotList {
            snapshots: self.snapshots.clone(),
            snapshots_total: self.snapshots_total,
            cur: Vec::new(),
        }
    }
}

impl<T> std::ops::Index<usize> for SnapshotList<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        self.get(index).unwrap()
    }
}

impl<T> std::ops::IndexMut<usize> for SnapshotList<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        self.get_mut(index).unwrap()
    }
}

impl<T> Default for SnapshotList<T> {
    fn default() -> SnapshotList<T> {
        SnapshotList {
            snapshots: Vec::new(),
            snapshots_total: 0,
            cur: Vec::new(),
        }
    }
}
