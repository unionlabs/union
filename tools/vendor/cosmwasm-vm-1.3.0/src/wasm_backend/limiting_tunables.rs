use std::ptr::NonNull;
use std::sync::Arc;

use loupe::MemoryUsage;
use wasmer::{
    vm::{self, MemoryError, MemoryStyle, TableStyle, VMMemoryDefinition, VMTableDefinition},
    MemoryType, Pages, TableType, Tunables,
};

/// A custom tunables that allows you to set a memory limit.
///
/// After adjusting the memory limits, it delegates all other logic
/// to the base tunables.
#[derive(MemoryUsage)]
pub struct LimitingTunables<T: Tunables> {
    /// The maxium a linear memory is allowed to be (in Wasm pages, 65 KiB each).
    /// Since Wasmer ensures there is only none or one memory, this is practically
    /// an upper limit for the guest memory.
    limit: Pages,
    /// The base implementation we delegate all the logic to
    base: T,
}

impl<T: Tunables> LimitingTunables<T> {
    pub fn new(base: T, limit: Pages) -> Self {
        Self { limit, base }
    }

    /// Takes in input memory type as requested by the guest and sets
    /// a maximum if missing. The resulting memory type is final if
    /// valid. However, this can produce invalid types, such that
    /// validate_memory must be called before creating the memory.
    fn adjust_memory(&self, requested: &MemoryType) -> MemoryType {
        let mut adjusted = *requested;
        if requested.maximum.is_none() {
            adjusted.maximum = Some(self.limit);
        }
        adjusted
    }

    /// Ensures the a given memory type does not exceed the memory limit.
    /// Call this after adjusting the memory.
    fn validate_memory(&self, ty: &MemoryType) -> Result<(), MemoryError> {
        if ty.minimum > self.limit {
            return Err(MemoryError::Generic(
                "Minimum exceeds the allowed memory limit".to_string(),
            ));
        }

        if let Some(max) = ty.maximum {
            if max > self.limit {
                return Err(MemoryError::Generic(
                    "Maximum exceeds the allowed memory limit".to_string(),
                ));
            }
        } else {
            return Err(MemoryError::Generic("Maximum unset".to_string()));
        }

        Ok(())
    }
}

impl<T: Tunables> Tunables for LimitingTunables<T> {
    /// Construct a `MemoryStyle` for the provided `MemoryType`
    ///
    /// Delegated to base.
    fn memory_style(&self, memory: &MemoryType) -> MemoryStyle {
        let adjusted = self.adjust_memory(memory);
        self.base.memory_style(&adjusted)
    }

    /// Construct a `TableStyle` for the provided `TableType`
    ///
    /// Delegated to base.
    fn table_style(&self, table: &TableType) -> TableStyle {
        self.base.table_style(table)
    }

    /// Create a memory owned by the host given a [`MemoryType`] and a [`MemoryStyle`].
    ///
    /// The requested memory type is validated, adjusted to the limited and then passed to base.
    fn create_host_memory(
        &self,
        ty: &MemoryType,
        style: &MemoryStyle,
    ) -> Result<Arc<dyn vm::Memory>, MemoryError> {
        let adjusted = self.adjust_memory(ty);
        self.validate_memory(&adjusted)?;
        self.base.create_host_memory(&adjusted, style)
    }

    /// Create a memory owned by the VM given a [`MemoryType`] and a [`MemoryStyle`].
    ///
    /// Delegated to base.
    unsafe fn create_vm_memory(
        &self,
        ty: &MemoryType,
        style: &MemoryStyle,
        vm_definition_location: NonNull<VMMemoryDefinition>,
    ) -> Result<Arc<dyn vm::Memory>, MemoryError> {
        let adjusted = self.adjust_memory(ty);
        self.validate_memory(&adjusted)?;
        self.base
            .create_vm_memory(&adjusted, style, vm_definition_location)
    }

    /// Create a table owned by the host given a [`TableType`] and a [`TableStyle`].
    ///
    /// Delegated to base.
    fn create_host_table(
        &self,
        ty: &TableType,
        style: &TableStyle,
    ) -> Result<Arc<dyn vm::Table>, String> {
        self.base.create_host_table(ty, style)
    }

    /// Create a table owned by the VM given a [`TableType`] and a [`TableStyle`].
    ///
    /// Delegated to base.
    unsafe fn create_vm_table(
        &self,
        ty: &TableType,
        style: &TableStyle,
        vm_definition_location: NonNull<VMTableDefinition>,
    ) -> Result<Arc<dyn vm::Table>, String> {
        self.base.create_vm_table(ty, style, vm_definition_location)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasmer::{BaseTunables, Target};

    #[test]
    fn adjust_memory_works() {
        let limit = Pages(12);
        let limiting = LimitingTunables::new(BaseTunables::for_target(&Target::default()), limit);

        // No maximum
        let requested = MemoryType::new(3, None, true);
        let adjusted = limiting.adjust_memory(&requested);
        assert_eq!(adjusted, MemoryType::new(3, Some(12), true));

        // Maximum smaller than limit
        let requested = MemoryType::new(3, Some(7), true);
        let adjusted = limiting.adjust_memory(&requested);
        assert_eq!(adjusted, requested);

        // Maximum equal to limit
        let requested = MemoryType::new(3, Some(12), true);
        let adjusted = limiting.adjust_memory(&requested);
        assert_eq!(adjusted, requested);

        // Maximum greater than limit
        let requested = MemoryType::new(3, Some(20), true);
        let adjusted = limiting.adjust_memory(&requested);
        assert_eq!(adjusted, requested);

        // Minimum greater than maximum (not our problem)
        let requested = MemoryType::new(5, Some(3), true);
        let adjusted = limiting.adjust_memory(&requested);
        assert_eq!(adjusted, requested);

        // Minimum greater than limit
        let requested = MemoryType::new(20, Some(20), true);
        let adjusted = limiting.adjust_memory(&requested);
        assert_eq!(adjusted, requested);
    }

    #[test]
    fn validate_memory_works() {
        let limit = Pages(12);
        let limiting = LimitingTunables::new(BaseTunables::for_target(&Target::default()), limit);

        // Maximum smaller than limit
        let memory = MemoryType::new(3, Some(7), true);
        limiting.validate_memory(&memory).unwrap();

        // Maximum equal to limit
        let memory = MemoryType::new(3, Some(12), true);
        limiting.validate_memory(&memory).unwrap();

        // Maximum greater than limit
        let memory = MemoryType::new(3, Some(20), true);
        let result = limiting.validate_memory(&memory);
        match result.unwrap_err() {
            MemoryError::Generic(msg) => {
                assert_eq!(msg, "Maximum exceeds the allowed memory limit")
            }
            err => panic!("Unexpected error: {:?}", err),
        }

        // Maximum not set
        let memory = MemoryType::new(3, None, true);
        let result = limiting.validate_memory(&memory);
        match result.unwrap_err() {
            MemoryError::Generic(msg) => assert_eq!(msg, "Maximum unset"),
            err => panic!("Unexpected error: {:?}", err),
        }

        // Minimum greater than maximum (not our problem)
        let memory = MemoryType::new(5, Some(3), true);
        limiting.validate_memory(&memory).unwrap();

        // Minimum greater than limit
        let memory = MemoryType::new(20, Some(20), true);
        let result = limiting.validate_memory(&memory);
        match result.unwrap_err() {
            MemoryError::Generic(msg) => {
                assert_eq!(msg, "Minimum exceeds the allowed memory limit")
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }
}
