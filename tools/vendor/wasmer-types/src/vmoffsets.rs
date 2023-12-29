// This file contains code from external sources.
// Attributions: https://github.com/wasmerio/wasmer/blob/master/ATTRIBUTIONS.md

//! Offsets and sizes of various structs in wasmer-vm's vmcontext
//! module.

#![deny(rustdoc::broken_intra_doc_links)]

use crate::{
    FunctionIndex, GlobalIndex, LocalGlobalIndex, LocalMemoryIndex, LocalTableIndex, MemoryIndex,
    ModuleInfo, SignatureIndex, TableIndex,
};
use more_asserts::assert_lt;
use std::convert::TryFrom;
use std::mem::size_of;

/// An index type for builtin functions.
#[derive(Copy, Clone, Debug)]
pub struct VMBuiltinFunctionIndex(u32);

impl VMBuiltinFunctionIndex {
    /// Returns an index for wasm's `memory.grow` builtin function.
    pub const fn get_memory32_grow_index() -> Self {
        Self(0)
    }
    /// Returns an index for wasm's imported `memory.grow` builtin function.
    pub const fn get_imported_memory32_grow_index() -> Self {
        Self(1)
    }
    /// Returns an index for wasm's `memory.size` builtin function.
    pub const fn get_memory32_size_index() -> Self {
        Self(2)
    }
    /// Returns an index for wasm's imported `memory.size` builtin function.
    pub const fn get_imported_memory32_size_index() -> Self {
        Self(3)
    }
    /// Returns an index for wasm's `table.copy` when both tables are locally
    /// defined.
    pub const fn get_table_copy_index() -> Self {
        Self(4)
    }
    /// Returns an index for wasm's `table.init`.
    pub const fn get_table_init_index() -> Self {
        Self(5)
    }
    /// Returns an index for wasm's `elem.drop`.
    pub const fn get_elem_drop_index() -> Self {
        Self(6)
    }
    /// Returns an index for wasm's `memory.copy` for locally defined memories.
    pub const fn get_memory_copy_index() -> Self {
        Self(7)
    }
    /// Returns an index for wasm's `memory.copy` for imported memories.
    pub const fn get_imported_memory_copy_index() -> Self {
        Self(8)
    }
    /// Returns an index for wasm's `memory.fill` for locally defined memories.
    pub const fn get_memory_fill_index() -> Self {
        Self(9)
    }
    /// Returns an index for wasm's `memory.fill` for imported memories.
    pub const fn get_imported_memory_fill_index() -> Self {
        Self(10)
    }
    /// Returns an index for wasm's `memory.init` instruction.
    pub const fn get_memory_init_index() -> Self {
        Self(11)
    }
    /// Returns an index for wasm's `data.drop` instruction.
    pub const fn get_data_drop_index() -> Self {
        Self(12)
    }
    /// Returns an index for wasm's `raise_trap` instruction.
    pub const fn get_raise_trap_index() -> Self {
        Self(13)
    }
    /// Returns an index for wasm's `table.size` instruction for local tables.
    pub const fn get_table_size_index() -> Self {
        Self(14)
    }
    /// Returns an index for wasm's `table.size` instruction for imported tables.
    pub const fn get_imported_table_size_index() -> Self {
        Self(15)
    }
    /// Returns an index for wasm's `table.grow` instruction for local tables.
    pub const fn get_table_grow_index() -> Self {
        Self(16)
    }
    /// Returns an index for wasm's `table.grow` instruction for imported tables.
    pub const fn get_imported_table_grow_index() -> Self {
        Self(17)
    }
    /// Returns an index for wasm's `table.get` instruction for local tables.
    pub const fn get_table_get_index() -> Self {
        Self(18)
    }
    /// Returns an index for wasm's `table.get` instruction for imported tables.
    pub const fn get_imported_table_get_index() -> Self {
        Self(19)
    }
    /// Returns an index for wasm's `table.set` instruction for local tables.
    pub const fn get_table_set_index() -> Self {
        Self(20)
    }
    /// Returns an index for wasm's `table.set` instruction for imported tables.
    pub const fn get_imported_table_set_index() -> Self {
        Self(21)
    }
    /// Returns an index for wasm's `func.ref` instruction.
    pub const fn get_func_ref_index() -> Self {
        Self(22)
    }
    /// Returns an index for wasm's `table.fill` instruction for local tables.
    pub const fn get_table_fill_index() -> Self {
        Self(23)
    }
    /// Returns an index for wasm's local `memory.atomic.wait32` builtin function.
    pub const fn get_memory_atomic_wait32_index() -> Self {
        Self(24)
    }
    /// Returns an index for wasm's imported `memory.atomic.wait32` builtin function.
    pub const fn get_imported_memory_atomic_wait32_index() -> Self {
        Self(25)
    }
    /// Returns an index for wasm's local `memory.atomic.wait64` builtin function.
    pub const fn get_memory_atomic_wait64_index() -> Self {
        Self(26)
    }
    /// Returns an index for wasm's imported `memory.atomic.wait64` builtin function.
    pub const fn get_imported_memory_atomic_wait64_index() -> Self {
        Self(27)
    }
    /// Returns an index for wasm's local `memory.atomic.notify` builtin function.
    pub const fn get_memory_atomic_notify_index() -> Self {
        Self(28)
    }
    /// Returns an index for wasm's imported `memory.atomic.notify` builtin function.
    pub const fn get_imported_memory_atomic_notify_index() -> Self {
        Self(29)
    }
    /// Returns the total number of builtin functions.
    pub const fn builtin_functions_total_number() -> u32 {
        30
    }

