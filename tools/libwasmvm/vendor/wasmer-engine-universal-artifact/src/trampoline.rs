//! Trampolines for libcalls.
//!
//! This is needed because the target of libcall relocations are not reachable
//! through normal branch instructions.

use enum_iterator::IntoEnumIterator;
use wasmer_compiler::{
    Architecture, CustomSection, CustomSectionProtection, Relocation, RelocationKind,
    RelocationTarget, SectionBody, Target,
};
use wasmer_types::LibCall;

// SystemV says that both x16 and x17 are available as intra-procedural scratch
// registers but Apple's ABI restricts us to use x17.
// LDR x17, [PC, #8]  51 00 00 58
// BR x17             20 02 1f d6
// JMPADDR            00 00 00 00 00 00 00 00
const AARCH64_TRAMPOLINE: [u8; 16] = [
    0x51, 0x00, 0x00, 0x58, 0x20, 0x02, 0x1f, 0xd6, 0, 0, 0, 0, 0, 0, 0, 0,
];

// 2 padding bytes are used to preserve alignment.
// JMP [RIP + 2]   FF 25 02 00 00 00 [00 00]
// 64-bit ADDR     00 00 00 00 00 00 00 00
const X86_64_TRAMPOLINE: [u8; 16] = [
    0xff, 0x25, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

fn make_trampoline(
    target: &Target,
    libcall: LibCall,
    code: &mut Vec<u8>,
    relocations: &mut Vec<Relocation>,
) {
    match target.triple().architecture {
        Architecture::Aarch64(_) => {
            code.extend(&AARCH64_TRAMPOLINE);
            relocations.push(Relocation {
                kind: RelocationKind::Abs8,
                reloc_target: RelocationTarget::LibCall(libcall),
                offset: code.len() as u32 - 8,
                addend: 0,
            });
        }
        Architecture::X86_64 => {
            code.extend(&X86_64_TRAMPOLINE);
            relocations.push(Relocation {
                kind: RelocationKind::Abs8,
                reloc_target: RelocationTarget::LibCall(libcall),
                offset: code.len() as u32 - 8,
                addend: 0,
            });
        }
        arch => panic!("Unsupported architecture: {}", arch),
    };
}

/// Returns the length of a libcall trampoline.
pub fn libcall_trampoline_len(target: &Target) -> usize {
    match target.triple().architecture {
        Architecture::Aarch64(_) => AARCH64_TRAMPOLINE.len(),
        Architecture::X86_64 => X86_64_TRAMPOLINE.len(),
        arch => panic!("Unsupported architecture: {}", arch),
    }
}

/// Creates a custom section containing the libcall trampolines.
pub fn make_libcall_trampolines(target: &Target) -> CustomSection {
    let mut code = vec![];
    let mut relocations = vec![];
    for libcall in LibCall::into_enum_iter() {
        make_trampoline(target, libcall, &mut code, &mut relocations);
    }
    CustomSection {
        protection: CustomSectionProtection::ReadExecute,
        bytes: SectionBody::new_with_vec(code),
        relocations,
    }
}

/// Returns the address of a trampoline in the libcall trampolines section.
pub fn get_libcall_trampoline(
    libcall: LibCall,
    libcall_trampolines: usize,
    libcall_trampoline_len: usize,
) -> usize {
    libcall_trampolines + libcall as usize * libcall_trampoline_len
}
