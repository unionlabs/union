// This file contains code from external sources.
// Attributions: https://github.com/wasmerio/wasmer/blob/master/ATTRIBUTIONS.md

//! An `Instance` contains all the runtime state used by execution of
//! a WebAssembly module (except its callstack and register state). An
//! `VMInstance` is a wrapper around `Instance` that manages
//! how it is allocated and deallocated.

mod allocator;

use crate::export::VMExtern;
use crate::imports::Imports;
use crate::store::{InternalStoreHandle, StoreObjects};
use crate::table::TableElement;
use crate::trap::{catch_traps, Trap, TrapCode};
use crate::vmcontext::{
    memory32_atomic_check32, memory32_atomic_check64, memory_copy, memory_fill,
    VMBuiltinFunctionsArray, VMCallerCheckedAnyfunc, VMContext, VMFunctionContext,
    VMFunctionImport, VMFunctionKind, VMGlobalDefinition, VMGlobalImport, VMMemoryDefinition,
    VMMemoryImport, VMSharedSignatureIndex, VMTableDefinition, VMTableImport, VMTrampoline,
};
use crate::{FunctionBodyPtr, MaybeInstanceOwned, TrapHandlerFn, VMFunctionBody};
use crate::{LinearMemory, NotifyLocation};
use crate::{VMConfig, VMFuncRef, VMFunction, VMGlobal, VMMemory, VMTable};
pub use allocator::InstanceAllocator;
use memoffset::offset_of;
use more_asserts::assert_lt;
use std::alloc::Layout;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::mem;
use std::ptr::{self, NonNull};
use std::slice;
use std::sync::Arc;
use wasmer_types::entity::{packed_option::ReservedValue, BoxedSlice, EntityRef, PrimaryMap};
use wasmer_types::{
    DataIndex, DataInitializer, ElemIndex, ExportIndex, FunctionIndex, GlobalIndex, GlobalInit,
    LocalFunctionIndex, LocalGlobalIndex, LocalMemoryIndex, LocalTableIndex, MemoryError,
    MemoryIndex, ModuleInfo, Pages, SignatureIndex, TableIndex, TableInitializer, VMOffsets,
};

/// A WebAssembly instance.
///
/// The type is dynamically-sized. Indeed, the `vmctx` field can
/// contain various data. That's why the type has a C representation
/// to ensure that the `vmctx` field is last. See the documentation of
/// the `vmctx` field to learn more.
#[repr(C)]
#[allow(clippy::type_complexity)]
pub(crate) struct Instance {
    /// The `ModuleInfo` this `Instance` was instantiated from.
    module: Arc<ModuleInfo>,

    /// Pointer to the object store of the context owning this instance.
    context: *mut StoreObjects,

    /// Offsets in the `vmctx` region.
    offsets: VMOffsets,

    /// WebAssembly linear memory data.
    memories: BoxedSlice<LocalMemoryIndex, InternalStoreHandle<VMMemory>>,

    /// WebAssembly table data.
    tables: BoxedSlice<LocalTableIndex, InternalStoreHandle<VMTable>>,

    /// WebAssembly global data.
    globals: BoxedSlice<LocalGlobalIndex, InternalStoreHandle<VMGlobal>>,

    /// Pointers to functions in executable memory.
    functions: BoxedSlice<LocalFunctionIndex, FunctionBodyPtr>,

    /// Pointers to function call trampolines in executable memory.
    function_call_trampolines: BoxedSlice<SignatureIndex, VMTrampoline>,

    /// Passive elements in this instantiation. As `elem.drop`s happen, these
    /// entries get removed.
    passive_elements: RefCell<HashMap<ElemIndex, Box<[Option<VMFuncRef>]>>>,

    /// Passive data segments from our module. As `data.drop`s happen, entries
    /// get removed. A missing entry is considered equivalent to an empty slice.
    passive_data: RefCell<HashMap<DataIndex, Arc<[u8]>>>,

    /// Mapping of function indices to their func ref backing data. `VMFuncRef`s
    /// will point to elements here for functions defined by this instance.
    funcrefs: BoxedSlice<LocalFunctionIndex, VMCallerCheckedAnyfunc>,

    /// Mapping of function indices to their func ref backing data. `VMFuncRef`s
    /// will point to elements here for functions imported by this instance.
    imported_funcrefs: BoxedSlice<FunctionIndex, NonNull<VMCallerCheckedAnyfunc>>,

    /// Additional context used by compiled WebAssembly code. This
    /// field is last, and represents a dynamically-sized array that
    /// extends beyond the nominal end of the struct (similar to a
    /// flexible array member).
    vmctx: VMContext,
}

impl fmt::Debug for Instance {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_struct("Instance").finish()
    }
}

#[allow(clippy::cast_ptr_alignment)]
impl Instance {
    /// Helper function to access various locations offset from our `*mut
    /// VMContext` object.
    unsafe fn vmctx_plus_offset<T>(&self, offset: u32) -> *mut T {
        (self.vmctx_ptr() as *mut u8)
            .add(usize::try_from(offset).unwrap())
            .cast()
    }

    fn module(&self) -> &Arc<ModuleInfo> {
        &self.module
    }

    pub(crate) fn module_ref(&self) -> &ModuleInfo {
        &self.module
    }

    fn context(&self) -> &StoreObjects {
        unsafe { &*self.context }
    }

    fn context_mut(&mut self) -> &mut StoreObjects {
        unsafe { &mut *self.context }
    }

    /// Offsets in the `vmctx` region.
    fn offsets(&self) -> &VMOffsets {
        &self.offsets
    }

    /// Return a pointer to the `VMSharedSignatureIndex`s.
    fn signature_ids_ptr(&self) -> *mut VMSharedSignatureIndex {
        unsafe { self.vmctx_plus_offset(self.offsets.vmctx_signature_ids_begin()) }
    }

    /// Return the indexed `VMFunctionImport`.
    fn imported_function(&self, index: FunctionIndex) -> &VMFunctionImport {
        let index = usize::try_from(index.as_u32()).unwrap();
        unsafe { &*self.imported_functions_ptr().add(index) }
    }

    /// Return a pointer to the `VMFunctionImport`s.
    fn imported_functions_ptr(&self) -> *mut VMFunctionImport {
        unsafe { self.vmctx_plus_offset(self.offsets.vmctx_imported_functions_begin()) }
    }

    /// Return the index `VMTableImport`.
    fn imported_table(&self, index: TableIndex) -> &VMTableImport {
        let index = usize::try_from(index.as_u32()).unwrap();
        unsafe { &*self.imported_tables_ptr().add(index) }
    }

