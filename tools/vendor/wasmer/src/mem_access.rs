use crate::access::WasmRefAccess;
use crate::externals::memory::MemoryBuffer;
use crate::{Memory32, Memory64, MemorySize, MemoryView, WasmPtr};
use crate::{RuntimeError, WasmSliceAccess};
use std::convert::TryInto;
use std::fmt;
use std::marker::PhantomData;
use std::mem::{self, MaybeUninit};
use std::ops::Range;
use std::slice;
use std::string::FromUtf8Error;
use thiserror::Error;
use wasmer_types::ValueType;

/// Error for invalid [`Memory`][super::Memory] access.
#[derive(Clone, Copy, Debug, Error)]
#[non_exhaustive]
pub enum MemoryAccessError {
    /// Memory access is outside heap bounds.
    #[error("memory access out of bounds")]
    HeapOutOfBounds,
    /// Address calculation overflow.
    #[error("address calculation overflow")]
    Overflow,
    /// String is not valid UTF-8.
    #[error("string is not valid utf-8")]
    NonUtf8String,
}

impl From<MemoryAccessError> for RuntimeError {
    fn from(err: MemoryAccessError) -> Self {
        Self::new(err.to_string())
    }
}
impl From<FromUtf8Error> for MemoryAccessError {
    fn from(_err: FromUtf8Error) -> Self {
        Self::NonUtf8String
    }
}

/// Reference to a value in Wasm memory.
///
/// The type of the value must satisfy the requirements of the `ValueType`
/// trait which guarantees that reading and writing such a value to untrusted
/// memory is safe.
///
/// The address is not required to be aligned: unaligned accesses are fully
/// supported.
///
/// This wrapper safely handles concurrent modifications of the data by another
/// thread.
#[derive(Clone, Copy)]
pub struct WasmRef<'a, T: ValueType> {
    #[allow(unused)]
    pub(crate) buffer: MemoryBuffer<'a>,
    pub(crate) offset: u64,
    marker: PhantomData<*mut T>,
}

impl<'a, T: ValueType> WasmRef<'a, T> {
    /// Creates a new `WasmRef` at the given offset in a memory.
    #[inline]
    pub fn new(view: &'a MemoryView, offset: u64) -> Self {
        Self {
            buffer: view.buffer(),
            offset,
            marker: PhantomData,
        }
    }

    /// Get the offset into Wasm linear memory for this `WasmRef`.
    #[inline]
    pub fn offset(self) -> u64 {
        self.offset
    }

    /// Get a `WasmPtr` for this `WasmRef`.
    #[inline]
    pub fn as_ptr32(self) -> WasmPtr<T, Memory32> {
        WasmPtr::new(self.offset as u32)
    }

    /// Get a 64-bit `WasmPtr` for this `WasmRef`.
    #[inline]
    pub fn as_ptr64(self) -> WasmPtr<T, Memory64> {
        WasmPtr::new(self.offset)
    }

    /// Get a `WasmPtr` fror this `WasmRef`.
    #[inline]
    pub fn as_ptr<M: MemorySize>(self) -> WasmPtr<T, M> {
        let offset: M::Offset = self
            .offset
            .try_into()
            .map_err(|_| "invalid offset into memory")
            .unwrap();
        WasmPtr::<T, M>::new(offset)
    }

    /// Reads the location pointed to by this `WasmRef`.
    #[inline]
    pub fn read(self) -> Result<T, MemoryAccessError> {
        let mut out = MaybeUninit::uninit();
        let buf =
            unsafe { slice::from_raw_parts_mut(out.as_mut_ptr() as *mut u8, mem::size_of::<T>()) };
        self.buffer.read(self.offset, buf)?;
        Ok(unsafe { out.assume_init() })
        // Ok(self.access()?.read())
    }

    /// Writes to the location pointed to by this `WasmRef`.
    #[inline]
    pub fn write(self, val: T) -> Result<(), MemoryAccessError> {
        self.access()?.write(val);
        Ok(())
    }

    /// Gains direct access to the memory of this slice
    #[inline]
    pub fn access(self) -> Result<WasmRefAccess<'a, T>, MemoryAccessError> {
        WasmRefAccess::new(self)
    }
}

impl<'a, T: ValueType> fmt::Debug for WasmRef<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "WasmRef(offset: {}, pointer: {:#x})",
            self.offset, self.offset
        )
    }
}

