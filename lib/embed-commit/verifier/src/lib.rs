#![warn(clippy::pedantic, clippy::unwrap_used)]

use std::{path::PathBuf, sync::Arc};

use anyhow::{Context, Result, anyhow};
use clap::Parser;
use elf::{ElfBytes, endian::AnyEndian};
use embed_commit::Rev;
use solana_sbpf::{
    aligned_memory::AlignedMemory,
    ebpf,
    elf::Executable,
    elf_parser::consts::EM_SBPF,
    memory_region::{MemoryMapping, MemoryRegion},
    program::{BuiltinProgram, FunctionRegistry, SBPFVersion},
    verifier::RequisiteVerifier,
    vm::{ContextObject, EbpfVm},
};
use wasmtime::{Engine, Linker, Module, Store};

#[derive(Parser)]
enum App {
    /// Extract the commit information embedded in the artifact.
    Extract { path: PathBuf },
}

/// Parse the git rev embedded in the `GIT_REV` note section of the provided elf binary bytes.
///
/// # Errors
///
/// This function will error if the elf binary bytes provided cannot be parsed, or if the embedded git rev cannot be parsed. If there is no embedded git rev then `Ok(None)` will be returned.
pub fn extract_elf(bz: &[u8]) -> Result<Option<Rev>> {
    let file = ElfBytes::<AnyEndian>::minimal_parse(bz).context("parsing elf file")?;

    if file.ehdr.e_machine == EM_SBPF {
        extract_solana(bz)
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

/// Parse the git rev embedded in the `GIT_REV` symbol of the provided solana sbpf binary.
///
/// # Errors
///
/// This function will error if the elf binary bytes provided cannot be parsed, or if the embedded git rev cannot be parsed. If there is no embedded git rev then `Ok(None)` will be returned.
pub fn extract_solana(bz: &[u8]) -> Result<Option<Rev>> {
    struct DummyContextObject;
    impl ContextObject for DummyContextObject {
        fn consume(&mut self, _amount: u64) {}

        fn get_remaining(&self) -> u64 {
            0
        }
    }

    let loader = Arc::new(BuiltinProgram::new_mock());
    let executable =
        Executable::<DummyContextObject>::load_with_strict_parser(bz, loader.clone()).unwrap();
    executable.verify::<RequisiteVerifier>().unwrap();

    const X: u64 = 3069975057;

    let mut stack =
        AlignedMemory::<{ ebpf::HOST_ALIGN }>::zero_filled(executable.get_config().stack_size());
    let stack_len = stack.len();
    let mut heap = AlignedMemory::<{ ebpf::HOST_ALIGN }>::with_capacity(0);

    let mut mem = &mut vec![];

    let regions: Vec<MemoryRegion> = vec![
        executable.get_ro_region(),
        MemoryRegion::new_writable(stack.as_slice_mut(), ebpf::MM_STACK_START),
        MemoryRegion::new_writable(heap.as_slice_mut(), ebpf::MM_HEAP_START),
        MemoryRegion::new_writable(mem, ebpf::MM_INPUT_START),
    ];

    let memory_mapping = MemoryMapping::new(
        regions,
        executable.get_config(),
        executable.get_sbpf_version(),
    )
    .unwrap();

    let mut ctx = DummyContextObject;

    let mut vm = EbpfVm::new(
        loader,
        executable.get_sbpf_version(),
        &mut ctx,
        memory_mapping,
        stack_len,
    );

    let res = vm.execute_program(&executable, true);

    dbg!(res);

    // bytemuck::checked::try_from_bytes::<Rev>(&bytes[0..std::mem::size_of::<Rev>()])
    //     .map_err(|e| anyhow!(e.to_string()))
    //     .context("parsing rev")
    //     .map(|rev| Some(*rev))

    todo!()
}
