// This file contains code from external sources.
// Attributions: https://github.com/wasmerio/wasmer/blob/master/ATTRIBUTIONS.md

use crate::translator::{
    type_to_irtype, FuncEnvironment as BaseFuncEnvironment, GlobalVariable, TargetEnvironment,
};
use cranelift_codegen::cursor::FuncCursor;
use cranelift_codegen::ir;
use cranelift_codegen::ir::condcodes::*;
use cranelift_codegen::ir::immediates::{Offset32, Uimm64};
use cranelift_codegen::ir::types::*;
use cranelift_codegen::ir::{AbiParam, ArgumentPurpose, Function, InstBuilder, Signature};
use cranelift_codegen::isa::TargetFrontendConfig;
use cranelift_frontend::FunctionBuilder;
use std::convert::TryFrom;
use wasmer_compiler::wasmparser::ValType;
use wasmer_types::entity::EntityRef;
use wasmer_types::entity::PrimaryMap;
use wasmer_types::VMBuiltinFunctionIndex;
use wasmer_types::VMOffsets;
use wasmer_types::{
    FunctionIndex, FunctionType, GlobalIndex, LocalFunctionIndex, MemoryIndex, ModuleInfo,
    SignatureIndex, TableIndex, Type as WasmerType,
};
use wasmer_types::{MemoryStyle, TableStyle};
use wasmer_types::{WasmError, WasmResult};

/// Compute an `ir::ExternalName` for a given wasm function index.
pub fn get_function_name(func_index: FunctionIndex) -> ir::ExternalName {
    ir::ExternalName::user(ir::UserExternalNameRef::from_u32(func_index.as_u32()))
}

/// The type of the `current_elements` field.
pub fn type_of_vmtable_definition_current_elements(vmoffsets: &VMOffsets) -> ir::Type {
    ir::Type::int(u16::from(vmoffsets.size_of_vmtable_definition_current_elements()) * 8).unwrap()
}

/// The `FuncEnvironment` implementation for use by the `ModuleEnvironment`.
pub struct FuncEnvironment<'module_environment> {
    /// Target-specified configuration.
    target_config: TargetFrontendConfig,

    /// The module-level environment which this function-level environment belongs to.
    module: &'module_environment ModuleInfo,

    /// A stack tracking the type of local variables.
    type_stack: Vec<WasmerType>,

    /// The module function signatures
    signatures: &'module_environment PrimaryMap<SignatureIndex, ir::Signature>,

    /// The Cranelift global holding the vmctx address.
    vmctx: Option<ir::GlobalValue>,

    /// The external function signature for implementing wasm's `memory.size`
    /// for locally-defined 32-bit memories.
    memory32_size_sig: Option<ir::SigRef>,

    /// The external function signature for implementing wasm's `table.size`
    /// for locally-defined tables.
    table_size_sig: Option<ir::SigRef>,

    /// The external function signature for implementing wasm's `memory.grow`
    /// for locally-defined memories.
    memory_grow_sig: Option<ir::SigRef>,

    /// The external function signature for implementing wasm's `table.grow`
    /// for locally-defined tables.
    table_grow_sig: Option<ir::SigRef>,

    /// The external function signature for implementing wasm's `table.copy`
    /// (it's the same for both local and imported tables).
    table_copy_sig: Option<ir::SigRef>,

    /// The external function signature for implementing wasm's `table.init`.
    table_init_sig: Option<ir::SigRef>,

    /// The external function signature for implementing wasm's `elem.drop`.
    elem_drop_sig: Option<ir::SigRef>,

    /// The external function signature for implementing wasm's `memory.copy`
    /// (it's the same for both local and imported memories).
    memory_copy_sig: Option<ir::SigRef>,

    /// The external function signature for implementing wasm's `memory.fill`
    /// (it's the same for both local and imported memories).
    memory_fill_sig: Option<ir::SigRef>,

    /// The external function signature for implementing wasm's `memory.init`.
    memory_init_sig: Option<ir::SigRef>,

    /// The external function signature for implementing wasm's `data.drop`.
    data_drop_sig: Option<ir::SigRef>,

    /// The external function signature for implementing wasm's `table.get`.
    table_get_sig: Option<ir::SigRef>,

    /// The external function signature for implementing wasm's `table.set`.
    table_set_sig: Option<ir::SigRef>,

    /// The external function signature for implementing wasm's `func.ref`.
    func_ref_sig: Option<ir::SigRef>,

    /// The external function signature for implementing wasm's `table.fill`.
    table_fill_sig: Option<ir::SigRef>,

    /// The external function signature for implementing wasm's `memory32.atomic.wait32`.
    memory32_atomic_wait32_sig: Option<ir::SigRef>,

    /// The external function signature for implementing wasm's `memory32.atomic.wait64`.
    memory32_atomic_wait64_sig: Option<ir::SigRef>,

    /// The external function signature for implementing wasm's `memory32.atomic.notify`.
    memory32_atomic_notify_sig: Option<ir::SigRef>,

    /// Offsets to struct fields accessed by JIT code.
    offsets: VMOffsets,

    /// The memory styles
    memory_styles: &'module_environment PrimaryMap<MemoryIndex, MemoryStyle>,

    /// The table styles
    table_styles: &'module_environment PrimaryMap<TableIndex, TableStyle>,
}

impl<'module_environment> FuncEnvironment<'module_environment> {
    pub fn new(
        target_config: TargetFrontendConfig,
        module: &'module_environment ModuleInfo,
        signatures: &'module_environment PrimaryMap<SignatureIndex, ir::Signature>,
        memory_styles: &'module_environment PrimaryMap<MemoryIndex, MemoryStyle>,
        table_styles: &'module_environment PrimaryMap<TableIndex, TableStyle>,
    ) -> Self {
        Self {
            target_config,
            module,
            signatures,
            type_stack: vec![],
            vmctx: None,
            memory32_size_sig: None,
            table_size_sig: None,
            memory_grow_sig: None,
            table_grow_sig: None,
            table_copy_sig: None,
            table_init_sig: None,
            elem_drop_sig: None,
            memory_copy_sig: None,
            memory_fill_sig: None,
            memory_init_sig: None,
            table_get_sig: None,
            table_set_sig: None,
            data_drop_sig: None,
            func_ref_sig: None,
            table_fill_sig: None,
            memory32_atomic_wait32_sig: None,
            memory32_atomic_wait64_sig: None,
            memory32_atomic_notify_sig: None,
            offsets: VMOffsets::new(target_config.pointer_bytes(), module),
            memory_styles,
            table_styles,
        }
    }

    fn pointer_type(&self) -> ir::Type {
        self.target_config.pointer_type()
    }

    fn vmctx(&mut self, func: &mut Function) -> ir::GlobalValue {
        self.vmctx.unwrap_or_else(|| {
            let vmctx = func.create_global_value(ir::GlobalValueData::VMContext);
            self.vmctx = Some(vmctx);
            vmctx
        })
    }

