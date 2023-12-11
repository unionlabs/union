// This file contains code from external sources.
// Attributions: https://github.com/wasmerio/wasmer/blob/master/ATTRIBUTIONS.md

//! This file declares `VMContext` and several related structs which contain
//! fields that compiled wasm code accesses directly.

use crate::global::VMGlobal;
use crate::instance::Instance;
use crate::memory::VMMemory;
use crate::store::InternalStoreHandle;
use crate::trap::{Trap, TrapCode};
use crate::VMFunctionBody;
use crate::VMTable;
use crate::{VMBuiltinFunctionIndex, VMFunction};
use std::convert::TryFrom;
use std::ptr::{self, NonNull};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::u32;
use wasmer_types::RawValue;

/// Union representing the first parameter passed when calling a function.
///
/// It may either be a pointer to the [`VMContext`] if it's a Wasm function
/// or a pointer to arbitrary data controlled by the host if it's a host function.
#[derive(Copy, Clone, Eq)]
pub union VMFunctionContext {
    /// Wasm functions take a pointer to [`VMContext`].
    pub vmctx: *mut VMContext,
    /// Host functions can have custom environments.
    pub host_env: *mut std::ffi::c_void,
}

impl VMFunctionContext {
    /// Check whether the pointer stored is null or not.
    pub fn is_null(&self) -> bool {
        unsafe { self.host_env.is_null() }
    }
}

impl std::fmt::Debug for VMFunctionContext {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("VMFunctionContext")
            .field("vmctx_or_hostenv", unsafe { &self.host_env })
            .finish()
    }
}

impl std::cmp::PartialEq for VMFunctionContext {
    fn eq(&self, rhs: &Self) -> bool {
        unsafe { self.host_env as usize == rhs.host_env as usize }
    }
}

impl std::hash::Hash for VMFunctionContext {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe {
            self.vmctx.hash(state);
        }
    }
}

/// An imported function.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct VMFunctionImport {
    /// A pointer to the imported function body.
    pub body: *const VMFunctionBody,

    /// A pointer to the `VMContext` that owns the function or host env data.
    pub environment: VMFunctionContext,

    /// Handle to the `VMFunction` in the context.
    pub handle: InternalStoreHandle<VMFunction>,
}

#[cfg(test)]
mod test_vmfunction_import {
    use super::VMFunctionImport;
    use memoffset::offset_of;
    use std::mem::size_of;
    use wasmer_types::ModuleInfo;
    use wasmer_types::VMOffsets;

    #[test]
    fn check_vmfunction_import_offsets() {
        let module = ModuleInfo::new();
        let offsets = VMOffsets::new(size_of::<*mut u8>() as u8, &module);
        assert_eq!(
            size_of::<VMFunctionImport>(),
            usize::from(offsets.size_of_vmfunction_import())
        );
        assert_eq!(
            offset_of!(VMFunctionImport, body),
            usize::from(offsets.vmfunction_import_body())
        );
        assert_eq!(
            offset_of!(VMFunctionImport, environment),
            usize::from(offsets.vmfunction_import_vmctx())
        );
    }
}

/// The `VMDynamicFunctionContext` is the context that dynamic
/// functions will receive when called (rather than `vmctx`).
/// A dynamic function is a function for which we don't know the signature
/// until runtime.
///
/// As such, we need to expose the dynamic function `context`
/// containing the relevant context for running the function indicated
/// in `address`.
#[repr(C)]
pub struct VMDynamicFunctionContext<T> {
    /// The address of the inner dynamic function.
    ///
    /// Note: The function must be on the form of
    /// `(*mut T, SignatureIndex, *mut i128)`.
    pub address: *const VMFunctionBody,

    /// The context that the inner dynamic function will receive.
    pub ctx: T,
}

// The `ctx` itself must be `Send`, `address` can be passed between
// threads because all usage is `unsafe` and synchronized.
unsafe impl<T: Sized + Send + Sync> Send for VMDynamicFunctionContext<T> {}
// The `ctx` itself must be `Sync`, `address` can be shared between
// threads because all usage is `unsafe` and synchronized.
unsafe impl<T: Sized + Send + Sync> Sync for VMDynamicFunctionContext<T> {}

