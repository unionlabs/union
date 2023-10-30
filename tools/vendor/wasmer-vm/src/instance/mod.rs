// This file contains code from external sources.
// Attributions: https://github.com/wasmerio/wasmer/blob/master/ATTRIBUTIONS.md

//! An `Instance` contains all the runtime state used by execution of
//! a WebAssembly module (except its callstack and register state). An
//! `InstanceRef` is a wrapper around `Instance` that manages
//! how it is allocated and deallocated. An `InstanceHandle` is a
//! wrapper around an `InstanceRef`.

mod allocator;
mod r#ref;

pub use allocator::InstanceAllocator;
pub use r#ref::{InstanceRef, WeakInstanceRef, WeakOrStrongInstanceRef};

use crate::export::VMExtern;
use crate::func_data_registry::VMFuncRef;
use crate::global::Global;
use crate::imports::Imports;
use crate::memory::{Memory, MemoryError};
use crate::table::{Table, TableElement};
use crate::trap::{catch_traps, Trap, TrapCode, TrapHandler};
use crate::vmcontext::{
    VMBuiltinFunctionsArray, VMCallerCheckedAnyfunc, VMContext, VMFunctionBody,
    VMFunctionEnvironment, VMFunctionImport, VMFunctionKind, VMGlobalDefinition, VMGlobalImport,
    VMMemoryDefinition, VMMemoryImport, VMSharedSignatureIndex, VMTableDefinition, VMTableImport,
    VMTrampoline,
};
use crate::{FunctionBodyPtr, VMOffsets};
use crate::{VMFunction, VMGlobal, VMMemory, VMTable};
use loupe::{MemoryUsage, MemoryUsageTracker};
use memoffset::offset_of;
use more_asserts::assert_lt;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::ffi;
use std::fmt;
use std::mem;
use std::ptr::{self, NonNull};
use std::slice;
use std::sync::Arc;
use wasmer_types::entity::{packed_option::ReservedValue, BoxedSlice, EntityRef, PrimaryMap};
use wasmer_types::{
    DataIndex, DataInitializer, ElemIndex, ExportIndex, FunctionIndex, GlobalIndex, GlobalInit,
    LocalFunctionIndex, LocalGlobalIndex, LocalMemoryIndex, LocalTableIndex, MemoryIndex,
    ModuleInfo, Pages, SignatureIndex, TableIndex, TableInitializer,
};

/// The function pointer to call with data and an [`Instance`] pointer to
/// finish initializing the host env.
pub type ImportInitializerFuncPtr<ResultErr = *mut ffi::c_void> =
    fn(*mut ffi::c_void, *const ffi::c_void) -> Result<(), ResultErr>;

/// A WebAssembly instance.
///
/// The type is dynamically-sized. Indeed, the `vmctx` field can
/// contain various data. That's why the type has a C representation
/// to ensure that the `vmctx` field is last. See the documentation of
/// the `vmctx` field to learn more.
#[derive(MemoryUsage)]
#[repr(C)]
pub(crate) struct Instance {
    /// The `ModuleInfo` this `Instance` was instantiated from.
    module: Arc<ModuleInfo>,

    /// Offsets in the `vmctx` region.
    offsets: VMOffsets,

    /// WebAssembly linear memory data.
    memories: BoxedSlice<LocalMemoryIndex, Arc<dyn Memory>>,

    /// WebAssembly table data.
    tables: BoxedSlice<LocalTableIndex, Arc<dyn Table>>,

    /// WebAssembly global data.
    globals: BoxedSlice<LocalGlobalIndex, Arc<Global>>,

    /// Pointers to functions in executable memory.
    functions: BoxedSlice<LocalFunctionIndex, FunctionBodyPtr>,

    /// Pointers to function call trampolines in executable memory.
    #[loupe(skip)]
    function_call_trampolines: BoxedSlice<SignatureIndex, VMTrampoline>,

    /// Passive elements in this instantiation. As `elem.drop`s happen, these
    /// entries get removed.
    passive_elements: RefCell<HashMap<ElemIndex, Box<[VMFuncRef]>>>,

