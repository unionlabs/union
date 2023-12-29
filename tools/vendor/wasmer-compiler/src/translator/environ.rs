// This file contains code from external sources.
// Attributions: https://github.com/wasmerio/wasmer/blob/master/ATTRIBUTIONS.md
use super::state::ModuleTranslationState;
use crate::lib::std::string::ToString;
use crate::lib::std::{boxed::Box, string::String, vec::Vec};
use crate::translate_module;
use crate::wasmparser::{Operator, ValType};
use std::convert::{TryFrom, TryInto};
use std::ops::Range;
use wasmer_types::entity::PrimaryMap;
use wasmer_types::FunctionType;
use wasmer_types::WasmResult;
use wasmer_types::{
    CustomSectionIndex, DataIndex, DataInitializer, DataInitializerLocation, ElemIndex,
    ExportIndex, FunctionIndex, GlobalIndex, GlobalInit, GlobalType, ImportIndex,
    LocalFunctionIndex, MemoryIndex, MemoryType, ModuleInfo, SignatureIndex, TableIndex,
    TableInitializer, TableType,
};

/// Contains function data: bytecode and its offset in the module.
#[derive(Hash)]
pub struct FunctionBodyData<'a> {
    /// Function body bytecode.
    pub data: &'a [u8],

    /// Body offset relative to the module file.
    pub module_offset: usize,
}

/// Trait for iterating over the operators of a Wasm Function
pub trait FunctionBinaryReader<'a> {
    /// Read a `count` indicating the number of times to call `read_local_decl`.
    fn read_local_count(&mut self) -> WasmResult<u32>;

    /// Read a `(count, value_type)` declaration of local variables of the same type.
    fn read_local_decl(&mut self) -> WasmResult<(u32, ValType)>;

    /// Reads the next available `Operator`.
    fn read_operator(&mut self) -> WasmResult<Operator<'a>>;

    /// Returns the current position.
    fn current_position(&self) -> usize;

    /// Returns the original position (with the offset)
    fn original_position(&self) -> usize;

    /// Returns the number of bytes remaining.
    fn bytes_remaining(&self) -> usize;

    /// Returns whether the readers has reached the end of the file.
    fn eof(&self) -> bool;

    /// Return the range (original offset, original offset + data length)
    fn range(&self) -> Range<usize>;
}

/// The result of translating via `ModuleEnvironment`. Function bodies are not
/// yet translated, and data initializers have not yet been copied out of the
/// original buffer.
/// The function bodies will be translated by a specific compiler backend.
pub struct ModuleEnvironment<'data> {
    /// ModuleInfo information.
    pub module: ModuleInfo,

    /// References to the function bodies.
    pub function_body_inputs: PrimaryMap<LocalFunctionIndex, FunctionBodyData<'data>>,

    /// References to the data initializers.
    pub data_initializers: Vec<DataInitializer<'data>>,

    /// The decoded Wasm types for the module.
    pub module_translation_state: Option<ModuleTranslationState>,
}

impl<'data> ModuleEnvironment<'data> {
    /// Allocates the environment data structures.
    pub fn new() -> Self {
        Self {
            module: ModuleInfo::new(),
            function_body_inputs: PrimaryMap::new(),
            data_initializers: Vec::new(),
            module_translation_state: None,
        }
    }

