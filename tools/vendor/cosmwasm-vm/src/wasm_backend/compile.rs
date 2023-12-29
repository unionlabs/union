use wasmer::{Engine, Module};

use crate::errors::VmResult;

/// Compiles a given Wasm bytecode into a module.
pub fn compile(engine: &Engine, code: &[u8]) -> VmResult<Module> {
    let module = Module::new(&engine, code)?;
    Ok(module)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wasm_backend::make_compiling_engine;

    static CONTRACT: &[u8] = include_bytes!("../../testdata/floaty.wasm");

    #[test]
    fn contract_with_floats_passes_check() {
        let engine = make_compiling_engine(None);
        assert!(compile(&engine, CONTRACT).is_ok());
    }
}