    /// Return the index as an u32 number.
    pub const fn index(self) -> u32 {
        self.0
    }
}

#[cfg(target_pointer_width = "32")]
fn cast_to_u32(sz: usize) -> u32 {
    u32::try_from(sz).unwrap()
}
#[cfg(target_pointer_width = "64")]
fn cast_to_u32(sz: usize) -> u32 {
    u32::try_from(sz).expect("overflow in cast from usize to u32")
}

/// Align an offset used in this module to a specific byte-width by rounding up
#[inline]
const fn align(offset: u32, width: u32) -> u32 {
    (offset + (width - 1)) / width * width
}

/// This class computes offsets to fields within VMContext and other
/// related structs that JIT code accesses directly.
#[derive(Clone, Debug)]
pub struct VMOffsets {
    /// The size in bytes of a pointer on the target.
    pointer_size: u8,
    /// The number of signature declarations in the module.
    num_signature_ids: u32,
    /// The number of imported functions in the module.
    num_imported_functions: u32,
    /// The number of imported tables in the module.
    num_imported_tables: u32,
    /// The number of imported memories in the module.
    num_imported_memories: u32,
    /// The number of imported globals in the module.
    num_imported_globals: u32,
    /// The number of defined tables in the module.
    num_local_tables: u32,
    /// The number of defined memories in the module.
    num_local_memories: u32,
    /// The number of defined globals in the module.
    num_local_globals: u32,

    vmctx_signature_ids_begin: u32,
    vmctx_imported_functions_begin: u32,
    vmctx_imported_tables_begin: u32,
    vmctx_imported_memories_begin: u32,
    vmctx_imported_globals_begin: u32,
    vmctx_tables_begin: u32,
    vmctx_memories_begin: u32,
    vmctx_globals_begin: u32,
    vmctx_builtin_functions_begin: u32,
    vmctx_trap_handler_begin: u32,
    vmctx_gas_limiter_pointer: u32,
    vmctx_stack_limit_begin: u32,
    vmctx_stack_limit_initial_begin: u32,
    size_of_vmctx: u32,
}

impl VMOffsets {
    /// Return a new `VMOffsets` instance, for a given pointer size.
    pub fn new(pointer_size: u8, module: &ModuleInfo) -> Self {
        let mut ret = Self {
            pointer_size,
            num_signature_ids: cast_to_u32(module.signatures.len()),
            num_imported_functions: cast_to_u32(module.num_imported_functions),
            num_imported_tables: cast_to_u32(module.num_imported_tables),
            num_imported_memories: cast_to_u32(module.num_imported_memories),
            num_imported_globals: cast_to_u32(module.num_imported_globals),
            num_local_tables: cast_to_u32(module.tables.len()),
            num_local_memories: cast_to_u32(module.memories.len()),
            num_local_globals: cast_to_u32(module.globals.len()),
            vmctx_signature_ids_begin: 0,
            vmctx_imported_functions_begin: 0,
            vmctx_imported_tables_begin: 0,
            vmctx_imported_memories_begin: 0,
            vmctx_imported_globals_begin: 0,
            vmctx_tables_begin: 0,
            vmctx_memories_begin: 0,
            vmctx_globals_begin: 0,
            vmctx_builtin_functions_begin: 0,
            vmctx_trap_handler_begin: 0,
            vmctx_gas_limiter_pointer: 0,
            vmctx_stack_limit_begin: 0,
            vmctx_stack_limit_initial_begin: 0,
            size_of_vmctx: 0,
        };
        ret.precompute();
        ret
    }

