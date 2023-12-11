use crate::store::AsStoreRef;
use crate::MemoryAccessError;
use rusty_jsc::JSObject;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::slice;
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
    pub(crate) buffer: MemoryBuffer<'a>,
    // pub(crate) size: Pages,
}

impl<'a> MemoryView<'a> {
    pub(crate) fn new(memory: &Memory, store: &'a (impl AsStoreRef + ?Sized)) -> Self {
        Self::new_raw(&memory.handle.memory, store)
    }

    pub(crate) fn new_raw(memory: &JSObject, store: &'a (impl AsStoreRef + ?Sized)) -> Self {
        let store_ref = store.as_store_ref();
        let engine = store_ref.engine();
        let context = engine.0.context();

        let buffer = memory
            .get_property(&context, "buffer".to_string())
            .to_object(&context)
            .unwrap();
        let typed_buffer = JSObject::create_typed_array_from_buffer(&context, buffer).unwrap();

        let mut buffer_data = typed_buffer.get_typed_array_buffer(&context).unwrap();
        // println!("BUFFER DATA {}", buffer_data.to_string(&context));

        // let definition = memory.handle.get(store.as_store_ref().objects()).vmmemory();
        // let def = unsafe { definition.as_ref() };
        Self {
            buffer: MemoryBuffer {
                base: buffer_data.as_mut_ptr(),
                len: buffer_data.len(),
                marker: PhantomData,
            },
            // size,
        }
    }

    /// Returns the pointer to the raw bytes of the `Memory`.
    //
    // This used by wasmer-emscripten and wasmer-c-api, but should be treated
    // as deprecated and not used in future code.
    #[doc(hidden)]
    pub fn data_ptr(&self) -> *mut u8 {
        self.buffer.base
    }

    /// Returns the size (in bytes) of the `Memory`.
    pub fn data_size(&self) -> u64 {
        self.buffer.len.try_into().unwrap()
    }

    /// Retrieve a slice of the memory contents.
    ///
    /// # Safety
    ///
    /// Until the returned slice is dropped, it is undefined behaviour to
    /// modify the memory contents in any way including by calling a wasm
    /// function that writes to the memory or by resizing the memory.
    #[doc(hidden)]
    pub unsafe fn data_unchecked(&self) -> &[u8] {
        self.data_unchecked_mut()
    }

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
        slice::from_raw_parts_mut(self.buffer.base, self.buffer.len)
    }

    /// Returns the size (in [`Pages`]) of the `Memory`.
    ///
    /// # Example
    ///
    /// ```
    /// # use wasmer::{Memory, MemoryType, Pages, Store, Type, Value};
    /// # let mut store = Store::default();
    /// #
    /// let m = Memory::new(&mut store, MemoryType::new(1, None, false)).unwrap();
    ///
    /// assert_eq!(m.view(&mut store).size(), Pages(1));
    /// ```
    pub fn size(&self) -> Pages {
        Bytes(self.buffer.len).try_into().unwrap()
    }

    #[inline]
    pub(crate) fn buffer(&'a self) -> MemoryBuffer<'a> {
        self.buffer
    }

    /// Safely reads bytes from the memory at the given offset.
    ///
    /// The full buffer will be filled, otherwise a `MemoryAccessError` is returned
    /// to indicate an out-of-bounds access.
    ///
    /// This method is guaranteed to be safe (from the host side) in the face of
    /// concurrent writes.
    pub fn read(&self, offset: u64, buf: &mut [u8]) -> Result<(), MemoryAccessError> {
        self.buffer.read(offset, buf)
    }

    /// Safely reads a single byte from memory at the given offset
    ///
    /// This method is guaranteed to be safe (from the host side) in the face of
    /// concurrent writes.
    pub fn read_u8(&self, offset: u64) -> Result<u8, MemoryAccessError> {
        let mut buf = [0u8; 1];
        self.read(offset, &mut buf)?;
        Ok(buf[0])
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
        self.buffer.read_uninit(offset, buf)
    }

    /// Safely writes bytes to the memory at the given offset.
    ///
    /// If the write exceeds the bounds of the memory then a `MemoryAccessError` is
    /// returned.
    ///
    /// This method is guaranteed to be safe (from the host side) in the face of
    /// concurrent reads/writes.
    pub fn write(&self, offset: u64, data: &[u8]) -> Result<(), MemoryAccessError> {
        self.buffer.write(offset, data)
    }

    /// Safely reads a single byte from memory at the given offset
    ///
    /// This method is guaranteed to be safe (from the host side) in the face of
    /// concurrent writes.
    pub fn write_u8(&self, offset: u64, val: u8) -> Result<(), MemoryAccessError> {
        let buf = [val];
        self.write(offset, &buf)?;
        Ok(())
    }

    #[allow(unused)]
    /// Copies the memory and returns it as a vector of bytes
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

    #[allow(unused)]
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