    fn get_table_fill_sig(&mut self, func: &mut Function) -> ir::SigRef {
        let sig = self.table_fill_sig.unwrap_or_else(|| {
            func.import_signature(Signature {
                params: vec![
                    AbiParam::special(self.pointer_type(), ArgumentPurpose::VMContext),
                    // table index
                    AbiParam::new(I32),
                    // dst
                    AbiParam::new(I32),
                    // value
                    AbiParam::new(R64),
                    // len
                    AbiParam::new(I32),
                ],
                returns: vec![],
                call_conv: self.target_config.default_call_conv,
            })
        });
        self.table_fill_sig = Some(sig);
        sig
    }

    fn get_table_fill_func(
        &mut self,
        func: &mut Function,
        table_index: TableIndex,
    ) -> (ir::SigRef, usize, VMBuiltinFunctionIndex) {
        (
            self.get_table_fill_sig(func),
            table_index.index(),
            VMBuiltinFunctionIndex::get_table_fill_index(),
        )
    }

    fn get_func_ref_sig(&mut self, func: &mut Function) -> ir::SigRef {
        let sig = self.func_ref_sig.unwrap_or_else(|| {
            func.import_signature(Signature {
                params: vec![
                    AbiParam::special(self.pointer_type(), ArgumentPurpose::VMContext),
                    AbiParam::new(I32),
                ],
                returns: vec![AbiParam::new(R64)],
                call_conv: self.target_config.default_call_conv,
            })
        });
        self.func_ref_sig = Some(sig);
        sig
    }

    fn get_func_ref_func(
        &mut self,
        func: &mut Function,
        function_index: FunctionIndex,
    ) -> (ir::SigRef, usize, VMBuiltinFunctionIndex) {
        (
            self.get_func_ref_sig(func),
            function_index.index(),
            VMBuiltinFunctionIndex::get_func_ref_index(),
        )
    }

    fn get_table_get_sig(&mut self, func: &mut Function) -> ir::SigRef {
        let sig = self.table_get_sig.unwrap_or_else(|| {
            func.import_signature(Signature {
                params: vec![
                    AbiParam::special(self.pointer_type(), ArgumentPurpose::VMContext),
                    AbiParam::new(I32),
                    AbiParam::new(I32),
                ],
                returns: vec![AbiParam::new(R64)],
                call_conv: self.target_config.default_call_conv,
            })
        });
        self.table_get_sig = Some(sig);
        sig
    }

    fn get_table_get_func(
        &mut self,
        func: &mut Function,
        table_index: TableIndex,
    ) -> (ir::SigRef, usize, VMBuiltinFunctionIndex) {
        if self.module.is_imported_table(table_index) {
            (
                self.get_table_get_sig(func),
                table_index.index(),
                VMBuiltinFunctionIndex::get_imported_table_get_index(),
            )
        } else {
            (
                self.get_table_get_sig(func),
                self.module.local_table_index(table_index).unwrap().index(),
                VMBuiltinFunctionIndex::get_table_get_index(),
            )
        }
    }

    fn get_table_set_sig(&mut self, func: &mut Function) -> ir::SigRef {
        let sig = self.table_set_sig.unwrap_or_else(|| {
            func.import_signature(Signature {
                params: vec![
                    AbiParam::special(self.pointer_type(), ArgumentPurpose::VMContext),
                    AbiParam::new(I32),
                    AbiParam::new(I32),
                    AbiParam::new(R64),
                ],
                returns: vec![],
                call_conv: self.target_config.default_call_conv,
            })
        });
        self.table_set_sig = Some(sig);
        sig
    }

    fn get_table_set_func(
        &mut self,
        func: &mut Function,
        table_index: TableIndex,
    ) -> (ir::SigRef, usize, VMBuiltinFunctionIndex) {
        if self.module.is_imported_table(table_index) {
            (
                self.get_table_set_sig(func),
                table_index.index(),
                VMBuiltinFunctionIndex::get_imported_table_set_index(),
            )
        } else {
            (
                self.get_table_set_sig(func),
                self.module.local_table_index(table_index).unwrap().index(),
                VMBuiltinFunctionIndex::get_table_set_index(),
            )
        }
    }

    fn get_table_grow_sig(&mut self, func: &mut Function) -> ir::SigRef {
        let sig = self.table_grow_sig.unwrap_or_else(|| {
            func.import_signature(Signature {
                params: vec![
                    AbiParam::special(self.pointer_type(), ArgumentPurpose::VMContext),
                    // TODO: figure out what the representation of a Wasm value is
                    AbiParam::new(R64),
                    AbiParam::new(I32),
                    AbiParam::new(I32),
                ],
                returns: vec![AbiParam::new(I32)],
                call_conv: self.target_config.default_call_conv,
            })
        });
        self.table_grow_sig = Some(sig);
        sig
    }

    /// Return the table.grow function signature to call for the given index, along with the
    /// translated index value to pass to it and its index in `VMBuiltinFunctionsArray`.
    fn get_table_grow_func(
        &mut self,
        func: &mut Function,
        index: TableIndex,
    ) -> (ir::SigRef, usize, VMBuiltinFunctionIndex) {
        if self.module.is_imported_table(index) {
            (
                self.get_table_grow_sig(func),
                index.index(),
                VMBuiltinFunctionIndex::get_imported_table_grow_index(),
            )
        } else {
            (
                self.get_table_grow_sig(func),
                self.module.local_table_index(index).unwrap().index(),
                VMBuiltinFunctionIndex::get_table_grow_index(),
            )
        }
    }

    fn get_memory_grow_sig(&mut self, func: &mut Function) -> ir::SigRef {
        let sig = self.memory_grow_sig.unwrap_or_else(|| {
            func.import_signature(Signature {
                params: vec![
                    AbiParam::special(self.pointer_type(), ArgumentPurpose::VMContext),
                    AbiParam::new(I32),
                    AbiParam::new(I32),
                ],
                returns: vec![AbiParam::new(I32)],
                call_conv: self.target_config.default_call_conv,
            })
        });
        self.memory_grow_sig = Some(sig);
        sig
    }

    /// Return the memory.grow function signature to call for the given index, along with the
    /// translated index value to pass to it and its index in `VMBuiltinFunctionsArray`.
    fn get_memory_grow_func(
        &mut self,
        func: &mut Function,
        index: MemoryIndex,
    ) -> (ir::SigRef, usize, VMBuiltinFunctionIndex) {
        if self.module.is_imported_memory(index) {
            (
                self.get_memory_grow_sig(func),
                index.index(),
                VMBuiltinFunctionIndex::get_imported_memory32_grow_index(),
            )
        } else {
            (
                self.get_memory_grow_sig(func),
                self.module.local_memory_index(index).unwrap().index(),
                VMBuiltinFunctionIndex::get_memory32_grow_index(),
            )
        }
    }

    fn get_table_size_sig(&mut self, func: &mut Function) -> ir::SigRef {
        let sig = self.table_size_sig.unwrap_or_else(|| {
            func.import_signature(Signature {
                params: vec![
                    AbiParam::special(self.pointer_type(), ArgumentPurpose::VMContext),
                    AbiParam::new(I32),
                ],
                returns: vec![AbiParam::new(I32)],
                call_conv: self.target_config.default_call_conv,
            })
        });
        self.table_size_sig = Some(sig);
        sig
    }