    /// Return a new `VMOffsets` instance, for a given pointer size
    /// skipping the `ModuleInfo`.
    ///
    /// Note: This should only when generating code for trampolines.
    pub fn new_for_trampolines(pointer_size: u8) -> Self {
        Self {
            pointer_size,
            num_signature_ids: 0,
            num_imported_functions: 0,
            num_imported_tables: 0,
            num_imported_memories: 0,
            num_imported_globals: 0,
            num_local_tables: 0,
            num_local_memories: 0,
            num_local_globals: 0,
            vmctx_signature_ids_begin: 0,
            vmctx_imported_functions_begin: 0,
            vmctx_imported_tables_begin: 0,
            vmctx_imported_memories_begin: 0,
            vmctx_imported_globals_begin: 0,
            vmctx_tables_begin: 0,
            vmctx_memories_begin: 0,
            vmctx_globals_begin: 0,
            vmctx_builtin_functions_begin: 0,
            vmctx_trap_handler_begin: 0,
            vmctx_gas_limiter_pointer: 0,
            vmctx_stack_limit_begin: 0,
            vmctx_stack_limit_initial_begin: 0,
            size_of_vmctx: 0,
        }
    }

    /// Number of local tables defined in the module
    pub fn num_local_tables(&self) -> u32 {
        self.num_local_tables
    }

    /// Number of local memories defined in the module
    pub fn num_local_memories(&self) -> u32 {
        self.num_local_memories
    }

    fn precompute(&mut self) {
        /// Offset base by num_items items of size item_size, panicking on overflow
        fn offset_by(base: u32, num_items: u32, item_size: u32) -> u32 {
            base.checked_add(num_items.checked_mul(item_size).unwrap())
                .unwrap()
        }
        /// Offset base by num_items items of size item_size, panicking on overflow
        /// Also, will align the value on pointer size boundary,
        /// to avoid misalignement issue
        fn offset_by_aligned(base: u32, num_items: u32, item_size: u32) -> u32 {
            align(
                base.checked_add(num_items.checked_mul(item_size).unwrap())
                    .unwrap(),
                size_of::<&u32>() as u32,
            )
        }

        self.vmctx_signature_ids_begin = 0;
        self.vmctx_imported_functions_begin = offset_by_aligned(
            self.vmctx_signature_ids_begin,
            self.num_signature_ids,
            u32::from(self.size_of_vmshared_signature_index()),
        );
        self.vmctx_imported_tables_begin = offset_by_aligned(
            self.vmctx_imported_functions_begin,
            self.num_imported_functions,
            u32::from(self.size_of_vmfunction_import()),
        );
        self.vmctx_imported_memories_begin = offset_by_aligned(
            self.vmctx_imported_tables_begin,
            self.num_imported_tables,
            u32::from(self.size_of_vmtable_import()),
        );
        self.vmctx_imported_globals_begin = offset_by_aligned(
            self.vmctx_imported_memories_begin,
            self.num_imported_memories,
            u32::from(self.size_of_vmmemory_import()),
        );
        self.vmctx_tables_begin = offset_by_aligned(
            self.vmctx_imported_globals_begin,
            self.num_imported_globals,
            u32::from(self.size_of_vmglobal_import()),
        );
        self.vmctx_memories_begin = offset_by_aligned(
            self.vmctx_tables_begin,
            self.num_local_tables,
            u32::from(self.size_of_vmtable_definition()),
        );
        self.vmctx_globals_begin = align(
            offset_by(
                self.vmctx_memories_begin,
                self.num_local_memories,
                u32::from(self.size_of_vmmemory_definition()),
            ),
            16,
        );
        self.vmctx_builtin_functions_begin = offset_by(
            self.vmctx_globals_begin,
            self.num_local_globals,
            u32::from(self.size_of_vmglobal_local()),
        );
        self.vmctx_trap_handler_begin = offset_by(
            self.vmctx_builtin_functions_begin,
            VMBuiltinFunctionIndex::builtin_functions_total_number(),
            u32::from(self.pointer_size),
        );
        self.vmctx_gas_limiter_pointer = offset_by(
            self.vmctx_trap_handler_begin,
            1,
            u32::from(self.pointer_size),
        );
        self.vmctx_stack_limit_begin = offset_by(
            self.vmctx_gas_limiter_pointer,
            1,
            u32::from(self.pointer_size),
        );
        self.vmctx_stack_limit_initial_begin = self.vmctx_stack_limit_begin.checked_add(4).unwrap();
        self.size_of_vmctx = self.vmctx_stack_limit_begin.checked_add(4).unwrap();
    }
}