/// Reference to an array of values in Wasm memory.
///
/// The type of the value must satisfy the requirements of the `ValueType`
/// trait which guarantees that reading and writing such a value to untrusted
/// memory is safe.
///
/// The address is not required to be aligned: unaligned accesses are fully
/// supported.
///
/// This wrapper safely handles concurrent modifications of the data by another
/// thread.
#[derive(Clone, Copy)]
pub struct WasmSlice<'a, T: ValueType> {
    pub(crate) buffer: MemoryBuffer<'a>,
    pub(crate) offset: u64,
    pub(crate) len: u64,
    marker: PhantomData<*mut T>,
}

impl<'a, T: ValueType> WasmSlice<'a, T> {
    /// Creates a new `WasmSlice` starting at the given offset in memory and
    /// with the given number of elements.
    ///
    /// Returns a `MemoryAccessError` if the slice length overflows.
    #[inline]
    pub fn new(view: &'a MemoryView, offset: u64, len: u64) -> Result<Self, MemoryAccessError> {
        let total_len = len
            .checked_mul(mem::size_of::<T>() as u64)
            .ok_or(MemoryAccessError::Overflow)?;
        offset
            .checked_add(total_len)
            .ok_or(MemoryAccessError::Overflow)?;
        Ok(Self {
            buffer: view.buffer(),
            offset,
            len,
            marker: PhantomData,
        })
    }

    /// Get the offset into Wasm linear memory for this `WasmSlice`.
    #[inline]
    pub fn offset(self) -> u64 {
        self.offset
    }

    /// Get a 32-bit `WasmPtr` for this `WasmRef`.
    #[inline]
    pub fn as_ptr32(self) -> WasmPtr<T, Memory32> {
        WasmPtr::new(self.offset as u32)
    }

    /// Get a 64-bit `WasmPtr` for this `WasmRef`.
    #[inline]
    pub fn as_ptr64(self) -> WasmPtr<T, Memory64> {
        WasmPtr::new(self.offset)
    }

    /// Get the number of elements in this slice.
    #[inline]
    pub fn len(self) -> u64 {
        self.len
    }

    /// Returns `true` if the number of elements is 0.
    #[inline]
    pub fn is_empty(self) -> bool {
        self.len == 0
    }