impl<T: Sized + Clone + Send + Sync> Clone for VMDynamicFunctionContext<T> {
    fn clone(&self) -> Self {
        Self {
            address: self.address,
            ctx: self.ctx.clone(),
        }
    }
}

#[cfg(test)]
mod test_vmdynamicfunction_import_context {
    use super::VMDynamicFunctionContext;
    use crate::VMOffsets;
    use memoffset::offset_of;
    use std::mem::size_of;
    use wasmer_types::ModuleInfo;

    #[test]
    fn check_vmdynamicfunction_import_context_offsets() {
        let module = ModuleInfo::new();
        let offsets = VMOffsets::new(size_of::<*mut u8>() as u8, &module);
        assert_eq!(
            size_of::<VMDynamicFunctionContext<usize>>(),
            usize::from(offsets.size_of_vmdynamicfunction_import_context())
        );
        assert_eq!(
            offset_of!(VMDynamicFunctionContext<usize>, address),
            usize::from(offsets.vmdynamicfunction_import_context_address())
        );
        assert_eq!(
            offset_of!(VMDynamicFunctionContext<usize>, ctx),
            usize::from(offsets.vmdynamicfunction_import_context_ctx())
        );
    }
}

/// A function kind is a calling convention into and out of wasm code.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub enum VMFunctionKind {
    /// A static function has the native signature:
    /// `extern "C" (vmctx, arg1, arg2...) -> (result1, result2, ...)`.
    ///
    /// This is the default for functions that are defined:
    /// 1. In the Host, natively
    /// 2. In the WebAssembly file
    Static,

    /// A dynamic function has the native signature:
    /// `extern "C" (ctx, &[Value]) -> Vec<Value>`.
    ///
    /// This is the default for functions that are defined:
    /// 1. In the Host, dynamically
    Dynamic,
}

/// The fields compiled code needs to access to utilize a WebAssembly table
/// imported from another instance.
#[derive(Clone)]
#[repr(C)]
pub struct VMTableImport {
    /// A pointer to the imported table description.
    pub definition: NonNull<VMTableDefinition>,

    /// Handle to the `VMTable` in the context.
    pub handle: InternalStoreHandle<VMTable>,
}

#[cfg(test)]
mod test_vmtable_import {
    use super::VMTableImport;
    use crate::VMOffsets;
    use memoffset::offset_of;
    use std::mem::size_of;
    use wasmer_types::ModuleInfo;

    #[test]
    fn check_vmtable_import_offsets() {
        let module = ModuleInfo::new();
        let offsets = VMOffsets::new(size_of::<*mut u8>() as u8, &module);
        assert_eq!(
            size_of::<VMTableImport>(),
            usize::from(offsets.size_of_vmtable_import())
        );
        assert_eq!(
            offset_of!(VMTableImport, definition),
            usize::from(offsets.vmtable_import_definition())
        );
    }
}

/// The fields compiled code needs to access to utilize a WebAssembly linear
/// memory imported from another instance.
#[derive(Clone)]
#[repr(C)]
pub struct VMMemoryImport {
    /// A pointer to the imported memory description.
    pub definition: NonNull<VMMemoryDefinition>,

    /// A handle to the `Memory` that owns the memory description.
    pub handle: InternalStoreHandle<VMMemory>,
}

#[cfg(test)]
mod test_vmmemory_import {
    use super::VMMemoryImport;
    use crate::VMOffsets;
    use memoffset::offset_of;
    use std::mem::size_of;
    use wasmer_types::ModuleInfo;

    #[test]
    fn check_vmmemory_import_offsets() {
        let module = ModuleInfo::new();
        let offsets = VMOffsets::new(size_of::<*mut u8>() as u8, &module);
        assert_eq!(
            size_of::<VMMemoryImport>(),
            usize::from(offsets.size_of_vmmemory_import())
        );
        assert_eq!(
            offset_of!(VMMemoryImport, definition),
            usize::from(offsets.vmmemory_import_definition())
        );
        assert_eq!(
            offset_of!(VMMemoryImport, handle),
            usize::from(offsets.vmmemory_import_handle())
        );
    }
}