    /// Return the memory.size function signature to call for the given index, along with the
    /// translated index value to pass to it and its index in `VMBuiltinFunctionsArray`.
    fn get_table_size_func(
        &mut self,
        func: &mut Function,
        index: TableIndex,
    ) -> (ir::SigRef, usize, VMBuiltinFunctionIndex) {
        if self.module.is_imported_table(index) {
            (
                self.get_table_size_sig(func),
                index.index(),
                VMBuiltinFunctionIndex::get_imported_table_size_index(),
            )
        } else {
            (
                self.get_table_size_sig(func),
                self.module.local_table_index(index).unwrap().index(),
                VMBuiltinFunctionIndex::get_table_size_index(),
            )
        }
    }

    fn get_memory32_size_sig(&mut self, func: &mut Function) -> ir::SigRef {
        let sig = self.memory32_size_sig.unwrap_or_else(|| {
            func.import_signature(Signature {
                params: vec![
                    AbiParam::special(self.pointer_type(), ArgumentPurpose::VMContext),
                    AbiParam::new(I32),
                ],
                returns: vec![AbiParam::new(I32)],
                call_conv: self.target_config.default_call_conv,
            })
        });
        self.memory32_size_sig = Some(sig);
        sig
    }

    /// Return the memory.size function signature to call for the given index, along with the
    /// translated index value to pass to it and its index in `VMBuiltinFunctionsArray`.
    fn get_memory_size_func(
        &mut self,
        func: &mut Function,
        index: MemoryIndex,
    ) -> (ir::SigRef, usize, VMBuiltinFunctionIndex) {
        if self.module.is_imported_memory(index) {
            (
                self.get_memory32_size_sig(func),
                index.index(),
                VMBuiltinFunctionIndex::get_imported_memory32_size_index(),
            )
        } else {
            (
                self.get_memory32_size_sig(func),
                self.module.local_memory_index(index).unwrap().index(),
                VMBuiltinFunctionIndex::get_memory32_size_index(),
            )
        }
    }

    fn get_table_copy_sig(&mut self, func: &mut Function) -> ir::SigRef {
        let sig = self.table_copy_sig.unwrap_or_else(|| {
            func.import_signature(Signature {
                params: vec![
                    AbiParam::special(self.pointer_type(), ArgumentPurpose::VMContext),
                    // Destination table index.
                    AbiParam::new(I32),
                    // Source table index.
                    AbiParam::new(I32),
                    // Index within destination table.
                    AbiParam::new(I32),
                    // Index within source table.
                    AbiParam::new(I32),
                    // Number of elements to copy.
                    AbiParam::new(I32),
                ],
                returns: vec![],
                call_conv: self.target_config.default_call_conv,
            })
        });
        self.table_copy_sig = Some(sig);
        sig
    }

    fn get_table_copy_func(
        &mut self,
        func: &mut Function,
        dst_table_index: TableIndex,
        src_table_index: TableIndex,
    ) -> (ir::SigRef, usize, usize, VMBuiltinFunctionIndex) {
        let sig = self.get_table_copy_sig(func);
        (
            sig,
            dst_table_index.as_u32() as usize,
            src_table_index.as_u32() as usize,
            VMBuiltinFunctionIndex::get_table_copy_index(),
        )
    }

    fn get_table_init_sig(&mut self, func: &mut Function) -> ir::SigRef {
        let sig = self.table_init_sig.unwrap_or_else(|| {
            func.import_signature(Signature {
                params: vec![
                    AbiParam::special(self.pointer_type(), ArgumentPurpose::VMContext),
                    // Table index.
                    AbiParam::new(I32),
                    // Segment index.
                    AbiParam::new(I32),
                    // Destination index within table.
                    AbiParam::new(I32),
                    // Source index within segment.
                    AbiParam::new(I32),
                    // Number of elements to initialize.
                    AbiParam::new(I32),
                ],
                returns: vec![],
                call_conv: self.target_config.default_call_conv,
            })
        });
        self.table_init_sig = Some(sig);
        sig
    }

    fn get_table_init_func(
        &mut self,
        func: &mut Function,
        table_index: TableIndex,
    ) -> (ir::SigRef, usize, VMBuiltinFunctionIndex) {
        let sig = self.get_table_init_sig(func);
        let table_index = table_index.as_u32() as usize;
        (
            sig,
            table_index,
            VMBuiltinFunctionIndex::get_table_init_index(),
        )
    }

    fn get_elem_drop_sig(&mut self, func: &mut Function) -> ir::SigRef {
        let sig = self.elem_drop_sig.unwrap_or_else(|| {
            func.import_signature(Signature {
                params: vec![
                    AbiParam::special(self.pointer_type(), ArgumentPurpose::VMContext),
                    // Element index.
                    AbiParam::new(I32),
                ],
                returns: vec![],
                call_conv: self.target_config.default_call_conv,
            })
        });
        self.elem_drop_sig = Some(sig);
        sig
    }

    fn get_elem_drop_func(&mut self, func: &mut Function) -> (ir::SigRef, VMBuiltinFunctionIndex) {
        let sig = self.get_elem_drop_sig(func);
        (sig, VMBuiltinFunctionIndex::get_elem_drop_index())
    }

    fn get_memory_copy_sig(&mut self, func: &mut Function) -> ir::SigRef {
        let sig = self.memory_copy_sig.unwrap_or_else(|| {
            func.import_signature(Signature {
                params: vec![
                    AbiParam::special(self.pointer_type(), ArgumentPurpose::VMContext),
                    // Memory index.
                    AbiParam::new(I32),
                    // Destination address.
                    AbiParam::new(I32),
                    // Source address.
                    AbiParam::new(I32),
                    // Length.
                    AbiParam::new(I32),
                ],
                returns: vec![],
                call_conv: self.target_config.default_call_conv,
            })
        });
        self.memory_copy_sig = Some(sig);
        sig
    }

    fn get_memory_copy_func(
        &mut self,
        func: &mut Function,
        memory_index: MemoryIndex,
    ) -> (ir::SigRef, usize, VMBuiltinFunctionIndex) {
        let sig = self.get_memory_copy_sig(func);
        if let Some(local_memory_index) = self.module.local_memory_index(memory_index) {
            (
                sig,
                local_memory_index.index(),
                VMBuiltinFunctionIndex::get_memory_copy_index(),
            )
        } else {
            (
                sig,
                memory_index.index(),
                VMBuiltinFunctionIndex::get_imported_memory_copy_index(),
            )
        }
    }

    fn get_memory_fill_sig(&mut self, func: &mut Function) -> ir::SigRef {
        let sig = self.memory_fill_sig.unwrap_or_else(|| {
            func.import_signature(Signature {
                params: vec![
                    AbiParam::special(self.pointer_type(), ArgumentPurpose::VMContext),
                    // Memory index.
                    AbiParam::new(I32),
                    // Destination address.
                    AbiParam::new(I32),
                    // Value.
                    AbiParam::new(I32),
                    // Length.
                    AbiParam::new(I32),
                ],
                returns: vec![],
                call_conv: self.target_config.default_call_conv,
            })
        });
        self.memory_fill_sig = Some(sig);
        sig
    }

