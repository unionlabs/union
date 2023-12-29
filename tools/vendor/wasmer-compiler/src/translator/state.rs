// This file contains code from external sources.
// Attributions: https://github.com/wasmerio/wasmer/blob/master/ATTRIBUTIONS.md

use std::boxed::Box;
use wasmer_types::entity::PrimaryMap;
use wasmer_types::{SignatureIndex, WasmResult};

/// Map of signatures to a function's parameter and return types.
pub(crate) type WasmTypes =
    PrimaryMap<SignatureIndex, (Box<[wasmparser::ValType]>, Box<[wasmparser::ValType]>)>;

/// Contains information decoded from the Wasm module that must be referenced
/// during each Wasm function's translation.
///
/// This is only for data that is maintained by `wasmer-compiler` itself, as
/// opposed to being maintained by the embedder. Data that is maintained by the
/// embedder is represented with `ModuleEnvironment`.
#[derive(Debug)]
pub struct ModuleTranslationState {
    /// A map containing a Wasm module's original, raw signatures.
    ///
    /// This is used for translating multi-value Wasm blocks inside functions,
    /// which are encoded to refer to their type signature via index.
    pub(crate) wasm_types: WasmTypes,
}

impl ModuleTranslationState {
    /// Creates a new empty ModuleTranslationState.
    pub fn new() -> Self {
        Self {
            wasm_types: PrimaryMap::new(),
        }
    }

    /// Get the parameter and result types for the given Wasm blocktype.
    pub fn blocktype_params_results(
        &self,
        ty_or_ft: wasmparser::BlockType,
    ) -> WasmResult<(&[wasmparser::ValType], &[wasmparser::ValType])> {
        Ok(match ty_or_ft {
            wasmparser::BlockType::Type(ty) => match ty {
                wasmparser::ValType::I32 => (&[], &[wasmparser::ValType::I32]),
                wasmparser::ValType::I64 => (&[], &[wasmparser::ValType::I64]),
                wasmparser::ValType::F32 => (&[], &[wasmparser::ValType::F32]),
                wasmparser::ValType::F64 => (&[], &[wasmparser::ValType::F64]),
                wasmparser::ValType::V128 => (&[], &[wasmparser::ValType::V128]),
                wasmparser::ValType::ExternRef => (&[], &[wasmparser::ValType::ExternRef]),
                wasmparser::ValType::FuncRef => (&[], &[wasmparser::ValType::FuncRef]),
            },
            wasmparser::BlockType::FuncType(ty_index) => {
                let sig_idx = SignatureIndex::from_u32(ty_index);
                let (ref params, ref results) = self.wasm_types[sig_idx];
                (params, results)
            }
            wasmparser::BlockType::Empty => (&[], &[]),
        })
    }
}
