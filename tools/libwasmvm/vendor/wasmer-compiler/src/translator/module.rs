// This file contains code from external sources.
// Attributions: https://github.com/wasmerio/wasmer/blob/master/ATTRIBUTIONS.md

//! Translation skeleton that traverses the whole WebAssembly module and call helper functions
//! to deal with each part of it.
use super::environ::ModuleEnvironment;
use super::sections::{
    parse_data_section, parse_element_section, parse_export_section, parse_function_section,
    parse_global_section, parse_import_section, parse_memory_section, parse_name_section,
    parse_start_section, parse_table_section, parse_type_section,
};
use super::state::ModuleTranslationState;
use crate::WasmResult;
use wasmparser::{NameSectionReader, Parser, Payload};

/// Translate a sequence of bytes forming a valid Wasm binary into a
/// parsed ModuleInfo `ModuleTranslationState`.
pub fn translate_module<'data>(
    data: &'data [u8],
    environ: &mut ModuleEnvironment<'data>,
) -> WasmResult<ModuleTranslationState> {
    let mut module_translation_state = ModuleTranslationState::new();

    for payload in Parser::new(0).parse_all(data) {
        match payload? {
            Payload::Version { .. } | Payload::End => {}

            Payload::TypeSection(types) => {
                parse_type_section(types, &mut module_translation_state, environ)?;
            }

            Payload::ImportSection(imports) => {
                parse_import_section(imports, environ)?;
            }

            Payload::FunctionSection(functions) => {
                parse_function_section(functions, environ)?;
            }

            Payload::TableSection(tables) => {
                parse_table_section(tables, environ)?;
            }

            Payload::MemorySection(memories) => {
                parse_memory_section(memories, environ)?;
            }

            Payload::GlobalSection(globals) => {
                parse_global_section(globals, environ)?;
            }

            Payload::ExportSection(exports) => {
                parse_export_section(exports, environ)?;
            }

            Payload::StartSection { func, .. } => {
                parse_start_section(func, environ)?;
            }

            Payload::ElementSection(elements) => {
                parse_element_section(elements, environ)?;
            }

            Payload::CodeSectionStart { .. } => {}
            Payload::CodeSectionEntry(code) => {
                let mut code = code.get_binary_reader();
                let size = code.bytes_remaining();
                let offset = code.original_position();
                environ.define_function_body(
                    &module_translation_state,
                    code.read_bytes(size)?,
                    offset,
                )?;
            }

            Payload::DataSection(data) => {
                parse_data_section(data, environ)?;
            }

            Payload::DataCountSection { count, .. } => {
                environ.reserve_passive_data(count)?;
            }

            Payload::InstanceSection(_)
            | Payload::AliasSection(_)
            | Payload::ModuleSectionStart { .. }
            | Payload::ModuleSectionEntry { .. } => {
                unimplemented!("module linking not implemented yet")
            }

            Payload::TagSection(_) => {
                unimplemented!("exception handling not implemented yet")
            }

            Payload::CustomSection {
                name: "name",
                data,
                data_offset,
                ..
            } => parse_name_section(NameSectionReader::new(data, data_offset)?, environ)?,

            Payload::CustomSection { name, data, .. } => environ.custom_section(name, data)?,

            Payload::UnknownSection { .. } => unreachable!(),
        }
    }

    Ok(module_translation_state)
}
