use crate::mem_access::MemoryAccessError;
use crate::store::AsStoreRef;
use std::convert::TryInto;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::slice;
#[cfg(feature = "tracing")]
use tracing::warn;
use wasm_bindgen::JsCast;

use wasmer_types::{Bytes, Pages};

use super::memory::{Memory, MemoryBuffer};

/// A WebAssembly `memory` view.
///
/// A memory view is used to read and write to the linear memory.
///
/// After a memory is grown a view must not be used anymore. Views are
/// created using the Memory.grow() method.
#[derive(Debug)]
pub struct MemoryView<'a> {
    view: js_sys::Uint8Array,
    size: u64,
    marker: PhantomData<&'a Memory>,
}

impl<'a> MemoryView<'a> {
    pub(crate) fn new(memory: &Memory, _store: &'a (impl AsStoreRef + ?Sized)) -> Self {
        Self::new_raw(&memory.handle.memory)
    }

    pub(crate) fn new_raw(memory: &js_sys::WebAssembly::Memory) -> Self {
        let buffer = memory.buffer();

        // This also works for SharedArrayBuffer.
        let size = buffer
            .unchecked_ref::<js_sys::ArrayBuffer>()
            .byte_length()
            .into();

        let view = js_sys::Uint8Array::new(&buffer);

        Self {
            view,
            size,
            marker: PhantomData,
        }
    }

    /// Returns the pointer to the raw bytes of the `Memory`.
    #[doc(hidden)]
    pub fn data_ptr(&self) -> *mut u8 {
        unimplemented!("direct data pointer access is not possible in JavaScript");
    }

    /// Returns the size (in bytes) of the `Memory`.
    pub fn data_size(&self) -> u64 {
        self.size
    }

    // TODO: do we want a proper implementation here instead?
    /// Retrieve a slice of the memory contents.
    ///
    /// # Safety
    ///
    /// Until the returned slice is dropped, it is undefined behaviour to
    /// modify the memory contents in any way including by calling a wasm
    /// function that writes to the memory or by resizing the memory.
    #[doc(hidden)]
    pub unsafe fn data_unchecked(&self) -> &[u8] {
        unimplemented!("direct data pointer access is not possible in JavaScript");
    }

    // TODO: do we want a proper implementation here instead?
    /// Retrieve a mutable slice of the memory contents.
    ///
    /// # Safety
    ///
    /// This method provides interior mutability without an UnsafeCell. Until
    /// the returned value is dropped, it is undefined behaviour to read or
    /// write to the pointed-to memory in any way except through this slice,
    /// including by calling a wasm function that reads the memory contents or
    /// by resizing this Memory.
    #[allow(clippy::mut_from_ref)]
    #[doc(hidden)]
    pub unsafe fn data_unchecked_mut(&self) -> &mut [u8] {
        unimplemented!("direct data pointer access is not possible in JavaScript");
    }

    /// Returns the size (in [`Pages`]) of the `Memory`.
    ///
    /// # Example
    ///
    /// ```
    /// # use wasmer::{Memory, MemoryType, Pages, Store, Type, Value};
    /// # let mut store = Store::default();
    /// #
    /// let m = Memory::new(&store, MemoryType::new(1, None, false)).unwrap();
    ///
    /// assert_eq!(m.size(), Pages(1));
    /// ```
    pub fn size(&self) -> Pages {
        Bytes(self.size as usize).try_into().unwrap()
    }