    fn get_memory_fill_func(
        &mut self,
        func: &mut Function,
        memory_index: MemoryIndex,
    ) -> (ir::SigRef, usize, VMBuiltinFunctionIndex) {
        let sig = self.get_memory_fill_sig(func);
        if let Some(local_memory_index) = self.module.local_memory_index(memory_index) {
            (
                sig,
                local_memory_index.index(),
                VMBuiltinFunctionIndex::get_memory_fill_index(),
            )
        } else {
            (
                sig,
                memory_index.index(),
                VMBuiltinFunctionIndex::get_imported_memory_fill_index(),
            )
        }
    }

    fn get_memory_init_sig(&mut self, func: &mut Function) -> ir::SigRef {
        let sig = self.memory_init_sig.unwrap_or_else(|| {
            func.import_signature(Signature {
                params: vec![
                    AbiParam::special(self.pointer_type(), ArgumentPurpose::VMContext),
                    // Memory index.
                    AbiParam::new(I32),
                    // Data index.
                    AbiParam::new(I32),
                    // Destination address.
                    AbiParam::new(I32),
                    // Source index within the data segment.
                    AbiParam::new(I32),
                    // Length.
                    AbiParam::new(I32),
                ],
                returns: vec![],
                call_conv: self.target_config.default_call_conv,
            })
        });
        self.memory_init_sig = Some(sig);
        sig
    }

    fn get_memory_init_func(
        &mut self,
        func: &mut Function,
    ) -> (ir::SigRef, VMBuiltinFunctionIndex) {
        let sig = self.get_memory_init_sig(func);
        (sig, VMBuiltinFunctionIndex::get_memory_init_index())
    }

    fn get_data_drop_sig(&mut self, func: &mut Function) -> ir::SigRef {
        let sig = self.data_drop_sig.unwrap_or_else(|| {
            func.import_signature(Signature {
                params: vec![
                    AbiParam::special(self.pointer_type(), ArgumentPurpose::VMContext),
                    // Data index.
                    AbiParam::new(I32),
                ],
                returns: vec![],
                call_conv: self.target_config.default_call_conv,
            })
        });
        self.data_drop_sig = Some(sig);
        sig
    }

    fn get_data_drop_func(&mut self, func: &mut Function) -> (ir::SigRef, VMBuiltinFunctionIndex) {
        let sig = self.get_data_drop_sig(func);
        (sig, VMBuiltinFunctionIndex::get_data_drop_index())
    }

    fn get_memory32_atomic_wait32_sig(&mut self, func: &mut Function) -> ir::SigRef {
        let sig = self.memory32_atomic_wait32_sig.unwrap_or_else(|| {
            func.import_signature(Signature {
                params: vec![
                    AbiParam::special(self.pointer_type(), ArgumentPurpose::VMContext),
                    // Memory Index
                    AbiParam::new(I32),
                    // Dst
                    AbiParam::new(I32),
                    // Val
                    AbiParam::new(I32),
                    // Timeout
                    AbiParam::new(I64),
                ],
                returns: vec![AbiParam::new(I32)],
                call_conv: self.target_config.default_call_conv,
            })
        });
        self.memory32_atomic_wait32_sig = Some(sig);
        sig
    }

    /// Return the memory.atomic.wait32 function signature to call for the given index,
    /// along with the translated index value to pass to it
    /// and its index in `VMBuiltinFunctionsArray`.
    fn get_memory_atomic_wait32_func(
        &mut self,
        func: &mut Function,
        index: MemoryIndex,
    ) -> (ir::SigRef, usize, VMBuiltinFunctionIndex) {
        if self.module.is_imported_memory(index) {
            (
                self.get_memory32_atomic_wait32_sig(func),
                index.index(),
                VMBuiltinFunctionIndex::get_imported_memory_atomic_wait32_index(),
            )
        } else {
            (
                self.get_memory32_atomic_wait32_sig(func),
                self.module.local_memory_index(index).unwrap().index(),
                VMBuiltinFunctionIndex::get_memory_atomic_wait32_index(),
            )
        }
    }

    fn get_memory32_atomic_wait64_sig(&mut self, func: &mut Function) -> ir::SigRef {
        let sig = self.memory32_atomic_wait64_sig.unwrap_or_else(|| {
            func.import_signature(Signature {
                params: vec![
                    AbiParam::special(self.pointer_type(), ArgumentPurpose::VMContext),
                    // Memory Index
                    AbiParam::new(I32),
                    // Dst
                    AbiParam::new(I32),
                    // Val
                    AbiParam::new(I64),
                    // Timeout
                    AbiParam::new(I64),
                ],
                returns: vec![AbiParam::new(I32)],
                call_conv: self.target_config.default_call_conv,
            })
        });
        self.memory32_atomic_wait64_sig = Some(sig);
        sig
    }

    /// Return the memory.atomic.wait64 function signature to call for the given index,
    /// along with the translated index value to pass to it
    /// and its index in `VMBuiltinFunctionsArray`.
    fn get_memory_atomic_wait64_func(
        &mut self,
        func: &mut Function,
        index: MemoryIndex,
    ) -> (ir::SigRef, usize, VMBuiltinFunctionIndex) {
        if self.module.is_imported_memory(index) {
            (
                self.get_memory32_atomic_wait64_sig(func),
                index.index(),
                VMBuiltinFunctionIndex::get_imported_memory_atomic_wait64_index(),
            )
        } else {
            (
                self.get_memory32_atomic_wait64_sig(func),
                self.module.local_memory_index(index).unwrap().index(),
                VMBuiltinFunctionIndex::get_memory_atomic_wait64_index(),
            )
        }
    }

    fn get_memory32_atomic_notify_sig(&mut self, func: &mut Function) -> ir::SigRef {
        let sig = self.memory32_atomic_notify_sig.unwrap_or_else(|| {
            func.import_signature(Signature {
                params: vec![
                    AbiParam::special(self.pointer_type(), ArgumentPurpose::VMContext),
                    // Memory Index
                    AbiParam::new(I32),
                    // Dst
                    AbiParam::new(I32),
                    // Count
                    AbiParam::new(I32),
                ],
                returns: vec![AbiParam::new(I32)],
                call_conv: self.target_config.default_call_conv,
            })
        });
        self.memory32_atomic_notify_sig = Some(sig);
        sig
    }

    /// Return the memory.atomic.notify function signature to call for the given index,
    /// along with the translated index value to pass to it
    /// and its index in `VMBuiltinFunctionsArray`.
    fn get_memory_atomic_notify_func(
        &mut self,
        func: &mut Function,
        index: MemoryIndex,
    ) -> (ir::SigRef, usize, VMBuiltinFunctionIndex) {
        if self.module.is_imported_memory(index) {
            (
                self.get_memory32_atomic_notify_sig(func),
                index.index(),
                VMBuiltinFunctionIndex::get_imported_memory_atomic_notify_index(),
            )
        } else {
            (
                self.get_memory32_atomic_notify_sig(func),
                self.module.local_memory_index(index).unwrap().index(),
                VMBuiltinFunctionIndex::get_memory_atomic_notify_index(),
            )
        }
    }