    /// Get a `WasmRef` to an element in the slice.
    #[inline]
    pub fn index(self, idx: u64) -> WasmRef<'a, T> {
        if idx >= self.len {
            panic!("WasmSlice out of bounds");
        }
        let offset = self.offset + idx * mem::size_of::<T>() as u64;
        WasmRef {
            buffer: self.buffer,
            offset,
            marker: PhantomData,
        }
    }

    /// Get a `WasmSlice` for a subslice of this slice.
    #[inline]
    pub fn subslice(self, range: Range<u64>) -> WasmSlice<'a, T> {
        if range.start > range.end || range.end > self.len {
            panic!("WasmSlice out of bounds");
        }
        let offset = self.offset + range.start * mem::size_of::<T>() as u64;
        Self {
            buffer: self.buffer,
            offset,
            len: range.end - range.start,
            marker: PhantomData,
        }
    }

    /// Get an iterator over the elements in this slice.
    #[inline]
    pub fn iter(self) -> WasmSliceIter<'a, T> {
        WasmSliceIter { slice: self }
    }

    /// Gains direct access to the memory of this slice
    #[inline]
    pub fn access(self) -> Result<WasmSliceAccess<'a, T>, MemoryAccessError> {
        WasmSliceAccess::new(self)
    }

    /// Reads an element of this slice.
    #[inline]
    pub fn read(self, idx: u64) -> Result<T, MemoryAccessError> {
        self.index(idx).read()
    }

    /// Writes to an element of this slice.
    #[inline]
    pub fn write(self, idx: u64, val: T) -> Result<(), MemoryAccessError> {
        self.index(idx).write(val)
    }

    /// Reads the entire slice into the given buffer.
    ///
    /// The length of the buffer must match the length of the slice.
    #[inline]
    pub fn read_slice(self, buf: &mut [T]) -> Result<(), MemoryAccessError> {
        assert_eq!(
            buf.len() as u64,
            self.len,
            "slice length doesn't match WasmSlice length"
        );
        let size = std::mem::size_of_val(buf);
        let bytes =
            unsafe { slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut MaybeUninit<u8>, size) };
        self.buffer.read_uninit(self.offset, bytes)?;
        Ok(())
    }

    /// Reads the entire slice into the given uninitialized buffer.
    ///
    /// The length of the buffer must match the length of the slice.
    ///
    /// This method returns an initialized view of the buffer.
    #[inline]
    pub fn read_slice_uninit(
        self,
        buf: &mut [MaybeUninit<T>],
    ) -> Result<&mut [T], MemoryAccessError> {
        assert_eq!(
            buf.len() as u64,
            self.len,
            "slice length doesn't match WasmSlice length"
        );
        let bytes = unsafe {
            slice::from_raw_parts_mut(
                buf.as_mut_ptr() as *mut MaybeUninit<u8>,
                buf.len() * mem::size_of::<T>(),
            )
        };
        self.buffer.read_uninit(self.offset, bytes)?;
        Ok(unsafe { slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut T, buf.len()) })
    }

    /// Write the given slice into this `WasmSlice`.
    ///
    /// The length of the slice must match the length of the `WasmSlice`.
    #[inline]
    pub fn write_slice(self, data: &[T]) -> Result<(), MemoryAccessError> {
        assert_eq!(
            data.len() as u64,
            self.len,
            "slice length doesn't match WasmSlice length"
        );
        let size = std::mem::size_of_val(data);
        let bytes = unsafe { slice::from_raw_parts(data.as_ptr() as *const u8, size) };
        self.buffer.write(self.offset, bytes)
    }

    /// Reads this `WasmSlice` into a `slice`.
    #[inline]
    pub fn read_to_slice(self, buf: &mut [MaybeUninit<u8>]) -> Result<usize, MemoryAccessError> {
        let len = self.len.try_into().expect("WasmSlice length overflow");
        self.buffer.read_uninit(self.offset, buf)?;
        Ok(len)
    }

    /// Reads this `WasmSlice` into a `Vec`.
    #[inline]
    pub fn read_to_vec(self) -> Result<Vec<T>, MemoryAccessError> {
        let len = self.len.try_into().expect("WasmSlice length overflow");
        let mut vec = Vec::with_capacity(len);
        let bytes = unsafe {
            slice::from_raw_parts_mut(
                vec.as_mut_ptr() as *mut MaybeUninit<u8>,
                len * mem::size_of::<T>(),
            )
        };
        self.buffer.read_uninit(self.offset, bytes)?;
        unsafe {
            vec.set_len(len);
        }
        Ok(vec)
    }

    /// Reads this `WasmSlice` into a `BytesMut`
    #[inline]
    pub fn read_to_bytes(self) -> Result<bytes::BytesMut, MemoryAccessError> {
        let len = self.len.try_into().expect("WasmSlice length overflow");
        let mut ret = bytes::BytesMut::with_capacity(len);
        let bytes = unsafe {
            slice::from_raw_parts_mut(
                ret.as_mut_ptr() as *mut MaybeUninit<u8>,
                len * mem::size_of::<T>(),
            )
        };
        self.buffer.read_uninit(self.offset, bytes)?;
        unsafe {
            ret.set_len(len);
        }
        Ok(ret)
    }
}

impl<'a, T: ValueType> fmt::Debug for WasmSlice<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "WasmSlice(offset: {}, len: {}, pointer: {:#x})",
            self.offset, self.len, self.offset
        )
    }
}

/// Iterator over the elements of a `WasmSlice`.
pub struct WasmSliceIter<'a, T: ValueType> {
    slice: WasmSlice<'a, T>,
}

impl<'a, T: ValueType> Iterator for WasmSliceIter<'a, T> {
    type Item = WasmRef<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.slice.is_empty() {
            let elem = self.slice.index(0);
            self.slice = self.slice.subslice(1..self.slice.len());
            Some(elem)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0..self.slice.len()).size_hint()
    }
}

impl<'a, T: ValueType> DoubleEndedIterator for WasmSliceIter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if !self.slice.is_empty() {
            let elem = self.slice.index(self.slice.len() - 1);
            self.slice = self.slice.subslice(0..self.slice.len() - 1);
            Some(elem)
        } else {
            None
        }
    }
}

impl<'a, T: ValueType> ExactSizeIterator for WasmSliceIter<'a, T> {}