    /// Return a pointer to the `VMTableImports`s.
    fn imported_tables_ptr(&self) -> *mut VMTableImport {
        unsafe { self.vmctx_plus_offset(self.offsets.vmctx_imported_tables_begin()) }
    }

    /// Return the indexed `VMMemoryImport`.
    fn imported_memory(&self, index: MemoryIndex) -> &VMMemoryImport {
        let index = usize::try_from(index.as_u32()).unwrap();
        unsafe { &*self.imported_memories_ptr().add(index) }
    }

    /// Return a pointer to the `VMMemoryImport`s.
    fn imported_memories_ptr(&self) -> *mut VMMemoryImport {
        unsafe { self.vmctx_plus_offset(self.offsets.vmctx_imported_memories_begin()) }
    }

    /// Return the indexed `VMGlobalImport`.
    fn imported_global(&self, index: GlobalIndex) -> &VMGlobalImport {
        let index = usize::try_from(index.as_u32()).unwrap();
        unsafe { &*self.imported_globals_ptr().add(index) }
    }

    /// Return a pointer to the `VMGlobalImport`s.
    fn imported_globals_ptr(&self) -> *mut VMGlobalImport {
        unsafe { self.vmctx_plus_offset(self.offsets.vmctx_imported_globals_begin()) }
    }

    /// Return the indexed `VMTableDefinition`.
    #[allow(dead_code)]
    fn table(&self, index: LocalTableIndex) -> VMTableDefinition {
        unsafe { *self.table_ptr(index).as_ref() }
    }

    #[allow(dead_code)]
    /// Updates the value for a defined table to `VMTableDefinition`.
    fn set_table(&self, index: LocalTableIndex, table: &VMTableDefinition) {
        unsafe {
            *self.table_ptr(index).as_ptr() = *table;
        }
    }

    /// Return the indexed `VMTableDefinition`.
    fn table_ptr(&self, index: LocalTableIndex) -> NonNull<VMTableDefinition> {
        let index = usize::try_from(index.as_u32()).unwrap();
        NonNull::new(unsafe { self.tables_ptr().add(index) }).unwrap()
    }

    /// Return a pointer to the `VMTableDefinition`s.
    fn tables_ptr(&self) -> *mut VMTableDefinition {
        unsafe { self.vmctx_plus_offset(self.offsets.vmctx_tables_begin()) }
    }

    #[allow(dead_code)]
    /// Get a locally defined or imported memory.
    fn get_memory(&self, index: MemoryIndex) -> VMMemoryDefinition {
        if let Some(local_index) = self.module.local_memory_index(index) {
            self.memory(local_index)
        } else {
            let import = self.imported_memory(index);
            unsafe { *import.definition.as_ref() }
        }
    }

    /// Return the indexed `VMMemoryDefinition`.
    fn memory(&self, index: LocalMemoryIndex) -> VMMemoryDefinition {
        unsafe { *self.memory_ptr(index).as_ref() }
    }

    #[allow(dead_code)]
    /// Set the indexed memory to `VMMemoryDefinition`.
    fn set_memory(&self, index: LocalMemoryIndex, mem: &VMMemoryDefinition) {
        unsafe {
            *self.memory_ptr(index).as_ptr() = *mem;
        }
    }

    /// Return the indexed `VMMemoryDefinition`.
    fn memory_ptr(&self, index: LocalMemoryIndex) -> NonNull<VMMemoryDefinition> {
        let index = usize::try_from(index.as_u32()).unwrap();
        NonNull::new(unsafe { self.memories_ptr().add(index) }).unwrap()
    }

    /// Return a pointer to the `VMMemoryDefinition`s.
    fn memories_ptr(&self) -> *mut VMMemoryDefinition {
        unsafe { self.vmctx_plus_offset(self.offsets.vmctx_memories_begin()) }
    }

    /// Get a locally defined or imported memory.
    fn get_vmmemory(&self, index: MemoryIndex) -> &VMMemory {
        if let Some(local_index) = self.module.local_memory_index(index) {
            unsafe {
                self.memories
                    .get(local_index)
                    .unwrap()
                    .get(self.context.as_ref().unwrap())
            }
        } else {
            let import = self.imported_memory(index);
            unsafe { import.handle.get(self.context.as_ref().unwrap()) }
        }
    }

    /// Get a locally defined or imported memory.
    fn get_vmmemory_mut(&mut self, index: MemoryIndex) -> &mut VMMemory {
        if let Some(local_index) = self.module.local_memory_index(index) {
            unsafe {
                self.memories
                    .get_mut(local_index)
                    .unwrap()
                    .get_mut(self.context.as_mut().unwrap())
            }
        } else {
            let import = self.imported_memory(index);
            unsafe { import.handle.get_mut(self.context.as_mut().unwrap()) }
        }
    }

    /// Get a locally defined memory as mutable.
    fn get_local_vmmemory_mut(&mut self, local_index: LocalMemoryIndex) -> &mut VMMemory {
        unsafe {
            self.memories
                .get_mut(local_index)
                .unwrap()
                .get_mut(self.context.as_mut().unwrap())
        }
    }

    /// Return the indexed `VMGlobalDefinition`.
    fn global(&self, index: LocalGlobalIndex) -> VMGlobalDefinition {
        unsafe { self.global_ptr(index).as_ref().clone() }
    }

    /// Set the indexed global to `VMGlobalDefinition`.
    #[allow(dead_code)]
    fn set_global(&self, index: LocalGlobalIndex, global: &VMGlobalDefinition) {
        unsafe {
            *self.global_ptr(index).as_ptr() = global.clone();
        }
    }

    /// Return the indexed `VMGlobalDefinition`.
    fn global_ptr(&self, index: LocalGlobalIndex) -> NonNull<VMGlobalDefinition> {
        let index = usize::try_from(index.as_u32()).unwrap();
        // TODO:
        NonNull::new(unsafe { *self.globals_ptr().add(index) }).unwrap()
    }

    /// Return a pointer to the `VMGlobalDefinition`s.
    fn globals_ptr(&self) -> *mut *mut VMGlobalDefinition {
        unsafe { self.vmctx_plus_offset(self.offsets.vmctx_globals_begin()) }
    }

    /// Return a pointer to the `VMBuiltinFunctionsArray`.
    fn builtin_functions_ptr(&self) -> *mut VMBuiltinFunctionsArray {
        unsafe { self.vmctx_plus_offset(self.offsets.vmctx_builtin_functions_begin()) }
    }

    /// Return a reference to the vmctx used by compiled wasm code.
    fn vmctx(&self) -> &VMContext {
        &self.vmctx
    }

    /// Return a raw pointer to the vmctx used by compiled wasm code.
    fn vmctx_ptr(&self) -> *mut VMContext {
        self.vmctx() as *const VMContext as *mut VMContext
    }

