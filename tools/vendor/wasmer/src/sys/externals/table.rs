use crate::store::{AsStoreMut, AsStoreRef};
use crate::sys::engine::NativeEngineExt;
use crate::TableType;
use crate::Value;
use crate::{vm::VMExternTable, ExternRef, Function, RuntimeError};
use wasmer_vm::{StoreHandle, TableElement, Trap, VMExtern, VMTable};

#[derive(Debug, Clone)]
pub struct Table {
    handle: StoreHandle<VMTable>,
}

fn set_table_item(
    table: &mut VMTable,
    item_index: u32,
    item: TableElement,
) -> Result<(), RuntimeError> {
    table.set(item_index, item).map_err(|e| e.into())
}

fn value_to_table_element(
    store: &mut impl AsStoreMut,
    val: Value,
) -> Result<wasmer_vm::TableElement, RuntimeError> {
    if !val.is_from_store(store) {
        return Err(RuntimeError::new("cannot pass Value across contexts"));
    }
    Ok(match val {
        Value::ExternRef(extern_ref) => {
            wasmer_vm::TableElement::ExternRef(extern_ref.map(|e| e.vm_externref()))
        }
        Value::FuncRef(func_ref) => {
            wasmer_vm::TableElement::FuncRef(func_ref.map(|f| f.vm_funcref(store)))
        }
        _ => return Err(RuntimeError::new("val is not reference")),
    })
}

fn value_from_table_element(store: &mut impl AsStoreMut, item: wasmer_vm::TableElement) -> Value {
    match item {
        wasmer_vm::TableElement::FuncRef(funcref) => {
            Value::FuncRef(funcref.map(|f| unsafe { Function::from_vm_funcref(store, f) }))
        }
        wasmer_vm::TableElement::ExternRef(extern_ref) => {
            Value::ExternRef(extern_ref.map(|e| unsafe { ExternRef::from_vm_externref(store, e) }))
        }
    }
}

impl Table {
    pub fn new(
        mut store: &mut impl AsStoreMut,
        ty: TableType,
        init: Value,
    ) -> Result<Self, RuntimeError> {
        let item = value_to_table_element(&mut store, init)?;
        let mut store = store.as_store_mut();
        let tunables = store.engine().tunables();
        let style = tunables.table_style(&ty);
        let mut table = tunables
            .create_host_table(&ty, &style)
            .map_err(RuntimeError::new)?;

        let num_elements = table.size();
        for i in 0..num_elements {
            set_table_item(&mut table, i, item.clone())?;
        }

        Ok(Self {
            handle: StoreHandle::new(store.objects_mut(), table),
        })
    }

    pub fn ty(&self, store: &impl AsStoreRef) -> TableType {
        *self.handle.get(store.as_store_ref().objects()).ty()
    }

    pub fn get(&self, store: &mut impl AsStoreMut, index: u32) -> Option<Value> {
        let item = self.handle.get(store.as_store_ref().objects()).get(index)?;
        Some(value_from_table_element(store, item))
    }

    pub fn set(
        &self,
        store: &mut impl AsStoreMut,
        index: u32,
        val: Value,
    ) -> Result<(), RuntimeError> {
        let item = value_to_table_element(store, val)?;
        set_table_item(self.handle.get_mut(store.objects_mut()), index, item)
    }

    pub fn size(&self, store: &impl AsStoreRef) -> u32 {
        self.handle.get(store.as_store_ref().objects()).size()
    }

    pub fn grow(
        &self,
        store: &mut impl AsStoreMut,
        delta: u32,
        init: Value,
    ) -> Result<u32, RuntimeError> {
        let item = value_to_table_element(store, init)?;
        self.handle
            .get_mut(store.objects_mut())
            .grow(delta, item)
            .ok_or_else(|| RuntimeError::new(format!("failed to grow table by `{}`", delta)))
    }

    pub fn copy(
        store: &mut impl AsStoreMut,
        dst_table: &Self,
        dst_index: u32,
        src_table: &Self,
        src_index: u32,
        len: u32,
    ) -> Result<(), RuntimeError> {
        if dst_table.handle.store_id() != src_table.handle.store_id() {
            return Err(RuntimeError::new(
                "cross-`Store` table copies are not supported",
            ));
        }
        let store = store;
        if dst_table.handle.internal_handle() == src_table.handle.internal_handle() {
            let table = dst_table.handle.get_mut(store.objects_mut());
            table.copy_within(dst_index, src_index, len)
        } else {
            let (src_table, dst_table) = store.objects_mut().get_2_mut(
                src_table.handle.internal_handle(),
                dst_table.handle.internal_handle(),
            );
            VMTable::copy(dst_table, src_table, dst_index, src_index, len)
        }
        .map_err(Into::<Trap>::into)?;
        Ok(())
    }

    pub(crate) fn from_vm_extern(store: &mut impl AsStoreMut, vm_extern: VMExternTable) -> Self {
        Self {
            handle: unsafe {
                StoreHandle::from_internal(store.as_store_ref().objects().id(), vm_extern)
            },
        }
    }

    /// Checks whether this `Table` can be used with the given context.
    pub fn is_from_store(&self, store: &impl AsStoreRef) -> bool {
        self.handle.store_id() == store.as_store_ref().objects().id()
    }

    pub(crate) fn to_vm_extern(&self) -> VMExtern {
        VMExtern::Table(self.handle.internal_handle())
    }
}

impl std::cmp::PartialEq for Table {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

impl std::cmp::Eq for Table {}
