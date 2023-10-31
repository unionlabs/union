use core::cmp::Ordering;
use core::fmt::{self, Debug};
use std::marker::PhantomData;

use js_sys::Uint8Array;

/// A mutable Wasm-memory location.
pub struct WasmCell<'a, T: ?Sized> {
    pub(crate) memory: Uint8Array,
    #[allow(dead_code)]
    phantom: &'a PhantomData<T>,
}

unsafe impl<T: ?Sized> Send for WasmCell<'_, T> where T: Send {}

unsafe impl<T: ?Sized> Sync for WasmCell<'_, T> {}

impl<'a, T: Copy> Clone for WasmCell<'a, T> {
    #[inline]
    fn clone(&self) -> WasmCell<'a, T> {
        WasmCell {
            memory: self.memory.clone(),
            phantom: &PhantomData,
        }
    }
}

impl<T: PartialEq + Copy> PartialEq for WasmCell<'_, T> {
    #[inline]
    fn eq(&self, other: &WasmCell<T>) -> bool {
        self.get() == other.get()
    }
}

impl<T: Eq + Copy> Eq for WasmCell<'_, T> {}

impl<T: PartialOrd + Copy> PartialOrd for WasmCell<'_, T> {
    #[inline]
    fn partial_cmp(&self, other: &WasmCell<T>) -> Option<Ordering> {
        self.get().partial_cmp(&other.get())
    }

    #[inline]
    fn lt(&self, other: &WasmCell<T>) -> bool {
        self.get() < other.get()
    }

    #[inline]
    fn le(&self, other: &WasmCell<T>) -> bool {
        self.get() <= other.get()
    }

    #[inline]
    fn gt(&self, other: &WasmCell<T>) -> bool {
        self.get() > other.get()
    }

    #[inline]
    fn ge(&self, other: &WasmCell<T>) -> bool {
        self.get() >= other.get()
    }
}

impl<T: Ord + Copy> Ord for WasmCell<'_, T> {
    #[inline]
    fn cmp(&self, other: &WasmCell<T>) -> Ordering {
        self.get().cmp(&other.get())
    }
}

impl<'a, T> WasmCell<'a, T> {
    /// Creates a new `WasmCell` containing the given value.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cell::Cell;
    /// use wasmer::WasmCell;
    ///
    /// let cell = Cell::new(5);
    /// let wasm_cell = WasmCell::new(&cell);
    /// ```
    #[inline]
    pub const fn new(memory: Uint8Array) -> WasmCell<'a, T> {
        WasmCell {
            memory,
            phantom: &PhantomData,
        }
    }
}

impl<'a, T: Copy> WasmCell<'a, T> {
    /// Returns a copy of the contained value.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cell::Cell;
    /// use wasmer::WasmCell;
    ///
    /// let cell = Cell::new(5);
    /// let wasm_cell = WasmCell::new(&cell);
    /// let five = wasm_cell.get();
    /// ```
    #[inline]
    pub fn get(&self) -> T {
        let vec = self.memory.to_vec();
        unsafe { *(vec.as_ptr() as *const T) }
    }
}

impl<T: Debug + Copy> Debug for WasmCell<'_, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WasmCell({:?})", self.get())
    }
}

impl<T: Sized> WasmCell<'_, T> {
    /// Sets the contained value.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cell::Cell;
    /// use wasmer::WasmCell;
    ///
    /// let cell = Cell::new(5);
    /// let wasm_cell = WasmCell::new(&cell);
    /// wasm_cell.set(10);
    /// assert_eq!(cell.get(), 10);
    /// ```
    #[inline]
    pub fn set(&self, val: T) {
        let size = std::mem::size_of::<T>();
        let ptr = &val as *const T as *const u8;
        let slice = unsafe { std::slice::from_raw_parts(ptr, size) };
        self.memory.copy_from(slice);
    }
}