    /// Invoke the WebAssembly start function of the instance, if one is present.
    fn invoke_start_function(
        &self,
        config: &VMConfig,
        trap_handler: Option<*const TrapHandlerFn<'static>>,
    ) -> Result<(), Trap> {
        let start_index = match self.module.start_function {
            Some(idx) => idx,
            None => return Ok(()),
        };

        let (callee_address, callee_vmctx) = match self.module.local_func_index(start_index) {
            Some(local_index) => {
                let body = self
                    .functions
                    .get(local_index)
                    .expect("function index is out of bounds")
                    .0;
                (
                    body as *const _,
                    VMFunctionContext {
                        vmctx: self.vmctx_ptr(),
                    },
                )
            }
            None => {
                assert_lt!(start_index.index(), self.module.num_imported_functions);
                let import = self.imported_function(start_index);
                (import.body, import.environment)
            }
        };

        // Make the call.
        unsafe {
            catch_traps(trap_handler, config, || {
                mem::transmute::<*const VMFunctionBody, unsafe extern "C" fn(VMFunctionContext)>(
                    callee_address,
                )(callee_vmctx)
            })
        }
    }

    /// Return the offset from the vmctx pointer to its containing `Instance`.
    #[inline]
    pub(crate) fn vmctx_offset() -> isize {
        offset_of!(Self, vmctx) as isize
    }

    /// Return the table index for the given `VMTableDefinition`.
    pub(crate) fn table_index(&self, table: &VMTableDefinition) -> LocalTableIndex {
        let begin: *const VMTableDefinition = self.tables_ptr() as *const _;
        let end: *const VMTableDefinition = table;
        // TODO: Use `offset_from` once it stablizes.
        let index = LocalTableIndex::new(
            (end as usize - begin as usize) / mem::size_of::<VMTableDefinition>(),
        );
        assert_lt!(index.index(), self.tables.len());
        index
    }

    /// Return the memory index for the given `VMMemoryDefinition`.
    pub(crate) fn memory_index(&self, memory: &VMMemoryDefinition) -> LocalMemoryIndex {
        let begin: *const VMMemoryDefinition = self.memories_ptr() as *const _;
        let end: *const VMMemoryDefinition = memory;
        // TODO: Use `offset_from` once it stablizes.
        let index = LocalMemoryIndex::new(
            (end as usize - begin as usize) / mem::size_of::<VMMemoryDefinition>(),
        );
        assert_lt!(index.index(), self.memories.len());
        index
    }

    /// Grow memory by the specified amount of pages.
    ///
    /// Returns `None` if memory can't be grown by the specified amount
    /// of pages.
    pub(crate) fn memory_grow<IntoPages>(
        &mut self,
        memory_index: LocalMemoryIndex,
        delta: IntoPages,
    ) -> Result<Pages, MemoryError>
    where
        IntoPages: Into<Pages>,
    {
        let mem = *self
            .memories
            .get(memory_index)
            .unwrap_or_else(|| panic!("no memory for index {}", memory_index.index()));
        mem.get_mut(self.context_mut()).grow(delta.into())
    }

    /// Grow imported memory by the specified amount of pages.
    ///
    /// Returns `None` if memory can't be grown by the specified amount
    /// of pages.
    ///
    /// # Safety
    /// This and `imported_memory_size` are currently unsafe because they
    /// dereference the memory import's pointers.
    pub(crate) unsafe fn imported_memory_grow<IntoPages>(
        &mut self,
        memory_index: MemoryIndex,
        delta: IntoPages,
    ) -> Result<Pages, MemoryError>
    where
        IntoPages: Into<Pages>,
    {
        let import = self.imported_memory(memory_index);
        let mem = import.handle;
        mem.get_mut(self.context_mut()).grow(delta.into())
    }

    /// Returns the number of allocated wasm pages.
    pub(crate) fn memory_size(&self, memory_index: LocalMemoryIndex) -> Pages {
        let mem = *self
            .memories
            .get(memory_index)
            .unwrap_or_else(|| panic!("no memory for index {}", memory_index.index()));
        mem.get(self.context()).size()
    }

    /// Returns the number of allocated wasm pages in an imported memory.
    ///
    /// # Safety
    /// This and `imported_memory_grow` are currently unsafe because they
    /// dereference the memory import's pointers.
    pub(crate) unsafe fn imported_memory_size(&self, memory_index: MemoryIndex) -> Pages {
        let import = self.imported_memory(memory_index);
        let mem = import.handle;
        mem.get(self.context()).size()
    }

    /// Returns the number of elements in a given table.
    pub(crate) fn table_size(&self, table_index: LocalTableIndex) -> u32 {
        let table = self
            .tables
            .get(table_index)
            .unwrap_or_else(|| panic!("no table for index {}", table_index.index()));
        table.get(self.context()).size()
    }

    /// Returns the number of elements in a given imported table.
    ///
    /// # Safety
    /// `table_index` must be a valid, imported table index.
    pub(crate) unsafe fn imported_table_size(&self, table_index: TableIndex) -> u32 {
        let import = self.imported_table(table_index);
        let table = import.handle;
        table.get(self.context()).size()
    }

    /// Grow table by the specified amount of elements.
    ///
    /// Returns `None` if table can't be grown by the specified amount
    /// of elements.
    pub(crate) fn table_grow(
        &mut self,
        table_index: LocalTableIndex,
        delta: u32,
        init_value: TableElement,
    ) -> Option<u32> {
        let table = *self
            .tables
            .get(table_index)
            .unwrap_or_else(|| panic!("no table for index {}", table_index.index()));
        table.get_mut(self.context_mut()).grow(delta, init_value)
    }

    /// Grow table by the specified amount of elements.
    ///
    /// # Safety
    /// `table_index` must be a valid, imported table index.
    pub(crate) unsafe fn imported_table_grow(
        &mut self,
        table_index: TableIndex,
        delta: u32,
        init_value: TableElement,
    ) -> Option<u32> {
        let import = self.imported_table(table_index);
        let table = import.handle;
        table.get_mut(self.context_mut()).grow(delta, init_value)
    }

    /// Get table element by index.
    pub(crate) fn table_get(
        &self,
        table_index: LocalTableIndex,
        index: u32,
    ) -> Option<TableElement> {
        let table = self
            .tables
            .get(table_index)
            .unwrap_or_else(|| panic!("no table for index {}", table_index.index()));
        table.get(self.context()).get(index)
    }

    /// Returns the element at the given index.
    ///
    /// # Safety
    /// `table_index` must be a valid, imported table index.
    pub(crate) unsafe fn imported_table_get(
        &self,
        table_index: TableIndex,
        index: u32,
    ) -> Option<TableElement> {
        let import = self.imported_table(table_index);
        let table = import.handle;
        table.get(self.context()).get(index)
    }

