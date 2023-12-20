use crate::js::export::VMFunction;
use crate::js::export::{Export, VMTable};
use crate::js::exports::{ExportError, Exportable};
use crate::js::externals::{Extern, Function as WasmerFunction};
use crate::js::store::Store;
use crate::js::types::Val;
use crate::js::RuntimeError;
use crate::js::TableType;
use js_sys::Function;
use wasmer_types::FunctionType;

/// A WebAssembly `table` instance.
///
/// The `Table` struct is an array-like structure representing a WebAssembly Table,
/// which stores function references.
///
/// A table created by the host or in WebAssembly code will be accessible and
/// mutable from both host and WebAssembly.
///
/// Spec: <https://webassembly.github.io/spec/core/exec/runtime.html#table-instances>
#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    store: Store,
    vm_table: VMTable,
}

fn set_table_item(table: &VMTable, item_index: u32, item: &Function) -> Result<(), RuntimeError> {
    table.table.set(item_index, item).map_err(|e| e.into())
}

fn get_function(val: Val) -> Result<Function, RuntimeError> {
    match val {
        Val::FuncRef(func) => Ok(func.as_ref().unwrap().exported.function.clone().into()),
        // Only funcrefs is supported by the spec atm
        _ => unimplemented!(),
    }
}

impl Table {
    /// Creates a new `Table` with the provided [`TableType`] definition.
    ///
    /// All the elements in the table will be set to the `init` value.
    ///
    /// This function will construct the `Table` using the store
    /// [`BaseTunables`][crate::js::tunables::BaseTunables].
    pub fn new(store: &Store, ty: TableType, init: Val) -> Result<Self, RuntimeError> {
        let descriptor = js_sys::Object::new();
        js_sys::Reflect::set(&descriptor, &"initial".into(), &ty.minimum.into())?;
        if let Some(max) = ty.maximum {
            js_sys::Reflect::set(&descriptor, &"maximum".into(), &max.into())?;
        }
        js_sys::Reflect::set(&descriptor, &"element".into(), &"anyfunc".into())?;

        let js_table = js_sys::WebAssembly::Table::new(&descriptor)?;
        let table = VMTable::new(js_table, ty);

        let num_elements = table.table.length();
        let func = get_function(init)?;
        for i in 0..num_elements {
            set_table_item(&table, i, &func)?;
        }

        Ok(Self {
            store: store.clone(),
            vm_table: table,
        })
    }

    /// Returns the [`TableType`] of the `Table`.
    pub fn ty(&self) -> &TableType {
        &self.vm_table.ty
    }

    /// Returns the [`Store`] where the `Table` belongs.
    pub fn store(&self) -> &Store {
        &self.store
    }

    /// Retrieves an element of the table at the provided `index`.
    pub fn get(&self, index: u32) -> Option<Val> {
        let func = self.vm_table.table.get(index).ok()?;
        let ty = FunctionType::new(vec![], vec![]);
        Some(Val::FuncRef(Some(WasmerFunction::from_vm_export(
            &self.store,
            VMFunction::new(func, ty, None),
        ))))
    }

    /// Sets an element `val` in the Table at the provided `index`.
    pub fn set(&self, index: u32, val: Val) -> Result<(), RuntimeError> {
        let func = get_function(val)?;
        set_table_item(&self.vm_table, index, &func)?;
        Ok(())
    }

    /// Retrieves the size of the `Table` (in elements)
    pub fn size(&self) -> u32 {
        self.vm_table.table.length()
    }

    /// Grows the size of the `Table` by `delta`, initializating
    /// the elements with the provided `init` value.
    ///
    /// It returns the previous size of the `Table` in case is able
    /// to grow the Table successfully.
    ///
    /// # Errors
    ///
    /// Returns an error if the `delta` is out of bounds for the table.
    pub fn grow(&self, _delta: u32, _init: Val) -> Result<u32, RuntimeError> {
        unimplemented!();
    }

    /// Copies the `len` elements of `src_table` starting at `src_index`
    /// to the destination table `dst_table` at index `dst_index`.
    ///
    /// # Errors
    ///
    /// Returns an error if the range is out of bounds of either the source or
    /// destination tables.
    pub fn copy(
        _dst_table: &Self,
        _dst_index: u32,
        _src_table: &Self,
        _src_index: u32,
        _len: u32,
    ) -> Result<(), RuntimeError> {
        unimplemented!("Table.copy is not natively supported in Javascript");
    }

    pub(crate) fn from_vm_export(store: &Store, vm_table: VMTable) -> Self {
        Self {
            store: store.clone(),
            vm_table,
        }
    }

    /// Returns whether or not these two tables refer to the same data.
    pub fn same(&self, other: &Self) -> bool {
        self.vm_table == other.vm_table
    }

    /// Get access to the backing VM value for this extern. This function is for
    /// tests it should not be called by users of the Wasmer API.
    ///
    /// # Safety
    /// This function is unsafe to call outside of tests for the wasmer crate
    /// because there is no stability guarantee for the returned type and we may
    /// make breaking changes to it at any time or remove this method.
    #[doc(hidden)]
    pub unsafe fn get_vm_table(&self) -> &VMTable {
        &self.vm_table
    }
}

impl<'a> Exportable<'a> for Table {
    fn to_export(&self) -> Export {
        Export::Table(self.vm_table.clone())
    }

    fn get_self_from_extern(_extern: &'a Extern) -> Result<&'a Self, ExportError> {
        match _extern {
            Extern::Table(table) => Ok(table),
            _ => Err(ExportError::IncompatibleType),
        }
    }
}