/// The fields compiled code needs to access to utilize a WebAssembly global
/// variable imported from another instance.
#[derive(Clone)]
#[repr(C)]
pub struct VMGlobalImport {
    /// A pointer to the imported global variable description.
    pub definition: NonNull<VMGlobalDefinition>,

    /// A handle to the `Global` that owns the global description.
    pub handle: InternalStoreHandle<VMGlobal>,
}

/// # Safety
/// This data is safe to share between threads because it's plain data that
/// is the user's responsibility to synchronize. Additionally, all operations
/// on `from` are thread-safe through the use of a mutex in [`VMGlobal`].
unsafe impl Send for VMGlobalImport {}
/// # Safety
/// This data is safe to share between threads because it's plain data that
/// is the user's responsibility to synchronize. And because it's `Clone`, there's
/// really no difference between passing it by reference or by value as far as
/// correctness in a multi-threaded context is concerned.
unsafe impl Sync for VMGlobalImport {}

#[cfg(test)]
mod test_vmglobal_import {
    use super::VMGlobalImport;
    use crate::VMOffsets;
    use memoffset::offset_of;
    use std::mem::size_of;
    use wasmer_types::ModuleInfo;

    #[test]
    fn check_vmglobal_import_offsets() {
        let module = ModuleInfo::new();
        let offsets = VMOffsets::new(size_of::<*mut u8>() as u8, &module);
        assert_eq!(
            size_of::<VMGlobalImport>(),
            usize::from(offsets.size_of_vmglobal_import())
        );
        assert_eq!(
            offset_of!(VMGlobalImport, definition),
            usize::from(offsets.vmglobal_import_definition())
        );
    }
}

/// Do an unsynchronized, non-atomic `memory.copy` for the memory.
///
/// # Errors
///
/// Returns a `Trap` error when the source or destination ranges are out of
/// bounds.
///
/// # Safety
/// The memory is not copied atomically and is not synchronized: it's the
/// caller's responsibility to synchronize.
pub(crate) unsafe fn memory_copy(
    mem: &VMMemoryDefinition,
    dst: u32,
    src: u32,
    len: u32,
) -> Result<(), Trap> {
    // https://webassembly.github.io/reference-types/core/exec/instructions.html#exec-memory-copy
    if src
        .checked_add(len)
        .map_or(true, |n| usize::try_from(n).unwrap() > mem.current_length)
        || dst
            .checked_add(len)
            .map_or(true, |m| usize::try_from(m).unwrap() > mem.current_length)
    {
        return Err(Trap::lib(TrapCode::HeapAccessOutOfBounds));
    }

    let dst = usize::try_from(dst).unwrap();
    let src = usize::try_from(src).unwrap();

    // Bounds and casts are checked above, by this point we know that
    // everything is safe.
    let dst = mem.base.add(dst);
    let src = mem.base.add(src);
    ptr::copy(src, dst, len as usize);

    Ok(())
}

/// Perform the `memory.fill` operation for the memory in an unsynchronized,
/// non-atomic way.
///
/// # Errors
///
/// Returns a `Trap` error if the memory range is out of bounds.
///
/// # Safety
/// The memory is not filled atomically and is not synchronized: it's the
/// caller's responsibility to synchronize.
pub(crate) unsafe fn memory_fill(
    mem: &VMMemoryDefinition,
    dst: u32,
    val: u32,
    len: u32,
) -> Result<(), Trap> {
    if dst
        .checked_add(len)
        .map_or(true, |m| usize::try_from(m).unwrap() > mem.current_length)
    {
        return Err(Trap::lib(TrapCode::HeapAccessOutOfBounds));
    }

    let dst = isize::try_from(dst).unwrap();
    let val = val as u8;

    // Bounds and casts are checked above, by this point we know that
    // everything is safe.
    let dst = mem.base.offset(dst);
    ptr::write_bytes(dst, val, len as usize);

    Ok(())
}

