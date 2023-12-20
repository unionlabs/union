use crate::js::vm::{VMExtern, VMMemory};
use crate::mem_access::MemoryAccessError;
use crate::store::{AsStoreMut, AsStoreRef, StoreObjects};
use crate::MemoryType;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::slice;
#[cfg(feature = "tracing")]
use tracing::warn;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasmer_types::Pages;

use super::memory_view::MemoryView;

pub use wasmer_types::MemoryError;

#[wasm_bindgen]
extern "C" {
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/Memory)
    #[wasm_bindgen(js_namespace = WebAssembly, extends = js_sys::Object, typescript_type = "WebAssembly.Memory")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type JSMemory;

    /// The `grow()` protoype method of the `Memory` object increases the
    /// size of the memory instance by a specified number of WebAssembly
    /// pages.
    ///
    /// Takes the number of pages to grow (64KiB in size) and returns the
    /// previous size of memory, in pages.
    ///
    /// # Reimplementation
    ///
    /// We re-implement `WebAssembly.Memory.grow` because it is
    /// different from what `wasm-bindgen` declares. It marks the function
    /// as `catch`, which means it can throw an exception.
    ///
    /// See [the opened patch](https://github.com/rustwasm/wasm-bindgen/pull/2599).
    ///
    /// # Exceptions
    ///
    /// A `RangeError` is thrown if adding pages would exceed the maximum
    /// memory.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/Memory/grow)
    #[wasm_bindgen(catch, method, js_namespace = WebAssembly)]
    pub fn grow(this: &JSMemory, pages: u32) -> Result<u32, JsValue>;
}

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
        let vm_memory = VMMemory::new(Self::js_memory_from_type(&ty)?, ty);
        Ok(Self::from_vm_extern(store, vm_memory))
    }

    pub(crate) fn js_memory_from_type(
        ty: &MemoryType,
    ) -> Result<js_sys::WebAssembly::Memory, MemoryError> {
        let descriptor = js_sys::Object::new();
        // Annotation is here to prevent spurious IDE warnings.
        #[allow(unused_unsafe)]
        unsafe {
            js_sys::Reflect::set(&descriptor, &"initial".into(), &ty.minimum.0.into()).unwrap();
            if let Some(max) = ty.maximum {
                js_sys::Reflect::set(&descriptor, &"maximum".into(), &max.0.into()).unwrap();
            }
            js_sys::Reflect::set(&descriptor, &"shared".into(), &ty.shared.into()).unwrap();
        }

        let js_memory = js_sys::WebAssembly::Memory::new(&descriptor)
            .map_err(|_e| MemoryError::Generic("Error while creating the memory".to_owned()))?;

        Ok(js_memory)
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

    /// Creates a view into the memory that then allows for
    /// read and write
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
        let js_memory = &self.handle.memory;
        let our_js_memory: &JSMemory = JsCast::unchecked_from_js_ref(js_memory);
        let new_pages = our_js_memory.grow(pages.0).map_err(|err| {
            if err.is_instance_of::<js_sys::RangeError>() {
                MemoryError::CouldNotGrow {
                    current: self.view(&store.as_store_ref()).size(),
                    attempted_delta: pages,
                }
            } else {
                MemoryError::Generic(err.as_string().unwrap())
            }
        })?;
        Ok(Pages(new_pages))
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
        cloned.copy()
    }

    pub fn is_from_store(&self, _store: &impl AsStoreRef) -> bool {
        true
    }
}

impl std::cmp::PartialEq for Memory {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

/// Underlying buffer for a memory.
#[derive(Copy, Clone, Debug)]
pub(crate) struct MemoryBuffer<'a> {
    pub(crate) base: *mut js_sys::Uint8Array,
    pub(crate) marker: PhantomData<(&'a Memory, &'a StoreObjects)>,
}

impl<'a> MemoryBuffer<'a> {
    pub(crate) fn read(&self, offset: u64, buf: &mut [u8]) -> Result<(), MemoryAccessError> {
        let end = offset
            .checked_add(buf.len() as u64)
            .ok_or(MemoryAccessError::Overflow)?;
        let view = unsafe { &*(self.base) };
        if end > view.length().into() {
            #[cfg(feature = "tracing")]
            warn!(
                "attempted to read ({} bytes) beyond the bounds of the memory view ({} > {})",
                buf.len(),
                end,
                view.length()
            );
            return Err(MemoryAccessError::HeapOutOfBounds);
        }
        view.subarray(offset as _, end as _)
            .copy_to(unsafe { &mut slice::from_raw_parts_mut(buf.as_mut_ptr(), buf.len()) });
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
        let view = unsafe { &*(self.base) };
        if end > view.length().into() {
            #[cfg(feature = "tracing")]
            warn!(
                "attempted to read ({} bytes) beyond the bounds of the memory view ({} > {})",
                buf.len(),
                end,
                view.length()
            );
            return Err(MemoryAccessError::HeapOutOfBounds);
        }
        let buf_ptr = buf.as_mut_ptr() as *mut u8;
        view.subarray(offset as _, end as _)
            .copy_to(unsafe { &mut slice::from_raw_parts_mut(buf_ptr, buf.len()) });

        Ok(unsafe { slice::from_raw_parts_mut(buf_ptr, buf.len()) })
    }

    pub(crate) fn write(&self, offset: u64, data: &[u8]) -> Result<(), MemoryAccessError> {
        let end = offset
            .checked_add(data.len() as u64)
            .ok_or(MemoryAccessError::Overflow)?;
        let view = unsafe { &mut *(self.base) };
        if end > view.length().into() {
            #[cfg(feature = "tracing")]
            warn!(
                "attempted to write ({} bytes) beyond the bounds of the memory view ({} > {})",
                data.len(),
                end,
                view.length()
            );
            return Err(MemoryAccessError::HeapOutOfBounds);
        }
        view.subarray(offset as _, end as _).copy_from(data);

        Ok(())
    }
}
