//! Types for a reusable pointer abstraction for accessing Wasm linear memory.
//!
//! This abstraction is safe: it ensures the memory is in bounds and that the pointer
//! is aligned (avoiding undefined behavior).
//!
//! Therefore, you should use this abstraction whenever possible to avoid memory
//! related bugs when implementing an ABI.

use crate::sys::cell::WasmCell;
use crate::sys::{externals::Memory, FromToNativeWasmType};
use std::{cell::Cell, fmt, marker::PhantomData, mem};
use wasmer_types::ValueType;

/// The `Array` marker type. This type can be used like `WasmPtr<T, Array>`
/// to get access to methods
pub struct Array;
/// The `Item` marker type. This is the default and does not usually need to be
/// specified.
pub struct Item;

/// A zero-cost type that represents a pointer to something in Wasm linear
/// memory.
///
/// This type can be used directly in the host function arguments:
/// ```
/// # use wasmer::Memory;
/// # use wasmer::WasmPtr;
/// pub fn host_import(memory: Memory, ptr: WasmPtr<u32>) {
///     let derefed_ptr = ptr.deref(&memory).expect("pointer in bounds");
///     let inner_val: u32 = derefed_ptr.get();
///     println!("Got {} from Wasm memory address 0x{:X}", inner_val, ptr.offset());
///     // update the value being pointed to
///     derefed_ptr.set(inner_val + 1);
/// }
/// ```
///
/// This type can also be used with primitive-filled structs, but be careful of
/// guarantees required by `ValueType`.
/// ```
/// # use wasmer::Memory;
/// # use wasmer::WasmPtr;
/// # use wasmer::ValueType;
///
/// #[derive(Copy, Clone, Debug)]
/// #[repr(C)]
/// struct V3 {
///     x: f32,
///     y: f32,
///     z: f32
/// }
/// // This is safe as the 12 bytes represented by this struct
/// // are valid for all bit combinations.
/// unsafe impl ValueType for V3 {
/// }
///
/// fn update_vector_3(memory: Memory, ptr: WasmPtr<V3>) {
///     let derefed_ptr = ptr.deref(&memory).expect("pointer in bounds");
///     let mut inner_val: V3 = derefed_ptr.get();
///     println!("Got {:?} from Wasm memory address 0x{:X}", inner_val, ptr.offset());
///     // update the value being pointed to
///     inner_val.x = 10.4;
///     derefed_ptr.set(inner_val);
/// }
/// ```
#[repr(transparent)]
pub struct WasmPtr<T: Copy, Ty = Item> {
    offset: u32,
    _phantom: PhantomData<(T, Ty)>,
}

/// Methods relevant to all types of `WasmPtr`.
impl<T: Copy, Ty> WasmPtr<T, Ty> {
    /// Create a new `WasmPtr` at the given offset.
    #[inline]
    pub fn new(offset: u32) -> Self {
        Self {
            offset,
            _phantom: PhantomData,
        }
    }

    /// Get the offset into Wasm linear memory for this `WasmPtr`.
    #[inline]
    pub fn offset(self) -> u32 {
        self.offset
    }
}

#[inline(always)]
fn align_pointer(ptr: usize, align: usize) -> usize {
    // clears bits below aligment amount (assumes power of 2) to align pointer
    debug_assert!(align.count_ones() == 1);
    ptr & !(align - 1)
}

/// Methods for `WasmPtr`s to data that can be dereferenced, namely to types
/// that implement [`ValueType`], meaning that they're valid for all possible
/// bit patterns.
impl<T: Copy + ValueType> WasmPtr<T, Item> {
    /// Dereference the `WasmPtr` getting access to a `&Cell<T>` allowing for
    /// reading and mutating of the inner value.
    ///
    /// This method is unsound if used with unsynchronized shared memory.
    /// If you're unsure what that means, it likely does not apply to you.
    /// This invariant will be enforced in the future.
    #[inline]
    pub fn deref<'a>(self, memory: &'a Memory) -> Option<WasmCell<'a, T>> {
        let end = (self.offset as usize).checked_add(mem::size_of::<T>())?;
        if end > memory.size().bytes().0 || mem::size_of::<T>() == 0 {
            return None;
        }

        unsafe {
            let cell_ptr = align_pointer(
                memory.view::<u8>().as_ptr().add(self.offset as usize) as usize,
                mem::align_of::<T>(),
            ) as *const Cell<T>;
            Some(WasmCell::new(&*cell_ptr))
        }
    }
}