    /// Translates load of builtin function and returns a pair of values `vmctx`
    /// and address of the loaded function.
    fn translate_load_builtin_function_address(
        &mut self,
        pos: &mut FuncCursor<'_>,
        callee_func_idx: VMBuiltinFunctionIndex,
    ) -> (ir::Value, ir::Value) {
        // We use an indirect call so that we don't have to patch the code at runtime.
        let pointer_type = self.pointer_type();
        let vmctx = self.vmctx(pos.func);
        let base = pos.ins().global_value(pointer_type, vmctx);

        let mut mem_flags = ir::MemFlags::trusted();
        mem_flags.set_readonly();

        // Load the callee address.
        let body_offset =
            i32::try_from(self.offsets.vmctx_builtin_function(callee_func_idx)).unwrap();
        let func_addr = pos.ins().load(pointer_type, mem_flags, base, body_offset);

        (base, func_addr)
    }
}

impl<'module_environment> TargetEnvironment for FuncEnvironment<'module_environment> {
    fn target_config(&self) -> TargetFrontendConfig {
        self.target_config
    }
}

impl<'module_environment> BaseFuncEnvironment for FuncEnvironment<'module_environment> {
    fn is_wasm_parameter(&self, _signature: &ir::Signature, index: usize) -> bool {
        // The first parameter is the vmctx. The rest are the wasm parameters.
        index >= 1
    }

    fn make_table(&mut self, func: &mut ir::Function, index: TableIndex) -> WasmResult<ir::Table> {
        let pointer_type = self.pointer_type();

        let (ptr, base_offset, current_elements_offset) = {
            let vmctx = self.vmctx(func);
            if let Some(def_index) = self.module.local_table_index(index) {
                let base_offset =
                    i32::try_from(self.offsets.vmctx_vmtable_definition_base(def_index)).unwrap();
                let current_elements_offset = i32::try_from(
                    self.offsets
                        .vmctx_vmtable_definition_current_elements(def_index),
                )
                .unwrap();
                (vmctx, base_offset, current_elements_offset)
            } else {
                let from_offset = self.offsets.vmctx_vmtable_import_definition(index);
                let table = func.create_global_value(ir::GlobalValueData::Load {
                    base: vmctx,
                    offset: Offset32::new(i32::try_from(from_offset).unwrap()),
                    global_type: pointer_type,
                    readonly: true,
                });
                let base_offset = i32::from(self.offsets.vmtable_definition_base());
                let current_elements_offset =
                    i32::from(self.offsets.vmtable_definition_current_elements());
                (table, base_offset, current_elements_offset)
            }
        };

        let base_gv = func.create_global_value(ir::GlobalValueData::Load {
            base: ptr,
            offset: Offset32::new(base_offset),
            global_type: pointer_type,
            readonly: false,
        });
        let bound_gv = func.create_global_value(ir::GlobalValueData::Load {
            base: ptr,
            offset: Offset32::new(current_elements_offset),
            global_type: type_of_vmtable_definition_current_elements(&self.offsets),
            readonly: false,
        });

        let element_size = match self.table_styles[index] {
            TableStyle::CallerChecksSignature => u64::from(self.offsets.size_of_vm_funcref()),
        };

        Ok(func.create_table(ir::TableData {
            base_gv,
            min_size: Uimm64::new(0),
            bound_gv,
            element_size: Uimm64::new(element_size),
            index_type: I32,
        }))
    }

    fn translate_table_grow(
        &mut self,
        mut pos: cranelift_codegen::cursor::FuncCursor<'_>,
        table_index: TableIndex,
        _table: ir::Table,
        delta: ir::Value,
        init_value: ir::Value,
    ) -> WasmResult<ir::Value> {
        let (func_sig, index_arg, func_idx) = self.get_table_grow_func(pos.func, table_index);
        let table_index = pos.ins().iconst(I32, index_arg as i64);
        let (vmctx, func_addr) = self.translate_load_builtin_function_address(&mut pos, func_idx);
        let call_inst = pos.ins().call_indirect(
            func_sig,
            func_addr,
            &[vmctx, init_value, delta, table_index],
        );
        Ok(*pos.func.dfg.inst_results(call_inst).first().unwrap())
    }

    fn translate_table_get(
        &mut self,
        builder: &mut FunctionBuilder,
        table_index: TableIndex,
        _table: ir::Table,
        index: ir::Value,
    ) -> WasmResult<ir::Value> {
        let mut pos = builder.cursor();

        let (func_sig, table_index_arg, func_idx) = self.get_table_get_func(pos.func, table_index);
        let table_index = pos.ins().iconst(I32, table_index_arg as i64);
        let (vmctx, func_addr) = self.translate_load_builtin_function_address(&mut pos, func_idx);
        let call_inst = pos
            .ins()
            .call_indirect(func_sig, func_addr, &[vmctx, table_index, index]);
        Ok(*pos.func.dfg.inst_results(call_inst).first().unwrap())
    }

    fn translate_table_set(
        &mut self,
        builder: &mut FunctionBuilder,
        table_index: TableIndex,
        _table: ir::Table,
        value: ir::Value,
        index: ir::Value,
    ) -> WasmResult<()> {
        let mut pos = builder.cursor();

        let (func_sig, table_index_arg, func_idx) = self.get_table_set_func(pos.func, table_index);
        let table_index = pos.ins().iconst(I32, table_index_arg as i64);
        let (vmctx, func_addr) = self.translate_load_builtin_function_address(&mut pos, func_idx);
        pos.ins()
            .call_indirect(func_sig, func_addr, &[vmctx, table_index, index, value]);
        Ok(())
    }

    fn translate_table_fill(
        &mut self,
        mut pos: cranelift_codegen::cursor::FuncCursor<'_>,
        table_index: TableIndex,
        dst: ir::Value,
        val: ir::Value,
        len: ir::Value,
    ) -> WasmResult<()> {
        let (func_sig, table_index_arg, func_idx) = self.get_table_fill_func(pos.func, table_index);
        let (vmctx, func_addr) = self.translate_load_builtin_function_address(&mut pos, func_idx);

        let table_index_arg = pos.ins().iconst(I32, table_index_arg as i64);
        pos.ins().call_indirect(
            func_sig,
            func_addr,
            &[vmctx, table_index_arg, dst, val, len],
        );

        Ok(())
    }

    fn translate_ref_null(
        &mut self,
        mut pos: cranelift_codegen::cursor::FuncCursor,
        ty: ValType,
    ) -> WasmResult<ir::Value> {
        Ok(match ty {
            ValType::FuncRef => pos.ins().null(self.reference_type()),
            ValType::ExternRef => pos.ins().null(self.reference_type()),
            _ => {
                return Err(WasmError::Unsupported(
                    "`ref.null T` that is not a `funcref` or an `externref`".into(),
                ));
            }
        })
    }

