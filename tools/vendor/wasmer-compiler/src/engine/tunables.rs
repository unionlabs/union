use crate::engine::error::LinkError;
use std::ptr::NonNull;
use wasmer_types::entity::{EntityRef, PrimaryMap};
use wasmer_types::{
    GlobalType, LocalGlobalIndex, LocalMemoryIndex, LocalTableIndex, MemoryIndex, MemoryType,
    ModuleInfo, Pages, PointerWidth, TableIndex, TableType, Target,
};
use wasmer_vm::{InternalStoreHandle, MemoryError, StoreObjects};
use wasmer_vm::{MemoryStyle, TableStyle};
use wasmer_vm::{VMConfig, VMGlobal, VMMemory, VMTable};
use wasmer_vm::{VMMemoryDefinition, VMTableDefinition};

/// An engine delegates the creation of memories, tables, and globals
/// to a foreign implementor of this trait.
pub trait Tunables {
    /// Construct a `MemoryStyle` for the provided `MemoryType`
    fn memory_style(&self, memory: &MemoryType) -> MemoryStyle;

    /// Construct a `TableStyle` for the provided `TableType`
    fn table_style(&self, table: &TableType) -> TableStyle;

    /// Create a memory owned by the host given a [`MemoryType`] and a [`MemoryStyle`].
    fn create_host_memory(
        &self,
        ty: &MemoryType,
        style: &MemoryStyle,
    ) -> Result<VMMemory, MemoryError>;

    /// Create a memory owned by the VM given a [`MemoryType`] and a [`MemoryStyle`].
    ///
    /// # Safety
    /// - `vm_definition_location` must point to a valid location in VM memory.
    unsafe fn create_vm_memory(
        &self,
        ty: &MemoryType,
        style: &MemoryStyle,
        vm_definition_location: NonNull<VMMemoryDefinition>,
    ) -> Result<VMMemory, MemoryError>;

    /// Create a table owned by the host given a [`TableType`] and a [`TableStyle`].
    fn create_host_table(&self, ty: &TableType, style: &TableStyle) -> Result<VMTable, String>;

    /// Create a table owned by the VM given a [`TableType`] and a [`TableStyle`].
    ///
    /// # Safety
    /// - `vm_definition_location` must point to a valid location in VM memory.
    unsafe fn create_vm_table(
        &self,
        ty: &TableType,
        style: &TableStyle,
        vm_definition_location: NonNull<VMTableDefinition>,
    ) -> Result<VMTable, String>;

    /// Create a global with an unset value.
    fn create_global(&self, ty: GlobalType) -> Result<VMGlobal, String> {
        Ok(VMGlobal::new(ty))
    }

    /// Allocate memory for just the memories of the current module.
    ///
    /// # Safety
    /// - `memory_definition_locations` must point to a valid locations in VM memory.
    #[allow(clippy::result_large_err)]
    unsafe fn create_memories(
        &self,
        context: &mut StoreObjects,
        module: &ModuleInfo,
        memory_styles: &PrimaryMap<MemoryIndex, MemoryStyle>,
        memory_definition_locations: &[NonNull<VMMemoryDefinition>],
    ) -> Result<PrimaryMap<LocalMemoryIndex, InternalStoreHandle<VMMemory>>, LinkError> {
        let num_imports = module.num_imported_memories;
        let mut memories: PrimaryMap<LocalMemoryIndex, _> =
            PrimaryMap::with_capacity(module.memories.len() - num_imports);
        for (index, mdl) in memory_definition_locations
            .iter()
            .enumerate()
            .take(module.memories.len())
            .skip(num_imports)
        {
            let mi = MemoryIndex::new(index);
            let ty = &module.memories[mi];
            let style = &memory_styles[mi];
            memories.push(InternalStoreHandle::new(
                context,
                self.create_vm_memory(ty, style, *mdl)
                    .map_err(|e| LinkError::Resource(format!("Failed to create memory: {}", e)))?,
            ));
        }
        Ok(memories)
    }

    /// Allocate memory for just the tables of the current module.
    ///
    /// # Safety
    ///
    /// To be done
    #[allow(clippy::result_large_err)]
    unsafe fn create_tables(
        &self,
        context: &mut StoreObjects,
        module: &ModuleInfo,
        table_styles: &PrimaryMap<TableIndex, TableStyle>,
        table_definition_locations: &[NonNull<VMTableDefinition>],
    ) -> Result<PrimaryMap<LocalTableIndex, InternalStoreHandle<VMTable>>, LinkError> {
        let num_imports = module.num_imported_tables;
        let mut tables: PrimaryMap<LocalTableIndex, _> =
            PrimaryMap::with_capacity(module.tables.len() - num_imports);
        for (index, tdl) in table_definition_locations
            .iter()
            .enumerate()
            .take(module.tables.len())
            .skip(num_imports)
        {
            let ti = TableIndex::new(index);
            let ty = &module.tables[ti];
            let style = &table_styles[ti];
            tables.push(InternalStoreHandle::new(
                context,
                self.create_vm_table(ty, style, *tdl)
                    .map_err(LinkError::Resource)?,
            ));
        }
        Ok(tables)
    }

