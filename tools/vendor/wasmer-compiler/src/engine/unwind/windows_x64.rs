// This file contains code from external sources.
// Attributions: https://github.com/wasmerio/wasmer/blob/master/ATTRIBUTIONS.md

//! Module for Windows x64 ABI unwind registry.
use std::collections::HashMap;
use wasmer_types::CompiledFunctionUnwindInfoReference;
use winapi::um::winnt;

/// Represents a registry of function unwind information for Windows x64 ABI.
pub struct UnwindRegistry {
    // A hashmap mapping the baseaddress with the registered runtime functions
    functions: HashMap<usize, Vec<winnt::RUNTIME_FUNCTION>>,
    published: bool,
}

impl UnwindRegistry {
    /// Creates a new unwind registry with the given base address.
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            published: false,
        }
    }

    /// Registers a function given the start offset, length, and unwind information.
    pub fn register(
        &mut self,
        base_address: usize,
        func_start: u32,
        func_len: u32,
        info: &CompiledFunctionUnwindInfoReference,
    ) -> Result<(), String> {
        if self.published {
            return Err("unwind registry has already been published".to_string());
        }

        match info {
            CompiledFunctionUnwindInfoReference::WindowsX64(_) => {}
            _ => return Err("unsupported unwind information".to_string()),
        };

        let mut entry = winnt::RUNTIME_FUNCTION::default();

        entry.BeginAddress = func_start;
        entry.EndAddress = func_start + func_len;

        // The unwind information should be immediately following the function
        // with padding for 4 byte alignment
        unsafe {
            *entry.u.UnwindInfoAddress_mut() = (entry.EndAddress + 3) & !3;
        }
        let entries = self.functions.entry(base_address).or_insert_with(Vec::new);

        entries.push(entry);

        Ok(())
    }

    /// Publishes all registered functions.
    pub fn publish(&mut self, _eh_frame: Option<&[u8]>) -> Result<(), String> {
        if self.published {
            return Err("unwind registry has already been published".to_string());
        }

        self.published = true;

        if !self.functions.is_empty() {
            for (base_address, functions) in self.functions.iter_mut() {
                // Windows heap allocations are 32-bit aligned, but assert just in case
                assert_eq!(
                    (functions.as_mut_ptr() as u64) % 4,
                    0,
                    "function table allocation was not aligned"
                );
                unsafe {
                    if winnt::RtlAddFunctionTable(
                        functions.as_mut_ptr(),
                        functions.len() as u32,
                        *base_address as u64,
                    ) == 0
                    {
                        return Err("failed to register function tables".to_string());
                    }
                }
            }
        }

        Ok(())
    }
}

impl Drop for UnwindRegistry {
    fn drop(&mut self) {
        if self.published {
            unsafe {
                for functions in self.functions.values_mut() {
                    winnt::RtlDeleteFunctionTable(functions.as_mut_ptr());
                }
            }
        }
    }
}