    /// Translate a wasm module using this environment. This consumes the
    /// `ModuleEnvironment` and produces a `ModuleInfoTranslation`.
    pub fn translate(mut self, data: &'data [u8]) -> WasmResult<ModuleEnvironment<'data>> {
        assert!(self.module_translation_state.is_none());
        let module_translation_state = translate_module(data, &mut self)?;
        self.module_translation_state = Some(module_translation_state);
        Ok(self)
    }

    pub(crate) fn declare_export(&mut self, export: ExportIndex, name: &str) -> WasmResult<()> {
        self.module.exports.insert(String::from(name), export);
        Ok(())
    }

    pub(crate) fn declare_import(
        &mut self,
        import: ImportIndex,
        module: &str,
        field: &str,
    ) -> WasmResult<()> {
        self.module.imports.insert(
            (
                String::from(module),
                String::from(field),
                self.module.imports.len().try_into().unwrap(),
            )
                .into(),
            import,
        );
        Ok(())
    }

    pub(crate) fn reserve_signatures(&mut self, num: u32) -> WasmResult<()> {
        self.module
            .signatures
            .reserve_exact(usize::try_from(num).unwrap());
        Ok(())
    }

    pub(crate) fn declare_signature(&mut self, sig: FunctionType) -> WasmResult<()> {
        // TODO: Deduplicate signatures.
        self.module.signatures.push(sig);
        Ok(())
    }

    pub(crate) fn declare_func_import(
        &mut self,
        sig_index: SignatureIndex,
        module: &str,
        field: &str,
    ) -> WasmResult<()> {
        debug_assert_eq!(
            self.module.functions.len(),
            self.module.num_imported_functions,
            "Imported functions must be declared first"
        );
        self.declare_import(
            ImportIndex::Function(FunctionIndex::from_u32(
                self.module.num_imported_functions as _,
            )),
            module,
            field,
        )?;
        self.module.functions.push(sig_index);
        self.module.num_imported_functions += 1;
        Ok(())
    }

    pub(crate) fn declare_table_import(
        &mut self,
        table: TableType,
        module: &str,
        field: &str,
    ) -> WasmResult<()> {
        debug_assert_eq!(
            self.module.tables.len(),
            self.module.num_imported_tables,
            "Imported tables must be declared first"
        );
        self.declare_import(
            ImportIndex::Table(TableIndex::from_u32(self.module.num_imported_tables as _)),
            module,
            field,
        )?;
        self.module.tables.push(table);
        self.module.num_imported_tables += 1;
        Ok(())
    }

    pub(crate) fn declare_memory_import(
        &mut self,
        memory: MemoryType,
        module: &str,
        field: &str,
    ) -> WasmResult<()> {
        debug_assert_eq!(
            self.module.memories.len(),
            self.module.num_imported_memories,
            "Imported memories must be declared first"
        );
        self.declare_import(
            ImportIndex::Memory(MemoryIndex::from_u32(
                self.module.num_imported_memories as _,
            )),
            module,
            field,
        )?;
        self.module.memories.push(memory);
        self.module.num_imported_memories += 1;
        Ok(())
    }

    pub(crate) fn declare_global_import(
        &mut self,
        global: GlobalType,
        module: &str,
        field: &str,
    ) -> WasmResult<()> {
        debug_assert_eq!(
            self.module.globals.len(),
            self.module.num_imported_globals,
            "Imported globals must be declared first"
        );
        self.declare_import(
            ImportIndex::Global(GlobalIndex::from_u32(self.module.num_imported_globals as _)),
            module,
            field,
        )?;
        self.module.globals.push(global);
        self.module.num_imported_globals += 1;
        Ok(())
    }

    pub(crate) fn finish_imports(&mut self) -> WasmResult<()> {
        Ok(())
    }

    pub(crate) fn reserve_func_types(&mut self, num: u32) -> WasmResult<()> {
        self.module
            .functions
            .reserve_exact(usize::try_from(num).unwrap());
        self.function_body_inputs
            .reserve_exact(usize::try_from(num).unwrap());
        Ok(())
    }

    pub(crate) fn declare_func_type(&mut self, sig_index: SignatureIndex) -> WasmResult<()> {
        self.module.functions.push(sig_index);
        Ok(())
    }

    pub(crate) fn reserve_tables(&mut self, num: u32) -> WasmResult<()> {
        self.module
            .tables
            .reserve_exact(usize::try_from(num).unwrap());
        Ok(())
    }

    pub(crate) fn declare_table(&mut self, table: TableType) -> WasmResult<()> {
        self.module.tables.push(table);
        Ok(())
    }

    pub(crate) fn reserve_memories(&mut self, num: u32) -> WasmResult<()> {
        self.module
            .memories
            .reserve_exact(usize::try_from(num).unwrap());
        Ok(())
    }

    pub(crate) fn declare_memory(&mut self, memory: MemoryType) -> WasmResult<()> {
        self.module.memories.push(memory);
        Ok(())
    }

    pub(crate) fn reserve_globals(&mut self, num: u32) -> WasmResult<()> {
        self.module
            .globals
            .reserve_exact(usize::try_from(num).unwrap());
        Ok(())
    }

    pub(crate) fn declare_global(
        &mut self,
        global: GlobalType,
        initializer: GlobalInit,
    ) -> WasmResult<()> {
        self.module.globals.push(global);
        self.module.global_initializers.push(initializer);
        Ok(())
    }

    pub(crate) fn reserve_exports(&mut self, num: u32) -> WasmResult<()> {
        self.module.exports.reserve(usize::try_from(num).unwrap());
        Ok(())
    }

    pub(crate) fn declare_func_export(
        &mut self,
        func_index: FunctionIndex,
        name: &str,
    ) -> WasmResult<()> {
        self.declare_export(ExportIndex::Function(func_index), name)
    }

    pub(crate) fn declare_table_export(
        &mut self,
        table_index: TableIndex,
        name: &str,
    ) -> WasmResult<()> {
        self.declare_export(ExportIndex::Table(table_index), name)
    }

    pub(crate) fn declare_memory_export(
        &mut self,
        memory_index: MemoryIndex,
        name: &str,
    ) -> WasmResult<()> {
        self.declare_export(ExportIndex::Memory(memory_index), name)
    }

    pub(crate) fn declare_global_export(
        &mut self,
        global_index: GlobalIndex,
        name: &str,
    ) -> WasmResult<()> {
        self.declare_export(ExportIndex::Global(global_index), name)
    }

    pub(crate) fn declare_start_function(&mut self, func_index: FunctionIndex) -> WasmResult<()> {
        debug_assert!(self.module.start_function.is_none());
        self.module.start_function = Some(func_index);
        Ok(())
    }

    pub(crate) fn reserve_table_initializers(&mut self, num: u32) -> WasmResult<()> {
        self.module
            .table_initializers
            .reserve_exact(usize::try_from(num).unwrap());
        Ok(())
    }

    pub(crate) fn declare_table_initializers(
        &mut self,
        table_index: TableIndex,
        base: Option<GlobalIndex>,
        offset: usize,
        elements: Box<[FunctionIndex]>,
    ) -> WasmResult<()> {
        self.module.table_initializers.push(TableInitializer {
            table_index,
            base,
            offset,
            elements,
        });
        Ok(())
    }

    pub(crate) fn declare_passive_element(
        &mut self,
        elem_index: ElemIndex,
        segments: Box<[FunctionIndex]>,
    ) -> WasmResult<()> {
        let old = self.module.passive_elements.insert(elem_index, segments);
        debug_assert!(
            old.is_none(),
            "should never get duplicate element indices, that would be a bug in `wasmer_compiler`'s \
             translation"
        );
        Ok(())
    }

    pub(crate) fn define_function_body(
        &mut self,
        _module_translation_state: &ModuleTranslationState,
        body_bytes: &'data [u8],
        body_offset: usize,
    ) -> WasmResult<()> {
        self.function_body_inputs.push(FunctionBodyData {
            data: body_bytes,
            module_offset: body_offset,
        });
        Ok(())
    }

    pub(crate) fn reserve_data_initializers(&mut self, num: u32) -> WasmResult<()> {
        self.data_initializers
            .reserve_exact(usize::try_from(num).unwrap());
        Ok(())
    }

    pub(crate) fn declare_data_initialization(
        &mut self,
        memory_index: MemoryIndex,
        base: Option<GlobalIndex>,
        offset: usize,
        data: &'data [u8],
    ) -> WasmResult<()> {
        self.data_initializers.push(DataInitializer {
            location: DataInitializerLocation {
                memory_index,
                base,
                offset,
            },
            data,
        });
        Ok(())
    }

    pub(crate) fn reserve_passive_data(&mut self, count: u32) -> WasmResult<()> {
        let count = usize::try_from(count).unwrap();
        self.module.passive_data.reserve(count);
        Ok(())
    }

    pub(crate) fn declare_passive_data(
        &mut self,
        data_index: DataIndex,
        data: &'data [u8],
    ) -> WasmResult<()> {
        let old = self.module.passive_data.insert(data_index, Box::from(data));
        debug_assert!(
            old.is_none(),
            "a module can't have duplicate indices, this would be a wasmer-compiler bug"
        );
        Ok(())
    }

    pub(crate) fn declare_module_name(&mut self, name: &'data str) -> WasmResult<()> {
        self.module.name = Some(name.to_string());
        Ok(())
    }

    pub(crate) fn declare_function_name(
        &mut self,
        func_index: FunctionIndex,
        name: &'data str,
    ) -> WasmResult<()> {
        self.module
            .function_names
            .insert(func_index, name.to_string());
        Ok(())
    }

    /// Provides the number of imports up front. By default this does nothing, but
    /// implementations can use this to preallocate memory if desired.
    pub(crate) fn reserve_imports(&mut self, _num: u32) -> WasmResult<()> {
        Ok(())
    }

    /// Notifies the implementation that all exports have been declared.
    pub(crate) fn finish_exports(&mut self) -> WasmResult<()> {
        Ok(())
    }

    /// Indicates that a custom section has been found in the wasm file
    pub(crate) fn custom_section(&mut self, name: &'data str, data: &'data [u8]) -> WasmResult<()> {
        let custom_section = CustomSectionIndex::from_u32(
            self.module.custom_sections_data.len().try_into().unwrap(),
        );
        self.module
            .custom_sections
            .insert(String::from(name), custom_section);
        self.module.custom_sections_data.push(Box::from(data));
        Ok(())
    }
}
