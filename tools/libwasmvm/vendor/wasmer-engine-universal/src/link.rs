//! Linking for Universal-compiled code.

use std::ptr::{read_unaligned, write_unaligned};
use wasmer_compiler::{Relocation, RelocationKind, RelocationTarget, Relocations, SectionIndex};
use wasmer_engine::FunctionExtent;
use wasmer_engine_universal_artifact::get_libcall_trampoline;
use wasmer_types::entity::PrimaryMap;
use wasmer_types::{LocalFunctionIndex, ModuleInfo};
use wasmer_vm::libcalls::function_pointer;
use wasmer_vm::SectionBodyPtr;

fn apply_relocation(
    body: usize,
    r: &Relocation,
    allocated_functions: &PrimaryMap<LocalFunctionIndex, FunctionExtent>,
    allocated_sections: &PrimaryMap<SectionIndex, SectionBodyPtr>,
    libcall_trampolines: SectionIndex,
    libcall_trampoline_len: usize,
) {
    let target_func_address: usize = match r.reloc_target {
        RelocationTarget::LocalFunc(index) => *allocated_functions[index].ptr as usize,
        RelocationTarget::LibCall(libcall) => {
            // Use the direct target of the libcall if the relocation supports
            // a full 64-bit address. Otherwise use a trampoline.
            if r.kind == RelocationKind::Abs8 || r.kind == RelocationKind::X86PCRel8 {
                function_pointer(libcall)
            } else {
                get_libcall_trampoline(
                    libcall,
                    allocated_sections[libcall_trampolines].0 as usize,
                    libcall_trampoline_len,
                )
            }
        }
        RelocationTarget::CustomSection(custom_section) => {
            *allocated_sections[custom_section] as usize
        }
    };

    match r.kind {
        RelocationKind::Abs8 => unsafe {
            let (reloc_address, reloc_delta) = r.for_address(body, target_func_address as u64);
            write_unaligned(reloc_address as *mut u64, reloc_delta);
        },
        RelocationKind::X86PCRel4 => unsafe {
            let (reloc_address, reloc_delta) = r.for_address(body, target_func_address as u64);
            write_unaligned(reloc_address as *mut u32, reloc_delta as _);
        },
        RelocationKind::X86PCRel8 => unsafe {
            let (reloc_address, reloc_delta) = r.for_address(body, target_func_address as u64);
            write_unaligned(reloc_address as *mut u64, reloc_delta);
        },
        RelocationKind::X86CallPCRel4 => unsafe {
            let (reloc_address, reloc_delta) = r.for_address(body, target_func_address as u64);
            write_unaligned(reloc_address as *mut u32, reloc_delta as _);
        },
        RelocationKind::Arm64Call => unsafe {
            let (reloc_address, reloc_delta) = r.for_address(body, target_func_address as u64);
            if (reloc_delta as i64).abs() >= 0x1000_0000 {
                panic!(
                    "Relocation to big for {:?} for {:?} with {:x}, current val {:x}",
                    r.kind,
                    r.reloc_target,
                    reloc_delta,
                    read_unaligned(reloc_address as *mut u32)
                )
            }
            let reloc_delta = (((reloc_delta / 4) as u32) & 0x3ff_ffff)
                | (read_unaligned(reloc_address as *mut u32) & 0xfc00_0000);
            write_unaligned(reloc_address as *mut u32, reloc_delta);
        },
        kind => panic!(
            "Relocation kind unsupported in the current architecture {}",
            kind
        ),
    }
}

/// Links a module, patching the allocated functions with the
/// required relocations and jump tables.
pub fn link_module(
    _module: &ModuleInfo,
    allocated_functions: &PrimaryMap<LocalFunctionIndex, FunctionExtent>,
    function_relocations: Relocations,
    allocated_sections: &PrimaryMap<SectionIndex, SectionBodyPtr>,
    section_relocations: &PrimaryMap<SectionIndex, Vec<Relocation>>,
    libcall_trampolines: SectionIndex,
    trampoline_len: usize,
) {
    for (i, section_relocs) in section_relocations.iter() {
        let body = *allocated_sections[i] as usize;
        for r in section_relocs {
            apply_relocation(
                body,
                r,
                allocated_functions,
                allocated_sections,
                libcall_trampolines,
                trampoline_len,
            );
        }
    }
    for (i, function_relocs) in function_relocations.iter() {
        let body = *allocated_functions[i].ptr as usize;
        for r in function_relocs {
            apply_relocation(
                body,
                r,
                allocated_functions,
                allocated_sections,
                libcall_trampolines,
                trampoline_len,
            );
        }
    }
}
