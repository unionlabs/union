use std::mem::MaybeUninit;

use crate::mem_access::{WasmRef, WasmSlice};

pub(super) enum SliceCow<'a, T> {
    #[allow(dead_code)]
    Borrowed(&'a mut [T]),
    #[allow(dead_code)]
    Owned(Vec<T>, bool),
}

impl<'a, T> AsRef<[T]> for SliceCow<'a, T> {
    fn as_ref(&self) -> &[T] {
        match self {
            Self::Borrowed(buf) => buf,
            Self::Owned(buf, _) => buf,
        }
    }
}

impl<'a, T> AsMut<[T]> for SliceCow<'a, T> {
    fn as_mut(&mut self) -> &mut [T] {
        // Note: Zero padding is not required here as its a typed copy which does
        //       not leak the bytes into the memory
        // https://stackoverflow.com/questions/61114026/does-stdptrwrite-transfer-the-uninitialized-ness-of-the-bytes-it-writes
        match self {
            Self::Borrowed(buf) => buf,
            Self::Owned(buf, modified) => {
                *modified = true;
                buf.as_mut()
            }
        }
    }
}

/// Provides direct memory access to a piece of memory that
/// is owned by WASM
pub struct WasmSliceAccess<'a, T>
where
    T: wasmer_types::ValueType,
{
    pub(super) slice: WasmSlice<'a, T>,
    pub(super) buf: SliceCow<'a, T>,
}

impl<'a, T> AsRef<[T]> for WasmSliceAccess<'a, T>
where
    T: wasmer_types::ValueType,
{
    fn as_ref(&self) -> &[T] {
        self.buf.as_ref()
    }
}

impl<'a, T> AsMut<[T]> for WasmSliceAccess<'a, T>
where
    T: wasmer_types::ValueType,
{
    fn as_mut(&mut self) -> &mut [T] {
        self.buf.as_mut()
    }
}

impl<'a, T> WasmSliceAccess<'a, T>
where
    T: wasmer_types::ValueType,
{
    /// Returns an iterator of all the elements in the slice
    pub fn iter(&'a self) -> std::slice::Iter<'a, T> {
        self.as_ref().iter()
    }

    /// Returns an iterator of all the elements in the slice
    pub fn iter_mut(&'a mut self) -> std::slice::IterMut<'a, T> {
        self.buf.as_mut().iter_mut()
    }

    /// Number of elements in this slice
    pub fn len(&self) -> usize {
        self.buf.as_ref().len()
    }

    /// If the slice is empty
    pub fn is_empty(&self) -> bool {
        self.buf.as_ref().is_empty()
    }
}

impl<'a> WasmSliceAccess<'a, u8> {
    /// Writes to the address pointed to by this `WasmPtr` in a memory.
    #[inline]
    pub fn copy_from_slice(&mut self, src: &[u8]) {
        let dst = self.buf.as_mut();
        dst.copy_from_slice(src);
    }
}

impl<'a, T> Drop for WasmSliceAccess<'a, T>
where
    T: wasmer_types::ValueType,
{
    fn drop(&mut self) {
        if let SliceCow::Owned(buf, modified) = &self.buf {
            if *modified {
                self.slice.write_slice(buf.as_ref()).ok();
            }
        }
    }
}

pub(super) enum RefCow<'a, T> {
    #[allow(dead_code)]
    Borrowed(&'a mut T),
    #[allow(dead_code)]
    Owned(T, bool),
}

impl<'a, T> AsRef<T> for RefCow<'a, T> {
    fn as_ref(&self) -> &T {
        match self {
            Self::Borrowed(val) => val,
            Self::Owned(val, _) => val,
        }
    }
}

impl<'a, T> AsMut<T> for RefCow<'a, T> {
    fn as_mut(&mut self) -> &mut T {
        // Note: Zero padding is not required here as its a typed copy which does
        //       not leak the bytes into the memory
        // https://stackoverflow.com/questions/61114026/does-stdptrwrite-transfer-the-uninitialized-ness-of-the-bytes-it-writes
        match self {
            Self::Borrowed(val) => val,
            Self::Owned(val, modified) => {
                *modified = true;
                val
            }
        }
    }
}

/// Provides direct memory access to a piece of memory that
/// is owned by WASM
pub struct WasmRefAccess<'a, T>
where
    T: wasmer_types::ValueType,
{
    pub(super) ptr: WasmRef<'a, T>,
    pub(super) buf: RefCow<'a, T>,
}

impl<'a, T> AsRef<T> for WasmRefAccess<'a, T>
where
    T: wasmer_types::ValueType,
{
    fn as_ref(&self) -> &T {
        self.buf.as_ref()
    }
}

impl<'a, T> AsMut<T> for WasmRefAccess<'a, T>
where
    T: wasmer_types::ValueType,
{
    fn as_mut(&mut self) -> &mut T {
        self.buf.as_mut()
    }
}

impl<'a, T> Drop for WasmRefAccess<'a, T>
where
    T: wasmer_types::ValueType,
{
    fn drop(&mut self) {
        if let RefCow::Owned(val, modified) = &self.buf {
            if *modified {
                self.ptr.write(*val).ok();
            }
        }
    }
}

impl<'a, T> WasmSliceAccess<'a, T>
where
    T: wasmer_types::ValueType,
{
    /// Returns a mutable slice that is not yet initialized
    pub fn as_mut_uninit(&mut self) -> &mut [MaybeUninit<T>] {
        let ret: &mut [T] = self.buf.as_mut();
        let ret: &mut [MaybeUninit<T>] = unsafe { std::mem::transmute(ret) };
        ret
    }
}

impl<'a, T> WasmRefAccess<'a, T>
where
    T: wasmer_types::ValueType,
{
    /// Returns a reference to an unitialized reference to this value
    pub fn as_mut_uninit(&mut self) -> &mut MaybeUninit<T> {
        let ret: &mut T = self.buf.as_mut();
        let ret: &mut MaybeUninit<T> = unsafe { std::mem::transmute(ret) };
        ret
    }
}
