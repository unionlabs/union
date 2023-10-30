use crate::sys::{MemoryType, Pages, TableType};
use loupe::MemoryUsage;
use std::ptr::NonNull;
use std::sync::Arc;
use target_lexicon::PointerWidth;
use wasmer_compiler::Target;
use wasmer_engine::Tunables;
use wasmer_vm::MemoryError;
use wasmer_vm::{
    LinearMemory, LinearTable, Memory, MemoryStyle, Table, TableStyle, VMMemoryDefinition,
    VMTableDefinition,
};

/// Tunable parameters for WebAssembly compilation.
/// This is the reference implementation of the `Tunables` trait,
/// used by default.
///
/// You can use this as a template for creating a custom Tunables
/// implementation or use composition to wrap your Tunables around
/// this one. The later approach is demonstrated in the
/// tunables-limit-memory example.
#[derive(Clone, MemoryUsage)]
pub struct BaseTunables {
    /// For static heaps, the size in wasm pages of the heap protected by bounds checking.
    pub static_memory_bound: Pages,

    /// The size in bytes of the offset guard for static heaps.
    pub static_memory_offset_guard_size: u64,

    /// The size in bytes of the offset guard for dynamic heaps.
    pub dynamic_memory_offset_guard_size: u64,
}

impl BaseTunables {
    /// Get the `BaseTunables` for a specific Target
    pub fn for_target(target: &Target) -> Self {
        let triple = target.triple();
        let pointer_width: PointerWidth = triple.pointer_width().unwrap();
        let (static_memory_bound, static_memory_offset_guard_size): (Pages, u64) =
            match pointer_width {
                PointerWidth::U16 => (0x400.into(), 0x1000),
                PointerWidth::U32 => (0x4000.into(), 0x1_0000),
                // Static Memory Bound:
                //   Allocating 4 GiB of address space let us avoid the
                //   need for explicit bounds checks.
                // Static Memory Guard size:
                //   Allocating 2 GiB of address space lets us translate wasm
                //   offsets into x86 offsets as aggressively as we can.
                PointerWidth::U64 => (0x1_0000.into(), 0x8000_0000),
            };

        // Allocate a small guard to optimize common cases but without
        // wasting too much memory.
        // The Windows memory manager seems more laxed than the other ones
        // And a guard of just 1 page may not be enough is some borderline cases
        // So using 2 pages for guard on this platform
        #[cfg(target_os = "windows")]
        let dynamic_memory_offset_guard_size: u64 = 0x2_0000;
        #[cfg(not(target_os = "windows"))]
        let dynamic_memory_offset_guard_size: u64 = 0x1_0000;

        Self {
            static_memory_bound,
            static_memory_offset_guard_size,
            dynamic_memory_offset_guard_size,
        }
    }
}

impl Tunables for BaseTunables {
    /// Get a `MemoryStyle` for the provided `MemoryType`
    fn memory_style(&self, memory: &MemoryType) -> MemoryStyle {
        // A heap with a maximum that doesn't exceed the static memory bound specified by the
        // tunables make it static.
        //
        // If the module doesn't declare an explicit maximum treat it as 4GiB.
        let maximum = memory.maximum.unwrap_or_else(Pages::max_value);
        if maximum <= self.static_memory_bound {
            MemoryStyle::Static {
                // Bound can be larger than the maximum for performance reasons
                bound: self.static_memory_bound,
                offset_guard_size: self.static_memory_offset_guard_size,
            }
        } else {
            MemoryStyle::Dynamic {
                offset_guard_size: self.dynamic_memory_offset_guard_size,
            }
        }
    }

    /// Get a [`TableStyle`] for the provided [`TableType`].
    fn table_style(&self, _table: &TableType) -> TableStyle {
        TableStyle::CallerChecksSignature
    }

    /// Create a memory owned by the host given a [`MemoryType`] and a [`MemoryStyle`].
    fn create_host_memory(
        &self,
        ty: &MemoryType,
        style: &MemoryStyle,
    ) -> Result<Arc<dyn Memory>, MemoryError> {
        Ok(Arc::new(LinearMemory::new(ty, style)?))
    }

    /// Create a memory owned by the VM given a [`MemoryType`] and a [`MemoryStyle`].
    ///
    /// # Safety
    /// - `vm_definition_location` must point to a valid, owned `VMMemoryDefinition`,
    ///   for example in `VMContext`.
    unsafe fn create_vm_memory(
        &self,
        ty: &MemoryType,
        style: &MemoryStyle,
        vm_definition_location: NonNull<VMMemoryDefinition>,
    ) -> Result<Arc<dyn Memory>, MemoryError> {
        Ok(Arc::new(LinearMemory::from_definition(
            ty,
            style,
            vm_definition_location,
        )?))
    }

    /// Create a table owned by the host given a [`TableType`] and a [`TableStyle`].
    fn create_host_table(
        &self,
        ty: &TableType,
        style: &TableStyle,
    ) -> Result<Arc<dyn Table>, String> {
        Ok(Arc::new(LinearTable::new(ty, style)?))
    }

    /// Create a table owned by the VM given a [`TableType`] and a [`TableStyle`].
    ///
    /// # Safety
    /// - `vm_definition_location` must point to a valid, owned `VMTableDefinition`,
    ///   for example in `VMContext`.
    unsafe fn create_vm_table(
        &self,
        ty: &TableType,
        style: &TableStyle,
        vm_definition_location: NonNull<VMTableDefinition>,
    ) -> Result<Arc<dyn Table>, String> {
        Ok(Arc::new(LinearTable::from_definition(
            ty,
            style,
            vm_definition_location,
        )?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_style() {
        let tunables = BaseTunables {
            static_memory_bound: Pages(2048),
            static_memory_offset_guard_size: 128,
            dynamic_memory_offset_guard_size: 256,
        };

        // No maximum
        let requested = MemoryType::new(3, None, true);
        let style = tunables.memory_style(&requested);
        match style {
            MemoryStyle::Dynamic { offset_guard_size } => assert_eq!(offset_guard_size, 256),
            s => panic!("Unexpected memory style: {:?}", s),
        }

        // Large maximum
        let requested = MemoryType::new(3, Some(5_000_000), true);
        let style = tunables.memory_style(&requested);
        match style {
            MemoryStyle::Dynamic { offset_guard_size } => assert_eq!(offset_guard_size, 256),
            s => panic!("Unexpected memory style: {:?}", s),
        }

        // Small maximum
        let requested = MemoryType::new(3, Some(16), true);
        let style = tunables.memory_style(&requested);
        match style {
            MemoryStyle::Static {
                bound,
                offset_guard_size,
            } => {
                assert_eq!(bound, Pages(2048));
                assert_eq!(offset_guard_size, 128);
            }
            s => panic!("Unexpected memory style: {:?}", s),
        }
    }
}
