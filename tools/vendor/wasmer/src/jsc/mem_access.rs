use crate::access::{RefCow, SliceCow, WasmRefAccess, WasmSliceAccess};
use crate::{MemoryAccessError, WasmRef, WasmSlice};
use std::mem;

impl<'a, T> WasmSliceAccess<'a, T>
where
    T: wasmer_types::ValueType,
{
    pub(crate) fn new(slice: WasmSlice<'a, T>) -> Result<Self, MemoryAccessError> {
        let total_len = slice
            .len
            .checked_mul(mem::size_of::<T>() as u64)
            .ok_or(MemoryAccessError::Overflow)?;
        let end = slice
            .offset
            .checked_add(total_len)
            .ok_or(MemoryAccessError::Overflow)?;
        if end > slice.buffer.0.len as u64 {
            #[cfg(feature = "tracing")]
            warn!(
                "attempted to read ({} bytes) beyond the bounds of the memory view ({} > {})",
                total_len, end, slice.buffer.0.len
            );
            return Err(MemoryAccessError::HeapOutOfBounds);
        }
        let buf = unsafe {
            let buf_ptr: *mut u8 = slice.buffer.0.base.add(slice.offset as usize);
            let buf_ptr: *mut T = std::mem::transmute(buf_ptr);
            std::slice::from_raw_parts_mut(buf_ptr, slice.len as usize)
        };
        Ok(Self {
            slice,
            buf: SliceCow::Borrowed(buf),
        })
    }
}

impl<'a, T> WasmRefAccess<'a, T>
where
    T: wasmer_types::ValueType,
{
    pub(crate) fn new(ptr: WasmRef<'a, T>) -> Result<Self, MemoryAccessError> {
        let total_len = mem::size_of::<T>() as u64;
        let end = ptr
            .offset
            .checked_add(total_len)
            .ok_or(MemoryAccessError::Overflow)?;
        if end > ptr.buffer.0.len as u64 {
            #[cfg(feature = "tracing")]
            warn!(
                "attempted to read ({} bytes) beyond the bounds of the memory view ({} > {})",
                total_len, end, ptr.buffer.0.len
            );
            return Err(MemoryAccessError::HeapOutOfBounds);
        }
        let val = unsafe {
            let val_ptr: *mut u8 = ptr.buffer.0.base.add(ptr.offset as usize);
            let val_ptr: *mut T = std::mem::transmute(val_ptr);
            &mut *val_ptr
        };
        Ok(Self {
            ptr,
            buf: RefCow::Borrowed(val),
        })
    }
}

impl<'a, T> WasmRefAccess<'a, T>
where
    T: wasmer_types::ValueType,
{
    /// Reads the address pointed to by this `WasmPtr` in a memory.
    #[inline]
    #[allow(clippy::clone_on_copy)]
    pub fn read(&self) -> T
    where
        T: Clone,
    {
        self.as_ref().clone()
    }

    /// Writes to the address pointed to by this `WasmPtr` in a memory.
    #[inline]
    pub fn write(&mut self, val: T) {
        // Note: Zero padding is not required here as its a typed copy which does
        //       not leak the bytes into the memory
        // https://stackoverflow.com/questions/61114026/does-stdptrwrite-transfer-the-uninitialized-ness-of-the-bytes-it-writes
        *(self.as_mut()) = val;
    }
}
