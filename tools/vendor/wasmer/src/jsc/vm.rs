/// This module is mainly used to create the `VM` types that will hold both
/// the JS values of the `Memory`, `Table`, `Global` and `Function` and also
/// it's types.
/// This module should not be needed any longer (with the exception of the memory)
/// once the type reflection is added to the WebAssembly JS API.
/// https://github.com/WebAssembly/js-types/
use crate::store::AsStoreRef;
use rusty_jsc::{JSObject, JSObjectCallAsFunctionCallback, JSValue};
use std::any::Any;
use std::fmt;
#[cfg(feature = "tracing")]
use tracing::trace;
use wasmer_types::RawValue;
use wasmer_types::{
    FunctionType, GlobalType, MemoryError, MemoryType, Pages, TableType, WASM_PAGE_SIZE,
};

/// Represents linear memory that is managed by the javascript runtime
#[derive(Clone, Debug, PartialEq)]
pub struct VMMemory {
    pub(crate) memory: JSObject,
    pub(crate) ty: MemoryType,
}

unsafe impl Send for VMMemory {}
unsafe impl Sync for VMMemory {}

impl VMMemory {
    /// Creates a new memory directly from a WebAssembly javascript object
    pub fn new(memory: JSObject, ty: MemoryType) -> Self {
        Self { memory, ty }
    }

    /// Returns the size of the memory buffer in pages
    pub fn get_runtime_size(&self) -> u32 {
        unimplemented!();
        // let dummy: DummyBuffer = match serde_wasm_bindgen::from_value(self.memory.buffer()) {
        //     Ok(o) => o,
        //     Err(_) => return 0,
        // };
        // if dummy.byte_length == 0 {
        //     return 0;
        // }
        // dummy.byte_length / WASM_PAGE_SIZE as u32
    }

    /// Attempts to clone this memory (if its clonable)
    pub(crate) fn try_clone(&self) -> Result<VMMemory, MemoryError> {
        Ok(self.clone())
    }

    /// Copies this memory to a new memory
    #[deprecated = "use `copy` instead"]
    pub fn duplicate(
        &mut self,
        store: &impl AsStoreRef,
    ) -> Result<VMMemory, wasmer_types::MemoryError> {
        self.copy(store)
    }

    /// Copies this memory to a new memory
    pub fn copy(&self, store: &impl AsStoreRef) -> Result<VMMemory, wasmer_types::MemoryError> {
        let new_memory =
            crate::jsc::externals::memory::Memory::js_memory_from_type(&store, &self.ty)?;

        #[cfg(feature = "tracing")]
        trace!("memory copy started");

        let src = crate::jsc::externals::memory_view::MemoryView::new_raw(&self.memory, store);
        let amount = src.data_size() as usize;
        let mut dst = crate::jsc::externals::memory_view::MemoryView::new_raw(&new_memory, store);
        let dst_size = dst.data_size() as usize;

        // if amount > dst_size {
        //     let delta = amount - dst_size;
        //     let pages = ((delta - 1) / WASM_PAGE_SIZE) + 1;

        //     let our_js_memory: &crate::jsc::externals::memory::JSMemory =
        //         JsCast::unchecked_from_js_ref(&new_memory);
        //     our_js_memory.grow(pages as u32).map_err(|err| {
        //         if err.is_instance_of::<js_sys::RangeError>() {
        //             let cur_pages = dst_size;
        //             MemoryError::CouldNotGrow {
        //                 current: Pages(cur_pages as u32),
        //                 attempted_delta: Pages(pages as u32),
        //             }
        //         } else {
        //             MemoryError::Generic(err.as_string().unwrap())
        //         }
        //     })?;

        //     dst = crate::jsc::externals::memory_view::MemoryView::new_raw(&new_memory);
        // }

        src.copy_to_memory(amount as u64, &dst).map_err(|err| {
            wasmer_types::MemoryError::Generic(format!("failed to copy the memory - {}", err))
        })?;

        #[cfg(feature = "tracing")]
        trace!("memory copy finished (size={})", dst.size().bytes().0);

        Ok(Self {
            memory: new_memory,
            ty: self.ty.clone(),
        })
    }
}