/// Methods for `WasmPtr`s to arrays of data that can be dereferenced, namely to
/// types that implement [`ValueType`], meaning that they're valid for all
/// possible bit patterns.
impl<T: Copy + ValueType> WasmPtr<T, Array> {
    /// Dereference the `WasmPtr` getting access to a `&[Cell<T>]` allowing for
    /// reading and mutating of the inner values.
    ///
    /// This method is unsound if used with unsynchronized shared memory.
    /// If you're unsure what that means, it likely does not apply to you.
    /// This invariant will be enforced in the future.
    #[inline]
    pub fn deref<'a>(
        self,
        memory: &'a Memory,
        index: u32,
        length: u32,
    ) -> Option<Vec<WasmCell<'a, T>>> {
        // gets the size of the item in the array with padding added such that
        // for any index, we will always result an aligned memory access
        let item_size = mem::size_of::<T>();
        let slice_full_len = (index as usize).checked_add(length as usize)?;
        let memory_size = memory.size().bytes().0;
        let end = (self.offset as usize).checked_add(item_size.checked_mul(slice_full_len)?)?;
        if end > memory_size || item_size == 0 {
            return None;
        }

        let cell_ptrs = unsafe {
            let cell_ptr = align_pointer(
                memory.view::<u8>().as_ptr().add(self.offset as usize) as usize,
                mem::align_of::<T>(),
            ) as *const Cell<T>;
            &std::slice::from_raw_parts(cell_ptr, slice_full_len)[index as usize..slice_full_len]
        };

        let wasm_cells = cell_ptrs
            .iter()
            .map(|ptr| WasmCell::new(ptr))
            .collect::<Vec<_>>();
        Some(wasm_cells)
    }

    /// Get a UTF-8 string from the `WasmPtr` with the given length.
    ///
    /// Note that . The
    /// underlying data can be mutated if the Wasm is allowed to execute or
    /// an aliasing `WasmPtr` is used to mutate memory.
    ///
    /// # Safety
    /// This method returns a reference to Wasm linear memory. The underlying
    /// data can be mutated if the Wasm is allowed to execute or an aliasing
    /// `WasmPtr` is used to mutate memory.
    ///
    /// `str` has invariants that must not be broken by mutating Wasm memory.
    /// Thus the caller must ensure that the backing memory is not modified
    /// while the reference is held.
    ///
    /// Additionally, if `memory` is dynamic, the caller must also ensure that `memory`
    /// is not grown while the reference is held.
    pub unsafe fn get_utf8_str<'a>(self, memory: &'a Memory, str_len: u32) -> Option<&'a str> {
        let end = self.offset.checked_add(str_len)?;
        if end as usize > memory.size().bytes().0 {
            return None;
        }

        let ptr = memory.view::<u8>().as_ptr().add(self.offset as usize) as *const u8;
        let slice: &[u8] = std::slice::from_raw_parts(ptr, str_len as usize);
        std::str::from_utf8(slice).ok()
    }

    /// Get a UTF-8 `String` from the `WasmPtr` with the given length.
    ///
    /// an aliasing `WasmPtr` is used to mutate memory.
    pub fn get_utf8_string(self, memory: &Memory, str_len: u32) -> Option<String> {
        let end = self.offset.checked_add(str_len)?;
        if end as usize > memory.size().bytes().0 {
            return None;
        }

        // TODO: benchmark the internals of this function: there is likely room for
        // micro-optimization here and this may be a fairly common function in user code.
        let view = memory.view::<u8>();

        let mut vec: Vec<u8> = Vec::with_capacity(str_len as usize);
        let base = self.offset as usize;
        for i in 0..(str_len as usize) {
            let byte = view[base + i].get();
            vec.push(byte);
        }

        String::from_utf8(vec).ok()
    }

    /// Get a UTF-8 string from the `WasmPtr`, where the string is nul-terminated.
    ///
    /// Note that this does not account for UTF-8 strings that _contain_ nul themselves,
    /// [`WasmPtr::get_utf8_str`] has to be used for those.
    ///
    /// # Safety
    /// This method behaves similarly to [`WasmPtr::get_utf8_str`], all safety invariants on
    /// that method must also be upheld here.
    pub unsafe fn get_utf8_str_with_nul<'a>(self, memory: &'a Memory) -> Option<&'a str> {
        memory.view::<u8>()[(self.offset as usize)..]
            .iter()
            .map(|cell| cell.get())
            .position(|byte| byte == 0)
            .and_then(|length| self.get_utf8_str(memory, length as u32))
    }

    /// Get a UTF-8 `String` from the `WasmPtr`, where the string is nul-terminated.
    ///
    /// Note that this does not account for UTF-8 strings that _contain_ nul themselves,
    /// [`WasmPtr::get_utf8_string`] has to be used for those.
    pub fn get_utf8_string_with_nul(self, memory: &Memory) -> Option<String> {
        unsafe { self.get_utf8_str_with_nul(memory) }.map(|s| s.to_owned())
    }
}

unsafe impl<T: Copy, Ty> FromToNativeWasmType for WasmPtr<T, Ty> {
    type Native = i32;

    fn to_native(self) -> Self::Native {
        self.offset as i32
    }
    fn from_native(n: Self::Native) -> Self {
        Self {
            offset: n as u32,
            _phantom: PhantomData,
        }
    }
}

unsafe impl<T: Copy, Ty> ValueType for WasmPtr<T, Ty> {}

impl<T: Copy, Ty> Clone for WasmPtr<T, Ty> {
    fn clone(&self) -> Self {
        Self {
            offset: self.offset,
            _phantom: PhantomData,
        }
    }
}

