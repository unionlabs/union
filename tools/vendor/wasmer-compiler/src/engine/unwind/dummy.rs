//! Module for Dummy unwind registry.

use wasmer_types::CompiledFunctionUnwindInfo;

/// Represents a registry of function unwind information when the host system
/// support any one in specific.
pub struct DummyUnwindRegistry {}

impl DummyUnwindRegistry {
    /// Creates a new unwind registry with the given base address.
    pub fn new() -> Self {
        DummyUnwindRegistry {}
    }

    /// Registers a function given the start offset, length, and unwind information.
    pub fn register(
        &mut self,
        _base_address: usize,
        _func_start: u32,
        _func_len: u32,
        _info: &CompiledFunctionUnwindInfo,
    ) -> Result<(), String> {
        // Do nothing
        Ok(())
    }

    /// Publishes all registered functions.
    pub fn publish(&mut self, eh_frame: Option<&[u8]>) -> Result<(), String> {
        // Do nothing
        Ok(())
    }
}
