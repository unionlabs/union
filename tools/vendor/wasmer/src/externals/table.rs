#[cfg(feature = "js")]
use crate::js::externals::table as table_impl;
#[cfg(feature = "jsc")]
use crate::jsc::externals::table as table_impl;
#[cfg(feature = "sys")]
use crate::sys::externals::table as table_impl;

use crate::exports::{ExportError, Exportable};
use crate::store::{AsStoreMut, AsStoreRef};
use crate::vm::{VMExtern, VMExternTable};
use crate::Extern;
use crate::RuntimeError;
use crate::TableType;
use crate::Value;

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
pub struct Table(pub(crate) table_impl::Table);

impl Table {
    /// Creates a new `Table` with the provided [`TableType`] definition.
    ///
    /// All the elements in the table will be set to the `init` value.
    ///
    /// This function will construct the `Table` using the store
    /// [`BaseTunables`][crate::sys::BaseTunables].
    pub fn new(
        store: &mut impl AsStoreMut,
        ty: TableType,
        init: Value,
    ) -> Result<Self, RuntimeError> {
        Ok(Self(table_impl::Table::new(store, ty, init)?))
    }

    /// Returns the [`TableType`] of the `Table`.
    pub fn ty(&self, store: &impl AsStoreRef) -> TableType {
        self.0.ty(store)
    }

    /// Retrieves an element of the table at the provided `index`.
    pub fn get(&self, store: &mut impl AsStoreMut, index: u32) -> Option<Value> {
        self.0.get(store, index)
    }

    /// Sets an element `val` in the Table at the provided `index`.
    pub fn set(
        &self,
        store: &mut impl AsStoreMut,
        index: u32,
        val: Value,
    ) -> Result<(), RuntimeError> {
        self.0.set(store, index, val)
    }

    /// Retrieves the size of the `Table` (in elements)
    pub fn size(&self, store: &impl AsStoreRef) -> u32 {
        self.0.size(store)
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
    pub fn grow(
        &self,
        store: &mut impl AsStoreMut,
        delta: u32,
        init: Value,
    ) -> Result<u32, RuntimeError> {
        self.0.grow(store, delta, init)
    }

    /// Copies the `len` elements of `src_table` starting at `src_index`
    /// to the destination table `dst_table` at index `dst_index`.
    ///
    /// # Errors
    ///
    /// Returns an error if the range is out of bounds of either the source or
    /// destination tables.
    pub fn copy(
        store: &mut impl AsStoreMut,
        dst_table: &Self,
        dst_index: u32,
        src_table: &Self,
        src_index: u32,
        len: u32,
    ) -> Result<(), RuntimeError> {
        table_impl::Table::copy(store, &dst_table.0, dst_index, &src_table.0, src_index, len)
    }

    pub(crate) fn from_vm_extern(store: &mut impl AsStoreMut, extern_: VMExternTable) -> Self {
        Self(table_impl::Table::from_vm_extern(store, extern_))
    }

    /// Checks whether this `Table` can be used with the given context.
    pub fn is_from_store(&self, store: &impl AsStoreRef) -> bool {
        self.0.is_from_store(store)
    }

    pub(crate) fn to_vm_extern(&self) -> VMExtern {
        self.0.to_vm_extern()
    }
}

impl std::cmp::Eq for Table {}

impl<'a> Exportable<'a> for Table {
    fn get_self_from_extern(_extern: &'a Extern) -> Result<&'a Self, ExportError> {
        match _extern {
            Extern::Table(table) => Ok(table),
            _ => Err(ExportError::IncompatibleType),
        }
    }
}

/// Check the example from <https://github.com/wasmerio/wasmer/issues/3197>.
#[test]
fn test_table_grow_issue_3197() {
    use crate::{imports, Instance, Module, Store, Table, TableType, Type, Value};

    const WAT: &str = r#"(module (table (import "env" "table") 100 funcref))"#;

    // Tests that the table type of `table` is compatible with the export in the WAT
    // This tests that `wasmer_types::types::is_table_compatible` works as expected.
    let mut store = Store::default();
    let module = Module::new(&store, WAT).unwrap();
    let ty = TableType::new(Type::FuncRef, 0, None);
    let table = Table::new(&mut store, ty, Value::FuncRef(None)).unwrap();
    table.grow(&mut store, 100, Value::FuncRef(None)).unwrap();
    assert_eq!(table.ty(&store).minimum, 0);
    let imports = imports! {"env" => {"table" => table}};
    let _instance = Instance::new(&mut store, &module, &imports).unwrap();
}
