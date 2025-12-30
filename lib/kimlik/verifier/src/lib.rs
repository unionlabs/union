#![warn(clippy::pedantic, clippy::unwrap_used)]

use anyhow::{Context, Result};
use wasmtime::{Engine, Linker, Module, Store};

/// Extract the kimlik stored in the provided wasm blob.
///
/// # Errors
///
/// This function will return an error if the provided wasm blob does not contain the expected kimlik export.
pub fn extract(bz: &[u8]) -> Result<String> {
    let engine = Engine::default();
    let module = Module::from_binary(&engine, bz)?;
    let mut linker = Linker::new(&engine);
    let mut store: Store<()> = Store::new(&engine, ());

    // stub all imports as they're unused when evaluating kimlik.
    for import in module.imports() {
        linker.func_new(
            import.module(),
            import.name(),
            import.ty().unwrap_func().clone(),
            |_, _, _| unimplemented!(),
        )?;
    }

    let instance = linker.instantiate(&mut store, &module)?;

    let kimlik_fn = instance.get_typed_func::<i32, ()>(&mut store, "kimlik")?;

    kimlik_fn.call(&mut store, 0)?;

    let memory = instance
        .get_memory(&mut store, "memory")
        .context("reading memory export")?;

    let ptr = i32::from_le_bytes(
        memory.data(&store)[0..4]
            .try_into()
            .context("reading pointer")?,
    );
    let len = i32::from_le_bytes(
        memory.data(&store)[4..8]
            .try_into()
            .context("reading length")?,
    );

    #[allow(clippy::cast_sign_loss)]
    let kimlik = str::from_utf8(&memory.data(&store)[(ptr as usize)..((ptr + len) as usize)])
        .context("kimlik is not utf8")?;

    Ok(kimlik.to_owned())
}
