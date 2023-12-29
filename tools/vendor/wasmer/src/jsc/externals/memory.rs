use crate::jsc::vm::{VMExtern, VMMemory};
use crate::mem_access::MemoryAccessError;
use crate::store::{AsStoreMut, AsStoreRef, StoreObjects};
use crate::MemoryType;
use rusty_jsc::{JSObject, JSValue};
use std::convert::TryInto;
use std::marker::PhantomData;
use std::mem::{self, MaybeUninit};
use std::slice;
#[cfg(feature = "tracing")]
use tracing::warn;

use wasmer_types::{Pages, WASM_PAGE_SIZE};

use super::memory_view::MemoryView;

pub use wasmer_types::MemoryError;

#[derive(Debug, Clone)]
pub struct Memory {
    pub(crate) handle: VMMemory,
}

// Only SharedMemories can be Send in js, becuase they support `structuredClone`.
// Normal memories will fail while doing structuredClone.
// In this case, we implement Send just in case as it can be a shared memory.
// https://developer.mozilla.org/en-US/docs/Web/API/structuredClone
// ```js
// const memory = new WebAssembly.Memory({
//   initial: 10,
//   maximum: 100,
//   shared: true // <--- It must be shared, otherwise structuredClone will fail
// });
// structuredClone(memory)
// ```
unsafe impl Send for Memory {}
unsafe impl Sync for Memory {}

impl Memory {
    pub fn new(store: &mut impl AsStoreMut, ty: MemoryType) -> Result<Self, MemoryError> {
        let vm_memory = VMMemory::new(Self::js_memory_from_type(store, &ty)?, ty);
        Ok(Self::from_vm_extern(store, vm_memory))
    }

    pub(crate) fn js_memory_from_type(
        store: &impl AsStoreRef,
        ty: &MemoryType,
    ) -> Result<JSObject, MemoryError> {
        let store_ref = store.as_store_ref();
        let engine = store_ref.engine();
        let context = engine.0.context();

        let mut descriptor = JSObject::new(&context);
        descriptor.set_property(
            &context,
            "initial".to_string(),
            JSValue::number(&context, ty.minimum.0.into()),
        );
        if let Some(max) = ty.maximum {
            descriptor.set_property(
                &context,
                "maximum".to_string(),
                JSValue::number(&context, max.0.into()),
            );
        }
        descriptor.set_property(
            &context,
            "shared".to_string(),
            JSValue::boolean(&context, ty.shared),
        );

        engine
            .0
            .wasm_memory_type()
            .construct(&context, &[descriptor.to_jsvalue()])
            .map_err(|e| MemoryError::Generic(format!("{:?}", e)))
    }

    pub fn new_from_existing(new_store: &mut impl AsStoreMut, memory: VMMemory) -> Self {
        Self::from_vm_extern(new_store, memory)
    }

    pub(crate) fn to_vm_extern(&self) -> VMExtern {
        VMExtern::Memory(self.handle.clone())
    }

    pub fn ty(&self, _store: &impl AsStoreRef) -> MemoryType {
        self.handle.ty
    }

