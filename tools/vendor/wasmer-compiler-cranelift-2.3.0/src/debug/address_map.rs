//! Data structures to provide transformation of the source
// addresses of a WebAssembly module into the native code.

use cranelift_codegen::ir;
use wasmer_types::entity::PrimaryMap;
use wasmer_types::LocalFunctionIndex;

/// Value ranges for functions.
pub type ValueLabelsRanges = PrimaryMap<LocalFunctionIndex, cranelift_codegen::ValueLabelsRanges>;

/// Stack slots for functions.
pub type StackSlots = PrimaryMap<LocalFunctionIndex, ir::StackSlots>;

/// Memory definition offset in the VMContext structure.
#[derive(Debug, Clone)]
pub enum ModuleInfoMemoryOffset {
    /// Not available.
    None,
    /// Offset to the defined memory.
    Defined(u32),
    /// Offset to the imported memory.
    Imported(u32),
}

/// ModuleInfo `vmctx` related info.
#[derive(Debug, Clone)]
pub struct ModuleInfoVmctxInfo {
    /// The memory definition offset in the VMContext structure.
    pub memory_offset: ModuleInfoMemoryOffset,

    /// The functions stack slots.
    pub stack_slots: StackSlots,
}