    /// Set table element by index.
    pub(crate) fn table_set(
        &mut self,
        table_index: LocalTableIndex,
        index: u32,
        val: TableElement,
    ) -> Result<(), Trap> {
        let table = *self
            .tables
            .get(table_index)
            .unwrap_or_else(|| panic!("no table for index {}", table_index.index()));
        table.get_mut(self.context_mut()).set(index, val)
    }

    /// Set table element by index for an imported table.
    ///
    /// # Safety
    /// `table_index` must be a valid, imported table index.
    pub(crate) unsafe fn imported_table_set(
        &mut self,
        table_index: TableIndex,
        index: u32,
        val: TableElement,
    ) -> Result<(), Trap> {
        let import = self.imported_table(table_index);
        let table = import.handle;
        table.get_mut(self.context_mut()).set(index, val)
    }

    /// Get a `VMFuncRef` for the given `FunctionIndex`.
    pub(crate) fn func_ref(&self, function_index: FunctionIndex) -> Option<VMFuncRef> {
        if function_index == FunctionIndex::reserved_value() {
            None
        } else if let Some(local_function_index) = self.module.local_func_index(function_index) {
            Some(VMFuncRef(NonNull::from(
                &self.funcrefs[local_function_index],
            )))
        } else {
            Some(VMFuncRef(self.imported_funcrefs[function_index]))
        }
    }

    /// The `table.init` operation: initializes a portion of a table with a
    /// passive element.
    ///
    /// # Errors
    ///
    /// Returns a `Trap` error when the range within the table is out of bounds
    /// or the range within the passive element is out of bounds.
    pub(crate) fn table_init(
        &mut self,
        table_index: TableIndex,
        elem_index: ElemIndex,
        dst: u32,
        src: u32,
        len: u32,
    ) -> Result<(), Trap> {
        // https://webassembly.github.io/bulk-memory-operations/core/exec/instructions.html#exec-table-init

        let table = self.get_table_handle(table_index);
        let table = unsafe { table.get_mut(&mut *self.context) };
        let passive_elements = self.passive_elements.borrow();
        let elem = passive_elements
            .get(&elem_index)
            .map_or::<&[Option<VMFuncRef>], _>(&[], |e| &**e);

        if src
            .checked_add(len)
            .map_or(true, |n| n as usize > elem.len())
            || dst.checked_add(len).map_or(true, |m| m > table.size())
        {
            return Err(Trap::lib(TrapCode::TableAccessOutOfBounds));
        }

        for (dst, src) in (dst..dst + len).zip(src..src + len) {
            table
                .set(dst, TableElement::FuncRef(elem[src as usize]))
                .expect("should never panic because we already did the bounds check above");
        }

        Ok(())
    }

    /// The `table.fill` operation: fills a portion of a table with a given value.
    ///
    /// # Errors
    ///
    /// Returns a `Trap` error when the range within the table is out of bounds
    pub(crate) fn table_fill(
        &mut self,
        table_index: TableIndex,
        start_index: u32,
        item: TableElement,
        len: u32,
    ) -> Result<(), Trap> {
        // https://webassembly.github.io/bulk-memory-operations/core/exec/instructions.html#exec-table-init

        let table = self.get_table(table_index);
        let table_size = table.size() as usize;

        if start_index
            .checked_add(len)
            .map_or(true, |n| n as usize > table_size)
        {
            return Err(Trap::lib(TrapCode::TableAccessOutOfBounds));
        }

        for i in start_index..(start_index + len) {
            table
                .set(i, item.clone())
                .expect("should never panic because we already did the bounds check above");
        }

        Ok(())
    }

    /// Drop an element.
    pub(crate) fn elem_drop(&self, elem_index: ElemIndex) {
        // https://webassembly.github.io/reference-types/core/exec/instructions.html#exec-elem-drop

        let mut passive_elements = self.passive_elements.borrow_mut();
        passive_elements.remove(&elem_index);
        // Note that we don't check that we actually removed an element because
        // dropping a non-passive element is a no-op (not a trap).
    }

    /// Do a `memory.copy` for a locally defined memory.
    ///
    /// # Errors
    ///
    /// Returns a `Trap` error when the source or destination ranges are out of
    /// bounds.
    pub(crate) fn local_memory_copy(
        &self,
        memory_index: LocalMemoryIndex,
        dst: u32,
        src: u32,
        len: u32,
    ) -> Result<(), Trap> {
        // https://webassembly.github.io/reference-types/core/exec/instructions.html#exec-memory-copy

        let memory = self.memory(memory_index);
        // The following memory copy is not synchronized and is not atomic:
        unsafe { memory_copy(&memory, dst, src, len) }
    }

    /// Perform a `memory.copy` on an imported memory.
    pub(crate) fn imported_memory_copy(
        &self,
        memory_index: MemoryIndex,
        dst: u32,
        src: u32,
        len: u32,
    ) -> Result<(), Trap> {
        let import = self.imported_memory(memory_index);
        let memory = unsafe { import.definition.as_ref() };
        // The following memory copy is not synchronized and is not atomic:
        unsafe { memory_copy(memory, dst, src, len) }
    }

    /// Perform the `memory.fill` operation on a locally defined memory.
    ///
    /// # Errors
    ///
    /// Returns a `Trap` error if the memory range is out of bounds.
    pub(crate) fn local_memory_fill(
        &self,
        memory_index: LocalMemoryIndex,
        dst: u32,
        val: u32,
        len: u32,
    ) -> Result<(), Trap> {
        let memory = self.memory(memory_index);
        // The following memory fill is not synchronized and is not atomic:
        unsafe { memory_fill(&memory, dst, val, len) }
    }

    /// Perform the `memory.fill` operation on an imported memory.
    ///
    /// # Errors
    ///
    /// Returns a `Trap` error if the memory range is out of bounds.
    pub(crate) fn imported_memory_fill(
        &self,
        memory_index: MemoryIndex,
        dst: u32,
        val: u32,
        len: u32,
    ) -> Result<(), Trap> {
        let import = self.imported_memory(memory_index);
        let memory = unsafe { import.definition.as_ref() };
        // The following memory fill is not synchronized and is not atomic:
        unsafe { memory_fill(memory, dst, val, len) }
    }