// impl From<VMMemory> for JSValue {
//     fn from(value: VMMemory) -> Self {
//         JSValue::from(value.memory)
//     }
// }

/// The shared memory is the same as the normal memory
pub type VMSharedMemory = VMMemory;

/// The VM Global type
#[derive(Clone, Debug, PartialEq)]
pub struct VMGlobal {
    pub(crate) global: JSObject,
    pub(crate) ty: GlobalType,
}

impl VMGlobal {
    pub(crate) fn new(global: JSObject, ty: GlobalType) -> Self {
        Self { global, ty }
    }
}

unsafe impl Send for VMGlobal {}
unsafe impl Sync for VMGlobal {}

/// The VM Table type
#[derive(Clone, Debug, PartialEq)]
pub struct VMTable {
    pub(crate) table: JSObject,
    pub(crate) ty: TableType,
}

unsafe impl Send for VMTable {}
unsafe impl Sync for VMTable {}

impl VMTable {
    pub(crate) fn new(table: JSObject, ty: TableType) -> Self {
        Self { table, ty }
    }

    /// Get the table size at runtime
    pub fn get_runtime_size(&self) -> u32 {
        unimplemented!();
        // self.table.length()
    }
}

/// The VM Function type
#[derive(Clone)]
pub struct VMFunction {
    pub(crate) function: JSObject,
    pub(crate) ty: FunctionType,
}

unsafe impl Send for VMFunction {}
unsafe impl Sync for VMFunction {}

impl VMFunction {
    pub(crate) fn new(function: JSObject, ty: FunctionType) -> Self {
        Self { function, ty }
    }
}

impl PartialEq for VMFunction {
    fn eq(&self, other: &Self) -> bool {
        self.function == other.function
    }
}

impl fmt::Debug for VMFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VMFunction")
            .field("function", &self.function)
            .finish()
    }
}

/// The value of an export passed from one instance to another.
pub enum VMExtern {
    /// A function export value.
    Function(VMFunction),

    /// A table export value.
    Table(VMTable),

    /// A memory export value.
    Memory(VMMemory),

    /// A global export value.
    Global(VMGlobal),
}

pub type VMInstance = JSObject;

/// Underlying FunctionEnvironment used by a `VMFunction`.
#[derive(Debug)]
pub struct VMFunctionEnvironment {
    contents: Box<dyn Any + Send + 'static>,
}

impl VMFunctionEnvironment {
    /// Wraps the given value to expose it to Wasm code as a function context.
    pub fn new(val: impl Any + Send + 'static) -> Self {
        Self {
            contents: Box::new(val),
        }
    }

    #[allow(clippy::should_implement_trait)]
    /// Returns a reference to the underlying value.
    pub fn as_ref(&self) -> &(dyn Any + Send + 'static) {
        &*self.contents
    }

    #[allow(clippy::should_implement_trait)]
    /// Returns a mutable reference to the underlying value.
    pub fn as_mut(&mut self) -> &mut (dyn Any + Send + 'static) {
        &mut *self.contents
    }
}

pub(crate) struct VMExternRef;

impl VMExternRef {
    /// Converts the `VMExternRef` into a `RawValue`.
    pub fn into_raw(self) -> RawValue {
        unimplemented!();
    }

    /// Extracts a `VMExternRef` from a `RawValue`.
    ///
    /// # Safety
    /// `raw` must be a valid `VMExternRef` instance.
    pub unsafe fn from_raw(_raw: RawValue) -> Option<Self> {
        unimplemented!();
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct VMFuncRef;

impl VMFuncRef {
    /// Converts the `VMFuncRef` into a `RawValue`.
    pub fn into_raw(self) -> RawValue {
        unimplemented!();
    }

    /// Extracts a `VMFuncRef` from a `RawValue`.
    ///
    /// # Safety
    /// `raw.funcref` must be a valid pointer.
    pub unsafe fn from_raw(_raw: RawValue) -> Option<Self> {
        unimplemented!();
    }
}

pub struct VMTrampoline;

pub(crate) type VMExternTable = VMTable;
pub(crate) type VMExternMemory = VMMemory;
pub(crate) type VMExternGlobal = VMGlobal;
pub(crate) type VMExternFunction = VMFunction;

pub type VMFunctionCallback = JSObjectCallAsFunctionCallback;
