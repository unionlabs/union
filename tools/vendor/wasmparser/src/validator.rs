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
    limits::*, BinaryReaderError, Encoding, FunctionBody, Parser, Payload, Result, SectionReader,
    SectionWithLimitedItems, ValType, WASM_COMPONENT_VERSION, WASM_MODULE_VERSION,
};
use std::mem;
use std::ops::Range;
use std::sync::Arc;

/// Test whether the given buffer contains a valid WebAssembly module or component,
/// analogous to [`WebAssembly.validate`][js] in the JS API.
///
/// This functions requires the bytes to validate are entirely resident in memory.
/// Additionally this validates the given bytes with the default set of WebAssembly
/// features implemented by `wasmparser`.
///
/// For more fine-tuned control over validation it's recommended to review the
/// documentation of [`Validator`].
///
/// Upon success, the type information for the top-level module or component will
/// be returned.
///
/// [js]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/validate
pub fn validate(bytes: &[u8]) -> Result<Types> {
    Validator::new().validate_all(bytes)
}

#[test]
fn test_validate() {
    assert!(validate(&[0x0, 0x61, 0x73, 0x6d, 0x1, 0x0, 0x0, 0x0]).is_ok());
    assert!(validate(&[0x0, 0x61, 0x73, 0x6d, 0x2, 0x0, 0x0, 0x0]).is_err());
}

mod component;
mod core;
mod func;
mod operators;
pub mod types;

use self::component::*;
pub use self::core::ValidatorResources;
use self::core::*;
use self::types::{TypeList, Types, TypesRef};
pub use func::{FuncToValidate, FuncValidator, FuncValidatorAllocations};
pub use operators::{Frame, FrameKind};

fn check_max(cur_len: usize, amt_added: u32, max: usize, desc: &str, offset: usize) -> Result<()> {
    if max
        .checked_sub(cur_len)
        .and_then(|amt| amt.checked_sub(amt_added as usize))
        .is_none()
    {
        if max == 1 {
            bail!(offset, "multiple {desc}");
        }

        bail!(offset, "{desc} count exceeds limit of {max}");
    }

    Ok(())
}

fn combine_type_sizes(a: usize, b: usize, offset: usize) -> Result<usize> {
    match a.checked_add(b) {
        Some(sum) if sum < MAX_WASM_TYPE_SIZE => Ok(sum),
        _ => Err(format_err!(
            offset,
            "effective type size exceeds the limit of {MAX_WASM_TYPE_SIZE}",
        )),
    }
}

/// Validator for a WebAssembly binary module or component.
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
    /// The current state of the validator.
    state: State,

    /// The global type space used by the validator and any sub-validators.
    types: TypeList,

    /// The module state when parsing a WebAssembly module.
    module: Option<ModuleState>,

    /// With the component model enabled, this stores the pushed component states.
    /// The top of the stack is the current component state.
    components: Vec<ComponentState>,

    /// Enabled WebAssembly feature flags, dictating what's valid and what
    /// isn't.
    features: WasmFeatures,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum State {
    /// A header has not yet been parsed.
    ///
    /// The value is the expected encoding for the header.
    Unparsed(Option<Encoding>),
    /// A module header has been parsed.
    ///
    /// The associated module state is available via [`Validator::module`].
    Module,
    /// A component header has been parsed.
    ///
    /// The associated component state exists at the top of the
    /// validator's [`Validator::components`] stack.
    Component,
    /// The parse has completed and no more data is expected.
    End,
}

impl State {
    fn ensure_parsable(&self, offset: usize) -> Result<()> {
        match self {
            Self::Module | Self::Component => Ok(()),
            Self::Unparsed(_) => Err(BinaryReaderError::new(
                "unexpected section before header was parsed",
                offset,
            )),
            Self::End => Err(BinaryReaderError::new(
                "unexpected section after parsing has completed",
                offset,
            )),
        }
    }

    fn ensure_module(&self, section: &str, offset: usize) -> Result<()> {
        self.ensure_parsable(offset)?;

        match self {
            Self::Module => Ok(()),
            Self::Component => Err(format_err!(
                offset,
                "unexpected module {section} section while parsing a component",
            )),
            _ => unreachable!(),
        }
    }

    fn ensure_component(&self, section: &str, offset: usize) -> Result<()> {
        self.ensure_parsable(offset)?;

        match self {
            Self::Component => Ok(()),
            Self::Module => Err(format_err!(
                offset,
                "unexpected component {section} section while parsing a module",
            )),
            _ => unreachable!(),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::Unparsed(None)
    }
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
    /// The WebAssembly component model proposal.
    pub component_model: bool,
}

impl WasmFeatures {
    pub(crate) fn check_value_type(&self, ty: ValType) -> Result<(), &'static str> {
        match ty {
            ValType::I32 | ValType::I64 | ValType::F32 | ValType::F64 => Ok(()),
            ValType::FuncRef | ValType::ExternRef => {
                if self.reference_types {
                    Ok(())
                } else {
                    Err("reference types support is not enabled")
                }
            }
            ValType::V128 => {
                if self.simd {
                    Ok(())
                } else {
                    Err("SIMD support is not enabled")
                }
            }
        }
    }
}