    /// Performs the `memory.init` operation.
    ///
    /// # Errors
    ///
    /// Returns a `Trap` error if the destination range is out of this module's
    /// memory's bounds or if the source range is outside the data segment's
    /// bounds.
    pub(crate) fn memory_init(
        &self,
        memory_index: MemoryIndex,
        data_index: DataIndex,
        dst: u32,
        src: u32,
        len: u32,
    ) -> Result<(), Trap> {
        // https://webassembly.github.io/bulk-memory-operations/core/exec/instructions.html#exec-memory-init

        let memory = self.get_vmmemory(memory_index);
        let passive_data = self.passive_data.borrow();
        let data = passive_data.get(&data_index).map_or(&[][..], |d| &**d);

        let current_length = unsafe { memory.vmmemory().as_ref().current_length };
        if src
            .checked_add(len)
            .map_or(true, |n| n as usize > data.len())
            || dst
                .checked_add(len)
                .map_or(true, |m| usize::try_from(m).unwrap() > current_length)
        {
            return Err(Trap::lib(TrapCode::HeapAccessOutOfBounds));
        }
        let src_slice = &data[src as usize..(src + len) as usize];
        unsafe { memory.initialize_with_data(dst as usize, src_slice) }
    }

    /// Drop the given data segment, truncating its length to zero.
    pub(crate) fn data_drop(&self, data_index: DataIndex) {
        let mut passive_data = self.passive_data.borrow_mut();
        passive_data.remove(&data_index);
    }

    /// Get a table by index regardless of whether it is locally-defined or an
    /// imported, foreign table.
    pub(crate) fn get_table(&mut self, table_index: TableIndex) -> &mut VMTable {
        if let Some(local_table_index) = self.module.local_table_index(table_index) {
            self.get_local_table(local_table_index)
        } else {
            self.get_foreign_table(table_index)
        }
    }

    /// Get a locally-defined table.
    pub(crate) fn get_local_table(&mut self, index: LocalTableIndex) -> &mut VMTable {
        let table = self.tables[index];
        table.get_mut(self.context_mut())
    }

    /// Get an imported, foreign table.
    pub(crate) fn get_foreign_table(&mut self, index: TableIndex) -> &mut VMTable {
        let import = self.imported_table(index);
        let table = import.handle;
        table.get_mut(self.context_mut())
    }

    /// Get a table handle by index regardless of whether it is locally-defined
    /// or an imported, foreign table.
    pub(crate) fn get_table_handle(
        &mut self,
        table_index: TableIndex,
    ) -> InternalStoreHandle<VMTable> {
        if let Some(local_table_index) = self.module.local_table_index(table_index) {
            self.tables[local_table_index]
        } else {
            self.imported_table(table_index).handle
        }
    }

    fn memory_wait(memory: &mut VMMemory, dst: u32, timeout: i64) -> Result<u32, Trap> {
        let location = NotifyLocation { address: dst };
        let timeout = if timeout < 0 {
            None
        } else {
            Some(std::time::Duration::from_nanos(timeout as u64))
        };
        let waiter = memory.do_wait(location, timeout);
        if waiter.is_err() {
            // ret is None if there is more than 2^32 waiter in queue or some other error
            return Err(Trap::lib(TrapCode::TableAccessOutOfBounds));
        }
        Ok(waiter.unwrap())
    }

    /// Perform an Atomic.Wait32
    pub(crate) fn local_memory_wait32(
        &mut self,
        memory_index: LocalMemoryIndex,
        dst: u32,
        val: u32,
        timeout: i64,
    ) -> Result<u32, Trap> {
        let memory = self.memory(memory_index);
        //if ! memory.shared {
        // We should trap according to spec, but official test rely on not trapping...
        //}

        let ret = unsafe { memory32_atomic_check32(&memory, dst, val) };

        if let Ok(mut ret) = ret {
            if ret == 0 {
                let memory = self.get_local_vmmemory_mut(memory_index);
                ret = Self::memory_wait(memory, dst, timeout)?;
            }
            Ok(ret)
        } else {
            ret
        }
    }

    /// Perform an Atomic.Wait32
    pub(crate) fn imported_memory_wait32(
        &mut self,
        memory_index: MemoryIndex,
        dst: u32,
        val: u32,
        timeout: i64,
    ) -> Result<u32, Trap> {
        let import = self.imported_memory(memory_index);
        let memory = unsafe { import.definition.as_ref() };
        //if ! memory.shared {
        // We should trap according to spec, but official test rely on not trapping...
        //}

        let ret = unsafe { memory32_atomic_check32(memory, dst, val) };
        if let Ok(mut ret) = ret {
            if ret == 0 {
                let memory = self.get_vmmemory_mut(memory_index);
                ret = Self::memory_wait(memory, dst, timeout)?;
            }
            Ok(ret)
        } else {
            ret
        }
    }

    /// Perform an Atomic.Wait64
    pub(crate) fn local_memory_wait64(
        &mut self,
        memory_index: LocalMemoryIndex,
        dst: u32,
        val: u64,
        timeout: i64,
    ) -> Result<u32, Trap> {
        let memory = self.memory(memory_index);
        //if ! memory.shared {
        // We should trap according to spec, but official test rely on not trapping...
        //}

        let ret = unsafe { memory32_atomic_check64(&memory, dst, val) };

        if let Ok(mut ret) = ret {
            if ret == 0 {
                let memory = self.get_local_vmmemory_mut(memory_index);
                ret = Self::memory_wait(memory, dst, timeout)?;
            }
            Ok(ret)
        } else {
            ret
        }
    }

    /// Perform an Atomic.Wait64
    pub(crate) fn imported_memory_wait64(
        &mut self,
        memory_index: MemoryIndex,
        dst: u32,
        val: u64,
        timeout: i64,
    ) -> Result<u32, Trap> {
        let import = self.imported_memory(memory_index);
        let memory = unsafe { import.definition.as_ref() };
        //if ! memory.shared {
        // We should trap according to spec, but official test rely on not trapping...
        //}

        let ret = unsafe { memory32_atomic_check64(memory, dst, val) };

        if let Ok(mut ret) = ret {
            if ret == 0 {
                let memory = self.get_vmmemory_mut(memory_index);
                ret = Self::memory_wait(memory, dst, timeout)?;
            }
            Ok(ret)
        } else {
            ret
        }
    }

    /// Perform an Atomic.Notify
    pub(crate) fn local_memory_notify(
        &mut self,
        memory_index: LocalMemoryIndex,
        dst: u32,
        count: u32,
    ) -> Result<u32, Trap> {
        let memory = self.get_local_vmmemory_mut(memory_index);
        // fetch the notifier
        let location = NotifyLocation { address: dst };
        Ok(memory.do_notify(location, count))
    }

    /// Perform an Atomic.Notify
    pub(crate) fn imported_memory_notify(
        &mut self,
        memory_index: MemoryIndex,
        dst: u32,
        count: u32,
    ) -> Result<u32, Trap> {
        let memory = self.get_vmmemory_mut(memory_index);
        // fetch the notifier
        let location = NotifyLocation { address: dst };
        Ok(memory.do_notify(location, count))
    }
}