    /// Allocate memory for just the globals of the current module,
    /// with initializers applied.
    #[allow(clippy::result_large_err)]
    fn create_globals(
        &self,
        context: &mut StoreObjects,
        module: &ModuleInfo,
    ) -> Result<PrimaryMap<LocalGlobalIndex, InternalStoreHandle<VMGlobal>>, LinkError> {
        let num_imports = module.num_imported_globals;
        let mut vmctx_globals = PrimaryMap::with_capacity(module.globals.len() - num_imports);

        for &global_type in module.globals.values().skip(num_imports) {
            vmctx_globals.push(InternalStoreHandle::new(
                context,
                self.create_global(global_type)
                    .map_err(LinkError::Resource)?,
            ));
        }

        Ok(vmctx_globals)
    }

    /// Get the VMConfig for this tunables
    /// Currently, VMConfig have optional Stack size
    /// If wasm_stack_size is left to None (the default value)
    /// then the global stack size will be use
    /// Else the defined stack size will be used. Size is in byte
    /// and the value might be rounded to sane value is needed.
    fn vmconfig(&self) -> &VMConfig {
        &VMConfig {
            wasm_stack_size: None,
        }
    }
}

/// Tunable parameters for WebAssembly compilation.
/// This is the reference implementation of the `Tunables` trait,
/// used by default.
///
/// You can use this as a template for creating a custom Tunables
/// implementation or use composition to wrap your Tunables around
/// this one. The later approach is demonstrated in the
/// tunables-limit-memory example.
#[derive(Clone)]
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
    ) -> Result<VMMemory, MemoryError> {
        VMMemory::new(ty, style)
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
    ) -> Result<VMMemory, MemoryError> {
        VMMemory::from_definition(ty, style, vm_definition_location)
    }

    /// Create a table owned by the host given a [`TableType`] and a [`TableStyle`].
    fn create_host_table(&self, ty: &TableType, style: &TableStyle) -> Result<VMTable, String> {
        VMTable::new(ty, style)
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
    ) -> Result<VMTable, String> {
        VMTable::from_definition(ty, style, vm_definition_location)
    }
}

impl Tunables for Box<dyn Tunables + Send + Sync> {
    fn memory_style(&self, memory: &MemoryType) -> MemoryStyle {
        self.as_ref().memory_style(memory)
    }

    fn table_style(&self, table: &TableType) -> TableStyle {
        self.as_ref().table_style(table)
    }

    fn create_host_memory(
        &self,
        ty: &MemoryType,
        style: &MemoryStyle,
    ) -> Result<VMMemory, MemoryError> {
        self.as_ref().create_host_memory(ty, style)
    }

    unsafe fn create_vm_memory(
        &self,
        ty: &MemoryType,
        style: &MemoryStyle,
        vm_definition_location: NonNull<VMMemoryDefinition>,
    ) -> Result<VMMemory, MemoryError> {
        self.as_ref()
            .create_vm_memory(ty, style, vm_definition_location)
    }

    fn create_host_table(&self, ty: &TableType, style: &TableStyle) -> Result<VMTable, String> {
        self.as_ref().create_host_table(ty, style)
    }

    unsafe fn create_vm_table(
        &self,
        ty: &TableType,
        style: &TableStyle,
        vm_definition_location: NonNull<VMTableDefinition>,
    ) -> Result<VMTable, String> {
        self.as_ref()
            .create_vm_table(ty, style, vm_definition_location)
    }
}

impl Tunables for std::sync::Arc<dyn Tunables + Send + Sync> {
    fn memory_style(&self, memory: &MemoryType) -> MemoryStyle {
        self.as_ref().memory_style(memory)
    }

    fn table_style(&self, table: &TableType) -> TableStyle {
        self.as_ref().table_style(table)
    }

    fn create_host_memory(
        &self,
        ty: &MemoryType,
        style: &MemoryStyle,
    ) -> Result<VMMemory, MemoryError> {
        self.as_ref().create_host_memory(ty, style)
    }

    unsafe fn create_vm_memory(
        &self,
        ty: &MemoryType,
        style: &MemoryStyle,
        vm_definition_location: NonNull<VMMemoryDefinition>,
    ) -> Result<VMMemory, MemoryError> {
        self.as_ref()
            .create_vm_memory(ty, style, vm_definition_location)
    }

    fn create_host_table(&self, ty: &TableType, style: &TableStyle) -> Result<VMTable, String> {
        self.as_ref().create_host_table(ty, style)
    }

    unsafe fn create_vm_table(
        &self,
        ty: &TableType,
        style: &TableStyle,
        vm_definition_location: NonNull<VMTableDefinition>,
    ) -> Result<VMTable, String> {
        self.as_ref()
            .create_vm_table(ty, style, vm_definition_location)
    }
}
