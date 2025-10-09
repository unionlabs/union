#![warn(clippy::pedantic, clippy::unwrap_used)]

use anyhow::{Context, Result, anyhow};
use elf::{ElfBytes, endian::AnyEndian};
use embed_commit::Rev;
use wasmtime::{Engine, Linker, Module, Store};

const EM_SBPF: u16 = 263;

/// Parse the git rev embedded in the `GIT_REV` note section of the provided elf binary bytes.
///
/// # Errors
///
/// This function will error if the elf binary bytes provided cannot be parsed, or if the embedded git rev cannot be parsed. If there is no embedded git rev then `Ok(None)` will be returned.
pub fn extract_elf(bz: &[u8]) -> Result<Option<Rev>> {
    let file = ElfBytes::<AnyEndian>::minimal_parse(bz).context("parsing elf file")?;

    if file.ehdr.e_machine == EM_SBPF {
        let Some(dynsym) = file
            .dynamic_symbol_table()
            .context("reading dynamic symbol table")?
        else {
            return Ok(None);
        };

        let Some(res) = dynsym.0.iter().find(|s| {
            dynsym
                .1
                .get(s.st_name as usize)
                .is_ok_and(|s| s == "GIT_REV")
        }) else {
            return Ok(None);
        };

        bytemuck::checked::try_from_bytes::<Rev>(
            &bz[res.st_value.try_into()?..(res.st_value + res.st_size).try_into()?],
        )
        .map_err(|e| anyhow!(e.to_string()))
        .context("parsing rev")
        .map(|rev| Some(*rev))
    } else {
        let Some(section) = file
            .section_header_by_name(".note.embed_commit.GIT_REV")
            .context("reading GIT_REV note section")?
        else {
            return Ok(None);
        };

        let (bytes, _) = file
            .section_data(&section)
            .context("reading GIT_REV note section data")?;

        bytemuck::checked::try_from_bytes::<Rev>(&bytes[0..std::mem::size_of::<Rev>()])
            .map_err(|e| anyhow!(e.to_string()))
            .context("parsing rev")
            .map(|rev| Some(*rev))
    }
}

/// Retrieve the git rev from the provided wasm binary bytes.
///
/// # Errors
///
/// This function will error if the wasm binary bytes provided cannot be parsed, or if the returned git rev cannot be parsed. If there is no `commit_hash` export then `Ok(None)` will be returned.
pub fn extract_wasm(bz: &[u8]) -> Result<Option<Rev>> {
    let engine = Engine::default();
    let module = Module::from_binary(&engine, bz)?;
    let mut linker = Linker::new(&engine);
    let mut store: Store<()> = Store::new(&engine, ());

    // stub all imports as they're unused when evaluating commit_hash
    for import in module.imports() {
        linker.func_new(
            import.module(),
            import.name(),
            import.ty().unwrap_func().clone(),
            |_, _, _| unimplemented!(),
        )?;
    }

    let instance = linker.instantiate(&mut store, &module)?;

    let Ok(commit_hash_fn) = instance.get_typed_func::<i32, ()>(&mut store, "commit_hash") else {
        return Ok(None);
    };

    commit_hash_fn.call(&mut store, 0)?;

    let memory = instance
        .get_memory(&mut store, "memory")
        .context("reading memory export")?;

    bytemuck::checked::try_from_bytes::<Rev>(&memory.data(&store)[0..std::mem::size_of::<Rev>()])
        .map_err(|e| anyhow!(e.to_string()))
        .context("parsing rev")
        .map(|rev| Some(*rev))
}
