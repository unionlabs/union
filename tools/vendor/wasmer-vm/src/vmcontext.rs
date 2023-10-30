// This file contains code from external sources.
// Attributions: https://github.com/wasmerio/wasmer/blob/master/ATTRIBUTIONS.md

//! This file declares `VMContext` and several related structs which contain
//! fields that compiled wasm code accesses directly.

use crate::func_data_registry::VMFuncRef;
use crate::global::Global;
use crate::instance::Instance;
use crate::memory::Memory;
use crate::table::Table;
use crate::trap::{Trap, TrapCode};
use crate::VMBuiltinFunctionIndex;
use crate::VMExternRef;
use loupe::{MemoryUsage, MemoryUsageTracker, POINTER_BYTE_SIZE};
use std::any::Any;
use std::convert::TryFrom;
use std::fmt;
use std::mem;
use std::ptr::{self, NonNull};
use std::sync::Arc;
use std::u32;
pub use wasmer_artifact::VMFunctionBody;

/// Union representing the first parameter passed when calling a function.
///
/// It may either be a pointer to the [`VMContext`] if it's a Wasm function
/// or a pointer to arbitrary data controlled by the host if it's a host function.
#[derive(Copy, Clone, Eq)]
pub union VMFunctionEnvironment {
    /// Wasm functions take a pointer to [`VMContext`].
    pub vmctx: *mut VMContext,
    /// Host functions can have custom environments.
    pub host_env: *mut std::ffi::c_void,
}

impl VMFunctionEnvironment {
    /// Check whether the pointer stored is null or not.
    pub fn is_null(&self) -> bool {
        unsafe { self.host_env.is_null() }
    }
}

impl std::fmt::Debug for VMFunctionEnvironment {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("VMFunctionEnvironment")
            .field("vmctx_or_hostenv", unsafe { &self.host_env })
            .finish()
    }
}

impl std::cmp::PartialEq for VMFunctionEnvironment {
    fn eq(&self, rhs: &Self) -> bool {
        unsafe { self.host_env as usize == rhs.host_env as usize }
    }
}

impl std::hash::Hash for VMFunctionEnvironment {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe {
            self.vmctx.hash(state);
        }
    }
}

impl MemoryUsage for VMFunctionEnvironment {
    fn size_of_val(&self, _: &mut dyn MemoryUsageTracker) -> usize {
        mem::size_of_val(self)
    }
}

/// An imported function.
#[derive(Debug, Copy, Clone, MemoryUsage)]
#[repr(C)]
pub struct VMFunctionImport {
    /// A pointer to the imported function body.
    pub body: *const VMFunctionBody,

    /// A pointer to the `VMContext` that owns the function or host env data.
    pub environment: VMFunctionEnvironment,
}

#[cfg(test)]
mod test_vmfunction_import {
    use super::VMFunctionImport;
    use crate::VMOffsets;
    use memoffset::offset_of;
    use std::mem::size_of;
    use wasmer_types::ModuleInfo;

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
pub struct VMDynamicFunctionContext<T: Sized + Send + Sync> {
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
#[derive(Debug, Copy, Clone, PartialEq, MemoryUsage)]
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
#[derive(Debug, Clone, MemoryUsage)]
#[repr(C)]
pub struct VMTableImport {
    /// A pointer to the imported table description.
    pub definition: NonNull<VMTableDefinition>,

    /// A pointer to the `Table` that owns the table description.
    pub from: Arc<dyn Table>,
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
        assert_eq!(
            offset_of!(VMTableImport, from),
            usize::from(offsets.vmtable_import_from())
        );
    }
}

/// The fields compiled code needs to access to utilize a WebAssembly linear
/// memory imported from another instance.
#[derive(Debug, Clone, MemoryUsage)]
#[repr(C)]
pub struct VMMemoryImport {
    /// A pointer to the imported memory description.
    pub definition: NonNull<VMMemoryDefinition>,

    /// A pointer to the `Memory` that owns the memory description.
    pub from: Arc<dyn Memory>,
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
            offset_of!(VMMemoryImport, from),
            usize::from(offsets.vmmemory_import_from())
        );
    }
}