/// Perform the `memory32.atomic.check32` operation for the memory. Return 0 if same, 1 if different
///
/// # Errors
///
/// Returns a `Trap` error if the memory range is out of bounds or 32bits unligned.
///
/// # Safety
/// memory access is unsafe
pub(crate) unsafe fn memory32_atomic_check32(
    mem: &VMMemoryDefinition,
    dst: u32,
    val: u32,
) -> Result<u32, Trap> {
    if usize::try_from(dst).unwrap() > mem.current_length {
        return Err(Trap::lib(TrapCode::HeapAccessOutOfBounds));
    }

    let dst = isize::try_from(dst).unwrap();
    if dst & 0b11 != 0 {
        return Err(Trap::lib(TrapCode::UnalignedAtomic));
    }

    // Bounds and casts are checked above, by this point we know that
    // everything is safe.
    let dst = mem.base.offset(dst) as *mut u32;
    let atomic_dst = AtomicPtr::new(dst);
    let read_val = *atomic_dst.load(Ordering::Acquire);
    let ret = if read_val == val { 0 } else { 1 };
    Ok(ret)
}

/// Perform the `memory32.atomic.check64` operation for the memory. Return 0 if same, 1 if different
///
/// # Errors
///
/// Returns a `Trap` error if the memory range is out of bounds or 64bits unaligned.
///
/// # Safety
/// memory access is unsafe
pub(crate) unsafe fn memory32_atomic_check64(
    mem: &VMMemoryDefinition,
    dst: u32,
    val: u64,
) -> Result<u32, Trap> {
    if usize::try_from(dst).unwrap() > mem.current_length {
        return Err(Trap::lib(TrapCode::HeapAccessOutOfBounds));
    }

    let dst = isize::try_from(dst).unwrap();
    if dst & 0b111 != 0 {
        return Err(Trap::lib(TrapCode::UnalignedAtomic));
    }

    // Bounds and casts are checked above, by this point we know that
    // everything is safe.
    let dst = mem.base.offset(dst) as *mut u64;
    let atomic_dst = AtomicPtr::new(dst);
    let read_val = *atomic_dst.load(Ordering::Acquire);
    let ret = if read_val == val { 0 } else { 1 };
    Ok(ret)
}

/// The fields compiled code needs to access to utilize a WebAssembly table
/// defined within the instance.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct VMTableDefinition {
    /// Pointer to the table data.
    pub base: *mut u8,

    /// The current number of elements in the table.
    pub current_elements: u32,
}

#[cfg(test)]
mod test_vmtable_definition {
    use super::VMTableDefinition;
    use crate::VMOffsets;
    use memoffset::offset_of;
    use std::mem::size_of;
    use wasmer_types::ModuleInfo;

    #[test]
    fn check_vmtable_definition_offsets() {
        let module = ModuleInfo::new();
        let offsets = VMOffsets::new(size_of::<*mut u8>() as u8, &module);
        assert_eq!(
            size_of::<VMTableDefinition>(),
            usize::from(offsets.size_of_vmtable_definition())
        );
        assert_eq!(
            offset_of!(VMTableDefinition, base),
            usize::from(offsets.vmtable_definition_base())
        );
        assert_eq!(
            offset_of!(VMTableDefinition, current_elements),
            usize::from(offsets.vmtable_definition_current_elements())
        );
    }
}

/// The storage for a WebAssembly global defined within the instance.
///
/// TODO: Pack the globals more densely, rather than using the same size
/// for every type.
#[derive(Debug, Clone)]
#[repr(C, align(16))]
pub struct VMGlobalDefinition {
    /// Raw value of the global.
    pub val: RawValue,
}

#[cfg(test)]
mod test_vmglobal_definition {
    use super::VMGlobalDefinition;
    use crate::{VMFuncRef, VMOffsets};
    use more_asserts::assert_ge;
    use std::mem::{align_of, size_of};
    use wasmer_types::ModuleInfo;

    #[test]
    fn check_vmglobal_definition_alignment() {
        assert_ge!(align_of::<VMGlobalDefinition>(), align_of::<i32>());
        assert_ge!(align_of::<VMGlobalDefinition>(), align_of::<i64>());
        assert_ge!(align_of::<VMGlobalDefinition>(), align_of::<f32>());
        assert_ge!(align_of::<VMGlobalDefinition>(), align_of::<f64>());
        assert_ge!(align_of::<VMGlobalDefinition>(), align_of::<VMFuncRef>());
        assert_ge!(align_of::<VMGlobalDefinition>(), align_of::<[u8; 16]>());
    }

