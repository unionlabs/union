use core::mem::{self, MaybeUninit};
use core::ops::{Deref, DerefMut};
use core::{ptr, slice};

use crate::Result;
use crate::{PartialAllocStrategy, PhysallocFlags, PhysmapFlags};
use crate::arch::PAGE_SIZE;

/// An RAII guard of a physical memory allocation. Currently all physically allocated memory are
/// page-aligned and take up at least 4k of space (on x86_64).
#[derive(Debug)]
pub struct PhysBox {
    address: usize,
    size: usize
}

const fn round_up(x: usize) -> usize {
    (x + PAGE_SIZE - 1) / PAGE_SIZE * PAGE_SIZE
}
fn assert_aligned(x: usize) {
    assert_eq!(x % PAGE_SIZE, 0);
}

#[cfg(target_arch = "aarch64")]
fn physmap_flags() -> PhysmapFlags {
    // aarch64 currently must map DMA memory without caching to ensure coherence
    crate::PHYSMAP_NO_CACHE | crate::PHYSMAP_WRITE
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn physmap_flags() -> PhysmapFlags {
    // x86 ensures cache coherence with DMA memory
    crate::PHYSMAP_WRITE
}

impl PhysBox {
    /// Construct a PhysBox from an address and a size. The address must be page-aligned, and the
    /// size must similarly be a multiple of the page size.
    ///
    /// # Safety
    /// This function is unsafe because when dropping, Self has to a valid allocation.
    pub unsafe fn from_raw_parts(address: usize, size: usize) -> Self {
        assert_aligned(address);
        assert_aligned(size);

        Self {
            address,
            size,
        }
    }

    /// Retrieve the byte address in physical memory, of this allocation.
    pub fn address(&self) -> usize {
        self.address
    }

    /// Retrieve the size in bytes of the alloc.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Allocate physical memory that must reside in 32-bit space.
    pub fn new_in_32bit_space(size: usize) -> Result<Self> {
        Self::new_with_flags(size, PhysallocFlags::SPACE_32)
    }

    pub fn new_with_flags(size: usize, flags: PhysallocFlags) -> Result<Self> {
        assert!(!flags.contains(PhysallocFlags::PARTIAL_ALLOC));
        assert_aligned(size);

        let address = unsafe { crate::physalloc2(size, flags.bits())? };
        Ok(unsafe { Self::from_raw_parts(address, size) })
    }

    /// "Partially" allocate physical memory, in the sense that the allocation may be smaller than
    /// expected, but still with a minimum limit. This is particularly useful when the physical
    /// memory space is fragmented, and a device supports scatter-gather I/O. In that case, the
    /// driver can optimistically request e.g. 1 alloc of 1 MiB, with the minimum of 512 KiB. If
    /// that first allocation only returns half the size, the driver can do another allocation
    /// and then let the device use both buffers.
    pub fn new_partial_allocation(size: usize, flags: PhysallocFlags, strategy: Option<PartialAllocStrategy>, mut min: usize) -> Result<Self> {
        assert_aligned(size);
        debug_assert!(!(flags.contains(PhysallocFlags::PARTIAL_ALLOC) && strategy.is_none()));

        let address = unsafe { crate::physalloc3(size, flags.bits() | strategy.map_or(0, |s| s as usize), &mut min)? };
        Ok(unsafe { Self::from_raw_parts(address, size) })
    }

    pub fn new(size: usize) -> Result<Self> {
        assert_aligned(size);

        let address = unsafe { crate::physalloc(size)? };
        Ok(unsafe { Self::from_raw_parts(address, size) })
    }
}

impl Drop for PhysBox {
    fn drop(&mut self) {
        let _ = unsafe { crate::physfree(self.address, self.size) };
    }
}

pub struct Dma<T: ?Sized> {
    phys: PhysBox,
    virt: *mut T,
}

impl<T> Dma<T> {
    pub fn from_physbox_uninit(phys: PhysBox) -> Result<Dma<MaybeUninit<T>>> {
        let virt = unsafe { crate::physmap(phys.address, phys.size, physmap_flags())? } as *mut MaybeUninit<T>;

        Ok(Dma {
            phys,
            virt,
        })
    }
    pub fn from_physbox_zeroed(phys: PhysBox) -> Result<Dma<MaybeUninit<T>>> {
        let this = Self::from_physbox_uninit(phys)?;
        unsafe { ptr::write_bytes(this.virt as *mut MaybeUninit<u8>, 0, this.phys.size) }
        Ok(this)
    }

    pub fn from_physbox(phys: PhysBox, value: T) -> Result<Self> {
        let this = Self::from_physbox_uninit(phys)?;

        Ok(unsafe {
            ptr::write(this.virt, MaybeUninit::new(value));
            this.assume_init()
        })
    }

    pub fn new(value: T) -> Result<Self> {
        let phys = PhysBox::new(round_up(mem::size_of::<T>()))?;
        Self::from_physbox(phys, value)
    }
    pub fn zeroed() -> Result<Dma<MaybeUninit<T>>> {
        let phys = PhysBox::new(round_up(mem::size_of::<T>()))?;
        Self::from_physbox_zeroed(phys)
    }
}

impl<T> Dma<MaybeUninit<T>> {
    pub unsafe fn assume_init(self) -> Dma<T> {
        let &Dma { phys: PhysBox { address, size }, virt } = &self;
        mem::forget(self);

        Dma {
            phys: PhysBox { address, size },
            virt: virt as *mut T,
        }
    }
}
impl<T: ?Sized> Dma<T> {
    pub fn physical(&self) -> usize {
        self.phys.address()
    }
    pub fn size(&self) -> usize {
        self.phys.size()
    }
    pub fn phys(&self) -> &PhysBox {
        &self.phys
    }
}

impl<T> Dma<[T]> {
    pub fn from_physbox_uninit_unsized(phys: PhysBox, len: usize) -> Result<Dma<[MaybeUninit<T>]>> {
        let max_len = phys.size() / mem::size_of::<T>();
        assert!(len <= max_len);

        Ok(Dma {
            virt: unsafe { slice::from_raw_parts_mut(crate::physmap(phys.address, phys.size, physmap_flags())? as *mut MaybeUninit<T>, len) } as *mut [MaybeUninit<T>],
            phys,
        })
    }
    pub fn from_physbox_zeroed_unsized(phys: PhysBox, len: usize) -> Result<Dma<[MaybeUninit<T>]>> {
        let this = Self::from_physbox_uninit_unsized(phys, len)?;
        unsafe { ptr::write_bytes(this.virt as *mut MaybeUninit<u8>, 0, this.phys.size()) }
        Ok(this)
    }
    /// Creates a new DMA buffer with a size only known at runtime.
    /// ## Safety
    /// * `T` must be properly aligned.
    /// * `T` must be valid as zeroed (i.e. no NonNull pointers).
    pub unsafe fn zeroed_unsized(count: usize) -> Result<Self> {
        let phys = PhysBox::new(round_up(mem::size_of::<T>() * count))?;
        Ok(Self::from_physbox_zeroed_unsized(phys, count)?.assume_init())
    }
}
impl<T> Dma<[MaybeUninit<T>]> {
    pub unsafe fn assume_init(self) -> Dma<[T]> {
        let &Dma { phys: PhysBox { address, size }, virt } = &self;
        mem::forget(self);

        Dma {
            phys: PhysBox { address, size },
            virt: virt as *mut [T],
        }
    }
}

impl<T: ?Sized> Deref for Dma<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.virt }
    }
}

impl<T: ?Sized> DerefMut for Dma<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.virt }
    }
}

impl<T: ?Sized> Drop for Dma<T> {
    fn drop(&mut self) {
        unsafe { ptr::drop_in_place(self.virt) }
        let _ = unsafe { crate::funmap(self.virt as *mut u8 as usize, self.phys.size) };
    }
}