/// The fields compiled code needs to access to utilize a WebAssembly global
/// variable imported from another instance.
#[derive(Debug, Clone, MemoryUsage)]
#[repr(C)]
pub struct VMGlobalImport {
    /// A pointer to the imported global variable description.
    pub definition: NonNull<VMGlobalDefinition>,

    /// A pointer to the `Global` that owns the global description.
    pub from: Arc<Global>,
}

/// # Safety
/// This data is safe to share between threads because it's plain data that
/// is the user's responsibility to synchronize. Additionally, all operations
/// on `from` are thread-safe through the use of a mutex in [`Global`].
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
        assert_eq!(
            offset_of!(VMGlobalImport, from),
            usize::from(offsets.vmglobal_import_from())
        );
    }
}

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

impl MemoryUsage for VMMemoryDefinition {
    fn size_of_val(&self, tracker: &mut dyn MemoryUsageTracker) -> usize {
        if tracker.track(self.base as *const _ as *const ()) {
            POINTER_BYTE_SIZE * self.current_length
        } else {
            0
        }
    }
}

impl VMMemoryDefinition {
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
    pub(crate) unsafe fn memory_copy(&self, dst: u32, src: u32, len: u32) -> Result<(), Trap> {
        // https://webassembly.github.io/reference-types/core/exec/instructions.html#exec-memory-copy
        if src
            .checked_add(len)
            .map_or(true, |n| usize::try_from(n).unwrap() > self.current_length)
            || dst
                .checked_add(len)
                .map_or(true, |m| usize::try_from(m).unwrap() > self.current_length)
        {
            return Err(Trap::lib(TrapCode::HeapAccessOutOfBounds));
        }

        let dst = usize::try_from(dst).unwrap();
        let src = usize::try_from(src).unwrap();

        // Bounds and casts are checked above, by this point we know that
        // everything is safe.
        let dst = self.base.add(dst);
        let src = self.base.add(src);
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
    pub(crate) unsafe fn memory_fill(&self, dst: u32, val: u32, len: u32) -> Result<(), Trap> {
        if dst
            .checked_add(len)
            .map_or(true, |m| usize::try_from(m).unwrap() > self.current_length)
        {
            return Err(Trap::lib(TrapCode::HeapAccessOutOfBounds));
        }

        let dst = isize::try_from(dst).unwrap();
        let val = val as u8;

        // Bounds and casts are checked above, by this point we know that
        // everything is safe.
        let dst = self.base.offset(dst);
        ptr::write_bytes(dst, val, len as usize);

        Ok(())
    }
}

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

impl MemoryUsage for VMTableDefinition {
    fn size_of_val(&self, tracker: &mut dyn MemoryUsageTracker) -> usize {
        if tracker.track(self.base as *const _ as *const ()) {
            POINTER_BYTE_SIZE * (self.current_elements as usize)
        } else {
            0
        }
    }
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

/// A typesafe wrapper around the storage for a global variables.
///
/// # Safety
///
/// Accessing the different members of this union is always safe because there
/// are no invalid values for any of the types and the whole object is
/// initialized by VMGlobalDefinition::new().
#[derive(Clone, Copy)]
#[repr(C, align(16))]
pub union VMGlobalDefinitionStorage {
    as_i32: i32,
    as_u32: u32,
    as_f32: f32,
    as_i64: i64,
    as_u64: u64,
    as_f64: f64,
    as_u128: u128,
    as_funcref: VMFuncRef,
    as_externref: VMExternRef,
    bytes: [u8; 16],
}

impl fmt::Debug for VMGlobalDefinitionStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VMGlobalDefinitionStorage")
            .field("bytes", unsafe { &self.bytes })
            .finish()
    }
}

impl MemoryUsage for VMGlobalDefinitionStorage {
    fn size_of_val(&self, _: &mut dyn MemoryUsageTracker) -> usize {
        mem::size_of_val(self)
    }
}

