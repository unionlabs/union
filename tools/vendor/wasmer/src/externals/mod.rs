pub(crate) mod function;
mod global;
pub(crate) mod memory;
mod memory_view;
mod table;

pub use self::function::{Function, HostFunction};
pub use self::global::Global;
pub use self::memory::Memory;
pub use self::memory_view::MemoryView;
pub use self::table::Table;

use crate::exports::{ExportError, Exportable};
use crate::ExternType;
use std::fmt;

#[cfg(feature = "js")]
use crate::js::vm::VMExtern;
#[cfg(feature = "jsc")]
use crate::jsc::vm::VMExtern;
#[cfg(feature = "sys")]
use wasmer_vm::VMExtern;

use crate::store::{AsStoreMut, AsStoreRef};

/// An `Extern` is the runtime representation of an entity that
/// can be imported or exported.
///
/// Spec: <https://webassembly.github.io/spec/core/exec/runtime.html#external-values>
#[derive(Clone, PartialEq, Eq)]
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
    pub fn ty(&self, store: &impl AsStoreRef) -> ExternType {
        match self {
            Self::Function(ft) => ExternType::Function(ft.ty(store)),
            Self::Memory(ft) => ExternType::Memory(ft.ty(store)),
            Self::Table(tt) => ExternType::Table(tt.ty(store)),
            Self::Global(gt) => ExternType::Global(gt.ty(store)),
        }
    }

    /// Create an `Extern` from an `wasmer_engine::Export`.
    pub fn from_vm_extern(store: &mut impl AsStoreMut, vm_extern: VMExtern) -> Self {
        match vm_extern {
            VMExtern::Function(f) => Self::Function(Function::from_vm_extern(store, f)),
            VMExtern::Memory(m) => Self::Memory(Memory::from_vm_extern(store, m)),
            VMExtern::Global(g) => Self::Global(Global::from_vm_extern(store, g)),
            VMExtern::Table(t) => Self::Table(Table::from_vm_extern(store, t)),
        }
    }

    /// Checks whether this `Extern` can be used with the given context.
    pub fn is_from_store(&self, store: &impl AsStoreRef) -> bool {
        match self {
            Self::Function(f) => f.is_from_store(store),
            Self::Global(g) => g.is_from_store(store),
            Self::Memory(m) => m.is_from_store(store),
            Self::Table(t) => t.is_from_store(store),
        }
    }

    /// To `VMExtern`.
    pub fn to_vm_extern(&self) -> VMExtern {
        match self {
            Self::Function(f) => f.to_vm_extern(),
            Self::Global(g) => g.to_vm_extern(),
            Self::Memory(m) => m.to_vm_extern(),
            Self::Table(t) => t.to_vm_extern(),
        }
    }
}

impl<'a> Exportable<'a> for Extern {
    fn get_self_from_extern(_extern: &'a Self) -> Result<&'a Self, ExportError> {
        // Since this is already an extern, we can just return it.
        Ok(_extern)
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