    #[test]
    fn check_vmglobal_definition_offsets() {
        let module = ModuleInfo::new();
        let offsets = VMOffsets::new(size_of::<*mut u8>() as u8, &module);
        assert_eq!(
            size_of::<*const VMGlobalDefinition>(),
            usize::from(offsets.size_of_vmglobal_local())
        );
    }

    #[test]
    fn check_vmglobal_begins_aligned() {
        let module = ModuleInfo::new();
        let offsets = VMOffsets::new(size_of::<*mut u8>() as u8, &module);
        assert_eq!(offsets.vmctx_globals_begin() % 16, 0);
    }
}

impl VMGlobalDefinition {
    /// Construct a `VMGlobalDefinition`.
    pub fn new() -> Self {
        Self {
            val: Default::default(),
        }
    }
}

/// An index into the shared signature registry, usable for checking signatures
/// at indirect calls.
#[repr(C)]
#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
pub struct VMSharedSignatureIndex(u32);

#[cfg(test)]
mod test_vmshared_signature_index {
    use super::VMSharedSignatureIndex;
    use std::mem::size_of;
    use wasmer_types::{ModuleInfo, TargetSharedSignatureIndex, VMOffsets};

    #[test]
    fn check_vmshared_signature_index() {
        let module = ModuleInfo::new();
        let offsets = VMOffsets::new(size_of::<*mut u8>() as u8, &module);
        assert_eq!(
            size_of::<VMSharedSignatureIndex>(),
            usize::from(offsets.size_of_vmshared_signature_index())
        );
    }

    #[test]
    fn check_target_shared_signature_index() {
        assert_eq!(
            size_of::<VMSharedSignatureIndex>(),
            size_of::<TargetSharedSignatureIndex>()
        );
    }
}

impl VMSharedSignatureIndex {
    /// Create a new `VMSharedSignatureIndex`.
    pub fn new(value: u32) -> Self {
        Self(value)
    }
}

impl Default for VMSharedSignatureIndex {
    fn default() -> Self {
        Self::new(u32::MAX)
    }
}

/// The VM caller-checked "anyfunc" record, for caller-side signature checking.
/// It consists of the actual function pointer and a signature id to be checked
/// by the caller.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[repr(C)]
pub struct VMCallerCheckedAnyfunc {
    /// Function body.
    pub func_ptr: *const VMFunctionBody,
    /// Function signature id.
    pub type_index: VMSharedSignatureIndex,
    /// Function `VMContext` or host env.
    pub vmctx: VMFunctionContext,
    /// Address of the function call trampoline to invoke this function using
    /// a dynamic argument list.
    pub call_trampoline: VMTrampoline,
    // If more elements are added here, remember to add offset_of tests below!
}

#[cfg(test)]
mod test_vmcaller_checked_anyfunc {
    use super::VMCallerCheckedAnyfunc;
    use crate::VMOffsets;
    use memoffset::offset_of;
    use std::mem::size_of;
    use wasmer_types::ModuleInfo;

    #[test]
    fn check_vmcaller_checked_anyfunc_offsets() {
        let module = ModuleInfo::new();
        let offsets = VMOffsets::new(size_of::<*mut u8>() as u8, &module);
        assert_eq!(
            size_of::<VMCallerCheckedAnyfunc>(),
            usize::from(offsets.size_of_vmcaller_checked_anyfunc())
        );
        assert_eq!(
            offset_of!(VMCallerCheckedAnyfunc, func_ptr),
            usize::from(offsets.vmcaller_checked_anyfunc_func_ptr())
        );
        assert_eq!(
            offset_of!(VMCallerCheckedAnyfunc, type_index),
            usize::from(offsets.vmcaller_checked_anyfunc_type_index())
        );
        assert_eq!(
            offset_of!(VMCallerCheckedAnyfunc, vmctx),
            usize::from(offsets.vmcaller_checked_anyfunc_vmctx())
        );
    }
}

