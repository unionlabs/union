#![warn(clippy::pedantic, clippy::unwrap_used)]

use std::path::PathBuf;

use anyhow::{anyhow, bail, Context, Result};
use clap::Parser;
use elf::{endian::AnyEndian, ElfBytes};
use embed_commit::Rev;
use wasmtime::{Engine, Linker, Module, Store};

#[derive(Parser)]
enum App {
    /// Extract the commit information embedded in the artifact.
    Extract { path: PathBuf },
}

fn main() -> Result<()> {
    match App::parse() {
        App::Extract { path } => {
            let file = std::fs::read(path).context("reading input artifact")?;

            let rev = match &file.get(0..4) {
                Some(b"\0asm") => extract_wasm(&file)?,
                Some(b"\0elf") => extract_elf(&file)?,
                Some(b"\x7FELF") => extract_elf(&file)?,
                Some(magic) => bail!("unknown file magic {magic:?}"),
                None => bail!("file is < 4 bytes"),
            };

            match rev {
                Some(rev) => println!("{rev}"),
                None => println!("none"),
            }
        }
    }

    Ok(())
}

fn extract_elf(bz: &[u8]) -> Result<Option<Rev>> {
    let file = ElfBytes::<AnyEndian>::minimal_parse(bz).context("parsing elf file")?;

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

fn extract_wasm(bz: &[u8]) -> Result<Option<Rev>> {
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

    let commit_hash_fn = instance.get_typed_func::<i32, ()>(&mut store, "commit_hash")?;

    commit_hash_fn.call(&mut store, 0)?;

    let memory = instance
        .get_memory(&mut store, "memory")
        .context("reading memory export")?;

    bytemuck::checked::try_from_bytes::<Rev>(&memory.data(&store)[0..std::mem::size_of::<Rev>()])
        .map_err(|e| anyhow!(e.to_string()))
        .context("parsing rev")
        .map(|rev| Some(*rev))
}
