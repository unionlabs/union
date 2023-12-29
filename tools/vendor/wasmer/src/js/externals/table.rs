use crate::errors::RuntimeError;
use crate::store::{AsStoreMut, AsStoreRef};
use crate::value::Value;
use crate::vm::VMExternTable;
use crate::vm::{VMExtern, VMFunction, VMTable};
use crate::{FunctionType, TableType};
use js_sys::Function;

#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    pub(crate) handle: VMTable,
}

// Table can't be Send in js because it dosen't support `structuredClone`
// https://developer.mozilla.org/en-US/docs/Web/API/structuredClone
// unsafe impl Send for Table {}

fn set_table_item(table: &VMTable, item_index: u32, item: &Function) -> Result<(), RuntimeError> {
    table.table.set(item_index, item).map_err(|e| e.into())
}

fn get_function(store: &mut impl AsStoreMut, val: Value) -> Result<Function, RuntimeError> {
    if !val.is_from_store(store) {
        return Err(RuntimeError::new("cannot pass Value across contexts"));
    }
    match val {
        Value::FuncRef(Some(ref func)) => Ok(func.0.handle.function.clone().into_inner()),
        // Only funcrefs is supported by the spec atm
        _ => unimplemented!("The {val:?} is not yet supported"),
    }
}

impl Table {
    pub fn new(
        store: &mut impl AsStoreMut,
        ty: TableType,
        init: Value,
    ) -> Result<Self, RuntimeError> {
        let mut store = store;
        let descriptor = js_sys::Object::new();
        js_sys::Reflect::set(&descriptor, &"initial".into(), &ty.minimum.into())?;
        if let Some(max) = ty.maximum {
            js_sys::Reflect::set(&descriptor, &"maximum".into(), &max.into())?;
        }
        js_sys::Reflect::set(&descriptor, &"element".into(), &"anyfunc".into())?;

        let js_table = js_sys::WebAssembly::Table::new(&descriptor)?;
        let table = VMTable::new(js_table, ty);

        let num_elements = table.table.length();
        let func = get_function(&mut store, init)?;
        for i in 0..num_elements {
            set_table_item(&table, i, &func)?;
        }

        Ok(Self { handle: table })
    }

    pub fn to_vm_extern(&self) -> VMExtern {
        VMExtern::Table(self.handle.clone())
    }

    pub fn ty(&self, _store: &impl AsStoreRef) -> TableType {
        self.handle.ty
    }

    pub fn get(&self, store: &mut impl AsStoreMut, index: u32) -> Option<Value> {
        if let Some(func) = self.handle.table.get(index).ok() {
            let ty = FunctionType::new(vec![], vec![]);
            let vm_function = VMFunction::new(func, ty);
            let function = crate::Function::from_vm_extern(store, vm_function);
            Some(Value::FuncRef(Some(function)))
        } else {
            None
        }
    }

    pub fn set(
        &self,
        store: &mut impl AsStoreMut,
        index: u32,
        val: Value,
    ) -> Result<(), RuntimeError> {
        let item = get_function(store, val)?;
        set_table_item(&self.handle, index, &item)
    }

    pub fn size(&self, _store: &impl AsStoreRef) -> u32 {
        self.handle.table.length()
    }

    pub fn grow(
        &self,
        _store: &mut impl AsStoreMut,
        _delta: u32,
        _init: Value,
    ) -> Result<u32, RuntimeError> {
        unimplemented!();
    }

    pub fn copy(
        _store: &mut impl AsStoreMut,
        _dst_table: &Self,
        _dst_index: u32,
        _src_table: &Self,
        _src_index: u32,
        _len: u32,
    ) -> Result<(), RuntimeError> {
        unimplemented!("Table.copy is not natively supported in Javascript");
    }

    pub(crate) fn from_vm_extern(_store: &mut impl AsStoreMut, vm_extern: VMExternTable) -> Self {
        Self { handle: vm_extern }
    }

    pub fn is_from_store(&self, _store: &impl AsStoreRef) -> bool {
        true
    }
}
