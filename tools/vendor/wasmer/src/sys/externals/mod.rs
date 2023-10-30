pub(crate) mod function;
mod global;
mod memory;
mod table;

pub use self::function::{
    FromToNativeWasmType, Function, HostFunction, WasmTypeList, WithEnv, WithoutEnv,
};

pub use self::global::Global;
pub use self::memory::Memory;
pub use self::table::Table;

use crate::sys::exports::{ExportError, Exportable};
use crate::sys::store::{Store, StoreObject};
use crate::sys::ExternType;
use loupe::MemoryUsage;
use std::fmt;
use wasmer_engine::Export;

/// An `Extern` is the runtime representation of an entity that
/// can be imported or exported.
///
/// Spec: <https://webassembly.github.io/spec/core/exec/runtime.html#external-values>
#[derive(Clone, MemoryUsage)]
pub enum Extern {
    /// A external [`Function`].
    Function(Function),
    /// A external [`Global`].
    Global(Global),
    /// A external [`Table`].
    Table(Table),
    /// A external [`Memory`].
    Memory(Memory),
}

impl Extern {
    /// Return the underlying type of the inner `Extern`.
    pub fn ty(&self) -> ExternType {
        match self {
            Self::Function(ft) => ExternType::Function(ft.ty().clone()),
            Self::Memory(ft) => ExternType::Memory(ft.ty()),
            Self::Table(tt) => ExternType::Table(*tt.ty()),
            Self::Global(gt) => ExternType::Global(*gt.ty()),
        }
    }

    /// Create an `Extern` from an `wasmer_engine::Export`.
    pub fn from_vm_export(store: &Store, export: Export) -> Self {
        match export {
            Export::Function(f) => Self::Function(Function::from_vm_export(store, f)),
            Export::Memory(m) => Self::Memory(Memory::from_vm_export(store, m)),
            Export::Global(g) => Self::Global(Global::from_vm_export(store, g)),
            Export::Table(t) => Self::Table(Table::from_vm_export(store, t)),
        }
    }
}

impl<'a> Exportable<'a> for Extern {
    fn to_export(&self) -> Export {
        match self {
            Self::Function(f) => f.to_export(),
            Self::Global(g) => g.to_export(),
            Self::Memory(m) => m.to_export(),
            Self::Table(t) => t.to_export(),
        }
    }

    fn get_self_from_extern(_extern: &'a Self) -> Result<&'a Self, ExportError> {
        // Since this is already an extern, we can just return it.
        Ok(_extern)
    }

    fn into_weak_instance_ref(&mut self) {
        match self {
            Self::Function(f) => f.into_weak_instance_ref(),
            Self::Global(g) => g.into_weak_instance_ref(),
            Self::Memory(m) => m.into_weak_instance_ref(),
            Self::Table(t) => t.into_weak_instance_ref(),
        }
    }
}

impl StoreObject for Extern {
    fn comes_from_same_store(&self, store: &Store) -> bool {
        let my_store = match self {
            Self::Function(f) => f.store(),
            Self::Global(g) => g.store(),
            Self::Memory(m) => m.store(),
            Self::Table(t) => t.store(),
        };
        Store::same(my_store, store)
    }
}

impl fmt::Debug for Extern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Function(_) => "Function(...)",
                Self::Global(_) => "Global(...)",
                Self::Memory(_) => "Memory(...)",
                Self::Table(_) => "Table(...)",
            }
        )
    }
}

impl From<Function> for Extern {
    fn from(r: Function) -> Self {
        Self::Function(r)
    }
}

impl From<Global> for Extern {
    fn from(r: Global) -> Self {
        Self::Global(r)
    }
}

impl From<Memory> for Extern {
    fn from(r: Memory) -> Self {
        Self::Memory(r)
    }
}

impl From<Table> for Extern {
    fn from(r: Table) -> Self {
        Self::Table(r)
    }
}