/// A handle holding an `Instance` of a WebAssembly module.
///
/// This is more or less a public facade of the private `Instance`,
/// providing useful higher-level API.
#[derive(Debug, Eq, PartialEq)]
pub struct VMInstance {
    /// The layout of `Instance` (which can vary).
    instance_layout: Layout,

    /// The `Instance` itself.
    ///
    /// `Instance` must not be dropped manually by Rust, because it's
    /// allocated manually with `alloc` and a specific layout (Rust
    /// would be able to drop `Instance` itself but it will imply a
    /// memory leak because of `alloc`).
    ///
    /// No one in the code has a copy of the `Instance`'s
    /// pointer. `Self` is the only one.
    instance: NonNull<Instance>,
}

/// VMInstance are created with an InstanceAllocator
/// and it will "consume" the memory
/// So the Drop here actualy free it (else it would be leaked)
impl Drop for VMInstance {
    fn drop(&mut self) {
        let instance_ptr = self.instance.as_ptr();

        unsafe {
            // Need to drop all the actual Instance members
            instance_ptr.drop_in_place();
            // And then free the memory allocated for the Instance itself
            std::alloc::dealloc(instance_ptr as *mut u8, self.instance_layout);
        }
    }
}

impl VMInstance {
    /// Create a new `VMInstance` pointing at a new [`Instance`].
    ///
    /// # Safety
    ///
    /// This method is not necessarily inherently unsafe to call, but in general
    /// the APIs of an `Instance` are quite unsafe and have not been really
    /// audited for safety that much. As a result the unsafety here on this
    /// method is a low-overhead way of saying “this is an extremely unsafe type
    /// to work with”.
    ///
    /// Extreme care must be taken when working with `VMInstance` and it's
    /// recommended to have relatively intimate knowledge of how it works
    /// internally if you'd like to do so. If possible it's recommended to use
    /// the `wasmer` crate API rather than this type since that is vetted for
    /// safety.
    ///
    /// However the following must be taken care of before calling this function:
    /// - The memory at `instance.tables_ptr()` must be initialized with data for
    ///   all the local tables.
    /// - The memory at `instance.memories_ptr()` must be initialized with data for
    ///   all the local memories.
    #[allow(clippy::too_many_arguments)]
    pub unsafe fn new(
        allocator: InstanceAllocator,
        module: Arc<ModuleInfo>,
        context: &mut StoreObjects,
        finished_functions: BoxedSlice<LocalFunctionIndex, FunctionBodyPtr>,
        finished_function_call_trampolines: BoxedSlice<SignatureIndex, VMTrampoline>,
        finished_memories: BoxedSlice<LocalMemoryIndex, InternalStoreHandle<VMMemory>>,
        finished_tables: BoxedSlice<LocalTableIndex, InternalStoreHandle<VMTable>>,
        finished_globals: BoxedSlice<LocalGlobalIndex, InternalStoreHandle<VMGlobal>>,
        imports: Imports,
        vmshared_signatures: BoxedSlice<SignatureIndex, VMSharedSignatureIndex>,
    ) -> Result<Self, Trap> {
        let vmctx_globals = finished_globals
            .values()
            .map(|m| m.get(context).vmglobal())
            .collect::<PrimaryMap<LocalGlobalIndex, _>>()
            .into_boxed_slice();
        let passive_data = RefCell::new(
            module
                .passive_data
                .clone()
                .into_iter()
                .map(|(idx, bytes)| (idx, Arc::from(bytes)))
                .collect::<HashMap<_, _>>(),
        );

        let handle = {
            let offsets = allocator.offsets().clone();
            // use dummy value to create an instance so we can get the vmctx pointer
            let funcrefs = PrimaryMap::new().into_boxed_slice();
            let imported_funcrefs = PrimaryMap::new().into_boxed_slice();
            // Create the `Instance`. The unique, the One.
            let instance = Instance {
                module,
                context,
                offsets,
                memories: finished_memories,
                tables: finished_tables,
                globals: finished_globals,
                functions: finished_functions,
                function_call_trampolines: finished_function_call_trampolines,
                passive_elements: Default::default(),
                passive_data,
                funcrefs,
                imported_funcrefs,
                vmctx: VMContext {},
            };

            let mut instance_handle = allocator.into_vminstance(instance);

            // Set the funcrefs after we've built the instance
            {
                let instance = instance_handle.instance_mut();
                let vmctx_ptr = instance.vmctx_ptr();
                (instance.funcrefs, instance.imported_funcrefs) = build_funcrefs(
                    &instance.module,
                    context,
                    &imports,
                    &instance.functions,
                    &vmshared_signatures,
                    &instance.function_call_trampolines,
                    vmctx_ptr,
                );
            }

            instance_handle
        };
        let instance = handle.instance();

        ptr::copy(
            vmshared_signatures.values().as_slice().as_ptr(),
            instance.signature_ids_ptr() as *mut VMSharedSignatureIndex,
            vmshared_signatures.len(),
        );
        ptr::copy(
            imports.functions.values().as_slice().as_ptr(),
            instance.imported_functions_ptr() as *mut VMFunctionImport,
            imports.functions.len(),
        );
        ptr::copy(
            imports.tables.values().as_slice().as_ptr(),
            instance.imported_tables_ptr() as *mut VMTableImport,
            imports.tables.len(),
        );
        ptr::copy(
            imports.memories.values().as_slice().as_ptr(),
            instance.imported_memories_ptr() as *mut VMMemoryImport,
            imports.memories.len(),
        );
        ptr::copy(
            imports.globals.values().as_slice().as_ptr(),
            instance.imported_globals_ptr() as *mut VMGlobalImport,
            imports.globals.len(),
        );
        // these should already be set, add asserts here? for:
        // - instance.tables_ptr() as *mut VMTableDefinition
        // - instance.memories_ptr() as *mut VMMemoryDefinition
        ptr::copy(
            vmctx_globals.values().as_slice().as_ptr(),
            instance.globals_ptr() as *mut NonNull<VMGlobalDefinition>,
            vmctx_globals.len(),
        );
        ptr::write(
            instance.builtin_functions_ptr() as *mut VMBuiltinFunctionsArray,
            VMBuiltinFunctionsArray::initialized(),
        );

        // Perform infallible initialization in this constructor, while fallible
        // initialization is deferred to the `initialize` method.
        initialize_passive_elements(instance);
        initialize_globals(instance);

        Ok(handle)
    }

