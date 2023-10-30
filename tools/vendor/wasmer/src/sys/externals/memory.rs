use crate::sys::exports::{ExportError, Exportable};
use crate::sys::externals::Extern;
use crate::sys::store::Store;
use crate::sys::{MemoryType, MemoryView};
use loupe::MemoryUsage;
use std::convert::TryInto;
use std::slice;
use std::sync::Arc;
use wasmer_engine::Export;
use wasmer_types::{Pages, ValueType};
use wasmer_vm::{MemoryError, VMMemory};

/// A WebAssembly `memory` instance.
///
/// A memory instance is the runtime representation of a linear memory.
/// It consists of a vector of bytes and an optional maximum size.
///
/// The length of the vector always is a multiple of the WebAssembly
/// page size, which is defined to be the constant 65536 – abbreviated 64Ki.
/// Like in a memory type, the maximum size in a memory instance is
/// given in units of this page size.
///
/// A memory created by the host or in WebAssembly code will be accessible and
/// mutable from both host and WebAssembly.
///
/// Spec: <https://webassembly.github.io/spec/core/exec/runtime.html#memory-instances>
#[derive(Debug, MemoryUsage)]
pub struct Memory {
    store: Store,
    vm_memory: VMMemory,
}

impl Memory {
    /// Creates a new host `Memory` from the provided [`MemoryType`].
    ///
    /// This function will construct the `Memory` using the store
    /// [`BaseTunables`][crate::sys::BaseTunables].
    ///
    /// # Example
    ///
    /// ```
    /// # use wasmer::{Memory, MemoryType, Pages, Store, Type, Value};
    /// # let store = Store::default();
    /// #
    /// let m = Memory::new(&store, MemoryType::new(1, None, false)).unwrap();
    /// ```
    pub fn new(store: &Store, ty: MemoryType) -> Result<Self, MemoryError> {
        let tunables = store.tunables();
        let style = tunables.memory_style(&ty);
        let memory = tunables.create_host_memory(&ty, &style)?;

        Ok(Self {
            store: store.clone(),
            vm_memory: VMMemory {
                from: memory,
                // We are creating it from the host, and therefore there is no
                // associated instance with this memory
                instance_ref: None,
            },
        })
    }

    /// Returns the [`MemoryType`] of the `Memory`.
    ///
    /// # Example
    ///
    /// ```
    /// # use wasmer::{Memory, MemoryType, Pages, Store, Type, Value};
    /// # let store = Store::default();
    /// #
    /// let mt = MemoryType::new(1, None, false);
    /// let m = Memory::new(&store, mt).unwrap();
    ///
    /// assert_eq!(m.ty(), mt);
    /// ```
    pub fn ty(&self) -> MemoryType {
        self.vm_memory.from.ty()
    }

    /// Returns the [`Store`] where the `Memory` belongs.
    ///
    /// # Example
    ///
    /// ```
    /// # use wasmer::{Memory, MemoryType, Pages, Store, Type, Value};
    /// # let store = Store::default();
    /// #
    /// let m = Memory::new(&store, MemoryType::new(1, None, false)).unwrap();
    ///
    /// assert_eq!(m.store(), &store);
    /// ```
    pub fn store(&self) -> &Store {
        &self.store
    }

    /// Retrieve a slice of the memory contents.
    ///
    /// # Safety
    ///
    /// Until the returned slice is dropped, it is undefined behaviour to
    /// modify the memory contents in any way including by calling a wasm
    /// function that writes to the memory or by resizing the memory.
    pub unsafe fn data_unchecked(&self) -> &[u8] {
        self.data_unchecked_mut()
    }

