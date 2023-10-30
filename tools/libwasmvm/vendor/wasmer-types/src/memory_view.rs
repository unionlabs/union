use crate::lib::std::cell::Cell;
use crate::lib::std::marker::PhantomData;
use crate::lib::std::ops::Deref;
// use crate::lib::std::ops::{Bound, RangeBounds};
use crate::lib::std::slice;
use crate::lib::std::sync::atomic::{
    AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicU16, AtomicU32, AtomicU64, AtomicU8,
};
use crate::native::ValueType;

pub trait Atomic {
    type Output;
}

macro_rules! atomic {
    ( $($for:ty => $output:ty),+ ) => {
        $(
            impl Atomic for $for {
                type Output = $output;
            }
        )+
    }
}

atomic!(
    i8 => AtomicI8,
    i16 => AtomicI16,
    i32 => AtomicI32,
    i64 => AtomicI64,
    u8 => AtomicU8,
    u16 => AtomicU16,
    u32 => AtomicU32,
    u64 => AtomicU64,
    f32 => AtomicU32,
    f64 => AtomicU64
);

/// A trait that represants an atomic type.
pub trait Atomicity {}

/// Atomically.
pub struct Atomically;
impl Atomicity for Atomically {}

/// Non-atomically.
pub struct NonAtomically;
impl Atomicity for NonAtomically {}

/// A view into a memory.
pub struct MemoryView<'a, T: 'a, A = NonAtomically> {
    ptr: *mut T,
    // Note: the length is in the terms of `size::<T>()`.
    // The total length in memory is `size::<T>() * length`.
    length: usize,
    _phantom: PhantomData<(&'a [Cell<T>], A)>,
}

impl<'a, T> MemoryView<'a, T, NonAtomically>
where
    T: ValueType,
{
    /// Creates a new MemoryView given a `pointer` and `length`.
    pub unsafe fn new(ptr: *mut T, length: u32) -> Self {
        Self {
            ptr,
            length: length as usize,
            _phantom: PhantomData,
        }
    }

    /// Creates a subarray view from this `MemoryView`.
    pub fn subarray(&self, start: u32, end: u32) -> Self {
        assert!(
            (start as usize) < self.length,
            "The range start is bigger than current length"
        );
        assert!(
            (end as usize) < self.length,
            "The range end is bigger than current length"
        );

        Self {
            ptr: unsafe { self.ptr.add(start as usize) },
            length: (end - start) as usize,
            _phantom: PhantomData,
        }
    }

    /// Copy the contents of the source slice into this `MemoryView`.
    ///
    /// This function will efficiently copy the memory from within the wasm
    /// module’s own linear memory to this typed array.
    ///
    /// # Safety
    ///
    /// This method is unsafe because the caller will need to make sure
    /// there are no data races when copying memory into the view.
    pub unsafe fn copy_from(&self, src: &[T]) {
        // We cap at a max length
        let sliced_src = &src[..self.length];
        for (i, byte) in sliced_src.iter().enumerate() {
            *self.ptr.offset(i as isize) = *byte;
        }
    }
}

impl<'a, T: Atomic> MemoryView<'a, T> {
    /// Get atomic access to a memory view.
    pub fn atomically(&self) -> MemoryView<'a, T::Output, Atomically> {
        MemoryView {
            ptr: self.ptr as *mut T::Output,
            length: self.length,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T> Deref for MemoryView<'a, T, NonAtomically> {
    type Target = [Cell<T>];
    fn deref(&self) -> &[Cell<T>] {
        let mut_slice: &mut [T] = unsafe { slice::from_raw_parts_mut(self.ptr, self.length) };
        let cell_slice: &Cell<[T]> = Cell::from_mut(mut_slice);
        cell_slice.as_slice_of_cells()
    }
}

impl<'a, T> Deref for MemoryView<'a, T, Atomically> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.ptr as *const T, self.length) }
    }
}