    /// Passive data segments from our module. As `data.drop`s happen, entries
    /// get removed. A missing entry is considered equivalent to an empty slice.
    passive_data: RefCell<HashMap<DataIndex, Arc<[u8]>>>,

    /// Mapping of function indices to their func ref backing data. `VMFuncRef`s
    /// will point to elements here for functions defined or imported by this
    /// instance.
    funcrefs: BoxedSlice<FunctionIndex, VMCallerCheckedAnyfunc>,

    /// Hosts can store arbitrary per-instance information here.
    host_state: Box<dyn Any>,

    /// Functions to operate on host environments in the imports
    /// and pointers to the environments.
    ///
    /// TODO: Be sure to test with serialize/deserialize and imported
    /// functions from other Wasm modules.
    imported_function_envs: BoxedSlice<FunctionIndex, ImportFunctionEnv>,

    /// Additional context used by compiled WebAssembly code. This
    /// field is last, and represents a dynamically-sized array that
    /// extends beyond the nominal end of the struct (similar to a
    /// flexible array member).
    #[loupe(skip)]
    vmctx: VMContext,
}

/// A collection of data about host envs used by imported functions.
#[derive(Debug)]
pub enum ImportFunctionEnv {
    /// The `vmctx` pointer does not refer to a host env, there is no
    /// metadata about it.
    NoEnv,
    /// We're dealing with a user-defined host env.
    ///
    /// This host env may be either unwrapped (the user-supplied host env
    /// directly) or wrapped. i.e. in the case of Dynamic functions, we
    /// store our own extra data along with the user supplied env,
    /// thus the `env` pointer here points to the outermost type.
    Env {
        /// The function environment. This is not always the user-supplied
        /// env.
        env: *mut ffi::c_void,

        /// A clone function for duplicating the env.
        clone: fn(*mut ffi::c_void) -> *mut ffi::c_void,
        /// This field is not always present. When it is present, it
        /// should be set to `None` after use to prevent double
        /// initialization.
        initializer: Option<ImportInitializerFuncPtr>,
        /// The destructor to clean up the type in `env`.
        ///
        /// # Safety
        /// - This function must be called ina synchronized way. For
        ///   example, in the `Drop` implementation of this type.
        destructor: unsafe fn(*mut ffi::c_void),
    },
}

impl ImportFunctionEnv {
    /// Get the `initializer` function pointer if it exists.
    fn initializer(&self) -> Option<ImportInitializerFuncPtr> {
        match self {
            Self::Env { initializer, .. } => *initializer,
            _ => None,
        }
    }
}

impl Clone for ImportFunctionEnv {
    fn clone(&self) -> Self {
        match &self {
            Self::NoEnv => Self::NoEnv,
            Self::Env {
                env,
                clone,
                destructor,
                initializer,
            } => {
                let new_env = (*clone)(*env);
                Self::Env {
                    env: new_env,
                    clone: *clone,
                    destructor: *destructor,
                    initializer: *initializer,
                }
            }
        }
    }
}

impl Drop for ImportFunctionEnv {
    fn drop(&mut self) {
        match self {
            Self::Env {
                env, destructor, ..
            } => {
                // # Safety
                // - This is correct because we know no other references
                //   to this data can exist if we're dropping it.
                unsafe {
                    (destructor)(*env);
                }
            }
            Self::NoEnv => (),
        }
    }
}

impl MemoryUsage for ImportFunctionEnv {
    fn size_of_val(&self, _: &mut dyn MemoryUsageTracker) -> usize {
        mem::size_of_val(self)
    }
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
        &*self.module
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