impl<T: Copy, Ty> Copy for WasmPtr<T, Ty> {}

impl<T: Copy, Ty> PartialEq for WasmPtr<T, Ty> {
    fn eq(&self, other: &Self) -> bool {
        self.offset == other.offset
    }
}

impl<T: Copy, Ty> Eq for WasmPtr<T, Ty> {}

impl<T: Copy, Ty> fmt::Debug for WasmPtr<T, Ty> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "WasmPtr(offset: {}, pointer: {:#x}, align: {})",
            self.offset,
            self.offset,
            mem::align_of::<T>()
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::sys::{Memory, MemoryType, Store};

    /// Ensure that memory accesses work on the edges of memory and that out of
    /// bounds errors are caught with `deref`
    #[test]
    fn wasm_ptr_memory_bounds_checks_hold() {
        // create a memory
        let store = Store::default();
        let memory_descriptor = MemoryType::new(1, Some(1), false);
        let memory = Memory::new(&store, memory_descriptor).unwrap();

        // test that basic access works and that len = 0 works, but oob does not
        let start_wasm_ptr: WasmPtr<u8> = WasmPtr::new(0);
        let start_wasm_ptr_array: WasmPtr<u8, Array> = WasmPtr::new(0);

        assert!(start_wasm_ptr.deref(&memory).is_some());
        assert!(start_wasm_ptr_array.deref(&memory, 0, 0).is_some());
        assert!(unsafe { start_wasm_ptr_array.get_utf8_str(&memory, 0).is_some() });
        assert!(start_wasm_ptr_array.get_utf8_string(&memory, 0).is_some());
        assert!(start_wasm_ptr_array.deref(&memory, 0, 1).is_some());

        // test that accessing the last valid memory address works correctly and OOB is caught
        let last_valid_address_for_u8 = (memory.size().bytes().0 - 1) as u32;
        let end_wasm_ptr: WasmPtr<u8> = WasmPtr::new(last_valid_address_for_u8);
        assert!(end_wasm_ptr.deref(&memory).is_some());

        let end_wasm_ptr_array: WasmPtr<u8, Array> = WasmPtr::new(last_valid_address_for_u8);

        assert!(end_wasm_ptr_array.deref(&memory, 0, 1).is_some());
        let invalid_idx_len_combos: [(u32, u32); 3] =
            [(last_valid_address_for_u8 + 1, 0), (0, 2), (1, 1)];
        for &(idx, len) in invalid_idx_len_combos.iter() {
            assert!(end_wasm_ptr_array.deref(&memory, idx, len).is_none());
        }
        assert!(unsafe { end_wasm_ptr_array.get_utf8_str(&memory, 2).is_none() });
        assert!(end_wasm_ptr_array.get_utf8_string(&memory, 2).is_none());

        // test that accesing the last valid memory address for a u32 is valid
        // (same as above test but with more edge cases to assert on)
        let last_valid_address_for_u32 = (memory.size().bytes().0 - 4) as u32;
        let end_wasm_ptr: WasmPtr<u32> = WasmPtr::new(last_valid_address_for_u32);
        assert!(end_wasm_ptr.deref(&memory).is_some());
        assert!(end_wasm_ptr.deref(&memory).is_some());

        let end_wasm_ptr_oob_array: [WasmPtr<u32>; 4] = [
            WasmPtr::new(last_valid_address_for_u32 + 1),
            WasmPtr::new(last_valid_address_for_u32 + 2),
            WasmPtr::new(last_valid_address_for_u32 + 3),
            WasmPtr::new(last_valid_address_for_u32 + 4),
        ];
        for oob_end_ptr in end_wasm_ptr_oob_array.iter() {
            assert!(oob_end_ptr.deref(&memory).is_none());
        }
        let end_wasm_ptr_array: WasmPtr<u32, Array> = WasmPtr::new(last_valid_address_for_u32);
        assert!(end_wasm_ptr_array.deref(&memory, 0, 1).is_some());

        let invalid_idx_len_combos: [(u32, u32); 3] =
            [(last_valid_address_for_u32 + 1, 0), (0, 2), (1, 1)];
        for &(idx, len) in invalid_idx_len_combos.iter() {
            assert!(end_wasm_ptr_array.deref(&memory, idx, len).is_none());
        }

        let end_wasm_ptr_array_oob_array: [WasmPtr<u32, Array>; 4] = [
            WasmPtr::new(last_valid_address_for_u32 + 1),
            WasmPtr::new(last_valid_address_for_u32 + 2),
            WasmPtr::new(last_valid_address_for_u32 + 3),
            WasmPtr::new(last_valid_address_for_u32 + 4),
        ];

        for oob_end_array_ptr in end_wasm_ptr_array_oob_array.iter() {
            assert!(oob_end_array_ptr.deref(&memory, 0, 1).is_none());
            assert!(oob_end_array_ptr.deref(&memory, 1, 0).is_none());
        }
    }
}
