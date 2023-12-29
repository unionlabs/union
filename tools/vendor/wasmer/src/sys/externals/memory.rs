use super::memory_view::MemoryView;
use crate::store::{AsStoreMut, AsStoreRef};
use crate::sys::engine::NativeEngineExt;
use crate::vm::VMExternMemory;
use crate::MemoryAccessError;
use crate::MemoryType;
use std::convert::TryInto;
use std::marker::PhantomData;
use std::mem;
use std::mem::MaybeUninit;
use std::slice;
#[cfg(feature = "tracing")]
use tracing::warn;
use wasmer_types::Pages;
use wasmer_vm::{LinearMemory, MemoryError, StoreHandle, VMExtern, VMMemory};

#[derive(Debug, Clone)]
pub struct Memory {
    pub(crate) handle: StoreHandle<VMMemory>,
}

impl Memory {
    pub fn new(store: &mut impl AsStoreMut, ty: MemoryType) -> Result<Self, MemoryError> {
        let mut store = store.as_store_mut();
        let tunables = store.engine().tunables();
        let style = tunables.memory_style(&ty);
        let memory = tunables.create_host_memory(&ty, &style)?;

        Ok(Self {
            handle: StoreHandle::new(store.objects_mut(), memory),
        })
    }

    pub fn new_from_existing(new_store: &mut impl AsStoreMut, memory: VMMemory) -> Self {
        let handle = StoreHandle::new(new_store.objects_mut(), memory);
        Self::from_vm_extern(new_store, handle.internal_handle())
    }

    pub fn ty(&self, store: &impl AsStoreRef) -> MemoryType {
        self.handle.get(store.as_store_ref().objects()).ty()
    }

    pub fn grow<IntoPages>(
        &self,
        store: &mut impl AsStoreMut,
        delta: IntoPages,
    ) -> Result<Pages, MemoryError>
    where
        IntoPages: Into<Pages>,
    {
        self.handle.get_mut(store.objects_mut()).grow(delta.into())
    }

    pub(crate) fn from_vm_extern(store: &impl AsStoreRef, vm_extern: VMExternMemory) -> Self {
        Self {
            handle: unsafe {
                StoreHandle::from_internal(store.as_store_ref().objects().id(), vm_extern)
            },
        }
    }

    /// Checks whether this `Memory` can be used with the given context.
    pub fn is_from_store(&self, store: &impl AsStoreRef) -> bool {
        self.handle.store_id() == store.as_store_ref().objects().id()
    }

    /// Cloning memory will create another reference to the same memory that
    /// can be put into a new store
    pub fn try_clone(&self, store: &impl AsStoreRef) -> Result<VMMemory, MemoryError> {
        let mem = self.handle.get(store.as_store_ref().objects());
        let cloned = mem.try_clone()?;
        Ok(cloned.into())
    }

    /// Copying the memory will actually copy all the bytes in the memory to
    /// a identical byte copy of the original that can be put into a new store
    pub fn try_copy(
        &self,
        store: &impl AsStoreRef,
    ) -> Result<Box<dyn LinearMemory + 'static>, MemoryError> {
        let mut mem = self.try_clone(store)?;
        mem.copy()
    }

    /// To `VMExtern`.
    pub(crate) fn to_vm_extern(&self) -> VMExtern {
        VMExtern::Memory(self.handle.internal_handle())
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