    #[inline]
    pub(crate) fn buffer(&self) -> MemoryBuffer<'a> {
        MemoryBuffer {
            base: &self.view as *const _ as *mut _,
            marker: PhantomData,
        }
    }

    /// Safely reads bytes from the memory at the given offset.
    ///
    /// The full buffer will be filled, otherwise a `MemoryAccessError` is returned
    /// to indicate an out-of-bounds access.
    ///
    /// This method is guaranteed to be safe (from the host side) in the face of
    /// concurrent writes.
    pub fn read(&self, offset: u64, data: &mut [u8]) -> Result<(), MemoryAccessError> {
        let view = &self.view;
        let offset: u32 = offset.try_into().map_err(|_| MemoryAccessError::Overflow)?;
        let len: u32 = data
            .len()
            .try_into()
            .map_err(|_| MemoryAccessError::Overflow)?;
        let end = offset.checked_add(len).ok_or(MemoryAccessError::Overflow)?;
        if end > view.length() {
            #[cfg(feature = "tracing")]
            warn!(
                "attempted to read ({} bytes) beyond the bounds of the memory view ({} > {})",
                len,
                end,
                view.length()
            );
            Err(MemoryAccessError::HeapOutOfBounds)?;
        }
        view.subarray(offset, end).copy_to(data);
        Ok(())
    }

    /// Safely reads a single byte from memory at the given offset
    ///
    /// This method is guaranteed to be safe (from the host side) in the face of
    /// concurrent writes.
    pub fn read_u8(&self, offset: u64) -> Result<u8, MemoryAccessError> {
        let view = &self.view;
        let offset: u32 = offset.try_into().map_err(|_| MemoryAccessError::Overflow)?;
        if offset >= view.length() {
            #[cfg(feature = "tracing")]
            warn!(
                "attempted to read beyond the bounds of the memory view ({} >= {})",
                offset,
                view.length()
            );
            Err(MemoryAccessError::HeapOutOfBounds)?;
        }
        Ok(view.get_index(offset))
    }

    /// Safely reads bytes from the memory at the given offset.
    ///
    /// This method is similar to `read` but allows reading into an
    /// uninitialized buffer. An initialized view of the buffer is returned.
    ///
    /// The full buffer will be filled, otherwise a `MemoryAccessError` is returned
    /// to indicate an out-of-bounds access.
    ///
    /// This method is guaranteed to be safe (from the host side) in the face of
    /// concurrent writes.
    pub fn read_uninit<'b>(
        &self,
        offset: u64,
        buf: &'b mut [MaybeUninit<u8>],
    ) -> Result<&'b mut [u8], MemoryAccessError> {
        let view = &self.view;
        let offset: u32 = offset.try_into().map_err(|_| MemoryAccessError::Overflow)?;
        let len: u32 = buf
            .len()
            .try_into()
            .map_err(|_| MemoryAccessError::Overflow)?;
        let end = offset.checked_add(len).ok_or(MemoryAccessError::Overflow)?;
        if end > view.length() {
            #[cfg(feature = "tracing")]
            warn!(
                "attempted to read ({} bytes) beyond the bounds of the memory view ({} > {})",
                len,
                end,
                view.length()
            );
            Err(MemoryAccessError::HeapOutOfBounds)?;
        }

        // Zero-initialize the buffer to avoid undefined behavior with
        // uninitialized data.
        for elem in buf.iter_mut() {
            *elem = MaybeUninit::new(0);
        }
        let buf = unsafe { slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, buf.len()) };

        view.subarray(offset, end).copy_to(buf);
        Ok(buf)
    }

    /// Safely writes bytes to the memory at the given offset.
    ///
    /// If the write exceeds the bounds of the memory then a `MemoryAccessError` is
    /// returned.
    ///
    /// This method is guaranteed to be safe (from the host side) in the face of
    /// concurrent reads/writes.
    pub fn write(&self, offset: u64, data: &[u8]) -> Result<(), MemoryAccessError> {
        let offset: u32 = offset.try_into().map_err(|_| MemoryAccessError::Overflow)?;
        let len: u32 = data
            .len()
            .try_into()
            .map_err(|_| MemoryAccessError::Overflow)?;
        let view = &self.view;
        let end = offset.checked_add(len).ok_or(MemoryAccessError::Overflow)?;
        if end > view.length() {
            #[cfg(feature = "tracing")]
            warn!(
                "attempted to write ({} bytes) beyond the bounds of the memory view ({} > {})",
                len,
                end,
                view.length()
            );
            Err(MemoryAccessError::HeapOutOfBounds)?;
        }
        view.subarray(offset, end).copy_from(data);
        Ok(())
    }

    /// Safely reads a single byte from memory at the given offset
    ///
    /// This method is guaranteed to be safe (from the host side) in the face of
    /// concurrent writes.
    pub fn write_u8(&self, offset: u64, val: u8) -> Result<(), MemoryAccessError> {
        let view = &self.view;
        let offset: u32 = offset.try_into().map_err(|_| MemoryAccessError::Overflow)?;
        if offset >= view.length() {
            #[cfg(feature = "tracing")]
            warn!(
                "attempted to write beyond the bounds of the memory view ({} >= {})",
                offset,
                view.length()
            );
            Err(MemoryAccessError::HeapOutOfBounds)?;
        }
        view.set_index(offset, val);
        Ok(())
    }

    /// Copies the memory and returns it as a vector of bytes
    #[allow(unused)]
    pub fn copy_to_vec(&self) -> Result<Vec<u8>, MemoryAccessError> {
        let mut new_memory = Vec::new();
        let mut offset = 0;
        let mut chunk = [0u8; 40960];
        while offset < self.data_size() {
            let remaining = self.data_size() - offset;
            let sublen = remaining.min(chunk.len() as u64) as usize;
            self.read(offset, &mut chunk[..sublen])?;
            new_memory.extend_from_slice(&chunk[..sublen]);
            offset += sublen as u64;
        }
        Ok(new_memory)
    }

    /// Copies the memory to another new memory object
    pub fn copy_to_memory(&self, amount: u64, new_memory: &Self) -> Result<(), MemoryAccessError> {
        let mut offset = 0;
        let mut chunk = [0u8; 40960];
        while offset < amount {
            let remaining = amount - offset;
            let sublen = remaining.min(chunk.len() as u64) as usize;
            self.read(offset, &mut chunk[..sublen])?;

            new_memory.write(offset, &chunk[..sublen])?;

            offset += sublen as u64;
        }
        Ok(())
    }
}
