use crate::sys::externals::Function;
use crate::sys::store::{Store, StoreObject};
use crate::sys::RuntimeError;
use wasmer_types::Value;
pub use wasmer_types::{
    ExportType, ExternType, FunctionType, GlobalType, ImportType, MemoryType, Mutability,
    TableType, Type as ValType,
};
use wasmer_vm::VMFuncRef;

/// WebAssembly computations manipulate values of basic value types:
/// * Integers (32 or 64 bit width)
/// * Floating-point (32 or 64 bit width)
/// * Vectors (128 bits, with 32 or 64 bit lanes)
///
/// Spec: <https://webassembly.github.io/spec/core/exec/runtime.html#values>
pub type Val = Value<Function>;

impl StoreObject for Val {
    fn comes_from_same_store(&self, store: &Store) -> bool {
        match self {
            Self::FuncRef(None) => true,
            Self::FuncRef(Some(f)) => Store::same(store, f.store()),
            // `ExternRef`s are not tied to specific stores
            Self::ExternRef(_) => true,
            Self::I32(_) | Self::I64(_) | Self::F32(_) | Self::F64(_) | Self::V128(_) => true,
        }
    }
}

impl From<Function> for Val {
    fn from(val: Function) -> Self {
        Self::FuncRef(Some(val))
    }
}

/// It provides useful functions for converting back and forth
/// from [`Val`] into `FuncRef`.
pub trait ValFuncRef {
    fn into_vm_funcref(&self, store: &Store) -> Result<VMFuncRef, RuntimeError>;

    fn from_vm_funcref(item: VMFuncRef, store: &Store) -> Self;

    fn into_table_reference(&self, store: &Store) -> Result<wasmer_vm::TableElement, RuntimeError>;

    fn from_table_reference(item: wasmer_vm::TableElement, store: &Store) -> Self;
}

impl ValFuncRef for Val {
    fn into_vm_funcref(&self, store: &Store) -> Result<VMFuncRef, RuntimeError> {
        if !self.comes_from_same_store(store) {
            return Err(RuntimeError::new("cross-`Store` values are not supported"));
        }
        Ok(match self {
            Self::FuncRef(None) => VMFuncRef::null(),
            Self::FuncRef(Some(f)) => f.vm_funcref(),
            _ => return Err(RuntimeError::new("val is not func ref")),
        })
    }

    fn from_vm_funcref(func_ref: VMFuncRef, store: &Store) -> Self {
        if func_ref.is_null() {
            return Self::FuncRef(None);
        }
        let item: &wasmer_vm::VMCallerCheckedAnyfunc = unsafe {
            let anyfunc: *const wasmer_vm::VMCallerCheckedAnyfunc = *func_ref;
            &*anyfunc
        };
        let signature = store
            .engine()
            .lookup_signature(item.type_index)
            .expect("Signature not found in store");
        let export = wasmer_engine::ExportFunction {
            // TODO:
            // figure out if we ever need a value here: need testing with complicated import patterns
            metadata: None,
            vm_function: wasmer_vm::VMFunction {
                address: item.func_ptr,
                signature,
                // TODO: review this comment (unclear if it's still correct):
                // All functions in tables are already Static (as dynamic functions
                // are converted to use the trampolines with static signatures).
                kind: wasmer_vm::VMFunctionKind::Static,
                vmctx: item.vmctx,
                call_trampoline: None,
                instance_ref: None,
            },
        };
        let f = Function::from_vm_export(store, export);
        Self::FuncRef(Some(f))
    }

    fn into_table_reference(&self, store: &Store) -> Result<wasmer_vm::TableElement, RuntimeError> {
        if !self.comes_from_same_store(store) {
            return Err(RuntimeError::new("cross-`Store` values are not supported"));
        }
        Ok(match self {
            // TODO(reftypes): review this clone
            Self::ExternRef(extern_ref) => wasmer_vm::TableElement::ExternRef(extern_ref.clone()),
            Self::FuncRef(None) => wasmer_vm::TableElement::FuncRef(VMFuncRef::null()),
            Self::FuncRef(Some(f)) => wasmer_vm::TableElement::FuncRef(f.vm_funcref()),
            _ => return Err(RuntimeError::new("val is not reference")),
        })
    }

    fn from_table_reference(item: wasmer_vm::TableElement, store: &Store) -> Self {
        match item {
            wasmer_vm::TableElement::FuncRef(f) => Self::from_vm_funcref(f, store),
            wasmer_vm::TableElement::ExternRef(extern_ref) => Self::ExternRef(extern_ref),
        }
    }
}
