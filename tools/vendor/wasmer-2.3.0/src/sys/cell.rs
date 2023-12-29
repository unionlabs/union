pub use std::cell::Cell;

use core::cmp::Ordering;
use core::fmt::{self, Debug};

/// A mutable Wasm-memory location.
#[repr(transparent)]
pub struct WasmCell<'a, T: ?Sized> {
    inner: &'a Cell<T>,
}

unsafe impl<T: ?Sized> Send for WasmCell<'_, T> where T: Send {}

unsafe impl<T: ?Sized> Sync for WasmCell<'_, T> {}

impl<'a, T: Copy> Clone for WasmCell<'a, T> {
    #[inline]
    fn clone(&self) -> WasmCell<'a, T> {
        WasmCell { inner: self.inner }
    }
}

impl<T: PartialEq + Copy> PartialEq for WasmCell<'_, T> {
    #[inline]
    fn eq(&self, other: &WasmCell<T>) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl<T: Eq + Copy> Eq for WasmCell<'_, T> {}

impl<T: PartialOrd + Copy> PartialOrd for WasmCell<'_, T> {
    #[inline]
    fn partial_cmp(&self, other: &WasmCell<T>) -> Option<Ordering> {
        self.inner.partial_cmp(&other.inner)
    }

    #[inline]
    fn lt(&self, other: &WasmCell<T>) -> bool {
        self.inner < other.inner
    }

    #[inline]
    fn le(&self, other: &WasmCell<T>) -> bool {
        self.inner <= other.inner
    }

    #[inline]
    fn gt(&self, other: &WasmCell<T>) -> bool {
        self.inner > other.inner
    }

    #[inline]
    fn ge(&self, other: &WasmCell<T>) -> bool {
        self.inner >= other.inner
    }
}

impl<T: Ord + Copy> Ord for WasmCell<'_, T> {
    #[inline]
    fn cmp(&self, other: &WasmCell<T>) -> Ordering {
        self.inner.cmp(&other.inner)
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
    pub const fn new(cell: &'a Cell<T>) -> WasmCell<'a, T> {
        WasmCell { inner: cell }
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
        self.inner.get()
    }

    /// Get an unsafe mutable pointer to the inner item
    /// in the Cell.
    ///
    /// # Safety
    ///
    /// This method is highly discouraged to use. We have it for
    /// compatibility reasons with Emscripten.
    /// It is unsafe because changing an item inline will change
    /// the underlying memory.
    ///
    /// It's highly encouraged to use the `set` method instead.
    #[deprecated(
        since = "2.0.0",
        note = "Please use the memory-safe set method instead"
    )]
    #[doc(hidden)]
    pub unsafe fn get_mut(&self) -> &'a mut T {
        &mut *self.inner.as_ptr()
    }
}

impl<T: Debug + Copy> Debug for WasmCell<'_, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WasmCell({:?})", self.inner.get())
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
        self.inner.set(val);
    }
}