/// The storage for a WebAssembly global defined within the instance.
///
/// TODO: Pack the globals more densely, rather than using the same size
/// for every type.
#[derive(Debug, Clone, MemoryUsage)]
#[repr(C, align(16))]
pub struct VMGlobalDefinition {
    storage: VMGlobalDefinitionStorage,
    // If more elements are added here, remember to add offset_of tests below!
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
            storage: VMGlobalDefinitionStorage { bytes: [0; 16] },
        }
    }

    /// Return the value as an i32.
    ///
    /// If this is not an I32 typed global it is unspecified what value is returned.
    pub fn to_i32(&self) -> i32 {
        unsafe { self.storage.as_i32 }
    }

    /// Return a mutable reference to the value as an i32.
    ///
    /// # Safety
    ///
    /// It is the callers responsibility to make sure the global has I32 type.
    /// Until the returned borrow is dropped, reads and writes of this global
    /// must be done exclusively through this borrow. That includes reads and
    /// writes of globals inside wasm functions.
    pub unsafe fn as_i32_mut(&mut self) -> &mut i32 {
        &mut self.storage.as_i32
    }

    /// Return a reference to the value as an u32.
    ///
    /// If this is not an I32 typed global it is unspecified what value is returned.
    pub fn to_u32(&self) -> u32 {
        unsafe { self.storage.as_u32 }
    }

    /// Return a mutable reference to the value as an u32.
    ///
    /// # Safety
    ///
    /// It is the callers responsibility to make sure the global has I32 type.
    /// Until the returned borrow is dropped, reads and writes of this global
    /// must be done exclusively through this borrow. That includes reads and
    /// writes of globals inside wasm functions.
    pub unsafe fn as_u32_mut(&mut self) -> &mut u32 {
        &mut self.storage.as_u32
    }

    /// Return a reference to the value as an i64.
    ///
    /// If this is not an I64 typed global it is unspecified what value is returned.
    pub fn to_i64(&self) -> i64 {
        unsafe { self.storage.as_i64 }
    }

    /// Return a mutable reference to the value as an i64.
    ///
    /// # Safety
    ///
    /// It is the callers responsibility to make sure the global has I32 type.
    /// Until the returned borrow is dropped, reads and writes of this global
    /// must be done exclusively through this borrow. That includes reads and
    /// writes of globals inside wasm functions.
    pub unsafe fn as_i64_mut(&mut self) -> &mut i64 {
        &mut self.storage.as_i64
    }

    /// Return a reference to the value as an u64.
    ///
    /// If this is not an I64 typed global it is unspecified what value is returned.
    pub fn to_u64(&self) -> u64 {
        unsafe { self.storage.as_u64 }
    }

    /// Return a mutable reference to the value as an u64.
    ///
    /// # Safety
    ///
    /// It is the callers responsibility to make sure the global has I64 type.
    /// Until the returned borrow is dropped, reads and writes of this global
    /// must be done exclusively through this borrow. That includes reads and
    /// writes of globals inside wasm functions.
    pub unsafe fn as_u64_mut(&mut self) -> &mut u64 {
        &mut self.storage.as_u64
    }

    /// Return a reference to the value as an f32.
    ///
    /// If this is not an F32 typed global it is unspecified what value is returned.
    pub fn to_f32(&self) -> f32 {
        unsafe { self.storage.as_f32 }
    }

    /// Return a mutable reference to the value as an f32.
    ///
    /// # Safety
    ///
    /// It is the callers responsibility to make sure the global has F32 type.
    /// Until the returned borrow is dropped, reads and writes of this global
    /// must be done exclusively through this borrow. That includes reads and
    /// writes of globals inside wasm functions.
    pub unsafe fn as_f32_mut(&mut self) -> &mut f32 {
        &mut self.storage.as_f32
    }

    /// Return a reference to the value as an f64.
    ///
    /// If this is not an F64 typed global it is unspecified what value is returned.
    pub fn to_f64(&self) -> f64 {
        unsafe { self.storage.as_f64 }
    }

    /// Return a mutable reference to the value as an f64.
    ///
    /// # Safety
    ///
    /// It is the callers responsibility to make sure the global has F64 type.
    /// Until the returned borrow is dropped, reads and writes of this global
    /// must be done exclusively through this borrow. That includes reads and
    /// writes of globals inside wasm functions.
    pub unsafe fn as_f64_mut(&mut self) -> &mut f64 {
        &mut self.storage.as_f64
    }

    /// Return a reference to the value as a `VMFuncRef`.
    ///
    /// If this is not a `VMFuncRef` typed global it is unspecified what value is returned.
    pub fn to_funcref(&self) -> VMFuncRef {
        unsafe { self.storage.as_funcref }
    }

    /// Return a mutable reference to the value as a `VMFuncRef`.
    ///
    /// # Safety
    ///
    /// It is the callers responsibility to make sure the global has `VMFuncRef` type.
    /// Until the returned borrow is dropped, reads and writes of this global
    /// must be done exclusively through this borrow. That includes reads and
    /// writes of globals inside wasm functions.
    pub unsafe fn as_funcref_mut(&mut self) -> &mut VMFuncRef {
        &mut self.storage.as_funcref
    }

    /// Return a mutable reference to the value as an `VMExternRef`.
    ///
    /// # Safety
    ///
    /// It is the callers responsibility to make sure the global has I32 type.
    /// Until the returned borrow is dropped, reads and writes of this global
    /// must be done exclusively through this borrow. That includes reads and
    /// writes of globals inside wasm functions.
    pub unsafe fn as_externref_mut(&mut self) -> &mut VMExternRef {
        &mut self.storage.as_externref
    }

    /// Return a reference to the value as an `VMExternRef`.
    ///
    /// If this is not an I64 typed global it is unspecified what value is returned.
    pub fn to_externref(&self) -> VMExternRef {
        unsafe { self.storage.as_externref }
    }

    /// Return a reference to the value as an u128.
    ///
    /// If this is not an V128 typed global it is unspecified what value is returned.
    pub fn to_u128(&self) -> u128 {
        unsafe { self.storage.as_u128 }
    }

    /// Return a mutable reference to the value as an u128.
    ///
    /// # Safety
    ///
    /// It is the callers responsibility to make sure the global has V128 type.
    /// Until the returned borrow is dropped, reads and writes of this global
    /// must be done exclusively through this borrow. That includes reads and
    /// writes of globals inside wasm functions.
    pub unsafe fn as_u128_mut(&mut self) -> &mut u128 {
        &mut self.storage.as_u128
    }

    /// Return a reference to the value as bytes.
    pub fn to_bytes(&self) -> [u8; 16] {
        unsafe { self.storage.bytes }
    }

    /// Return a mutable reference to the value as bytes.
    ///
    /// # Safety
    ///
    /// Until the returned borrow is dropped, reads and writes of this global
    /// must be done exclusively through this borrow. That includes reads and
    /// writes of globals inside wasm functions.
    pub unsafe fn as_bytes_mut(&mut self) -> &mut [u8; 16] {
        &mut self.storage.bytes
    }
}

