//! The `vm` module re-exports wasmer-vm types.

#[cfg(feature = "js")]
pub(crate) use crate::js::vm::{
    VMExtern, VMExternFunction, VMExternGlobal, VMExternMemory, VMExternRef, VMExternTable,
    VMFuncRef, VMFunctionCallback, VMFunctionEnvironment, VMInstance, VMTrampoline,
};

#[cfg(feature = "jsc")]
pub(crate) use crate::jsc::vm::{
    VMExtern, VMExternFunction, VMExternGlobal, VMExternMemory, VMExternRef, VMExternTable,
    VMFuncRef, VMFunctionCallback, VMFunctionEnvironment, VMInstance, VMTrampoline,
};

#[cfg(feature = "sys")]
pub(crate) use crate::sys::vm::{
    VMExtern, VMExternFunction, VMExternGlobal, VMExternMemory, VMExternRef, VMExternTable,
    VMFuncRef, VMFunctionCallback, VMFunctionEnvironment, VMInstance, VMTrampoline,
};

#[cfg(feature = "js")]
pub use crate::js::vm::{VMFunction, VMGlobal, VMMemory, VMSharedMemory, VMTable};

#[cfg(feature = "sys")]
pub use wasmer_vm::{VMConfig, VMFunction, VMGlobal, VMMemory, VMSharedMemory, VMTable};

#[cfg(feature = "jsc")]
pub use crate::jsc::vm::{VMFunction, VMGlobal, VMMemory, VMSharedMemory, VMTable};

// Needed for tunables customization (those are public types now)
#[cfg(feature = "sys")]
pub use wasmer_vm::{
    // An extra one for VMMemory implementors
    LinearMemory,
    VMMemoryDefinition,
    VMTableDefinition,
};

// Deprecated exports
pub use wasmer_types::{MemoryError, MemoryStyle, TableStyle};