/// Offsets for `VMFunctionImport`.
impl VMOffsets {
    /// The offset of the `body` field.
    #[allow(clippy::erasing_op)]
    pub const fn vmfunction_import_body(&self) -> u8 {
        0 * self.pointer_size
    }

    /// The offset of the `vmctx` field.
    #[allow(clippy::identity_op)]
    pub const fn vmfunction_import_vmctx(&self) -> u8 {
        1 * self.pointer_size
    }

    /// The offset of the `handle` field.
    pub const fn vmfunction_import_handle(&self) -> u8 {
        2 * self.pointer_size
    }

    /// Return the size of `VMFunctionImport`.
    pub const fn size_of_vmfunction_import(&self) -> u8 {
        3 * self.pointer_size
    }
}

/// Offsets for `VMDynamicFunctionContext`.
impl VMOffsets {
    /// The offset of the `address` field.
    #[allow(clippy::erasing_op)]
    pub const fn vmdynamicfunction_import_context_address(&self) -> u8 {
        0 * self.pointer_size
    }

    /// The offset of the `ctx` field.
    #[allow(clippy::identity_op)]
    pub const fn vmdynamicfunction_import_context_ctx(&self) -> u8 {
        1 * self.pointer_size
    }

    /// Return the size of `VMDynamicFunctionContext`.
    pub const fn size_of_vmdynamicfunction_import_context(&self) -> u8 {
        2 * self.pointer_size
    }
}

/// Offsets for `*const VMFunctionBody`.
impl VMOffsets {
    /// The size of the `current_elements` field.
    #[allow(clippy::identity_op)]
    pub const fn size_of_vmfunction_body_ptr(&self) -> u8 {
        1 * self.pointer_size
    }
}

/// Offsets for `VMTableImport`.
impl VMOffsets {
    /// The offset of the `definition` field.
    #[allow(clippy::erasing_op)]
    pub const fn vmtable_import_definition(&self) -> u8 {
        0 * self.pointer_size
    }

    /// The offset of the `handle` field.
    #[allow(clippy::identity_op)]
    pub const fn vmtable_import_handle(&self) -> u8 {
        1 * self.pointer_size
    }

    /// Return the size of `VMTableImport`.
    pub const fn size_of_vmtable_import(&self) -> u8 {
        2 * self.pointer_size
    }
}

/// Offsets for `VMTableDefinition`.
impl VMOffsets {
    /// The offset of the `base` field.
    #[allow(clippy::erasing_op)]
    pub const fn vmtable_definition_base(&self) -> u8 {
        0 * self.pointer_size
    }

    /// The offset of the `current_elements` field.
    #[allow(clippy::identity_op)]
    pub const fn vmtable_definition_current_elements(&self) -> u8 {
        1 * self.pointer_size
    }

    /// The size of the `current_elements` field.
    pub const fn size_of_vmtable_definition_current_elements(&self) -> u8 {
        4
    }

    /// Return the size of `VMTableDefinition`.
    pub const fn size_of_vmtable_definition(&self) -> u8 {
        2 * self.pointer_size
    }
}

/// Offsets for `VMMemoryImport`.
impl VMOffsets {
    /// The offset of the `from` field.
    #[allow(clippy::erasing_op)]
    pub const fn vmmemory_import_definition(&self) -> u8 {
        0 * self.pointer_size
    }