/// An array that stores addresses of builtin functions. We translate code
/// to use indirect calls. This way, we don't have to patch the code.
#[repr(C)]
pub struct VMBuiltinFunctionsArray {
    ptrs: [usize; Self::len()],
}

impl VMBuiltinFunctionsArray {
    pub const fn len() -> usize {
        VMBuiltinFunctionIndex::builtin_functions_total_number() as usize
    }

    pub fn initialized() -> Self {
        use crate::libcalls::*;

        let mut ptrs = [0; Self::len()];

        ptrs[VMBuiltinFunctionIndex::get_memory32_grow_index().index() as usize] =
            wasmer_vm_memory32_grow as usize;
        ptrs[VMBuiltinFunctionIndex::get_imported_memory32_grow_index().index() as usize] =
            wasmer_vm_imported_memory32_grow as usize;

        ptrs[VMBuiltinFunctionIndex::get_memory32_size_index().index() as usize] =
            wasmer_vm_memory32_size as usize;
        ptrs[VMBuiltinFunctionIndex::get_imported_memory32_size_index().index() as usize] =
            wasmer_vm_imported_memory32_size as usize;

        ptrs[VMBuiltinFunctionIndex::get_table_copy_index().index() as usize] =
            wasmer_vm_table_copy as usize;

        ptrs[VMBuiltinFunctionIndex::get_table_init_index().index() as usize] =
            wasmer_vm_table_init as usize;
        ptrs[VMBuiltinFunctionIndex::get_elem_drop_index().index() as usize] =
            wasmer_vm_elem_drop as usize;

        ptrs[VMBuiltinFunctionIndex::get_memory_copy_index().index() as usize] =
            wasmer_vm_memory32_copy as usize;
        ptrs[VMBuiltinFunctionIndex::get_imported_memory_copy_index().index() as usize] =
            wasmer_vm_imported_memory32_copy as usize;
        ptrs[VMBuiltinFunctionIndex::get_memory_fill_index().index() as usize] =
            wasmer_vm_memory32_fill as usize;
        ptrs[VMBuiltinFunctionIndex::get_imported_memory_fill_index().index() as usize] =
            wasmer_vm_imported_memory32_fill as usize;
        ptrs[VMBuiltinFunctionIndex::get_memory_init_index().index() as usize] =
            wasmer_vm_memory32_init as usize;
        ptrs[VMBuiltinFunctionIndex::get_data_drop_index().index() as usize] =
            wasmer_vm_data_drop as usize;
        ptrs[VMBuiltinFunctionIndex::get_raise_trap_index().index() as usize] =
            wasmer_vm_raise_trap as usize;
        ptrs[VMBuiltinFunctionIndex::get_table_size_index().index() as usize] =
            wasmer_vm_table_size as usize;
        ptrs[VMBuiltinFunctionIndex::get_imported_table_size_index().index() as usize] =
            wasmer_vm_imported_table_size as usize;
        ptrs[VMBuiltinFunctionIndex::get_table_grow_index().index() as usize] =
            wasmer_vm_table_grow as usize;
        ptrs[VMBuiltinFunctionIndex::get_imported_table_grow_index().index() as usize] =
            wasmer_vm_imported_table_grow as usize;
        ptrs[VMBuiltinFunctionIndex::get_table_get_index().index() as usize] =
            wasmer_vm_table_get as usize;
        ptrs[VMBuiltinFunctionIndex::get_imported_table_get_index().index() as usize] =
            wasmer_vm_imported_table_get as usize;
        ptrs[VMBuiltinFunctionIndex::get_table_set_index().index() as usize] =
            wasmer_vm_table_set as usize;
        ptrs[VMBuiltinFunctionIndex::get_imported_table_set_index().index() as usize] =
            wasmer_vm_imported_table_set as usize;
        ptrs[VMBuiltinFunctionIndex::get_func_ref_index().index() as usize] =
            wasmer_vm_func_ref as usize;
        ptrs[VMBuiltinFunctionIndex::get_table_fill_index().index() as usize] =
            wasmer_vm_table_fill as usize;

        ptrs[VMBuiltinFunctionIndex::get_memory_atomic_wait32_index().index() as usize] =
            wasmer_vm_memory32_atomic_wait32 as usize;
        ptrs[VMBuiltinFunctionIndex::get_imported_memory_atomic_wait32_index().index() as usize] =
            wasmer_vm_imported_memory32_atomic_wait32 as usize;
        ptrs[VMBuiltinFunctionIndex::get_memory_atomic_wait64_index().index() as usize] =
            wasmer_vm_memory32_atomic_wait64 as usize;
        ptrs[VMBuiltinFunctionIndex::get_imported_memory_atomic_wait64_index().index() as usize] =
            wasmer_vm_imported_memory32_atomic_wait64 as usize;
        ptrs[VMBuiltinFunctionIndex::get_memory_atomic_notify_index().index() as usize] =
            wasmer_vm_memory32_atomic_notify as usize;
        ptrs[VMBuiltinFunctionIndex::get_imported_memory_atomic_notify_index().index() as usize] =
            wasmer_vm_imported_memory32_atomic_notify as usize;

        debug_assert!(ptrs.iter().cloned().all(|p| p != 0));

        Self { ptrs }
    }
}

