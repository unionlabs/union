use crate::js::instance::Instance;
use crate::js::wasm_bindgen_polyfill::Global;
use crate::js::HostEnvInitError;
use crate::js::WasmerEnv;
use js_sys::Function;
use js_sys::WebAssembly::{Memory, Table};
use std::cell::RefCell;
use std::fmt;
use std::sync::Arc;
use wasm_bindgen::{JsCast, JsValue};
use wasmer_types::{ExternType, FunctionType, GlobalType, MemoryType, TableType};

#[derive(Clone, Debug, PartialEq)]
pub struct VMMemory {
    pub(crate) memory: Memory,
    pub(crate) ty: MemoryType,
}

unsafe impl Send for VMMemory {}
unsafe impl Sync for VMMemory {}

impl VMMemory {
    pub(crate) fn new(memory: Memory, ty: MemoryType) -> Self {
        Self { memory, ty }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VMGlobal {
    pub(crate) global: Global,
    pub(crate) ty: GlobalType,
}

impl VMGlobal {
    pub(crate) fn new(global: Global, ty: GlobalType) -> Self {
        Self { global, ty }
    }
}

unsafe impl Send for VMGlobal {}
unsafe impl Sync for VMGlobal {}

#[derive(Clone, Debug, PartialEq)]
pub struct VMTable {
    pub(crate) table: Table,
    pub(crate) ty: TableType,
}

unsafe impl Send for VMTable {}
unsafe impl Sync for VMTable {}

impl VMTable {
    pub(crate) fn new(table: Table, ty: TableType) -> Self {
        Self { table, ty }
    }
}

#[derive(Clone)]
pub struct VMFunction {
    pub(crate) function: Function,
    pub(crate) ty: FunctionType,
    pub(crate) environment: Option<Arc<RefCell<Box<dyn WasmerEnv>>>>,
}

unsafe impl Send for VMFunction {}
unsafe impl Sync for VMFunction {}

impl VMFunction {
    pub(crate) fn new(
        function: Function,
        ty: FunctionType,
        environment: Option<Box<dyn WasmerEnv>>,
    ) -> Self {
        Self {
            function,
            ty,
            environment: environment.map(|env| Arc::new(RefCell::new(env))),
        }
    }

    pub(crate) fn init_envs(&self, instance: &Instance) -> Result<(), HostEnvInitError> {
        if let Some(env) = &self.environment {
            let mut borrowed_env = env.borrow_mut();
            borrowed_env.init_with_instance(instance)?;
        }
        Ok(())
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
#[derive(Debug, Clone)]
pub enum Export {
    /// A function export value.
    Function(VMFunction),

    /// A table export value.
    Table(VMTable),

    /// A memory export value.
    Memory(VMMemory),

    /// A global export value.
    Global(VMGlobal),
}

impl Export {
    /// Return the export as a `JSValue`.
    pub fn as_jsvalue(&self) -> &JsValue {
        match self {
            Export::Memory(js_wasm_memory) => js_wasm_memory.memory.as_ref(),
            Export::Function(js_func) => js_func.function.as_ref(),
            Export::Table(js_wasm_table) => js_wasm_table.table.as_ref(),
            Export::Global(js_wasm_global) => js_wasm_global.global.as_ref(),
        }
    }
}

impl From<(JsValue, ExternType)> for Export {
    fn from((val, extern_type): (JsValue, ExternType)) -> Export {
        match extern_type {
            ExternType::Memory(memory_type) => {
                if val.is_instance_of::<Memory>() {
                    return Export::Memory(VMMemory::new(
                        val.unchecked_into::<Memory>(),
                        memory_type,
                    ));
                } else {
                    panic!("Extern type doesn't match js value type");
                }
            }
            ExternType::Global(global_type) => {
                if val.is_instance_of::<Global>() {
                    return Export::Global(VMGlobal::new(
                        val.unchecked_into::<Global>(),
                        global_type,
                    ));
                } else {
                    panic!("Extern type doesn't match js value type");
                }
            }
            ExternType::Function(function_type) => {
                if val.is_instance_of::<Function>() {
                    return Export::Function(VMFunction::new(
                        val.unchecked_into::<Function>(),
                        function_type,
                        None,
                    ));
                } else {
                    panic!("Extern type doesn't match js value type");
                }
            }
            ExternType::Table(table_type) => {
                if val.is_instance_of::<Table>() {
                    return Export::Table(VMTable::new(val.unchecked_into::<Table>(), table_type));
                } else {
                    panic!("Extern type doesn't match js value type");
                }
            }
        }
    }
}