    /// Retrieve a mutable slice of the memory contents.
    ///
    /// # Safety
    ///
    /// This method provides interior mutability without an UnsafeCell. Until
    /// the returned value is dropped, it is undefined behaviour to read or
    /// write to the pointed-to memory in any way except through this slice,
    /// including by calling a wasm function that reads the memory contents or
    /// by resizing this Memory.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn data_unchecked_mut(&self) -> &mut [u8] {
        let definition = self.vm_memory.from.vmmemory();
        let def = definition.as_ref();
        slice::from_raw_parts_mut(def.base, def.current_length.try_into().unwrap())
    }

    /// Returns the pointer to the raw bytes of the `Memory`.
    pub fn data_ptr(&self) -> *mut u8 {
        let definition = self.vm_memory.from.vmmemory();
        let def = unsafe { definition.as_ref() };
        def.base
    }

    /// Returns the size (in bytes) of the `Memory`.
    pub fn data_size(&self) -> u64 {
        let definition = self.vm_memory.from.vmmemory();
        let def = unsafe { definition.as_ref() };
        def.current_length.try_into().unwrap()
    }

    /// Returns the size (in [`Pages`]) of the `Memory`.
    ///
    /// # Example
    ///
    /// ```
    /// # use wasmer::{Memory, MemoryType, Pages, Store, Type, Value};
    /// # let store = Store::default();
    /// #
    /// let m = Memory::new(&store, MemoryType::new(1, None, false)).unwrap();
    ///
    /// assert_eq!(m.size(), Pages(1));
    /// ```
    pub fn size(&self) -> Pages {
        self.vm_memory.from.size()
    }

    /// Grow memory by the specified amount of WebAssembly [`Pages`] and return
    /// the previous memory size.
    ///
    /// # Example
    ///
    /// ```
    /// # use wasmer::{Memory, MemoryType, Pages, Store, Type, Value, WASM_MAX_PAGES};
    /// # let store = Store::default();
    /// #
    /// let m = Memory::new(&store, MemoryType::new(1, Some(3), false)).unwrap();
    /// let p = m.grow(2).unwrap();
    ///
    /// assert_eq!(p, Pages(1));
    /// assert_eq!(m.size(), Pages(3));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if memory can't be grown by the specified amount
    /// of pages.
    ///
    /// ```should_panic
    /// # use wasmer::{Memory, MemoryType, Pages, Store, Type, Value, WASM_MAX_PAGES};
    /// # let store = Store::default();
    /// #
    /// let m = Memory::new(&store, MemoryType::new(1, Some(1), false)).unwrap();
    ///
    /// // This results in an error: `MemoryError::CouldNotGrow`.
    /// let s = m.grow(1).unwrap();
    /// ```
    pub fn grow<IntoPages>(&self, delta: IntoPages) -> Result<Pages, MemoryError>
    where
        IntoPages: Into<Pages>,
    {
        self.vm_memory.from.grow(delta.into())
    }

    /// Return a "view" of the currently accessible memory. By
    /// default, the view is unsynchronized, using regular memory
    /// accesses. You can force a memory view to use atomic accesses
    /// by calling the [`MemoryView::atomically`] method.
    ///
    /// # Notes:
    ///
    /// This method is safe (as in, it won't cause the host to crash or have UB),
    /// but it doesn't obey rust's rules involving data races, especially concurrent ones.
    /// Therefore, if this memory is shared between multiple threads, a single memory
    /// location can be mutated concurrently without synchronization.
    ///
    /// # Usage:
    ///
    /// ```
    /// # use wasmer::{Memory, MemoryView};
    /// # use std::{cell::Cell, sync::atomic::Ordering};
    /// # fn view_memory(memory: Memory) {
    /// // Without synchronization.
    /// let view: MemoryView<u8> = memory.view();
    /// for byte in view[0x1000 .. 0x1010].iter().map(Cell::get) {
    ///     println!("byte: {}", byte);
    /// }
    ///
    /// // With synchronization.
    /// let atomic_view = view.atomically();
    /// for byte in atomic_view[0x1000 .. 0x1010].iter().map(|atom| atom.load(Ordering::SeqCst)) {
    ///     println!("byte: {}", byte);
    /// }
    /// # }
    /// ```
    pub fn view<T: ValueType>(&self) -> MemoryView<T> {
        let base = self.data_ptr();

        let length = self.size().bytes().0 / std::mem::size_of::<T>();

        unsafe { MemoryView::new(base as _, length as u32) }
    }

    /// A shortcut to [`Self::view::<u8>`][self::view].
    ///
    /// This code is going to be refactored. Use it as your own risks.
    #[doc(hidden)]
    pub fn uint8view(&self) -> MemoryView<u8> {
        self.view()
    }

    pub(crate) fn from_vm_export(store: &Store, vm_memory: VMMemory) -> Self {
        Self {
            store: store.clone(),
            vm_memory,
        }
    }

    /// Returns whether or not these two memories refer to the same data.
    ///
    /// # Example
    ///
    /// ```
    /// # use wasmer::{Memory, MemoryType, Store, Value};
    /// # let store = Store::default();
    /// #
    /// let m = Memory::new(&store, MemoryType::new(1, None, false)).unwrap();
    ///
    /// assert!(m.same(&m));
    /// ```
    pub fn same(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.vm_memory.from, &other.vm_memory.from)
    }

    /// Get access to the backing VM value for this extern. This function is for
    /// tests it should not be called by users of the Wasmer API.
    ///
    /// # Safety
    /// This function is unsafe to call outside of tests for the wasmer crate
    /// because there is no stability guarantee for the returned type and we may
    /// make breaking changes to it at any time or remove this method.
    #[doc(hidden)]
    pub unsafe fn get_vm_memory(&self) -> &VMMemory {
        &self.vm_memory
    }
}

impl Clone for Memory {
    fn clone(&self) -> Self {
        let mut vm_memory = self.vm_memory.clone();
        vm_memory.upgrade_instance_ref().unwrap();

        Self {
            store: self.store.clone(),
            vm_memory,
        }
    }
}

impl<'a> Exportable<'a> for Memory {
    fn to_export(&self) -> Export {
        self.vm_memory.clone().into()
    }

    fn get_self_from_extern(_extern: &'a Extern) -> Result<&'a Self, ExportError> {
        match _extern {
            Extern::Memory(memory) => Ok(memory),
            _ => Err(ExportError::IncompatibleType),
        }
    }

    fn into_weak_instance_ref(&mut self) {
        self.vm_memory
            .instance_ref
            .as_mut()
            .map(|v| *v = v.downgrade());
    }
}