/// An index into the shared signature registry, usable for checking signatures
/// at indirect calls.
#[repr(C)]
#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, MemoryUsage)]
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
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, MemoryUsage)]
#[repr(C)]
pub struct VMCallerCheckedAnyfunc {
    /// Function body.
    pub func_ptr: *const VMFunctionBody,
    /// Function signature id.
    pub type_index: VMSharedSignatureIndex,
    /// Function `VMContext` or host env.
    pub vmctx: VMFunctionEnvironment,
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

impl Default for VMCallerCheckedAnyfunc {
    fn default() -> Self {
        Self {
            func_ptr: ptr::null_mut(),
            type_index: Default::default(),
            vmctx: VMFunctionEnvironment {
                vmctx: ptr::null_mut(),
            },
        }
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
        ptrs[VMBuiltinFunctionIndex::get_externref_inc_index().index() as usize] =
            wasmer_vm_externref_inc as usize;
        ptrs[VMBuiltinFunctionIndex::get_externref_dec_index().index() as usize] =
            wasmer_vm_externref_dec as usize;

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

    /// Return a reference to the host state associated with this `Instance`.
    ///
    /// # Safety
    /// This is unsafe because it doesn't work on just any `VMContext`, it must
    /// be a `VMContext` allocated as part of an `Instance`.
    #[inline]
    pub unsafe fn host_state(&self) -> &dyn Any {
        self.instance().host_state()
    }
}

///
pub type VMTrampoline = unsafe extern "C" fn(
    *mut VMContext,        // callee vmctx
    *const VMFunctionBody, // function we're actually calling
    *mut u128,             // space for arguments and return values
);