    /// Get the import initializer func at the given index if it exists.
    fn imported_function_env_initializer(
        &self,
        index: FunctionIndex,
    ) -> Option<ImportInitializerFuncPtr> {
        self.imported_function_envs[index].initializer()
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

    /// Return a reference to the custom state attached to this instance.
    #[inline]
    pub fn host_state(&self) -> &dyn Any {
        &*self.host_state
    }

    /// Invoke the WebAssembly start function of the instance, if one is present.
    fn invoke_start_function(
        &self,
        trap_handler: &(dyn TrapHandler + 'static),
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
                    VMFunctionEnvironment {
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
            catch_traps(trap_handler, || {
                mem::transmute::<*const VMFunctionBody, unsafe extern "C" fn(VMFunctionEnvironment)>(
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
        &self,
        memory_index: LocalMemoryIndex,
        delta: IntoPages,
    ) -> Result<Pages, MemoryError>
    where
        IntoPages: Into<Pages>,
    {
        let mem = self
            .memories
            .get(memory_index)
            .unwrap_or_else(|| panic!("no memory for index {}", memory_index.index()));
        mem.grow(delta.into())
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
        &self,
        memory_index: MemoryIndex,
        delta: IntoPages,
    ) -> Result<Pages, MemoryError>
    where
        IntoPages: Into<Pages>,
    {
        let import = self.imported_memory(memory_index);
        let from = import.from.as_ref();
        from.grow(delta.into())
    }

    /// Returns the number of allocated wasm pages.
    pub(crate) fn memory_size(&self, memory_index: LocalMemoryIndex) -> Pages {
        self.memories
            .get(memory_index)
            .unwrap_or_else(|| panic!("no memory for index {}", memory_index.index()))
            .size()
    }

    /// Returns the number of allocated wasm pages in an imported memory.
    ///
    /// # Safety
    /// This and `imported_memory_grow` are currently unsafe because they
    /// dereference the memory import's pointers.
    pub(crate) unsafe fn imported_memory_size(&self, memory_index: MemoryIndex) -> Pages {
        let import = self.imported_memory(memory_index);
        let from = import.from.as_ref();
        from.size()
    }

    /// Returns the number of elements in a given table.
    pub(crate) fn table_size(&self, table_index: LocalTableIndex) -> u32 {
        self.tables
            .get(table_index)
            .unwrap_or_else(|| panic!("no table for index {}", table_index.index()))
            .size()
    }

    /// Returns the number of elements in a given imported table.
    ///
    /// # Safety
    /// `table_index` must be a valid, imported table index.
    pub(crate) unsafe fn imported_table_size(&self, table_index: TableIndex) -> u32 {
        let import = self.imported_table(table_index);
        let from = import.from.as_ref();
        from.size()
    }

    /// Grow table by the specified amount of elements.
    ///
    /// Returns `None` if table can't be grown by the specified amount
    /// of elements.
    pub(crate) fn table_grow(
        &self,
        table_index: LocalTableIndex,
        delta: u32,
        init_value: TableElement,
    ) -> Option<u32> {
        let result = self
            .tables
            .get(table_index)
            .unwrap_or_else(|| panic!("no table for index {}", table_index.index()))
            .grow(delta, init_value);

        result
    }

    /// Grow table by the specified amount of elements.
    ///
    /// # Safety
    /// `table_index` must be a valid, imported table index.
    pub(crate) unsafe fn imported_table_grow(
        &self,
        table_index: TableIndex,
        delta: u32,
        init_value: TableElement,
    ) -> Option<u32> {
        let import = self.imported_table(table_index);
        let from = import.from.as_ref();
        from.grow(delta, init_value)
    }

    /// Get table element by index.
    pub(crate) fn table_get(
        &self,
        table_index: LocalTableIndex,
        index: u32,
    ) -> Option<TableElement> {
        self.tables
            .get(table_index)
            .unwrap_or_else(|| panic!("no table for index {}", table_index.index()))
            .get(index)
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
        let from = import.from.as_ref();
        from.get(index)
    }

    /// Set table element by index.
    pub(crate) fn table_set(
        &self,
        table_index: LocalTableIndex,
        index: u32,
        val: TableElement,
    ) -> Result<(), Trap> {
        self.tables
            .get(table_index)
            .unwrap_or_else(|| panic!("no table for index {}", table_index.index()))
            .set(index, val)
    }

    /// Set table element by index for an imported table.
    ///
    /// # Safety
    /// `table_index` must be a valid, imported table index.
    pub(crate) unsafe fn imported_table_set(
        &self,
        table_index: TableIndex,
        index: u32,
        val: TableElement,
    ) -> Result<(), Trap> {
        let import = self.imported_table(table_index);
        let from = import.from.as_ref();
        from.set(index, val)
    }

    pub(crate) fn func_ref(&self, function_index: FunctionIndex) -> Option<VMFuncRef> {
        Some(self.get_vm_funcref(function_index))
    }

    /// Get a `VMFuncRef` for the given `FunctionIndex`.
    fn get_vm_funcref(&self, index: FunctionIndex) -> VMFuncRef {
        if index == FunctionIndex::reserved_value() {
            return VMFuncRef::null();
        }
        VMFuncRef(&self.funcrefs[index])
    }

    /// The `table.init` operation: initializes a portion of a table with a
    /// passive element.
    ///
    /// # Errors
    ///
    /// Returns a `Trap` error when the range within the table is out of bounds
    /// or the range within the passive element is out of bounds.
    pub(crate) fn table_init(
        &self,
        table_index: TableIndex,
        elem_index: ElemIndex,
        dst: u32,
        src: u32,
        len: u32,
    ) -> Result<(), Trap> {
        // https://webassembly.github.io/bulk-memory-operations/core/exec/instructions.html#exec-table-init

        let table = self.get_table(table_index);
        let passive_elements = self.passive_elements.borrow();
        let elem = passive_elements
            .get(&elem_index)
            .map_or::<&[VMFuncRef], _>(&[], |e| &**e);

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
        &self,
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
        unsafe { memory.memory_copy(dst, src, len) }
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
        unsafe { memory.memory_copy(dst, src, len) }
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
        unsafe { memory.memory_fill(dst, val, len) }
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
        unsafe { memory.memory_fill(dst, val, len) }
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

        let memory = self.get_memory(memory_index);
        let passive_data = self.passive_data.borrow();
        let data = passive_data.get(&data_index).map_or(&[][..], |d| &**d);

        if src
            .checked_add(len)
            .map_or(true, |n| n as usize > data.len())
            || dst.checked_add(len).map_or(true, |m| {
                usize::try_from(m).unwrap() > memory.current_length
            })
        {
            return Err(Trap::lib(TrapCode::HeapAccessOutOfBounds));
        }

        let src_slice = &data[src as usize..(src + len) as usize];

        unsafe {
            let dst_start = memory.base.add(dst as usize);
            let dst_slice = slice::from_raw_parts_mut(dst_start, len as usize);
            dst_slice.copy_from_slice(src_slice);
        }

        Ok(())
    }

    /// Drop the given data segment, truncating its length to zero.
    pub(crate) fn data_drop(&self, data_index: DataIndex) {
        let mut passive_data = self.passive_data.borrow_mut();
        passive_data.remove(&data_index);
    }

    /// Get a table by index regardless of whether it is locally-defined or an
    /// imported, foreign table.
    pub(crate) fn get_table(&self, table_index: TableIndex) -> &dyn Table {
        if let Some(local_table_index) = self.module.local_table_index(table_index) {
            self.get_local_table(local_table_index)
        } else {
            self.get_foreign_table(table_index)
        }
    }

    /// Get a locally-defined table.
    pub(crate) fn get_local_table(&self, index: LocalTableIndex) -> &dyn Table {
        self.tables[index].as_ref()
    }

    /// Get an imported, foreign table.
    pub(crate) fn get_foreign_table(&self, index: TableIndex) -> &dyn Table {
        let import = self.imported_table(index);
        &*import.from
    }
}

/// A handle holding an `InstanceRef`, which holds an `Instance`
/// of a WebAssembly module.
///
/// This is more or less a public facade of the private `Instance`,
/// providing useful higher-level API.
#[derive(Debug, PartialEq, MemoryUsage)]
pub struct InstanceHandle {
    /// The [`InstanceRef`]. See its documentation to learn more.
    instance: InstanceRef,
}

impl InstanceHandle {
    /// Create a new `InstanceHandle` pointing at a new [`InstanceRef`].
    ///
    /// # Safety
    ///
    /// This method is not necessarily inherently unsafe to call, but in general
    /// the APIs of an `Instance` are quite unsafe and have not been really
    /// audited for safety that much. As a result the unsafety here on this
    /// method is a low-overhead way of saying “this is an extremely unsafe type
    /// to work with”.
    ///
    /// Extreme care must be taken when working with `InstanceHandle` and it's
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
        finished_functions: BoxedSlice<LocalFunctionIndex, FunctionBodyPtr>,
        finished_function_call_trampolines: BoxedSlice<SignatureIndex, VMTrampoline>,
        finished_memories: BoxedSlice<LocalMemoryIndex, Arc<dyn Memory>>,
        finished_tables: BoxedSlice<LocalTableIndex, Arc<dyn Table>>,
        finished_globals: BoxedSlice<LocalGlobalIndex, Arc<Global>>,
        imports: Imports,
        vmshared_signatures: BoxedSlice<SignatureIndex, VMSharedSignatureIndex>,
        host_state: Box<dyn Any>,
        imported_function_envs: BoxedSlice<FunctionIndex, ImportFunctionEnv>,
    ) -> Result<Self, Trap> {
        let vmctx_globals = finished_globals
            .values()
            .map(|m| m.vmglobal())
            .collect::<PrimaryMap<LocalGlobalIndex, _>>()
            .into_boxed_slice();
        let passive_data = RefCell::new(module.passive_data.clone());

        let handle = {
            let offsets = allocator.offsets().clone();
            // use dummy value to create an instance so we can get the vmctx pointer
            let funcrefs = PrimaryMap::new().into_boxed_slice();
            // Create the `Instance`. The unique, the One.
            let instance = Instance {
                module,
                offsets,
                memories: finished_memories,
                tables: finished_tables,
                globals: finished_globals,
                functions: finished_functions,
                function_call_trampolines: finished_function_call_trampolines,
                passive_elements: Default::default(),
                passive_data,
                host_state,
                funcrefs,
                imported_function_envs,
                vmctx: VMContext {},
            };

            let mut instance_ref = allocator.write_instance(instance);

            // Set the funcrefs after we've built the instance
            {
                let instance = instance_ref.as_mut().unwrap();
                let vmctx_ptr = instance.vmctx_ptr();
                instance.funcrefs = build_funcrefs(
                    &*instance.module,
                    &imports,
                    &instance.functions,
                    &vmshared_signatures,
                    vmctx_ptr,
                );
            }

            Self {
                instance: instance_ref,
            }
        };
        let instance = handle.instance().as_ref();

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
    pub(crate) fn instance(&self) -> &InstanceRef {
        &self.instance
    }

    /// Finishes the instantiation process started by `Instance::new`.
    ///
    /// # Safety
    ///
    /// Only safe to call immediately after instantiation.
    pub unsafe fn finish_instantiation(
        &self,
        trap_handler: &(dyn TrapHandler + 'static),
        data_initializers: &[DataInitializer<'_>],
    ) -> Result<(), Trap> {
        let instance = self.instance().as_ref();

        // Apply the initializers.
        initialize_tables(instance)?;
        initialize_memories(instance, data_initializers)?;

        // The WebAssembly spec specifies that the start function is
        // invoked automatically at instantiation time.
        instance.invoke_start_function(trap_handler)?;
        Ok(())
    }

    /// Return a reference to the vmctx used by compiled wasm code.
    pub fn vmctx(&self) -> &VMContext {
        self.instance().as_ref().vmctx()
    }

    /// Return a raw pointer to the vmctx used by compiled wasm code.
    pub fn vmctx_ptr(&self) -> *mut VMContext {
        self.instance().as_ref().vmctx_ptr()
    }

    /// Return a reference to the `VMOffsets` to get offsets in the
    /// `Self::vmctx_ptr` region. Be careful when doing pointer
    /// arithmetic!
    pub fn vmoffsets(&self) -> &VMOffsets {
        self.instance().as_ref().offsets()
    }

    /// Return a reference-counting pointer to a module.
    pub fn module(&self) -> &Arc<ModuleInfo> {
        self.instance().as_ref().module()
    }

    /// Return a reference to a module.
    pub fn module_ref(&self) -> &ModuleInfo {
        self.instance().as_ref().module_ref()
    }

    /// Lookup an export with the given name.
    pub fn lookup(&self, field: &str) -> Option<VMExtern> {
        let export = self.module_ref().exports.get(field)?;

        Some(self.lookup_by_declaration(&export))
    }

    /// Lookup an export with the given export declaration.
    // TODO: maybe EngineExport
    pub fn lookup_by_declaration(&self, export: &ExportIndex) -> VMExtern {
        let instance = self.instance().clone();
        let instance_ref = instance.as_ref();

        match export {
            ExportIndex::Function(index) => {
                let sig_index = &instance_ref.module.functions[*index];
                let (address, vmctx, _function_ptr) =
                    if let Some(def_index) = instance_ref.module.local_func_index(*index) {
                        (
                            instance_ref.functions[def_index].0 as *const _,
                            VMFunctionEnvironment {
                                vmctx: instance_ref.vmctx_ptr(),
                            },
                            None,
                        )
                    } else {
                        let import = instance_ref.imported_function(*index);
                        let initializer = instance_ref.imported_function_env_initializer(*index);
                        (import.body, import.environment, initializer)
                    };
                let call_trampoline = Some(instance_ref.function_call_trampolines[*sig_index]);
                let signature = instance_ref.module.signatures[*sig_index].clone();

                VMFunction {
                    address,
                    // Any function received is already static at this point as:
                    // 1. All locally defined functions in the Wasm have a static signature.
                    // 2. All the imported functions are already static (because
                    //    they point to the trampolines rather than the dynamic addresses).
                    kind: VMFunctionKind::Static,
                    signature,
                    vmctx,
                    call_trampoline,
                    instance_ref: Some(WeakOrStrongInstanceRef::Strong(instance)),
                }
                .into()
            }
            ExportIndex::Table(index) => {
                let from = if let Some(def_index) = instance_ref.module.local_table_index(*index) {
                    instance_ref.tables[def_index].clone()
                } else {
                    let import = instance_ref.imported_table(*index);
                    import.from.clone()
                };
                VMTable {
                    from,
                    instance_ref: Some(WeakOrStrongInstanceRef::Strong(instance)),
                }
                .into()
            }
            ExportIndex::Memory(index) => {
                let from = if let Some(def_index) = instance_ref.module.local_memory_index(*index) {
                    instance_ref.memories[def_index].clone()
                } else {
                    let import = instance_ref.imported_memory(*index);
                    import.from.clone()
                };
                VMMemory {
                    from,
                    instance_ref: Some(WeakOrStrongInstanceRef::Strong(instance)),
                }
                .into()
            }
            ExportIndex::Global(index) => {
                let from = {
                    if let Some(def_index) = instance_ref.module.local_global_index(*index) {
                        instance_ref.globals[def_index].clone()
                    } else {
                        let import = instance_ref.imported_global(*index);
                        import.from.clone()
                    }
                };
                VMGlobal {
                    from,
                    instance_ref: Some(WeakOrStrongInstanceRef::Strong(instance)),
                }
                .into()
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

    /// Return a reference to the custom state attached to this instance.
    pub fn host_state(&self) -> &dyn Any {
        self.instance().as_ref().host_state()
    }

    /// Return the memory index for the given `VMMemoryDefinition` in this instance.
    pub fn memory_index(&self, memory: &VMMemoryDefinition) -> LocalMemoryIndex {
        self.instance().as_ref().memory_index(memory)
    }

    /// Grow memory in this instance by the specified amount of pages.
    ///
    /// Returns `None` if memory can't be grown by the specified amount
    /// of pages.
    pub fn memory_grow<IntoPages>(
        &self,
        memory_index: LocalMemoryIndex,
        delta: IntoPages,
    ) -> Result<Pages, MemoryError>
    where
        IntoPages: Into<Pages>,
    {
        self.instance().as_ref().memory_grow(memory_index, delta)
    }

    /// Return the table index for the given `VMTableDefinition` in this instance.
    pub fn table_index(&self, table: &VMTableDefinition) -> LocalTableIndex {
        self.instance().as_ref().table_index(table)
    }

    /// Grow table in this instance by the specified amount of pages.
    ///
    /// Returns `None` if memory can't be grown by the specified amount
    /// of pages.
    pub fn table_grow(
        &self,
        table_index: LocalTableIndex,
        delta: u32,
        init_value: TableElement,
    ) -> Option<u32> {
        self.instance()
            .as_ref()
            .table_grow(table_index, delta, init_value)
    }

    /// Get table element reference.
    ///
    /// Returns `None` if index is out of bounds.
    pub fn table_get(&self, table_index: LocalTableIndex, index: u32) -> Option<TableElement> {
        self.instance().as_ref().table_get(table_index, index)
    }

    /// Set table element reference.
    ///
    /// Returns an error if the index is out of bounds
    pub fn table_set(
        &self,
        table_index: LocalTableIndex,
        index: u32,
        val: TableElement,
    ) -> Result<(), Trap> {
        self.instance().as_ref().table_set(table_index, index, val)
    }

    /// Get a table defined locally within this module.
    pub fn get_local_table(&self, index: LocalTableIndex) -> &dyn Table {
        self.instance().as_ref().get_local_table(index)
    }

    /// Initializes the host environments.
    ///
    /// # Safety
    /// - This function must be called with the correct `Err` type parameter: the error type is not
    ///   visible to code in `wasmer_vm`, so it's the caller's responsibility to ensure these
    ///   functions are called with the correct type.
    /// - `instance_ptr` must point to a valid `wasmer::Instance`.
    pub unsafe fn initialize_host_envs<Err: Sized>(
        &mut self,
        instance_ptr: *const ffi::c_void,
    ) -> Result<(), Err> {
        let instance_ref = self.instance.as_mut_unchecked();

        for import_function_env in instance_ref.imported_function_envs.values_mut() {
            match import_function_env {
                ImportFunctionEnv::Env {
                    env,
                    ref mut initializer,
                    ..
                } => {
                    if let Some(f) = initializer {
                        // transmute our function pointer into one with the correct error type
                        let f = mem::transmute::<
                            &ImportInitializerFuncPtr,
                            &ImportInitializerFuncPtr<Err>,
                        >(f);
                        f(*env, instance_ptr)?;
                    }
                    *initializer = None;
                }
                ImportFunctionEnv::NoEnv => (),
            }
        }
        Ok(())
    }
}

/// Compute the offset for a memory data initializer.
fn get_memory_init_start(init: &DataInitializer<'_>, instance: &Instance) -> usize {
    let mut start = init.location.offset;

    if let Some(base) = init.location.base {
        let val = unsafe {
            if let Some(def_index) = instance.module.local_global_index(base) {
                instance.global(def_index).to_u32()
            } else {
                instance.imported_global(base).definition.as_ref().to_u32()
            }
        };
        start += usize::try_from(val).unwrap();
    }

    start
}

#[allow(clippy::mut_from_ref)]
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
    slice::from_raw_parts_mut(memory.base, memory.current_length.try_into().unwrap())
}

/// Compute the offset for a table element initializer.
fn get_table_init_start(init: &TableInitializer, instance: &Instance) -> usize {
    let mut start = init.offset;

    if let Some(base) = init.base {
        let val = unsafe {
            if let Some(def_index) = instance.module.local_global_index(base) {
                instance.global(def_index).to_u32()
            } else {
                instance.imported_global(base).definition.as_ref().to_u32()
            }
        };
        start += usize::try_from(val).unwrap();
    }

    start
}

/// Initialize the table memory from the provided initializers.
fn initialize_tables(instance: &Instance) -> Result<(), Trap> {
    let module = Arc::clone(&instance.module);
    for init in &module.table_initializers {
        let start = get_table_init_start(init, instance);
        let table = instance.get_table(init.table_index);

        if start
            .checked_add(init.elements.len())
            .map_or(true, |end| end > table.size() as usize)
        {
            return Err(Trap::lib(TrapCode::TableAccessOutOfBounds));
        }

        for (i, func_idx) in init.elements.iter().enumerate() {
            let anyfunc = instance.get_vm_funcref(*func_idx);
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
                    segments
                        .iter()
                        .map(|s| instance.get_vm_funcref(*s))
                        .collect(),
                )
            }),
    );
}

/// Initialize the table memory from the provided initializers.
fn initialize_memories(
    instance: &Instance,
    data_initializers: &[DataInitializer<'_>],
) -> Result<(), Trap> {
    for init in data_initializers {
        let memory = instance.get_memory(init.location.memory_index);

        let start = get_memory_init_start(init, instance);
        if start
            .checked_add(init.data.len())
            .map_or(true, |end| end > memory.current_length.try_into().unwrap())
        {
            return Err(Trap::lib(TrapCode::HeapAccessOutOfBounds));
        }

        unsafe {
            let mem_slice = get_memory_slice(init, instance);
            let end = start + init.data.len();
            let to_init = &mut mem_slice[start..end];
            to_init.copy_from_slice(init.data);
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
                GlobalInit::I32Const(x) => *(*to).as_i32_mut() = *x,
                GlobalInit::I64Const(x) => *(*to).as_i64_mut() = *x,
                GlobalInit::F32Const(x) => *(*to).as_f32_mut() = *x,
                GlobalInit::F64Const(x) => *(*to).as_f64_mut() = *x,
                GlobalInit::V128Const(x) => *(*to).as_bytes_mut() = *x.bytes(),
                GlobalInit::GetGlobal(x) => {
                    let from: VMGlobalDefinition =
                        if let Some(def_x) = module.local_global_index(*x) {
                            instance.global(def_x)
                        } else {
                            instance.imported_global(*x).definition.as_ref().clone()
                        };
                    *to = from;
                }
                GlobalInit::RefNullConst => *(*to).as_funcref_mut() = VMFuncRef::null(),
                GlobalInit::RefFunc(func_idx) => {
                    let funcref = instance.func_ref(*func_idx).unwrap();
                    *(*to).as_funcref_mut() = funcref;
                }
            }
        }
    }
}

/// Eagerly builds all the `VMFuncRef`s for imported and local functions so that all
/// future funcref operations are just looking up this data.
fn build_funcrefs(
    module_info: &ModuleInfo,
    imports: &Imports,
    finished_functions: &BoxedSlice<LocalFunctionIndex, FunctionBodyPtr>,
    vmshared_signatures: &BoxedSlice<SignatureIndex, VMSharedSignatureIndex>,
    vmctx_ptr: *mut VMContext,
) -> BoxedSlice<FunctionIndex, VMCallerCheckedAnyfunc> {
    let mut func_refs = PrimaryMap::with_capacity(module_info.functions.len());

    // do imported functions
    for (index, import) in imports.functions.iter() {
        let sig_index = module_info.functions[index];
        let type_index = vmshared_signatures[sig_index];
        let anyfunc = VMCallerCheckedAnyfunc {
            func_ptr: import.body,
            type_index,
            vmctx: import.environment,
        };
        func_refs.push(anyfunc);
    }

    // do local functions
    for (local_index, func_ptr) in finished_functions.iter() {
        let index = module_info.func_index(local_index);
        let sig_index = module_info.functions[index];
        let type_index = vmshared_signatures[sig_index];
        let anyfunc = VMCallerCheckedAnyfunc {
            func_ptr: func_ptr.0,
            type_index,
            vmctx: VMFunctionEnvironment { vmctx: vmctx_ptr },
        };
        func_refs.push(anyfunc);
    }

    func_refs.into_boxed_slice()
}