    fn translate_ref_is_null(
        &mut self,
        mut pos: cranelift_codegen::cursor::FuncCursor,
        value: ir::Value,
    ) -> WasmResult<ir::Value> {
        let bool_is_null = match pos.func.dfg.value_type(value) {
            // `externref`
            ty if ty.is_ref() => pos.ins().is_null(value),
            // `funcref`
            ty if ty == self.pointer_type() => {
                pos.ins()
                    .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, value, 0)
            }
            _ => unreachable!(),
        };

        Ok(pos.ins().uextend(ir::types::I32, bool_is_null))
    }

    fn translate_ref_func(
        &mut self,
        mut pos: cranelift_codegen::cursor::FuncCursor<'_>,
        func_index: FunctionIndex,
    ) -> WasmResult<ir::Value> {
        // TODO: optimize this by storing a pointer to local func_index funcref metadata
        // so that local funcref is just (*global + offset) instead of a function call
        //
        // Actually we can do the above for both local and imported functions because
        // all of those are known statically.
        //
        // prototyping with a function call though

        let (func_sig, func_index_arg, func_idx) = self.get_func_ref_func(pos.func, func_index);
        let (vmctx, func_addr) = self.translate_load_builtin_function_address(&mut pos, func_idx);

        let func_index_arg = pos.ins().iconst(I32, func_index_arg as i64);
        let call_inst = pos
            .ins()
            .call_indirect(func_sig, func_addr, &[vmctx, func_index_arg]);

        Ok(*pos.func.dfg.inst_results(call_inst).first().unwrap())
    }

    fn translate_custom_global_get(
        &mut self,
        mut _pos: cranelift_codegen::cursor::FuncCursor<'_>,
        _index: GlobalIndex,
    ) -> WasmResult<ir::Value> {
        unreachable!("we don't make any custom globals")
    }

    fn translate_custom_global_set(
        &mut self,
        mut _pos: cranelift_codegen::cursor::FuncCursor<'_>,
        _index: GlobalIndex,
        _value: ir::Value,
    ) -> WasmResult<()> {
        unreachable!("we don't make any custom globals")
    }

    fn make_heap(&mut self, func: &mut ir::Function, index: MemoryIndex) -> WasmResult<ir::Heap> {
        let pointer_type = self.pointer_type();

        let (ptr, base_offset, current_length_offset) = {
            let vmctx = self.vmctx(func);
            if let Some(def_index) = self.module.local_memory_index(index) {
                let base_offset =
                    i32::try_from(self.offsets.vmctx_vmmemory_definition_base(def_index)).unwrap();
                let current_length_offset = i32::try_from(
                    self.offsets
                        .vmctx_vmmemory_definition_current_length(def_index),
                )
                .unwrap();
                (vmctx, base_offset, current_length_offset)
            } else {
                let from_offset = self.offsets.vmctx_vmmemory_import_definition(index);
                let memory = func.create_global_value(ir::GlobalValueData::Load {
                    base: vmctx,
                    offset: Offset32::new(i32::try_from(from_offset).unwrap()),
                    global_type: pointer_type,
                    readonly: true,
                });
                let base_offset = i32::from(self.offsets.vmmemory_definition_base());
                let current_length_offset =
                    i32::from(self.offsets.vmmemory_definition_current_length());
                (memory, base_offset, current_length_offset)
            }
        };

        // If we have a declared maximum, we can make this a "static" heap, which is
        // allocated up front and never moved.
        let (offset_guard_size, heap_style, readonly_base) = match self.memory_styles[index] {
            MemoryStyle::Dynamic { offset_guard_size } => {
                let heap_bound = func.create_global_value(ir::GlobalValueData::Load {
                    base: ptr,
                    offset: Offset32::new(current_length_offset),
                    global_type: pointer_type,
                    readonly: false,
                });
                (
                    Uimm64::new(offset_guard_size),
                    ir::HeapStyle::Dynamic {
                        bound_gv: heap_bound,
                    },
                    false,
                )
            }
            MemoryStyle::Static {
                bound,
                offset_guard_size,
            } => (
                Uimm64::new(offset_guard_size),
                ir::HeapStyle::Static {
                    bound: Uimm64::new(bound.bytes().0 as u64),
                },
                true,
            ),
        };

        let heap_base = func.create_global_value(ir::GlobalValueData::Load {
            base: ptr,
            offset: Offset32::new(base_offset),
            global_type: pointer_type,
            readonly: readonly_base,
        });
        Ok(func.create_heap(ir::HeapData {
            base: heap_base,
            min_size: 0.into(),
            offset_guard_size,
            style: heap_style,
            index_type: I32,
        }))
    }

    fn make_global(
        &mut self,
        func: &mut ir::Function,
        index: GlobalIndex,
    ) -> WasmResult<GlobalVariable> {
        let pointer_type = self.pointer_type();

        let (ptr, offset) = {
            let vmctx = self.vmctx(func);
            let from_offset = if let Some(def_index) = self.module.local_global_index(index) {
                self.offsets.vmctx_vmglobal_definition(def_index)
            } else {
                self.offsets.vmctx_vmglobal_import_definition(index)
            };
            let global = func.create_global_value(ir::GlobalValueData::Load {
                base: vmctx,
                offset: Offset32::new(i32::try_from(from_offset).unwrap()),
                global_type: pointer_type,
                readonly: true,
            });

            (global, 0)
        };

        Ok(GlobalVariable::Memory {
            gv: ptr,
            offset: offset.into(),
            ty: type_to_irtype(self.module.globals[index].ty, self.target_config())?,
        })
    }

    fn make_indirect_sig(
        &mut self,
        func: &mut ir::Function,
        index: SignatureIndex,
    ) -> WasmResult<ir::SigRef> {
        Ok(func.import_signature(self.signatures[index].clone()))
    }

    fn make_direct_func(
        &mut self,
        func: &mut ir::Function,
        index: FunctionIndex,
    ) -> WasmResult<ir::FuncRef> {
        let sigidx = self.module.functions[index];
        let signature = func.import_signature(self.signatures[sigidx].clone());
        let name = get_function_name(index);
        Ok(func.import_function(ir::ExtFuncData {
            name,
            signature,
            colocated: true,
        }))
    }

    fn translate_call_indirect(
        &mut self,
        mut pos: FuncCursor<'_>,
        table_index: TableIndex,
        table: ir::Table,
        sig_index: SignatureIndex,
        sig_ref: ir::SigRef,
        callee: ir::Value,
        call_args: &[ir::Value],
    ) -> WasmResult<ir::Inst> {
        let pointer_type = self.pointer_type();

        let table_entry_addr = pos.ins().table_addr(pointer_type, table, callee, 0);

        // Dereference table_entry_addr to get the function address.
        let mem_flags = ir::MemFlags::trusted();
        let table_entry_addr = pos.ins().load(
            pointer_type,
            mem_flags,
            table_entry_addr,
            i32::from(self.offsets.vm_funcref_anyfunc_ptr()),
        );

        // check if the funcref is null
        pos.ins()
            .trapz(table_entry_addr, ir::TrapCode::IndirectCallToNull);

        let func_addr = pos.ins().load(
            pointer_type,
            mem_flags,
            table_entry_addr,
            i32::from(self.offsets.vmcaller_checked_anyfunc_func_ptr()),
        );

        // If necessary, check the signature.
        match self.table_styles[table_index] {
            TableStyle::CallerChecksSignature => {
                let sig_id_size = self.offsets.size_of_vmshared_signature_index();
                let sig_id_type = ir::Type::int(u16::from(sig_id_size) * 8).unwrap();
                let vmctx = self.vmctx(pos.func);
                let base = pos.ins().global_value(pointer_type, vmctx);
                let offset =
                    i32::try_from(self.offsets.vmctx_vmshared_signature_id(sig_index)).unwrap();

                // Load the caller ID.
                let mut mem_flags = ir::MemFlags::trusted();
                mem_flags.set_readonly();
                let caller_sig_id = pos.ins().load(sig_id_type, mem_flags, base, offset);

                // Load the callee ID.
                let mem_flags = ir::MemFlags::trusted();
                let callee_sig_id = pos.ins().load(
                    sig_id_type,
                    mem_flags,
                    table_entry_addr,
                    i32::from(self.offsets.vmcaller_checked_anyfunc_type_index()),
                );

                // Check that they match.
                let cmp = pos.ins().icmp(IntCC::Equal, callee_sig_id, caller_sig_id);
                pos.ins().trapz(cmp, ir::TrapCode::BadSignature);
            }
        }

        let mut real_call_args = Vec::with_capacity(call_args.len() + 2);

        // First append the callee vmctx address.
        let vmctx = pos.ins().load(
            pointer_type,
            mem_flags,
            table_entry_addr,
            i32::from(self.offsets.vmcaller_checked_anyfunc_vmctx()),
        );
        real_call_args.push(vmctx);

        // Then append the regular call arguments.
        real_call_args.extend_from_slice(call_args);

        Ok(pos.ins().call_indirect(sig_ref, func_addr, &real_call_args))
    }

    fn translate_call(
        &mut self,
        mut pos: FuncCursor<'_>,
        callee_index: FunctionIndex,
        callee: ir::FuncRef,
        call_args: &[ir::Value],
    ) -> WasmResult<ir::Inst> {
        let mut real_call_args = Vec::with_capacity(call_args.len() + 2);

        // Handle direct calls to locally-defined functions.
        if !self.module.is_imported_function(callee_index) {
            // Let's get the caller vmctx
            let caller_vmctx = pos.func.special_param(ArgumentPurpose::VMContext).unwrap();
            // First append the callee vmctx address, which is the same as the caller vmctx in
            // this case.
            real_call_args.push(caller_vmctx);

            // Then append the regular call arguments.
            real_call_args.extend_from_slice(call_args);

            return Ok(pos.ins().call(callee, &real_call_args));
        }

        // Handle direct calls to imported functions. We use an indirect call
        // so that we don't have to patch the code at runtime.
        let pointer_type = self.pointer_type();
        let sig_ref = pos.func.dfg.ext_funcs[callee].signature;
        let vmctx = self.vmctx(pos.func);
        let base = pos.ins().global_value(pointer_type, vmctx);

        let mem_flags = ir::MemFlags::trusted();

        // Load the callee address.
        let body_offset =
            i32::try_from(self.offsets.vmctx_vmfunction_import_body(callee_index)).unwrap();
        let func_addr = pos.ins().load(pointer_type, mem_flags, base, body_offset);

        // First append the callee vmctx address.
        let vmctx_offset =
            i32::try_from(self.offsets.vmctx_vmfunction_import_vmctx(callee_index)).unwrap();
        let vmctx = pos.ins().load(pointer_type, mem_flags, base, vmctx_offset);
        real_call_args.push(vmctx);

        // Then append the regular call arguments.
        real_call_args.extend_from_slice(call_args);

        Ok(pos.ins().call_indirect(sig_ref, func_addr, &real_call_args))
    }

    fn translate_memory_grow(
        &mut self,
        mut pos: FuncCursor<'_>,
        index: MemoryIndex,
        _heap: ir::Heap,
        val: ir::Value,
    ) -> WasmResult<ir::Value> {
        let (func_sig, index_arg, func_idx) = self.get_memory_grow_func(pos.func, index);
        let memory_index = pos.ins().iconst(I32, index_arg as i64);
        let (vmctx, func_addr) = self.translate_load_builtin_function_address(&mut pos, func_idx);
        let call_inst = pos
            .ins()
            .call_indirect(func_sig, func_addr, &[vmctx, val, memory_index]);
        Ok(*pos.func.dfg.inst_results(call_inst).first().unwrap())
    }

    fn translate_memory_size(
        &mut self,
        mut pos: FuncCursor<'_>,
        index: MemoryIndex,
        _heap: ir::Heap,
    ) -> WasmResult<ir::Value> {
        let (func_sig, index_arg, func_idx) = self.get_memory_size_func(pos.func, index);
        let memory_index = pos.ins().iconst(I32, index_arg as i64);
        let (vmctx, func_addr) = self.translate_load_builtin_function_address(&mut pos, func_idx);
        let call_inst = pos
            .ins()
            .call_indirect(func_sig, func_addr, &[vmctx, memory_index]);
        Ok(*pos.func.dfg.inst_results(call_inst).first().unwrap())
    }

    fn translate_memory_copy(
        &mut self,
        mut pos: FuncCursor,
        src_index: MemoryIndex,
        _src_heap: ir::Heap,
        _dst_index: MemoryIndex,
        _dst_heap: ir::Heap,
        dst: ir::Value,
        src: ir::Value,
        len: ir::Value,
    ) -> WasmResult<()> {
        let (func_sig, src_index, func_idx) = self.get_memory_copy_func(pos.func, src_index);

        let src_index_arg = pos.ins().iconst(I32, src_index as i64);

        let (vmctx, func_addr) = self.translate_load_builtin_function_address(&mut pos, func_idx);

        pos.ins()
            .call_indirect(func_sig, func_addr, &[vmctx, src_index_arg, dst, src, len]);

        Ok(())
    }

    fn translate_memory_fill(
        &mut self,
        mut pos: FuncCursor,
        memory_index: MemoryIndex,
        _heap: ir::Heap,
        dst: ir::Value,
        val: ir::Value,
        len: ir::Value,
    ) -> WasmResult<()> {
        let (func_sig, memory_index, func_idx) = self.get_memory_fill_func(pos.func, memory_index);

        let memory_index_arg = pos.ins().iconst(I32, memory_index as i64);

        let (vmctx, func_addr) = self.translate_load_builtin_function_address(&mut pos, func_idx);

        pos.ins().call_indirect(
            func_sig,
            func_addr,
            &[vmctx, memory_index_arg, dst, val, len],
        );

        Ok(())
    }

    fn translate_memory_init(
        &mut self,
        mut pos: FuncCursor,
        memory_index: MemoryIndex,
        _heap: ir::Heap,
        seg_index: u32,
        dst: ir::Value,
        src: ir::Value,
        len: ir::Value,
    ) -> WasmResult<()> {
        let (func_sig, func_idx) = self.get_memory_init_func(pos.func);

        let memory_index_arg = pos.ins().iconst(I32, memory_index.index() as i64);
        let seg_index_arg = pos.ins().iconst(I32, seg_index as i64);

        let (vmctx, func_addr) = self.translate_load_builtin_function_address(&mut pos, func_idx);

        pos.ins().call_indirect(
            func_sig,
            func_addr,
            &[vmctx, memory_index_arg, seg_index_arg, dst, src, len],
        );

        Ok(())
    }

    fn translate_data_drop(&mut self, mut pos: FuncCursor, seg_index: u32) -> WasmResult<()> {
        let (func_sig, func_idx) = self.get_data_drop_func(pos.func);
        let seg_index_arg = pos.ins().iconst(I32, seg_index as i64);
        let (vmctx, func_addr) = self.translate_load_builtin_function_address(&mut pos, func_idx);
        pos.ins()
            .call_indirect(func_sig, func_addr, &[vmctx, seg_index_arg]);
        Ok(())
    }

    fn translate_table_size(
        &mut self,
        mut pos: FuncCursor,
        table_index: TableIndex,
        _table: ir::Table,
    ) -> WasmResult<ir::Value> {
        let (func_sig, index_arg, func_idx) = self.get_table_size_func(pos.func, table_index);
        let table_index = pos.ins().iconst(I32, index_arg as i64);
        let (vmctx, func_addr) = self.translate_load_builtin_function_address(&mut pos, func_idx);
        let call_inst = pos
            .ins()
            .call_indirect(func_sig, func_addr, &[vmctx, table_index]);
        Ok(*pos.func.dfg.inst_results(call_inst).first().unwrap())
    }

    fn translate_table_copy(
        &mut self,
        mut pos: FuncCursor,
        dst_table_index: TableIndex,
        _dst_table: ir::Table,
        src_table_index: TableIndex,
        _src_table: ir::Table,
        dst: ir::Value,
        src: ir::Value,
        len: ir::Value,
    ) -> WasmResult<()> {
        let (func_sig, dst_table_index_arg, src_table_index_arg, func_idx) =
            self.get_table_copy_func(pos.func, dst_table_index, src_table_index);

        let dst_table_index_arg = pos.ins().iconst(I32, dst_table_index_arg as i64);
        let src_table_index_arg = pos.ins().iconst(I32, src_table_index_arg as i64);

        let (vmctx, func_addr) = self.translate_load_builtin_function_address(&mut pos, func_idx);

        pos.ins().call_indirect(
            func_sig,
            func_addr,
            &[
                vmctx,
                dst_table_index_arg,
                src_table_index_arg,
                dst,
                src,
                len,
            ],
        );

        Ok(())
    }

    fn translate_table_init(
        &mut self,
        mut pos: FuncCursor,
        seg_index: u32,
        table_index: TableIndex,
        _table: ir::Table,
        dst: ir::Value,
        src: ir::Value,
        len: ir::Value,
    ) -> WasmResult<()> {
        let (func_sig, table_index_arg, func_idx) = self.get_table_init_func(pos.func, table_index);

        let table_index_arg = pos.ins().iconst(I32, table_index_arg as i64);
        let seg_index_arg = pos.ins().iconst(I32, seg_index as i64);

        let (vmctx, func_addr) = self.translate_load_builtin_function_address(&mut pos, func_idx);

        pos.ins().call_indirect(
            func_sig,
            func_addr,
            &[vmctx, table_index_arg, seg_index_arg, dst, src, len],
        );

        Ok(())
    }

    fn translate_elem_drop(&mut self, mut pos: FuncCursor, elem_index: u32) -> WasmResult<()> {
        let (func_sig, func_idx) = self.get_elem_drop_func(pos.func);

        let elem_index_arg = pos.ins().iconst(I32, elem_index as i64);

        let (vmctx, func_addr) = self.translate_load_builtin_function_address(&mut pos, func_idx);

        pos.ins()
            .call_indirect(func_sig, func_addr, &[vmctx, elem_index_arg]);

        Ok(())
    }

    fn translate_atomic_wait(
        &mut self,
        mut pos: FuncCursor,
        index: MemoryIndex,
        _heap: ir::Heap,
        addr: ir::Value,
        expected: ir::Value,
        timeout: ir::Value,
    ) -> WasmResult<ir::Value> {
        let (func_sig, index_arg, func_idx) = if pos.func.dfg.value_type(expected) == I64 {
            self.get_memory_atomic_wait64_func(pos.func, index)
        } else {
            self.get_memory_atomic_wait32_func(pos.func, index)
        };
        let memory_index = pos.ins().iconst(I32, index_arg as i64);
        let (vmctx, func_addr) = self.translate_load_builtin_function_address(&mut pos, func_idx);
        let call_inst = pos.ins().call_indirect(
            func_sig,
            func_addr,
            &[vmctx, memory_index, addr, expected, timeout],
        );
        Ok(*pos.func.dfg.inst_results(call_inst).first().unwrap())
    }

    fn translate_atomic_notify(
        &mut self,
        mut pos: FuncCursor,
        index: MemoryIndex,
        _heap: ir::Heap,
        addr: ir::Value,
        count: ir::Value,
    ) -> WasmResult<ir::Value> {
        let (func_sig, index_arg, func_idx) = self.get_memory_atomic_notify_func(pos.func, index);
        let memory_index = pos.ins().iconst(I32, index_arg as i64);
        let (vmctx, func_addr) = self.translate_load_builtin_function_address(&mut pos, func_idx);
        let call_inst =
            pos.ins()
                .call_indirect(func_sig, func_addr, &[vmctx, memory_index, addr, count]);
        Ok(*pos.func.dfg.inst_results(call_inst).first().unwrap())
    }

    fn get_global_type(&self, global_index: GlobalIndex) -> Option<WasmerType> {
        Some(self.module.globals.get(global_index)?.ty)
    }

    fn push_local_decl_on_stack(&mut self, ty: WasmerType) {
        self.type_stack.push(ty);
    }

    fn push_params_on_stack(&mut self, function_index: LocalFunctionIndex) {
        let func_index = self.module.func_index(function_index);
        let sig_idx = self.module.functions[func_index];
        let signature = &self.module.signatures[sig_idx];
        for param in signature.params() {
            self.type_stack.push(*param);
        }
    }

    fn get_local_type(&self, local_index: u32) -> Option<WasmerType> {
        self.type_stack.get(local_index as usize).cloned()
    }

    fn get_local_types(&self) -> &[WasmerType] {
        &self.type_stack
    }

    fn get_function_type(&self, function_index: FunctionIndex) -> Option<&FunctionType> {
        let sig_idx = self.module.functions.get(function_index)?;
        Some(&self.module.signatures[*sig_idx])
    }

    fn get_function_sig(&self, sig_index: SignatureIndex) -> Option<&FunctionType> {
        self.module.signatures.get(sig_index)
    }
}
