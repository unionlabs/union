use crate::access::{RefCow, SliceCow, WasmRefAccess, WasmSliceAccess};
use crate::{MemoryAccessError, WasmRef, WasmSlice};
use std::mem::{self, MaybeUninit};
use std::slice;

impl<'a, T> WasmSliceAccess<'a, T>
where
    T: wasmer_types::ValueType,
{
    pub(crate) fn new(slice: WasmSlice<'a, T>) -> Result<Self, MemoryAccessError> {
        let buf = slice.read_to_vec()?;
        Ok(Self {
            slice,
            buf: SliceCow::Owned(buf, false),
        })
    }
}

impl<'a, T> WasmRefAccess<'a, T>
where
    T: wasmer_types::ValueType,
{
    pub(crate) fn new(ptr: WasmRef<'a, T>) -> Result<Self, MemoryAccessError> {
        let mut out = MaybeUninit::uninit();
        let buf =
            unsafe { slice::from_raw_parts_mut(out.as_mut_ptr() as *mut u8, mem::size_of::<T>()) };
        ptr.buffer.read(ptr.offset, buf)?;
        let val = unsafe { out.assume_init() };

        Ok(Self {
            ptr,
            buf: RefCow::Owned(val, false),
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
        let mut data = MaybeUninit::new(val);
        let data = unsafe {
            slice::from_raw_parts_mut(
                data.as_mut_ptr() as *mut MaybeUninit<u8>,
                mem::size_of::<T>(),
            )
        };
        val.zero_padding_bytes(data);
        let data = unsafe { slice::from_raw_parts(data.as_ptr() as *const _, data.len()) };
        self.ptr.buffer.write(self.ptr.offset, data).unwrap()
    }
}