impl Default for WasmFeatures {
    fn default() -> WasmFeatures {
        WasmFeatures {
            // off-by-default features
            relaxed_simd: false,
            threads: false,
            tail_call: false,
            multi_memory: false,
            exceptions: false,
            memory64: false,
            extended_const: false,
            component_model: false,
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

/// Possible return values from [`Validator::payload`].
#[allow(clippy::large_enum_variant)]
pub enum ValidPayload<'a> {
    /// The payload validated, no further action need be taken.
    Ok,
    /// The payload validated, but it started a nested module or component.
    ///
    /// This result indicates that the specified parser should be used instead
    /// of the currently-used parser until this returned one ends.
    Parser(Parser),
    /// A function was found to be validate.
    Func(FuncToValidate<ValidatorResources>, FunctionBody<'a>),
    /// The end payload was validated and the types known to the validator
    /// are provided.
    End(Types),
}

impl Validator {
    /// Creates a new [`Validator`] ready to validate a WebAssembly module
    /// or component.
    ///
    /// The new validator will receive payloads parsed from
    /// [`Parser`], and expects the first payload received to be
    /// the version header from the parser.
    pub fn new() -> Validator {
        Validator::default()
    }

    /// Creates a new [`Validator`] which has the specified set of wasm
    /// features activated for validation.
    ///
    /// This function is the same as [`Validator::new`] except it also allows
    /// you to customize the active wasm features in use for validation. This
    /// can allow enabling experimental proposals or also turning off
    /// on-by-default wasm proposals.
    pub fn new_with_features(features: WasmFeatures) -> Validator {
        let mut ret = Validator::new();
        ret.features = features;
        ret
    }

    /// Returns the wasm features used for this validator.
    pub fn features(&self) -> &WasmFeatures {
        &self.features
    }

    /// Validates an entire in-memory module or component with this validator.
    ///
    /// This function will internally create a [`Parser`] to parse the `bytes`
    /// provided. The entire module or component specified by `bytes` will be
    /// parsed and validated.
    ///
    /// Upon success, the type information for the top-level module or component
    /// will be returned.
    pub fn validate_all(&mut self, bytes: &[u8]) -> Result<Types> {
        let mut functions_to_validate = Vec::new();
        let mut last_types = None;
        for payload in Parser::new(0).parse_all(bytes) {
            match self.payload(&payload?)? {
                ValidPayload::Func(a, b) => {
                    functions_to_validate.push((a, b));
                }
                ValidPayload::End(types) => {
                    // Only the last (top-level) type information will be returned
                    last_types = Some(types);
                }
                _ => {}
            }
        }

        let mut allocs = FuncValidatorAllocations::default();
        for (func, body) in functions_to_validate {
            let mut validator = func.into_validator(allocs);
            validator.validate(&body)?;
            allocs = validator.into_allocations();
        }

        Ok(last_types.unwrap())
    }

    /// Gets the types known by the validator so far within the
    /// module/component `level` modules/components up from the
    /// module/component currently being parsed.
    ///
    /// For instance, calling `validator.types(0)` will get the types of the
    /// module/component currently being parsed, and `validator.types(1)` will
    /// get the types of the component containing that module/component.
    ///
    /// Returns `None` if there is no module/component that many levels up.
    pub fn types(&self, mut level: usize) -> Option<TypesRef> {
        if let Some(module) = &self.module {
            if level == 0 {
                return Some(TypesRef::from_module(&self.types, &module.module));
            } else {
                level -= 1;
            }
        }

        self.components
            .iter()
            .nth_back(level)
            .map(|component| TypesRef::from_component(&self.types, component))
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
            Version {
                num,
                encoding,
                range,
            } => self.version(*num, *encoding, range)?,

            // Module sections
            TypeSection(s) => self.type_section(s)?,
            ImportSection(s) => self.import_section(s)?,
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
                let func_validator = self.code_section_entry(body)?;
                return Ok(ValidPayload::Func(func_validator, *body));
            }
            DataSection(s) => self.data_section(s)?,

            // Component sections
            ModuleSection { parser, range, .. } => {
                self.module_section(range)?;
                return Ok(ValidPayload::Parser(parser.clone()));
            }
            InstanceSection(s) => self.instance_section(s)?,
            CoreTypeSection(s) => self.core_type_section(s)?,
            ComponentSection { parser, range, .. } => {
                self.component_section(range)?;
                return Ok(ValidPayload::Parser(parser.clone()));
            }
            ComponentInstanceSection(s) => self.component_instance_section(s)?,
            ComponentAliasSection(s) => self.component_alias_section(s)?,
            ComponentTypeSection(s) => self.component_type_section(s)?,
            ComponentCanonicalSection(s) => self.component_canonical_section(s)?,
            ComponentStartSection(s) => self.component_start_section(s)?,
            ComponentImportSection(s) => self.component_import_section(s)?,
            ComponentExportSection(s) => self.component_export_section(s)?,

            End(offset) => return Ok(ValidPayload::End(self.end(*offset)?)),

            CustomSection { .. } => {} // no validation for custom sections
            UnknownSection { id, range, .. } => self.unknown_section(*id, range)?,
        }
        Ok(ValidPayload::Ok)
    }

    /// Validates [`Payload::Version`](crate::Payload).
    pub fn version(&mut self, num: u32, encoding: Encoding, range: &Range<usize>) -> Result<()> {
        match &self.state {
            State::Unparsed(expected) => {
                if let Some(expected) = expected {
                    if *expected != encoding {
                        bail!(
                            range.start,
                            "expected a version header for a {}",
                            match expected {
                                Encoding::Module => "module",
                                Encoding::Component => "component",
                            }
                        );
                    }
                }
            }
            _ => {
                return Err(BinaryReaderError::new(
                    "wasm version header out of order",
                    range.start,
                ))
            }
        }

        self.state = match (encoding, num) {
            (Encoding::Module, WASM_MODULE_VERSION) => {
                assert!(self.module.is_none());
                self.module = Some(ModuleState::default());
                State::Module
            }
            (Encoding::Component, WASM_COMPONENT_VERSION) => {
                if !self.features.component_model {
                    return Err(BinaryReaderError::new(
                        "WebAssembly component model feature not enabled",
                        range.start,
                    ));
                }

                self.components.push(ComponentState::default());
                State::Component
            }
            _ => {
                return Err(BinaryReaderError::new(
                    "unknown binary version",
                    range.start,
                ));
            }
        };

        Ok(())
    }

    /// Validates [`Payload::TypeSection`](crate::Payload).
    pub fn type_section(&mut self, section: &crate::TypeSectionReader<'_>) -> Result<()> {
        self.process_module_section(
            Order::Type,
            section,
            "type",
            |state, _, types, count, offset| {
                check_max(
                    state.module.types.len(),
                    count,
                    MAX_WASM_TYPES,
                    "types",
                    offset,
                )?;
                types.reserve(count as usize);
                state.module.assert_mut().types.reserve(count as usize);
                Ok(())
            },
            |state, features, types, def, offset| {
                state
                    .module
                    .assert_mut()
                    .add_type(def, features, types, offset, false /* checked above */)
            },
        )
    }

    /// Validates [`Payload::ImportSection`](crate::Payload).
    ///
    /// This method should only be called when parsing a module.
    pub fn import_section(&mut self, section: &crate::ImportSectionReader<'_>) -> Result<()> {
        self.process_module_section(
            Order::Import,
            section,
            "import",
            |_, _, _, _, _| Ok(()), // add_import will check limits
            |state, features, types, import, offset| {
                state
                    .module
                    .assert_mut()
                    .add_import(import, features, types, offset)
            },
        )
    }

    /// Validates [`Payload::FunctionSection`](crate::Payload).
    ///
    /// This method should only be called when parsing a module.
    pub fn function_section(&mut self, section: &crate::FunctionSectionReader<'_>) -> Result<()> {
        self.process_module_section(
            Order::Function,
            section,
            "function",
            |state, _, _, count, offset| {
                check_max(
                    state.module.functions.len(),
                    count,
                    MAX_WASM_FUNCTIONS,
                    "functions",
                    offset,
                )?;
                state.module.assert_mut().functions.reserve(count as usize);
                debug_assert!(state.expected_code_bodies.is_none());
                state.expected_code_bodies = Some(count);
                Ok(())
            },
            |state, _, types, ty, offset| state.module.assert_mut().add_function(ty, types, offset),
        )
    }

    /// Validates [`Payload::TableSection`](crate::Payload).
    ///
    /// This method should only be called when parsing a module.
    pub fn table_section(&mut self, section: &crate::TableSectionReader<'_>) -> Result<()> {
        let features = self.features;
        self.process_module_section(
            Order::Table,
            section,
            "table",
            |state, _, _, count, offset| {
                check_max(
                    state.module.tables.len(),
                    count,
                    state.module.max_tables(&features),
                    "tables",
                    offset,
                )?;
                state.module.assert_mut().tables.reserve(count as usize);
                Ok(())
            },
            |state, features, _, ty, offset| {
                state.module.assert_mut().add_table(ty, features, offset)
            },
        )
    }

    /// Validates [`Payload::MemorySection`](crate::Payload).
    ///
    /// This method should only be called when parsing a module.
    pub fn memory_section(&mut self, section: &crate::MemorySectionReader<'_>) -> Result<()> {
        self.process_module_section(
            Order::Memory,
            section,
            "memory",
            |state, features, _, count, offset| {
                check_max(
                    state.module.memories.len(),
                    count,
                    state.module.max_memories(features),
                    "memories",
                    offset,
                )?;
                state.module.assert_mut().memories.reserve(count as usize);
                Ok(())
            },
            |state, features, _, ty, offset| {
                state.module.assert_mut().add_memory(ty, features, offset)
            },
        )
    }

    /// Validates [`Payload::TagSection`](crate::Payload).
    ///
    /// This method should only be called when parsing a module.
    pub fn tag_section(&mut self, section: &crate::TagSectionReader<'_>) -> Result<()> {
        if !self.features.exceptions {
            return Err(BinaryReaderError::new(
                "exceptions proposal not enabled",
                section.range().start,
            ));
        }

        self.process_module_section(
            Order::Tag,
            section,
            "tag",
            |state, _, _, count, offset| {
                check_max(
                    state.module.tags.len(),
                    count,
                    MAX_WASM_TAGS,
                    "tags",
                    offset,
                )?;
                state.module.assert_mut().tags.reserve(count as usize);
                Ok(())
            },
            |state, features, types, ty, offset| {
                state
                    .module
                    .assert_mut()
                    .add_tag(ty, features, types, offset)
            },
        )
    }

    /// Validates [`Payload::GlobalSection`](crate::Payload).
    ///
    /// This method should only be called when parsing a module.
    pub fn global_section(&mut self, section: &crate::GlobalSectionReader<'_>) -> Result<()> {
        self.process_module_section(
            Order::Global,
            section,
            "global",
            |state, _, _, count, offset| {
                check_max(
                    state.module.globals.len(),
                    count,
                    MAX_WASM_GLOBALS,
                    "globals",
                    offset,
                )?;
                state.module.assert_mut().globals.reserve(count as usize);
                Ok(())
            },
            |state, features, types, global, offset| {
                state.add_global(global, features, types, offset)
            },
        )
    }

    /// Validates [`Payload::ExportSection`](crate::Payload).
    ///
    /// This method should only be called when parsing a module.
    pub fn export_section(&mut self, section: &crate::ExportSectionReader<'_>) -> Result<()> {
        self.process_module_section(
            Order::Export,
            section,
            "export",
            |state, _, _, count, offset| {
                check_max(
                    state.module.exports.len(),
                    count,
                    MAX_WASM_EXPORTS,
                    "exports",
                    offset,
                )?;
                state.module.assert_mut().exports.reserve(count as usize);
                Ok(())
            },
            |state, features, _, e, offset| {
                let state = state.module.assert_mut();
                let ty = state.export_to_entity_type(&e, offset)?;
                state.add_export(e.name, ty, features, offset, false /* checked above */)
            },
        )
    }

    /// Validates [`Payload::StartSection`](crate::Payload).
    ///
    /// This method should only be called when parsing a module.
    pub fn start_section(&mut self, func: u32, range: &Range<usize>) -> Result<()> {
        let offset = range.start;
        self.state.ensure_module("start", offset)?;
        let state = self.module.as_mut().unwrap();
        state.update_order(Order::Start, offset)?;

        let ty = state.module.get_func_type(func, &self.types, offset)?;
        if !ty.params().is_empty() || !ty.results().is_empty() {
            return Err(BinaryReaderError::new(
                "invalid start function type",
                offset,
            ));
        }

        Ok(())
    }

    /// Validates [`Payload::ElementSection`](crate::Payload).
    ///
    /// This method should only be called when parsing a module.
    pub fn element_section(&mut self, section: &crate::ElementSectionReader<'_>) -> Result<()> {
        self.process_module_section(
            Order::Element,
            section,
            "element",
            |state, _, _, count, offset| {
                check_max(
                    state.module.element_types.len(),
                    count,
                    MAX_WASM_ELEMENT_SEGMENTS,
                    "element segments",
                    offset,
                )?;
                state
                    .module
                    .assert_mut()
                    .element_types
                    .reserve(count as usize);
                Ok(())
            },
            |state, features, types, e, offset| {
                state.add_element_segment(e, features, types, offset)
            },
        )
    }

    /// Validates [`Payload::DataCountSection`](crate::Payload).
    ///
    /// This method should only be called when parsing a module.
    pub fn data_count_section(&mut self, count: u32, range: &Range<usize>) -> Result<()> {
        let offset = range.start;
        self.state.ensure_module("data count", offset)?;

        let state = self.module.as_mut().unwrap();
        state.update_order(Order::DataCount, offset)?;

        if count > MAX_WASM_DATA_SEGMENTS as u32 {
            return Err(BinaryReaderError::new(
                "data count section specifies too many data segments",
                offset,
            ));
        }

        state.module.assert_mut().data_count = Some(count);
        Ok(())
    }

    /// Validates [`Payload::CodeSectionStart`](crate::Payload).
    ///
    /// This method should only be called when parsing a module.
    pub fn code_section_start(&mut self, count: u32, range: &Range<usize>) -> Result<()> {
        let offset = range.start;
        self.state.ensure_module("code", offset)?;

        let state = self.module.as_mut().unwrap();
        state.update_order(Order::Code, offset)?;

        match state.expected_code_bodies.take() {
            Some(n) if n == count => {}
            Some(_) => {
                return Err(BinaryReaderError::new(
                    "function and code section have inconsistent lengths",
                    offset,
                ));
            }
            // empty code sections are allowed even if the function section is
            // missing
            None if count == 0 => {}
            None => {
                return Err(BinaryReaderError::new(
                    "code section without function section",
                    offset,
                ))
            }
        }

        // Take a snapshot of the types when we start the code section.
        state.module.assert_mut().snapshot = Some(Arc::new(self.types.commit()));

        Ok(())
    }

    /// Validates [`Payload::CodeSectionEntry`](crate::Payload).
    ///
    /// This function will prepare a [`FuncToValidate`] which can be used to
    /// create a [`FuncValidator`] to validate the function. The function body
    /// provided will not be parsed or validated by this function.
    ///
    /// Note that the returned [`FuncToValidate`] is "connected" to this
    /// [`Validator`] in that it uses the internal context of this validator for
    /// validating the function. The [`FuncToValidate`] can be sent to another
    /// thread, for example, to offload actual processing of functions
    /// elsewhere.
    ///
    /// This method should only be called when parsing a module.
    pub fn code_section_entry(
        &mut self,
        body: &crate::FunctionBody,
    ) -> Result<FuncToValidate<ValidatorResources>> {
        let offset = body.range().start;
        self.state.ensure_module("code", offset)?;

        let state = self.module.as_mut().unwrap();

        let (index, ty) = state.next_code_index_and_type(offset)?;
        Ok(FuncToValidate::new(
            index,
            ty,
            ValidatorResources(state.module.arc().clone()),
            &self.features,
        ))
    }

    /// Validates [`Payload::DataSection`](crate::Payload).
    ///
    /// This method should only be called when parsing a module.
    pub fn data_section(&mut self, section: &crate::DataSectionReader<'_>) -> Result<()> {
        self.process_module_section(
            Order::Data,
            section,
            "data",
            |state, _, _, count, offset| {
                state.data_segment_count = count;
                check_max(0, count, MAX_WASM_DATA_SEGMENTS, "data segments", offset)
            },
            |state, features, types, d, offset| state.add_data_segment(d, features, types, offset),
        )
    }

    /// Validates [`Payload::ModuleSection`](crate::Payload).
    ///
    /// This method should only be called when parsing a component.
    pub fn module_section(&mut self, range: &Range<usize>) -> Result<()> {
        self.state.ensure_component("module", range.start)?;

        let current = self.components.last_mut().unwrap();
        check_max(
            current.core_modules.len(),
            1,
            MAX_WASM_MODULES,
            "modules",
            range.start,
        )?;

        match mem::replace(&mut self.state, State::Unparsed(Some(Encoding::Module))) {
            State::Component => {}
            _ => unreachable!(),
        }

        Ok(())
    }

    /// Validates [`Payload::InstanceSection`](crate::Payload).
    ///
    /// This method should only be called when parsing a component.
    pub fn instance_section(&mut self, section: &crate::InstanceSectionReader) -> Result<()> {
        self.process_component_section(
            section,
            "core instance",
            |components, _, count, offset| {
                let current = components.last_mut().unwrap();
                check_max(
                    current.instance_count(),
                    count,
                    MAX_WASM_INSTANCES,
                    "instances",
                    offset,
                )?;
                current.core_instances.reserve(count as usize);
                Ok(())
            },
            |components, types, _, instance, offset| {
                components
                    .last_mut()
                    .unwrap()
                    .add_core_instance(instance, types, offset)
            },
        )
    }

    /// Validates [`Payload::CoreTypeSection`](crate::Payload).
    ///
    /// This method should only be called when parsing a component.
    pub fn core_type_section(&mut self, section: &crate::CoreTypeSectionReader<'_>) -> Result<()> {
        self.process_component_section(
            section,
            "core type",
            |components, types, count, offset| {
                let current = components.last_mut().unwrap();
                check_max(current.type_count(), count, MAX_WASM_TYPES, "types", offset)?;
                types.reserve(count as usize);
                current.core_types.reserve(count as usize);
                Ok(())
            },
            |components, types, features, ty, offset| {
                ComponentState::add_core_type(
                    components, ty, features, types, offset, false, /* checked above */
                )
            },
        )
    }

    /// Validates [`Payload::ComponentSection`](crate::Payload).
    ///
    /// This method should only be called when parsing a component.
    pub fn component_section(&mut self, range: &Range<usize>) -> Result<()> {
        self.state.ensure_component("component", range.start)?;

        let current = self.components.last_mut().unwrap();
        check_max(
            current.components.len(),
            1,
            MAX_WASM_COMPONENTS,
            "components",
            range.start,
        )?;

        match mem::replace(&mut self.state, State::Unparsed(Some(Encoding::Component))) {
            State::Component => {}
            _ => unreachable!(),
        }

        Ok(())
    }

    /// Validates [`Payload::ComponentInstanceSection`](crate::Payload).
    ///
    /// This method should only be called when parsing a component.
    pub fn component_instance_section(
        &mut self,
        section: &crate::ComponentInstanceSectionReader,
    ) -> Result<()> {
        self.process_component_section(
            section,
            "instance",
            |components, _, count, offset| {
                let current = components.last_mut().unwrap();
                check_max(
                    current.instance_count(),
                    count,
                    MAX_WASM_INSTANCES,
                    "instances",
                    offset,
                )?;
                current.instances.reserve(count as usize);
                Ok(())
            },
            |components, types, _, instance, offset| {
                components
                    .last_mut()
                    .unwrap()
                    .add_instance(instance, types, offset)
            },
        )
    }

    /// Validates [`Payload::ComponentAliasSection`](crate::Payload).
    ///
    /// This method should only be called when parsing a component.
    pub fn component_alias_section(
        &mut self,
        section: &crate::ComponentAliasSectionReader,
    ) -> Result<()> {
        self.process_component_section(
            section,
            "alias",
            |_, _, _, _| Ok(()), // maximums checked via `add_alias`
            |components, types, _, alias, offset| -> Result<(), BinaryReaderError> {
                ComponentState::add_alias(components, alias, types, offset)
            },
        )
    }

    /// Validates [`Payload::ComponentTypeSection`](crate::Payload).
    ///
    /// This method should only be called when parsing a component.
    pub fn component_type_section(
        &mut self,
        section: &crate::ComponentTypeSectionReader,
    ) -> Result<()> {
        self.process_component_section(
            section,
            "type",
            |components, types, count, offset| {
                let current = components.last_mut().unwrap();
                check_max(current.type_count(), count, MAX_WASM_TYPES, "types", offset)?;
                types.reserve(count as usize);
                current.types.reserve(count as usize);
                Ok(())
            },
            |components, types, features, ty, offset| {
                ComponentState::add_type(
                    components, ty, features, types, offset, false, /* checked above */
                )
            },
        )
    }

    /// Validates [`Payload::ComponentCanonicalSection`](crate::Payload).
    ///
    /// This method should only be called when parsing a component.
    pub fn component_canonical_section(
        &mut self,
        section: &crate::ComponentCanonicalSectionReader,
    ) -> Result<()> {
        self.process_component_section(
            section,
            "function",
            |components, _, count, offset| {
                let current = components.last_mut().unwrap();
                check_max(
                    current.function_count(),
                    count,
                    MAX_WASM_FUNCTIONS,
                    "functions",
                    offset,
                )?;
                current.funcs.reserve(count as usize);
                Ok(())
            },
            |components, types, _, func, offset| {
                let current = components.last_mut().unwrap();
                match func {
                    crate::CanonicalFunction::Lift {
                        core_func_index,
                        type_index,
                        options,
                    } => current.lift_function(
                        core_func_index,
                        type_index,
                        options.into_vec(),
                        types,
                        offset,
                    ),
                    crate::CanonicalFunction::Lower {
                        func_index,
                        options,
                    } => current.lower_function(func_index, options.into_vec(), types, offset),
                }
            },
        )
    }

    /// Validates [`Payload::ComponentStartSection`](crate::Payload).
    ///
    /// This method should only be called when parsing a component.
    pub fn component_start_section(
        &mut self,
        section: &crate::ComponentStartSectionReader,
    ) -> Result<()> {
        let range = section.range();
        self.state.ensure_component("start", range.start)?;

        let mut section = section.clone();
        let f = section.read()?;

        if !section.eof() {
            return Err(BinaryReaderError::new(
                "trailing data at the end of the start section",
                section.original_position(),
            ));
        }

        self.components.last_mut().unwrap().add_start(
            f.func_index,
            &f.arguments,
            f.results,
            &self.types,
            range.start,
        )
    }

    /// Validates [`Payload::ComponentImportSection`](crate::Payload).
    ///
    /// This method should only be called when parsing a component.
    pub fn component_import_section(
        &mut self,
        section: &crate::ComponentImportSectionReader,
    ) -> Result<()> {
        self.process_component_section(
            section,
            "import",
            |_, _, _, _| Ok(()), // add_import will check limits
            |components, types, _, import, offset| {
                components
                    .last_mut()
                    .unwrap()
                    .add_import(import, types, offset)
            },
        )
    }

    /// Validates [`Payload::ComponentExportSection`](crate::Payload).
    ///
    /// This method should only be called when parsing a component.
    pub fn component_export_section(
        &mut self,
        section: &crate::ComponentExportSectionReader,
    ) -> Result<()> {
        self.process_component_section(
            section,
            "export",
            |components, _, count, offset| {
                let current = components.last_mut().unwrap();
                check_max(
                    current.exports.len(),
                    count,
                    MAX_WASM_EXPORTS,
                    "exports",
                    offset,
                )?;
                current.exports.reserve(count as usize);
                Ok(())
            },
            |components, _, _, export, offset| {
                let current = components.last_mut().unwrap();
                let ty = current.export_to_entity_type(&export, offset)?;
                current.add_export(
                    export.name,
                    export.url,
                    ty,
                    offset,
                    false, /* checked above */
                )
            },
        )
    }

    /// Validates [`Payload::UnknownSection`](crate::Payload).
    ///
    /// Currently always returns an error.
    pub fn unknown_section(&mut self, id: u8, range: &Range<usize>) -> Result<()> {
        Err(format_err!(range.start, "malformed section id: {id}"))
    }

    /// Validates [`Payload::End`](crate::Payload).
    ///
    /// Returns the types known to the validator for the module or component.
    pub fn end(&mut self, offset: usize) -> Result<Types> {
        match std::mem::replace(&mut self.state, State::End) {
            State::Unparsed(_) => Err(BinaryReaderError::new(
                "cannot call `end` before a header has been parsed",
                offset,
            )),
            State::End => Err(BinaryReaderError::new(
                "cannot call `end` after parsing has completed",
                offset,
            )),
            State::Module => {
                let mut state = self.module.take().unwrap();
                state.validate_end(offset)?;

                // If there's a parent component, we'll add a module to the parent state
                // and continue to validate the component
                if let Some(parent) = self.components.last_mut() {
                    parent.add_core_module(&state.module, &mut self.types, offset)?;
                    self.state = State::Component;
                }

                Ok(Types::from_module(
                    self.types.commit(),
                    state.module.arc().clone(),
                ))
            }
            State::Component => {
                let mut component = self.components.pop().unwrap();

                // Validate that all values were used for the component
                if let Some(index) = component.values.iter().position(|(_, used)| !*used) {
                    return Err(
                        format_err!(offset,"value index {index} was not used as part of an instantiation, start function, or export"
                            )
                    );
                }

                // If there's a parent component, pop the stack, add it to the parent,
                // and continue to validate the component
                if let Some(parent) = self.components.last_mut() {
                    parent.add_component(&mut component, &mut self.types);
                    self.state = State::Component;
                }

                Ok(Types::from_component(self.types.commit(), component))
            }
        }
    }

    fn process_module_section<T>(
        &mut self,
        order: Order,
        section: &T,
        name: &str,
        validate_section: impl FnOnce(
            &mut ModuleState,
            &WasmFeatures,
            &mut TypeList,
            u32,
            usize,
        ) -> Result<()>,
        mut validate_item: impl FnMut(
            &mut ModuleState,
            &WasmFeatures,
            &mut TypeList,
            T::Item,
            usize,
        ) -> Result<()>,
    ) -> Result<()>
    where
        T: SectionReader + Clone + SectionWithLimitedItems,
    {
        let offset = section.range().start;
        self.state.ensure_module(name, offset)?;

        let state = self.module.as_mut().unwrap();
        state.update_order(order, offset)?;

        validate_section(
            state,
            &self.features,
            &mut self.types,
            section.get_count(),
            offset,
        )?;

        let mut section = section.clone();
        for _ in 0..section.get_count() {
            let offset = section.original_position();
            let item = section.read()?;
            validate_item(state, &self.features, &mut self.types, item, offset)?;
        }

        section.ensure_end()?;

        Ok(())
    }

    fn process_component_section<T>(
        &mut self,
        section: &T,
        name: &str,
        validate_section: impl FnOnce(&mut Vec<ComponentState>, &mut TypeList, u32, usize) -> Result<()>,
        mut validate_item: impl FnMut(
            &mut Vec<ComponentState>,
            &mut TypeList,
            &WasmFeatures,
            T::Item,
            usize,
        ) -> Result<()>,
    ) -> Result<()>
    where
        T: Clone + SectionWithLimitedItems,
    {
        let offset = section.range().start;

        if !self.features.component_model {
            return Err(BinaryReaderError::new(
                "component model feature is not enabled",
                offset,
            ));
        }

        self.state.ensure_component(name, offset)?;
        validate_section(
            &mut self.components,
            &mut self.types,
            section.get_count(),
            offset,
        )?;

        let mut section = section.clone();
        for _ in 0..section.get_count() {
            let offset = section.original_position();
            let item = section.read()?;
            validate_item(
                &mut self.components,
                &mut self.types,
                &self.features,
                item,
                offset,
            )?;
        }

        section.ensure_end()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{GlobalType, MemoryType, TableType, ValType, Validator, WasmFeatures};
    use anyhow::Result;

    #[test]
    fn test_module_type_information() -> Result<()> {
        let bytes = wat::parse_str(
            r#"
            (module
                (type (func (param i32 i64) (result i32)))
                (memory 1 5)
                (table 10 funcref)
                (global (mut i32) (i32.const 0))
                (func (type 0) (i32.const 0))
                (tag (param i64 i32))
                (elem funcref (ref.func 0))
            )
        "#,
        )?;

        let mut validator = Validator::new_with_features(WasmFeatures {
            exceptions: true,
            ..Default::default()
        });

        let types = validator.validate_all(&bytes)?;

        assert_eq!(types.type_count(), 2);
        assert_eq!(types.memory_count(), 1);
        assert_eq!(types.table_count(), 1);
        assert_eq!(types.global_count(), 1);
        assert_eq!(types.function_count(), 1);
        assert_eq!(types.tag_count(), 1);
        assert_eq!(types.element_count(), 1);
        assert_eq!(types.module_count(), 0);
        assert_eq!(types.component_count(), 0);
        assert_eq!(types.instance_count(), 0);
        assert_eq!(types.value_count(), 0);

        match types.func_type_at(0) {
            Some(ty) => {
                assert_eq!(ty.params(), [ValType::I32, ValType::I64]);
                assert_eq!(ty.results(), [ValType::I32]);
            }
            _ => unreachable!(),
        }

        match types.func_type_at(1) {
            Some(ty) => {
                assert_eq!(ty.params(), [ValType::I64, ValType::I32]);
                assert_eq!(ty.results(), []);
            }
            _ => unreachable!(),
        }

        assert_eq!(
            types.memory_at(0),
            Some(MemoryType {
                memory64: false,
                shared: false,
                initial: 1,
                maximum: Some(5)
            })
        );

        assert_eq!(
            types.table_at(0),
            Some(TableType {
                initial: 10,
                maximum: None,
                element_type: ValType::FuncRef,
            })
        );

        assert_eq!(
            types.global_at(0),
            Some(GlobalType {
                content_type: ValType::I32,
                mutable: true
            })
        );

        match types.function_at(0) {
            Some(ty) => {
                assert_eq!(ty.params(), [ValType::I32, ValType::I64]);
                assert_eq!(ty.results(), [ValType::I32]);
            }
            _ => unreachable!(),
        }

        match types.tag_at(0) {
            Some(ty) => {
                assert_eq!(ty.params(), [ValType::I64, ValType::I32]);
                assert_eq!(ty.results(), []);
            }
            _ => unreachable!(),
        }

        assert_eq!(types.element_at(0), Some(ValType::FuncRef));

        Ok(())
    }

    #[test]
    fn test_type_id_aliasing() -> Result<()> {
        let bytes = wat::parse_str(
            r#"
            (component
              (type $T (list string))
              (alias outer 0 $T (type $A1))
              (alias outer 0 $T (type $A2))
            )
        "#,
        )?;

        let mut validator = Validator::new_with_features(WasmFeatures {
            component_model: true,
            ..Default::default()
        });

        let types = validator.validate_all(&bytes)?;

        let t_id = types.id_from_type_index(0, false).unwrap();
        let a1_id = types.id_from_type_index(1, false).unwrap();
        let a2_id = types.id_from_type_index(2, false).unwrap();

        // The ids should all be different
        assert!(t_id != a1_id);
        assert!(t_id != a2_id);
        assert!(a1_id != a2_id);

        // However, they should all point to the same type
        assert!(std::ptr::eq(
            types.type_from_id(t_id).unwrap(),
            types.type_from_id(a1_id).unwrap()
        ));
        assert!(std::ptr::eq(
            types.type_from_id(t_id).unwrap(),
            types.type_from_id(a2_id).unwrap()
        ));

        Ok(())
    }
}