    /// Return a reference to the contained `Instance`.
    pub(crate) fn instance(&self) -> &Instance {
        unsafe { self.instance.as_ref() }
    }

    /// Return a mutable reference to the contained `Instance`.
    pub(crate) fn instance_mut(&mut self) -> &mut Instance {
        unsafe { self.instance.as_mut() }
    }

    /// Finishes the instantiation process started by `Instance::new`.
    ///
    /// # Safety
    ///
    /// Only safe to call immediately after instantiation.
    pub unsafe fn finish_instantiation(
        &mut self,
        config: &VMConfig,
        trap_handler: Option<*const TrapHandlerFn<'static>>,
        data_initializers: &[DataInitializer<'_>],
    ) -> Result<(), Trap> {
        let instance = self.instance_mut();

        // Apply the initializers.
        initialize_tables(instance)?;
        initialize_memories(instance, data_initializers)?;

        // The WebAssembly spec specifies that the start function is
        // invoked automatically at instantiation time.
        instance.invoke_start_function(config, trap_handler)?;
        Ok(())
    }

    /// Return a reference to the vmctx used by compiled wasm code.
    pub fn vmctx(&self) -> &VMContext {
        self.instance().vmctx()
    }

    /// Return a raw pointer to the vmctx used by compiled wasm code.
    pub fn vmctx_ptr(&self) -> *mut VMContext {
        self.instance().vmctx_ptr()
    }

    /// Return a reference to the `VMOffsets` to get offsets in the
    /// `Self::vmctx_ptr` region. Be careful when doing pointer
    /// arithmetic!
    pub fn vmoffsets(&self) -> &VMOffsets {
        self.instance().offsets()
    }

    /// Return a reference-counting pointer to a module.
    pub fn module(&self) -> &Arc<ModuleInfo> {
        self.instance().module()
    }

    /// Return a reference to a module.
    pub fn module_ref(&self) -> &ModuleInfo {
        self.instance().module_ref()
    }

    /// Lookup an export with the given name.
    pub fn lookup(&mut self, field: &str) -> Option<VMExtern> {
        let export = *self.module_ref().exports.get(field)?;

        Some(self.lookup_by_declaration(export))
    }

    /// Lookup an export with the given export declaration.
    pub fn lookup_by_declaration(&mut self, export: ExportIndex) -> VMExtern {
        let instance = self.instance();

        match export {
            ExportIndex::Function(index) => {
                let sig_index = &instance.module.functions[index];
                let handle = if let Some(def_index) = instance.module.local_func_index(index) {
                    // A VMFunction is lazily created only for functions that are
                    // exported.
                    let signature = instance.module.signatures[*sig_index].clone();
                    let vm_function = VMFunction {
                        anyfunc: MaybeInstanceOwned::Instance(NonNull::from(
                            &instance.funcrefs[def_index],
                        )),
                        signature,
                        // Any function received is already static at this point as:
                        // 1. All locally defined functions in the Wasm have a static signature.
                        // 2. All the imported functions are already static (because
                        //    they point to the trampolines rather than the dynamic addresses).
                        kind: VMFunctionKind::Static,
                        host_data: Box::new(()),
                    };
                    InternalStoreHandle::new(self.instance_mut().context_mut(), vm_function)
                } else {
                    let import = instance.imported_function(index);
                    import.handle
                };

                VMExtern::Function(handle)
            }
            ExportIndex::Table(index) => {
                let handle = if let Some(def_index) = instance.module.local_table_index(index) {
                    instance.tables[def_index]
                } else {
                    let import = instance.imported_table(index);
                    import.handle
                };
                VMExtern::Table(handle)
            }
            ExportIndex::Memory(index) => {
                let handle = if let Some(def_index) = instance.module.local_memory_index(index) {
                    instance.memories[def_index]
                } else {
                    let import = instance.imported_memory(index);
                    import.handle
                };
                VMExtern::Memory(handle)
            }
            ExportIndex::Global(index) => {
                let handle = if let Some(def_index) = instance.module.local_global_index(index) {
                    instance.globals[def_index]
                } else {
                    let import = instance.imported_global(index);
                    import.handle
                };
                VMExtern::Global(handle)
            }
        }
    }

    /// Return an iterator over the exports of this instance.
    ///
    /// Specifically, it provides access to the key-value pairs, where the keys
    /// are export names, and the values are export declarations which can be
    /// resolved `lookup_by_declaration`.
    pub fn exports(&self) -> indexmap::map::Iter<String, ExportIndex> {
        self.module().exports.iter()
    }

    /// Return the memory index for the given `VMMemoryDefinition` in this instance.
    pub fn memory_index(&self, memory: &VMMemoryDefinition) -> LocalMemoryIndex {
        self.instance().memory_index(memory)
    }

    /// Grow memory in this instance by the specified amount of pages.
    ///
    /// Returns `None` if memory can't be grown by the specified amount
    /// of pages.
    pub fn memory_grow<IntoPages>(
        &mut self,
        memory_index: LocalMemoryIndex,
        delta: IntoPages,
    ) -> Result<Pages, MemoryError>
    where
        IntoPages: Into<Pages>,
    {
        self.instance_mut().memory_grow(memory_index, delta)
    }

    /// Return the table index for the given `VMTableDefinition` in this instance.
    pub fn table_index(&self, table: &VMTableDefinition) -> LocalTableIndex {
        self.instance().table_index(table)
    }

    /// Grow table in this instance by the specified amount of pages.
    ///
    /// Returns `None` if memory can't be grown by the specified amount
    /// of pages.
    pub fn table_grow(
        &mut self,
        table_index: LocalTableIndex,
        delta: u32,
        init_value: TableElement,
    ) -> Option<u32> {
        self.instance_mut()
            .table_grow(table_index, delta, init_value)
    }

    /// Get table element reference.
    ///
    /// Returns `None` if index is out of bounds.
    pub fn table_get(&self, table_index: LocalTableIndex, index: u32) -> Option<TableElement> {
        self.instance().table_get(table_index, index)
    }

    /// Set table element reference.
    ///
    /// Returns an error if the index is out of bounds
    pub fn table_set(
        &mut self,
        table_index: LocalTableIndex,
        index: u32,
        val: TableElement,
    ) -> Result<(), Trap> {
        self.instance_mut().table_set(table_index, index, val)
    }

    /// Get a table defined locally within this module.
    pub fn get_local_table(&mut self, index: LocalTableIndex) -> &mut VMTable {
        self.instance_mut().get_local_table(index)
    }
}

/// Compute the offset for a memory data initializer.
fn get_memory_init_start(init: &DataInitializer<'_>, instance: &Instance) -> usize {
    let mut start = init.location.offset;

    if let Some(base) = init.location.base {
        let val = unsafe {
            if let Some(def_index) = instance.module.local_global_index(base) {
                instance.global(def_index).val.u32
            } else {
                instance.imported_global(base).definition.as_ref().val.u32
            }
        };
        start += usize::try_from(val).unwrap();
    }

    start
}

#[allow(clippy::mut_from_ref)]
#[allow(dead_code)]
/// Return a byte-slice view of a memory's data.
unsafe fn get_memory_slice<'instance>(
    init: &DataInitializer<'_>,
    instance: &'instance Instance,
) -> &'instance mut [u8] {
    let memory = if let Some(local_memory_index) = instance
        .module
        .local_memory_index(init.location.memory_index)
    {
        instance.memory(local_memory_index)
    } else {
        let import = instance.imported_memory(init.location.memory_index);
        *import.definition.as_ref()
    };
    slice::from_raw_parts_mut(memory.base, memory.current_length)
}