    /// The offset of the `handle` field.
    #[allow(clippy::identity_op)]
    pub const fn vmmemory_import_handle(&self) -> u8 {
        1 * self.pointer_size
    }

    /// Return the size of `VMMemoryImport`.
    pub const fn size_of_vmmemory_import(&self) -> u8 {
        2 * self.pointer_size
    }
}

/// Offsets for `VMMemoryDefinition`.
impl VMOffsets {
    /// The offset of the `base` field.
    #[allow(clippy::erasing_op)]
    pub const fn vmmemory_definition_base(&self) -> u8 {
        0 * self.pointer_size
    }

    /// The offset of the `current_length` field.
    #[allow(clippy::identity_op)]
    pub const fn vmmemory_definition_current_length(&self) -> u8 {
        1 * self.pointer_size
    }

    /// The size of the `current_length` field.
    pub const fn size_of_vmmemory_definition_current_length(&self) -> u8 {
        4
    }

    /// Return the size of `VMMemoryDefinition`.
    pub const fn size_of_vmmemory_definition(&self) -> u8 {
        2 * self.pointer_size
    }
}

/// Offsets for `VMGlobalImport`.
impl VMOffsets {
    /// The offset of the `definition` field.
    #[allow(clippy::erasing_op)]
    pub const fn vmglobal_import_definition(&self) -> u8 {
        0 * self.pointer_size
    }

    /// The offset of the `handle` field.
    #[allow(clippy::identity_op)]
    pub const fn vmglobal_import_handle(&self) -> u8 {
        1 * self.pointer_size
    }

    /// Return the size of `VMGlobalImport`.
    #[allow(clippy::identity_op)]
    pub const fn size_of_vmglobal_import(&self) -> u8 {
        2 * self.pointer_size
    }
}

/// Offsets for a non-null pointer to a `VMGlobalDefinition` used as a local global.
impl VMOffsets {
    /// Return the size of a pointer to a `VMGlobalDefinition`;
    ///
    /// The underlying global itself is the size of the largest value type (i.e. a V128),
    /// however the size of this type is just the size of a pointer.
    pub const fn size_of_vmglobal_local(&self) -> u8 {
        self.pointer_size
    }
}

/// Offsets for `VMSharedSignatureIndex`.
impl VMOffsets {
    /// Return the size of `VMSharedSignatureIndex`.
    pub const fn size_of_vmshared_signature_index(&self) -> u8 {
        4
    }
}

/// Offsets for `VMCallerCheckedAnyfunc`.
impl VMOffsets {
    /// The offset of the `func_ptr` field.
    #[allow(clippy::erasing_op)]
    pub const fn vmcaller_checked_anyfunc_func_ptr(&self) -> u8 {
        0 * self.pointer_size
    }

    /// The offset of the `type_index` field.
    #[allow(clippy::identity_op)]
    pub const fn vmcaller_checked_anyfunc_type_index(&self) -> u8 {
        1 * self.pointer_size
    }

    /// The offset of the `vmctx` field.
    pub const fn vmcaller_checked_anyfunc_vmctx(&self) -> u8 {
        2 * self.pointer_size
    }

    /// The offset of the `call_trampoline` field.
    pub const fn vmcaller_checked_anyfunc_call_trampoline(&self) -> u8 {
        3 * self.pointer_size
    }

    /// Return the size of `VMCallerCheckedAnyfunc`.
    pub const fn size_of_vmcaller_checked_anyfunc(&self) -> u8 {
        4 * self.pointer_size
    }
}

/// Offsets for `VMFuncRef`.
impl VMOffsets {
    /// The offset to the pointer to the anyfunc inside the ref.
    #[allow(clippy::erasing_op)]
    pub const fn vm_funcref_anyfunc_ptr(&self) -> u8 {
        0 * self.pointer_size
    }

    /// Return the size of `VMFuncRef`.
    #[allow(clippy::identity_op)]
    pub const fn size_of_vm_funcref(&self) -> u8 {
        1 * self.pointer_size
    }
}

/// Offsets for `VMContext`.
impl VMOffsets {
    /// The offset of the `signature_ids` array.
    pub fn vmctx_signature_ids_begin(&self) -> u32 {
        self.vmctx_signature_ids_begin
    }