    pub fn view<'a>(&self, store: &'a impl AsStoreRef) -> MemoryView<'a> {
        MemoryView::new(self, store)
    }

    pub fn grow<IntoPages>(
        &self,
        store: &mut impl AsStoreMut,
        delta: IntoPages,
    ) -> Result<Pages, MemoryError>
    where
        IntoPages: Into<Pages>,
    {
        let pages = delta.into();

        let store_mut = store.as_store_mut();
        let engine = store_mut.engine();
        let context = engine.0.context();
        let func = self
            .handle
            .memory
            .get_property(&context, "grow".to_string())
            .to_object(&context)
            .unwrap();
        match func.call(
            &context,
            Some(&self.handle.memory),
            &[JSValue::number(&context, pages.0 as _)],
        ) {
            Ok(val) => Ok(Pages(val.to_number(&context).unwrap() as _)),
            Err(e) => {
                let old_pages = pages;
                Err(MemoryError::CouldNotGrow {
                    current: old_pages,
                    attempted_delta: pages,
                })
            }
        }
        // let js_memory = &self.handle.memory;
        // let our_js_memory: &JSMemory = JsCast::unchecked_from_js_ref(js_memory);
        // let new_pages = our_js_memory.grow(pages.0).map_err(|err| {
        //     if err.is_instance_of::<js_sys::RangeError>() {
        //         MemoryError::CouldNotGrow {
        //             current: self.view(&store.as_store_ref()).size(),
        //             attempted_delta: pages,
        //         }
        //     } else {
        //         MemoryError::Generic(err.as_string().unwrap())
        //     }
        // })?;
        // Ok(Pages(new_pages))
    }

    pub fn copy_to_store(
        &self,
        store: &impl AsStoreRef,
        new_store: &mut impl AsStoreMut,
    ) -> Result<Self, MemoryError> {
        let view = self.view(store);
        let ty = self.ty(store);
        let amount = view.data_size() as usize;

        let new_memory = Self::new(new_store, ty)?;
        let mut new_view = new_memory.view(&new_store);
        let new_view_size = new_view.data_size() as usize;
        if amount > new_view_size {
            let delta = amount - new_view_size;
            let pages = ((delta - 1) / wasmer_types::WASM_PAGE_SIZE) + 1;
            new_memory.grow(new_store, Pages(pages as u32))?;
            new_view = new_memory.view(&new_store);
        }

        // Copy the bytes
        view.copy_to_memory(amount as u64, &new_view)
            .map_err(|err| MemoryError::Generic(err.to_string()))?;
        // // Return the new memory
        Ok(new_memory)
    }

    pub(crate) fn from_vm_extern(_store: &mut impl AsStoreMut, internal: VMMemory) -> Self {
        Self { handle: internal }
    }

    /// Cloning memory will create another reference to the same memory that
    /// can be put into a new store
    pub fn try_clone(&self, _store: &impl AsStoreRef) -> Result<VMMemory, MemoryError> {
        self.handle.try_clone()
    }

    /// Copying the memory will actually copy all the bytes in the memory to
    /// a identical byte copy of the original that can be put into a new store
    pub fn try_copy(&self, store: &impl AsStoreRef) -> Result<VMMemory, MemoryError> {
        let mut cloned = self.try_clone(store)?;
        cloned.copy(store)
    }

    pub fn is_from_store(&self, _store: &impl AsStoreRef) -> bool {
        true
    }

    #[allow(unused)]
    pub fn duplicate(&mut self, store: &impl AsStoreRef) -> Result<VMMemory, MemoryError> {
        self.handle.duplicate(store)
    }
}

impl std::cmp::PartialEq for Memory {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

impl std::cmp::Eq for Memory {}

/// Underlying buffer for a memory.
#[derive(Debug, Copy, Clone)]
pub(crate) struct MemoryBuffer<'a> {
    pub(crate) base: *mut u8,
    pub(crate) len: usize,
    pub(crate) marker: PhantomData<&'a MemoryView<'a>>,
}

impl<'a> MemoryBuffer<'a> {
    pub(crate) fn read(&self, offset: u64, buf: &mut [u8]) -> Result<(), MemoryAccessError> {
        let end = offset
            .checked_add(buf.len() as u64)
            .ok_or(MemoryAccessError::Overflow)?;
        if end > self.len.try_into().unwrap() {
            #[cfg(feature = "tracing")]
            warn!(
                "attempted to read ({} bytes) beyond the bounds of the memory view ({} > {})",
                buf.len(),
                end,
                self.len
            );
            return Err(MemoryAccessError::HeapOutOfBounds);
        }
        unsafe {
            volatile_memcpy_read(self.base.add(offset as usize), buf.as_mut_ptr(), buf.len());
        }
        Ok(())
    }