/// The VM "context", which is pointed to by the `vmctx` arg in the compiler.
/// This has information about globals, memories, tables, and other runtime
/// state associated with the current instance.
///
/// The struct here is empty, as the sizes of these fields are dynamic, and
/// we can't describe them in Rust's type system. Sufficient memory is
/// allocated at runtime.
///
/// TODO: We could move the globals into the `vmctx` allocation too.
#[derive(Debug)]
#[repr(C, align(16))] // align 16 since globals are aligned to that and contained inside
pub struct VMContext {}

impl VMContext {
    /// Return a mutable reference to the associated `Instance`.
    ///
    /// # Safety
    /// This is unsafe because it doesn't work on just any `VMContext`, it must
    /// be a `VMContext` allocated as part of an `Instance`.
    #[allow(clippy::cast_ptr_alignment)]
    #[inline]
    pub(crate) unsafe fn instance(&self) -> &Instance {
        &*((self as *const Self as *mut u8).offset(-Instance::vmctx_offset()) as *const Instance)
    }

    #[inline]
    pub(crate) unsafe fn instance_mut(&mut self) -> &mut Instance {
        &mut *((self as *const Self as *mut u8).offset(-Instance::vmctx_offset()) as *mut Instance)
    }
}

///
pub type VMTrampoline = unsafe extern "C" fn(
    *mut VMContext,        // callee vmctx
    *const VMFunctionBody, // function we're actually calling
    *mut RawValue,         // space for arguments and return values
);

/// The fields compiled code needs to access to utilize a WebAssembly linear
/// memory defined within the instance, namely the start address and the
/// size in bytes.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct VMMemoryDefinition {
    /// The start address which is always valid, even if the memory grows.
    pub base: *mut u8,

    /// The current logical size of this linear memory in bytes.
    pub current_length: usize,
}

/// # Safety
/// This data is safe to share between threads because it's plain data that
/// is the user's responsibility to synchronize.
unsafe impl Send for VMMemoryDefinition {}
/// # Safety
/// This data is safe to share between threads because it's plain data that
/// is the user's responsibility to synchronize. And it's `Copy` so there's
/// really no difference between passing it by reference or by value as far as
/// correctness in a multi-threaded context is concerned.
unsafe impl Sync for VMMemoryDefinition {}

#[cfg(test)]
mod test_vmmemory_definition {
    use super::VMMemoryDefinition;
    use crate::VMOffsets;
    use memoffset::offset_of;
    use std::mem::size_of;
    use wasmer_types::ModuleInfo;

    #[test]
    fn check_vmmemory_definition_offsets() {
        let module = ModuleInfo::new();
        let offsets = VMOffsets::new(size_of::<*mut u8>() as u8, &module);
        assert_eq!(
            size_of::<VMMemoryDefinition>(),
            usize::from(offsets.size_of_vmmemory_definition())
        );
        assert_eq!(
            offset_of!(VMMemoryDefinition, base),
            usize::from(offsets.vmmemory_definition_base())
        );
        assert_eq!(
            offset_of!(VMMemoryDefinition, current_length),
            usize::from(offsets.vmmemory_definition_current_length())
        );
    }
}