/// Compute the offset for a table element initializer.
fn get_table_init_start(init: &TableInitializer, instance: &Instance) -> usize {
    let mut start = init.offset;

    if let Some(base) = init.base {
        let val = unsafe {
            if let Some(def_index) = instance.module.local_global_index(base) {
                instance.global(def_index).val.u32
            } else {
                instance.imported_global(base).definition.as_ref().val.u32
            }
        };
        start += usize::try_from(val).unwrap();
    }

    start
}

/// Initialize the table memory from the provided initializers.
fn initialize_tables(instance: &mut Instance) -> Result<(), Trap> {
    let module = Arc::clone(&instance.module);
    for init in &module.table_initializers {
        let start = get_table_init_start(init, instance);
        let table = instance.get_table_handle(init.table_index);
        let table = unsafe { table.get_mut(&mut *instance.context) };

        if start
            .checked_add(init.elements.len())
            .map_or(true, |end| end > table.size() as usize)
        {
            return Err(Trap::lib(TrapCode::TableAccessOutOfBounds));
        }

        for (i, func_idx) in init.elements.iter().enumerate() {
            let anyfunc = instance.func_ref(*func_idx);
            table
                .set(
                    u32::try_from(start + i).unwrap(),
                    TableElement::FuncRef(anyfunc),
                )
                .unwrap();
        }
    }

    Ok(())
}

/// Initialize the `Instance::passive_elements` map by resolving the
/// `ModuleInfo::passive_elements`'s `FunctionIndex`s into `VMCallerCheckedAnyfunc`s for
/// this instance.
fn initialize_passive_elements(instance: &Instance) {
    let mut passive_elements = instance.passive_elements.borrow_mut();
    debug_assert!(
        passive_elements.is_empty(),
        "should only be called once, at initialization time"
    );

    passive_elements.extend(
        instance
            .module
            .passive_elements
            .iter()
            .filter(|(_, segments)| !segments.is_empty())
            .map(|(idx, segments)| {
                (
                    *idx,
                    segments.iter().map(|s| instance.func_ref(*s)).collect(),
                )
            }),
    );
}

/// Initialize the table memory from the provided initializers.
fn initialize_memories(
    instance: &mut Instance,
    data_initializers: &[DataInitializer<'_>],
) -> Result<(), Trap> {
    for init in data_initializers {
        let memory = instance.get_vmmemory(init.location.memory_index);

        let start = get_memory_init_start(init, instance);
        unsafe {
            let current_length = memory.vmmemory().as_ref().current_length;
            if start
                .checked_add(init.data.len())
                .map_or(true, |end| end > current_length)
            {
                return Err(Trap::lib(TrapCode::HeapAccessOutOfBounds));
            }
            memory.initialize_with_data(start, init.data)?;
        }
    }

    Ok(())
}

fn initialize_globals(instance: &Instance) {
    let module = Arc::clone(&instance.module);
    for (index, initializer) in module.global_initializers.iter() {
        unsafe {
            let to = instance.global_ptr(index).as_ptr();
            match initializer {
                GlobalInit::I32Const(x) => (*to).val.i32 = *x,
                GlobalInit::I64Const(x) => (*to).val.i64 = *x,
                GlobalInit::F32Const(x) => (*to).val.f32 = *x,
                GlobalInit::F64Const(x) => (*to).val.f64 = *x,
                GlobalInit::V128Const(x) => (*to).val.bytes = *x.bytes(),
                GlobalInit::GetGlobal(x) => {
                    let from: VMGlobalDefinition =
                        if let Some(def_x) = module.local_global_index(*x) {
                            instance.global(def_x)
                        } else {
                            instance.imported_global(*x).definition.as_ref().clone()
                        };
                    *to = from;
                }
                GlobalInit::RefNullConst => (*to).val.funcref = 0,
                GlobalInit::RefFunc(func_idx) => {
                    let funcref = instance.func_ref(*func_idx).unwrap();
                    (*to).val = funcref.into_raw();
                }
            }
        }
    }
}

/// Eagerly builds all the `VMFuncRef`s for imported and local functions so that all
/// future funcref operations are just looking up this data.
fn build_funcrefs(
    module_info: &ModuleInfo,
    ctx: &StoreObjects,
    imports: &Imports,
    finished_functions: &BoxedSlice<LocalFunctionIndex, FunctionBodyPtr>,
    vmshared_signatures: &BoxedSlice<SignatureIndex, VMSharedSignatureIndex>,
    function_call_trampolines: &BoxedSlice<SignatureIndex, VMTrampoline>,
    vmctx_ptr: *mut VMContext,
) -> (
    BoxedSlice<LocalFunctionIndex, VMCallerCheckedAnyfunc>,
    BoxedSlice<FunctionIndex, NonNull<VMCallerCheckedAnyfunc>>,
) {
    let mut func_refs =
        PrimaryMap::with_capacity(module_info.functions.len() - module_info.num_imported_functions);
    let mut imported_func_refs = PrimaryMap::with_capacity(module_info.num_imported_functions);

    // do imported functions
    for import in imports.functions.values() {
        imported_func_refs.push(import.handle.get(ctx).anyfunc.as_ptr());
    }

    // do local functions
    for (local_index, func_ptr) in finished_functions.iter() {
        let index = module_info.func_index(local_index);
        let sig_index = module_info.functions[index];
        let type_index = vmshared_signatures[sig_index];
        let call_trampoline = function_call_trampolines[sig_index];
        let anyfunc = VMCallerCheckedAnyfunc {
            func_ptr: func_ptr.0,
            type_index,
            vmctx: VMFunctionContext { vmctx: vmctx_ptr },
            call_trampoline,
        };
        func_refs.push(anyfunc);
    }
    (
        func_refs.into_boxed_slice(),
        imported_func_refs.into_boxed_slice(),
    )
}