    pub(crate) fn read_uninit<'b>(
        &self,
        offset: u64,
        buf: &'b mut [MaybeUninit<u8>],
    ) -> Result<&'b mut [u8], MemoryAccessError> {
        let end = offset
            .checked_add(buf.len() as u64)
            .ok_or(MemoryAccessError::Overflow)?;
        if end > self.len.try_into().unwrap() {
            #[cfg(feature = "tracing")]
            warn!(
                "attempted to read ({} bytes) beyond the bounds of the memory view ({} > {})",
                buf.len(),
                end,
                self.len
            );
            return Err(MemoryAccessError::HeapOutOfBounds);
        }
        let buf_ptr = buf.as_mut_ptr() as *mut u8;
        unsafe {
            volatile_memcpy_read(self.base.add(offset as usize), buf_ptr, buf.len());
        }

        Ok(unsafe { slice::from_raw_parts_mut(buf_ptr, buf.len()) })
    }

    pub(crate) fn write(&self, offset: u64, data: &[u8]) -> Result<(), MemoryAccessError> {
        let end = offset
            .checked_add(data.len() as u64)
            .ok_or(MemoryAccessError::Overflow)?;
        if end > self.len.try_into().unwrap() {
            #[cfg(feature = "tracing")]
            warn!(
                "attempted to write ({} bytes) beyond the bounds of the memory view ({} > {})",
                data.len(),
                end,
                self.len
            );
            return Err(MemoryAccessError::HeapOutOfBounds);
        }
        unsafe {
            volatile_memcpy_write(data.as_ptr(), self.base.add(offset as usize), data.len());
        }
        Ok(())
    }
}

// We can't use a normal memcpy here because it has undefined behavior if the
// memory is being concurrently modified. So we need to write our own memcpy
// implementation which uses volatile operations.
//
// The implementation of these functions can optimize very well when inlined
// with a fixed length: they should compile down to a single load/store
// instruction for small (8/16/32/64-bit) copies.
#[inline]
unsafe fn volatile_memcpy_read(mut src: *const u8, mut dst: *mut u8, mut len: usize) {
    #[inline]
    unsafe fn copy_one<T>(src: &mut *const u8, dst: &mut *mut u8, len: &mut usize) {
        #[repr(packed)]
        struct Unaligned<T>(T);
        let val = (*src as *const Unaligned<T>).read_volatile();
        (*dst as *mut Unaligned<T>).write(val);
        *src = src.add(mem::size_of::<T>());
        *dst = dst.add(mem::size_of::<T>());
        *len -= mem::size_of::<T>();
    }

    while len >= 8 {
        copy_one::<u64>(&mut src, &mut dst, &mut len);
    }
    if len >= 4 {
        copy_one::<u32>(&mut src, &mut dst, &mut len);
    }
    if len >= 2 {
        copy_one::<u16>(&mut src, &mut dst, &mut len);
    }
    if len >= 1 {
        copy_one::<u8>(&mut src, &mut dst, &mut len);
    }
}
#[inline]
unsafe fn volatile_memcpy_write(mut src: *const u8, mut dst: *mut u8, mut len: usize) {
    #[inline]
    unsafe fn copy_one<T>(src: &mut *const u8, dst: &mut *mut u8, len: &mut usize) {
        #[repr(packed)]
        struct Unaligned<T>(T);
        let val = (*src as *const Unaligned<T>).read();
        (*dst as *mut Unaligned<T>).write_volatile(val);
        *src = src.add(mem::size_of::<T>());
        *dst = dst.add(mem::size_of::<T>());
        *len -= mem::size_of::<T>();
    }

    while len >= 8 {
        copy_one::<u64>(&mut src, &mut dst, &mut len);
    }
    if len >= 4 {
        copy_one::<u32>(&mut src, &mut dst, &mut len);
    }
    if len >= 2 {
        copy_one::<u16>(&mut src, &mut dst, &mut len);
    }
    if len >= 1 {
        copy_one::<u8>(&mut src, &mut dst, &mut len);
    }
}