    /// The offset of the `tables` array.
    #[allow(clippy::erasing_op)]
    pub fn vmctx_imported_functions_begin(&self) -> u32 {
        self.vmctx_imported_functions_begin
    }

    /// The offset of the `tables` array.
    #[allow(clippy::identity_op)]
    pub fn vmctx_imported_tables_begin(&self) -> u32 {
        self.vmctx_imported_tables_begin
    }

    /// The offset of the `memories` array.
    pub fn vmctx_imported_memories_begin(&self) -> u32 {
        self.vmctx_imported_memories_begin
    }

    /// The offset of the `globals` array.
    pub fn vmctx_imported_globals_begin(&self) -> u32 {
        self.vmctx_imported_globals_begin
    }

    /// The offset of the `tables` array.
    pub fn vmctx_tables_begin(&self) -> u32 {
        self.vmctx_tables_begin
    }

    /// The offset of the `memories` array.
    pub fn vmctx_memories_begin(&self) -> u32 {
        self.vmctx_memories_begin
    }

    /// The offset of the `globals` array.
    pub fn vmctx_globals_begin(&self) -> u32 {
        self.vmctx_globals_begin
    }

    /// The offset of the builtin functions array.
    pub fn vmctx_builtin_functions_begin(&self) -> u32 {
        self.vmctx_builtin_functions_begin
    }

    /// Return the size of the `VMContext` allocation.
    pub fn size_of_vmctx(&self) -> u32 {
        self.size_of_vmctx
    }

    /// Return the offset to `VMSharedSignatureIndex` index `index`.
    pub fn vmctx_vmshared_signature_id(&self, index: SignatureIndex) -> u32 {
        assert_lt!(index.as_u32(), self.num_signature_ids);
        self.vmctx_signature_ids_begin
            + index.as_u32() * u32::from(self.size_of_vmshared_signature_index())
    }

    /// Return the offset to `VMFunctionImport` index `index`.
    pub fn vmctx_vmfunction_import(&self, index: FunctionIndex) -> u32 {
        assert_lt!(index.as_u32(), self.num_imported_functions);
        self.vmctx_imported_functions_begin
            + index.as_u32() * u32::from(self.size_of_vmfunction_import())
    }

    /// Return the offset to `VMTableImport` index `index`.
    pub fn vmctx_vmtable_import(&self, index: TableIndex) -> u32 {
        assert_lt!(index.as_u32(), self.num_imported_tables);
        self.vmctx_imported_tables_begin + index.as_u32() * u32::from(self.size_of_vmtable_import())
    }

    /// Return the offset to `VMMemoryImport` index `index`.
    pub fn vmctx_vmmemory_import(&self, index: MemoryIndex) -> u32 {
        assert_lt!(index.as_u32(), self.num_imported_memories);
        self.vmctx_imported_memories_begin
            + index.as_u32() * u32::from(self.size_of_vmmemory_import())
    }

    /// Return the offset to `VMGlobalImport` index `index`.
    pub fn vmctx_vmglobal_import(&self, index: GlobalIndex) -> u32 {
        assert_lt!(index.as_u32(), self.num_imported_globals);
        self.vmctx_imported_globals_begin
            + index.as_u32() * u32::from(self.size_of_vmglobal_import())
    }

    /// Return the offset to `VMTableDefinition` index `index`.
    pub fn vmctx_vmtable_definition(&self, index: LocalTableIndex) -> u32 {
        assert_lt!(index.as_u32(), self.num_local_tables);
        self.vmctx_tables_begin + index.as_u32() * u32::from(self.size_of_vmtable_definition())
    }

    /// Return the offset to `VMMemoryDefinition` index `index`.
    pub fn vmctx_vmmemory_definition(&self, index: LocalMemoryIndex) -> u32 {
        assert_lt!(index.as_u32(), self.num_local_memories);
        self.vmctx_memories_begin + index.as_u32() * u32::from(self.size_of_vmmemory_definition())
    }

    /// Return the offset to the `VMGlobalDefinition` index `index`.
    pub fn vmctx_vmglobal_definition(&self, index: LocalGlobalIndex) -> u32 {
        assert_lt!(index.as_u32(), self.num_local_globals);
        self.vmctx_globals_begin + index.as_u32() * u32::from(self.size_of_vmglobal_local())
    }

