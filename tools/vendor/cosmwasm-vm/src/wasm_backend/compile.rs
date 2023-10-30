use std::sync::Arc;

use wasmer::{Module, ModuleMiddleware};

use crate::errors::VmResult;
use crate::size::Size;

use super::store::make_compile_time_store;

/// Compiles a given Wasm bytecode into a module.
/// The given memory limit (in bytes) is used when memories are created.
/// If no memory limit is passed, the resulting compiled module should
/// not be used for execution.
pub fn compile(
    code: &[u8],
    memory_limit: Option<Size>,
    middlewares: &[Arc<dyn ModuleMiddleware>],
) -> VmResult<Module> {
    let store = make_compile_time_store(memory_limit, middlewares);
    let module = Module::new(&store, code)?;
    Ok(module)
}

#[cfg(test)]
mod tests {
    use super::*;

    static CONTRACT: &[u8] = include_bytes!("../../testdata/floaty.wasm");

    #[test]
    fn contract_with_floats_fails_check() {
        let err = compile(CONTRACT, None, &[]).unwrap_err();
        assert!(err.to_string().contains("Float operator detected:"));
    }
}
