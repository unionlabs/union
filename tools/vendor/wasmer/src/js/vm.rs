/// This module is mainly used to create the `VM` types that will hold both
/// the JS values of the `Memory`, `Table`, `Global` and `Function` and also
/// it's types.
/// This module should not be needed any longer (with the exception of the memory)
/// once the type reflection is added to the WebAssembly JS API.
/// https://github.com/WebAssembly/js-types/
use crate::js::{js_handle::JsHandle, wasm_bindgen_polyfill::Global as JsGlobal};
use js_sys::Function as JsFunction;
use js_sys::WebAssembly;
use js_sys::WebAssembly::{Memory as JsMemory, Table as JsTable};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::fmt;
#[cfg(feature = "tracing")]
use tracing::trace;
use wasm_bindgen::{JsCast, JsValue};
use wasmer_types::RawValue;
use wasmer_types::{
    FunctionType, GlobalType, MemoryError, MemoryType, Pages, TableType, WASM_PAGE_SIZE,
};

/// Represents linear memory that is managed by the javascript runtime
#[derive(Clone, Debug, PartialEq)]
pub struct VMMemory {
    pub(crate) memory: JsHandle<JsMemory>,
    pub(crate) ty: MemoryType,
}

unsafe impl Send for VMMemory {}
unsafe impl Sync for VMMemory {}

#[derive(Serialize, Deserialize)]
struct DummyBuffer {
    #[serde(rename = "byteLength")]
    byte_length: u32,
}

impl VMMemory {
    /// Creates a new memory directly from a WebAssembly javascript object
    pub fn new(memory: JsMemory, ty: MemoryType) -> Self {
        Self {
            memory: JsHandle::new(memory),
            ty,
        }
    }

    /// Returns the size of the memory buffer in pages
    pub fn get_runtime_size(&self) -> u32 {
        let dummy: DummyBuffer = match serde_wasm_bindgen::from_value(self.memory.buffer()) {
            Ok(o) => o,
            Err(_) => return 0,
        };
        if dummy.byte_length == 0 {
            return 0;
        }
        dummy.byte_length / WASM_PAGE_SIZE as u32
    }

    /// Attempts to clone this memory (if its clonable)
    pub(crate) fn try_clone(&self) -> Result<VMMemory, MemoryError> {
        Ok(self.clone())
    }

    /// Copies this memory to a new memory
    #[deprecated = "use `copy` instead"]
    pub fn duplicate(&mut self) -> Result<VMMemory, wasmer_types::MemoryError> {
        self.copy()
    }

    /// Copies this memory to a new memory
    pub fn copy(&mut self) -> Result<VMMemory, wasmer_types::MemoryError> {
        let new_memory = crate::js::externals::memory::Memory::js_memory_from_type(&self.ty)?;

        let src = crate::js::externals::memory_view::MemoryView::new_raw(&self.memory);
        let amount = src.data_size() as usize;

        #[cfg(feature = "tracing")]
        trace!(%amount, "memory copy started");

        let mut dst = crate::js::externals::memory_view::MemoryView::new_raw(&new_memory);
        let dst_size = dst.data_size() as usize;

        if amount > dst_size {
            let delta = amount - dst_size;
            let pages = ((delta - 1) / WASM_PAGE_SIZE) + 1;

            let our_js_memory: &crate::js::externals::memory::JSMemory =
                JsCast::unchecked_from_js_ref(&new_memory);
            our_js_memory.grow(pages as u32).map_err(|err| {
                if err.is_instance_of::<js_sys::RangeError>() {
                    let cur_pages = dst_size;
                    MemoryError::CouldNotGrow {
                        current: Pages(cur_pages as u32),
                        attempted_delta: Pages(pages as u32),
                    }
                } else {
                    MemoryError::Generic(err.as_string().unwrap())
                }
            })?;

            dst = crate::js::externals::memory_view::MemoryView::new_raw(&new_memory);
        }

        src.copy_to_memory(amount as u64, &dst).map_err(|err| {
            wasmer_types::MemoryError::Generic(format!("failed to copy the memory - {}", err))
        })?;

        #[cfg(feature = "tracing")]
        trace!("memory copy finished (size={})", dst.size().bytes().0);

        Ok(Self {
            memory: JsHandle::new(new_memory),
            ty: self.ty.clone(),
        })
    }
}

impl From<VMMemory> for JsValue {
    fn from(value: VMMemory) -> Self {
        JsValue::from(value.memory)
    }
}

impl From<VMMemory> for (JsValue, MemoryType) {
    fn from(value: VMMemory) -> Self {
        (JsValue::from(value.memory), value.ty)
    }
}

/// The shared memory is the same as the normal memory
pub type VMSharedMemory = VMMemory;

/// The VM Global type
#[derive(Clone, Debug, PartialEq)]
pub struct VMGlobal {
    pub(crate) global: JsGlobal,
    pub(crate) ty: GlobalType,
}

impl VMGlobal {
    pub(crate) fn new(global: JsGlobal, ty: GlobalType) -> Self {
        Self { global, ty }
    }
}

unsafe impl Send for VMGlobal {}
unsafe impl Sync for VMGlobal {}

/// The VM Table type
#[derive(Clone, Debug, PartialEq)]
pub struct VMTable {
    pub(crate) table: JsTable,
    pub(crate) ty: TableType,
}

unsafe impl Send for VMTable {}
unsafe impl Sync for VMTable {}

impl VMTable {
    pub(crate) fn new(table: JsTable, ty: TableType) -> Self {
        Self { table, ty }
    }

    /// Get the table size at runtime
    pub fn get_runtime_size(&self) -> u32 {
        self.table.length()
    }
}

/// The VM Function type
#[derive(Clone)]
pub struct VMFunction {
    pub(crate) function: JsHandle<JsFunction>,
    pub(crate) ty: FunctionType,
}

unsafe impl Send for VMFunction {}
unsafe impl Sync for VMFunction {}

impl VMFunction {
    pub(crate) fn new(function: JsFunction, ty: FunctionType) -> Self {
        Self {
            function: JsHandle::new(function),
            ty,
        }
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

pub type VMInstance = WebAssembly::Instance;

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

#[repr(C)]
pub struct VMFunctionBody(u8);

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

pub type VMFunctionCallback = *const VMFunctionBody;