    /// Return the offset to the `body` field in `*const VMFunctionBody` index `index`.
    /// Remember updating precompute upon changes
    pub fn vmctx_vmfunction_import_body(&self, index: FunctionIndex) -> u32 {
        self.vmctx_vmfunction_import(index) + u32::from(self.vmfunction_import_body())
    }

    /// Return the offset to the `vmctx` field in `*const VMFunctionBody` index `index`.
    /// Remember updating precompute upon changes
    pub fn vmctx_vmfunction_import_vmctx(&self, index: FunctionIndex) -> u32 {
        self.vmctx_vmfunction_import(index) + u32::from(self.vmfunction_import_vmctx())
    }

    /// Return the offset to the `definition` field in `VMTableImport` index `index`.
    /// Remember updating precompute upon changes
    pub fn vmctx_vmtable_import_definition(&self, index: TableIndex) -> u32 {
        self.vmctx_vmtable_import(index) + u32::from(self.vmtable_import_definition())
    }

    /// Return the offset to the `base` field in `VMTableDefinition` index `index`.
    /// Remember updating precompute upon changes
    pub fn vmctx_vmtable_definition_base(&self, index: LocalTableIndex) -> u32 {
        self.vmctx_vmtable_definition(index) + u32::from(self.vmtable_definition_base())
    }

    /// Return the offset to the `current_elements` field in `VMTableDefinition` index `index`.
    /// Remember updating precompute upon changes
    pub fn vmctx_vmtable_definition_current_elements(&self, index: LocalTableIndex) -> u32 {
        self.vmctx_vmtable_definition(index) + u32::from(self.vmtable_definition_current_elements())
    }

    /// Return the offset to the `from` field in `VMMemoryImport` index `index`.
    /// Remember updating precompute upon changes
    pub fn vmctx_vmmemory_import_definition(&self, index: MemoryIndex) -> u32 {
        self.vmctx_vmmemory_import(index) + u32::from(self.vmmemory_import_definition())
    }

    /// Return the offset to the `vmctx` field in `VMMemoryImport` index `index`.
    /// Remember updating precompute upon changes
    pub fn vmctx_vmmemory_import_handle(&self, index: MemoryIndex) -> u32 {
        self.vmctx_vmmemory_import(index) + u32::from(self.vmmemory_import_handle())
    }

    /// Return the offset to the `base` field in `VMMemoryDefinition` index `index`.
    /// Remember updating precompute upon changes
    pub fn vmctx_vmmemory_definition_base(&self, index: LocalMemoryIndex) -> u32 {
        self.vmctx_vmmemory_definition(index) + u32::from(self.vmmemory_definition_base())
    }

    /// Return the offset to the `current_length` field in `VMMemoryDefinition` index `index`.
    /// Remember updating precompute upon changes
    pub fn vmctx_vmmemory_definition_current_length(&self, index: LocalMemoryIndex) -> u32 {
        self.vmctx_vmmemory_definition(index) + u32::from(self.vmmemory_definition_current_length())
    }

    /// Return the offset to the `from` field in `VMGlobalImport` index `index`.
    /// Remember updating precompute upon changes
    pub fn vmctx_vmglobal_import_definition(&self, index: GlobalIndex) -> u32 {
        self.vmctx_vmglobal_import(index) + u32::from(self.vmglobal_import_definition())
    }

    /// Return the offset to builtin function in `VMBuiltinFunctionsArray` index `index`.
    /// Remember updating precompute upon changes
    pub fn vmctx_builtin_function(&self, index: VMBuiltinFunctionIndex) -> u32 {
        self.vmctx_builtin_functions_begin + index.index() * u32::from(self.pointer_size)
    }
}

/// Target specific type for shared signature index.
#[derive(Debug, Copy, Clone)]
pub struct TargetSharedSignatureIndex(u32);

impl TargetSharedSignatureIndex {
    /// Constructs `TargetSharedSignatureIndex`.
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns index value.
    pub const fn index(self) -> u32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::vmoffsets::align;

    #[test]
    fn alignment() {
        fn is_aligned(x: u32) -> bool {
            x % 16 == 0
        }
        assert!(is_aligned(align(0, 16)));
        assert!(is_aligned(align(32, 16)));
        assert!(is_aligned(align(33, 16)));
        assert!(is_aligned(align(31, 16)));
    }
}
