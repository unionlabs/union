//! Types for a reusable pointer abstraction for accessing Wasm linear memory.
//!
//! This abstraction is safe: it ensures the memory is in bounds and that the pointer
//! is aligned (avoiding undefined behavior).
//!
//! Therefore, you should use this abstraction whenever possible to avoid memory
//! related bugs when implementing an ABI.

use crate::js::cell::WasmCell;
use crate::js::{externals::Memory, FromToNativeWasmType};
use std::{fmt, marker::PhantomData, mem};
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
    pub fn deref<'a>(self, memory: &'a Memory) -> Option<WasmCell<T>> {
        let end = (self.offset as usize).checked_add(mem::size_of::<T>())?;
        if end > memory.size().bytes().0 || mem::size_of::<T>() == 0 {
            return None;
        }

        let subarray = memory.uint8view().subarray(self.offset, end as u32);
        Some(WasmCell::new(subarray))
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
    pub fn deref(self, memory: &Memory, index: u32, length: u32) -> Option<Vec<WasmCell<T>>> {
        // gets the size of the item in the array with padding added such that
        // for any index, we will always result an aligned memory access
        let item_size = mem::size_of::<T>() as u32;
        let slice_full_len = index.checked_add(length)?;
        let memory_size = memory.size().bytes().0 as u32;
        let end = self
            .offset
            .checked_add(item_size.checked_mul(slice_full_len)?)?;
        if end > memory_size || item_size == 0 {
            return None;
        }

        Some(
            (0..length)
                .map(|i| {
                    let subarray = memory.uint8view().subarray(
                        self.offset + i * item_size,
                        self.offset + (i + 1) * item_size,
                    );
                    WasmCell::new(subarray)
                })
                .collect::<Vec<_>>(),
        )
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
    pub unsafe fn get_utf8_str<'a>(
        self,
        memory: &'a Memory,
        str_len: u32,
    ) -> Option<std::borrow::Cow<'a, str>> {
        self.get_utf8_string(memory, str_len)
            .map(std::borrow::Cow::from)
    }

    /// Get a UTF-8 `String` from the `WasmPtr` with the given length.
    ///
    /// an aliasing `WasmPtr` is used to mutate memory.
    pub fn get_utf8_string(self, memory: &Memory, str_len: u32) -> Option<String> {
        let end = self.offset.checked_add(str_len)?;
        if end as usize > memory.size().bytes().0 {
            return None;
        }

        let view = memory.uint8view();
        // let subarray_as_vec = view.subarray(self.offset, str_len + 1).to_vec();

        let mut subarray_as_vec: Vec<u8> = Vec::with_capacity(str_len as usize);
        let base = self.offset;
        for i in 0..(str_len) {
            let byte = view.get_index(base + i);
            subarray_as_vec.push(byte);
        }

        String::from_utf8(subarray_as_vec).ok()
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
    use crate::js::{Memory, MemoryType, Store};
    use wasm_bindgen_test::*;

    /// Ensure that memory accesses work on the edges of memory and that out of
    /// bounds errors are caught with both `deref` and `deref_mut`.
    #[wasm_bindgen_test]
    fn wasm_ptr_is_functional() {
        let store = Store::default();
        let memory_descriptor = MemoryType::new(1, Some(1), false);
        let memory = Memory::new(&store, memory_descriptor).unwrap();

        let start_wasm_ptr: WasmPtr<u64> = WasmPtr::new(2);
        let val = start_wasm_ptr.deref(&memory).unwrap();
        assert_eq!(val.memory.to_vec(), vec![0; 8]);

        val.set(1200);

        assert_eq!(val.memory.to_vec(), vec![176, 4, 0, 0, 0, 0, 0, 0]);
        // Let's make sure the main memory is changed
        assert_eq!(
            memory.uint8view().subarray(0, 10).to_vec(),
            vec![0, 0, 176, 4, 0, 0, 0, 0, 0, 0]
        );

        val.memory.copy_from(&[10, 0, 0, 0, 0, 0, 0, 0]);

        let value = val.get();
        assert_eq!(value, 10);
    }

    /// Ensure that memory accesses work on the edges of memory and that out of
    /// bounds errors are caught with both `deref` and `deref_mut`.
    #[wasm_bindgen_test]
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
