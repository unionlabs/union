// This file contains code from external sources.
// Attributions: https://github.com/wasmerio/wasmer/blob/master/ATTRIBUTIONS.md

use crate::global::Global;
use crate::instance::WeakOrStrongInstanceRef;
use crate::memory::Memory;
use crate::table::Table;
use crate::vmcontext::{VMFunctionBody, VMFunctionEnvironment, VMFunctionKind, VMTrampoline};
use loupe::MemoryUsage;
use std::sync::Arc;
use wasmer_types::{FunctionType, MemoryStyle, MemoryType, TableStyle, TableType};

/// The value of an export passed from one instance to another.
#[derive(Debug)]
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

/// A function export value.
#[derive(Clone, Debug, PartialEq, MemoryUsage)]
pub struct VMFunction {
    /// The address of the native-code function.
    pub address: *const VMFunctionBody,

    /// Pointer to the containing `VMContext`.
    pub vmctx: VMFunctionEnvironment,

    /// The function type, used for compatibility checking.
    pub signature: FunctionType,

    /// The function kind (specifies the calling convention for the
    /// function).
    pub kind: VMFunctionKind,

    /// Address of the function call trampoline owned by the same
    /// VMContext that owns the VMFunctionBody.
    ///
    /// May be `None` when the function is a host function (`FunctionType`
    /// == `Dynamic` or `vmctx` == `nullptr`).
    #[loupe(skip)]
    pub call_trampoline: Option<VMTrampoline>,

    /// A “reference” to the instance through the
    /// `InstanceRef`. `None` if it is a host function.
    pub instance_ref: Option<WeakOrStrongInstanceRef>,
}

impl VMFunction {
    /// Converts the stored instance ref into a strong `InstanceRef` if it is weak.
    /// Returns None if it cannot be upgraded.
    pub fn upgrade_instance_ref(&mut self) -> Option<()> {
        if let Some(ref mut ir) = self.instance_ref {
            *ir = ir.upgrade()?;
        }
        Some(())
    }
}

/// # Safety
/// There is no non-threadsafe logic directly in this type. Calling the function
/// may not be threadsafe.
unsafe impl Send for VMFunction {}
/// # Safety
/// The members of an VMFunction are immutable after construction.
unsafe impl Sync for VMFunction {}

impl From<VMFunction> for VMExtern {
    fn from(func: VMFunction) -> Self {
        Self::Function(func)
    }
}

/// A table export value.
#[derive(Clone, Debug, MemoryUsage)]
pub struct VMTable {
    /// Pointer to the containing `Table`.
    pub from: Arc<dyn Table>,

    /// A “reference” to the instance through the
    /// `InstanceRef`. `None` if it is a host table.
    pub instance_ref: Option<WeakOrStrongInstanceRef>,
}

/// # Safety
/// This is correct because there is no non-threadsafe logic directly in this type;
/// correct use of the raw table from multiple threads via `definition` requires `unsafe`
/// and is the responsibilty of the user of this type.
unsafe impl Send for VMTable {}

/// # Safety
/// This is correct because the values directly in `definition` should be considered immutable
/// and the type is both `Send` and `Clone` (thus marking it `Sync` adds no new behavior, it
/// only makes this type easier to use)
unsafe impl Sync for VMTable {}

impl VMTable {
    /// Get the table type for this exported table
    pub fn ty(&self) -> &TableType {
        self.from.ty()
    }

    /// Get the style for this exported table
    pub fn style(&self) -> &TableStyle {
        self.from.style()
    }

    /// Returns whether or not the two `VMTable`s refer to the same Memory.
    pub fn same(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.from, &other.from)
    }

    /// Converts the stored instance ref into a strong `InstanceRef` if it is weak.
    /// Returns None if it cannot be upgraded.
    pub fn upgrade_instance_ref(&mut self) -> Option<()> {
        if let Some(ref mut ir) = self.instance_ref {
            *ir = ir.upgrade()?;
        }
        Some(())
    }
}

impl From<VMTable> for VMExtern {
    fn from(table: VMTable) -> Self {
        Self::Table(table)
    }
}

/// A memory export value.
#[derive(Debug, Clone, MemoryUsage)]
pub struct VMMemory {
    /// Pointer to the containing `Memory`.
    pub from: Arc<dyn Memory>,

    /// A “reference” to the instance through the
    /// `InstanceRef`. `None` if it is a host memory.
    pub instance_ref: Option<WeakOrStrongInstanceRef>,
}

/// # Safety
/// This is correct because there is no non-threadsafe logic directly in this type;
/// correct use of the raw memory from multiple threads via `definition` requires `unsafe`
/// and is the responsibilty of the user of this type.
unsafe impl Send for VMMemory {}

/// # Safety
/// This is correct because the values directly in `definition` should be considered immutable
/// and the type is both `Send` and `Clone` (thus marking it `Sync` adds no new behavior, it
/// only makes this type easier to use)
unsafe impl Sync for VMMemory {}

impl VMMemory {
    /// Get the type for this exported memory
    pub fn ty(&self) -> MemoryType {
        self.from.ty()
    }

    /// Get the style for this exported memory
    pub fn style(&self) -> &MemoryStyle {
        self.from.style()
    }

    /// Returns whether or not the two `VMMemory`s refer to the same Memory.
    pub fn same(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.from, &other.from)
    }

    /// Converts the stored instance ref into a strong `InstanceRef` if it is weak.
    /// Returns None if it cannot be upgraded.
    pub fn upgrade_instance_ref(&mut self) -> Option<()> {
        if let Some(ref mut ir) = self.instance_ref {
            *ir = ir.upgrade()?;
        }
        Some(())
    }
}

impl From<VMMemory> for VMExtern {
    fn from(memory: VMMemory) -> Self {
        Self::Memory(memory)
    }
}

/// A global export value.
#[derive(Debug, Clone, MemoryUsage)]
pub struct VMGlobal {
    /// The global declaration, used for compatibility checking.
    pub from: Arc<Global>,

    /// A “reference” to the instance through the
    /// `InstanceRef`. `None` if it is a host global.
    pub instance_ref: Option<WeakOrStrongInstanceRef>,
}

/// # Safety
/// This is correct because there is no non-threadsafe logic directly in this type;
/// correct use of the raw global from multiple threads via `definition` requires `unsafe`
/// and is the responsibilty of the user of this type.
unsafe impl Send for VMGlobal {}

/// # Safety
/// This is correct because the values directly in `definition` should be considered immutable
/// from the perspective of users of this type and the type is both `Send` and `Clone` (thus
/// marking it `Sync` adds no new behavior, it only makes this type easier to use)
unsafe impl Sync for VMGlobal {}

impl VMGlobal {
    /// Returns whether or not the two `VMGlobal`s refer to the same Global.
    pub fn same(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.from, &other.from)
    }

    /// Converts the stored instance ref into a strong `InstanceRef` if it is weak.
    /// Returns None if it cannot be upgraded.
    pub fn upgrade_instance_ref(&mut self) -> Option<()> {
        if let Some(ref mut ir) = self.instance_ref {
            *ir = ir.upgrade()?;
        }
        Some(())
    }
}

impl From<VMGlobal> for VMExtern {
    fn from(global: VMGlobal) -> Self {
        Self::Global(global)
    }
}
