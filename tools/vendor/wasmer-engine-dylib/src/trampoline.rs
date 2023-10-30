//! Trampolines for libcalls.
//!
//! This is needed because the target of libcall relocations are not reachable
//! through normal branch instructions.
//!
//! There is an additional complexity for dynamic libraries: we can't just
//! import the symbol from the host executable because executables don't export
//! dynamic symbols (it's possible but requires special linker options).
//!
//! Instead, we export a table of function pointers in the data section which is
//! manually filled in by the runtime after the dylib is loaded.

use enum_iterator::IntoEnumIterator;
use object::{
    elf, macho,
    write::{Object, Relocation, SectionId, StandardSection, Symbol, SymbolId, SymbolSection},
    BinaryFormat, RelocationEncoding, RelocationKind, SymbolFlags, SymbolKind, SymbolScope,
};
use wasmer_compiler::{Architecture, Target};
use wasmer_vm::libcalls::{function_pointer, LibCall};

/// Symbol exported from the dynamic library which points to the trampoline table.
pub const WASMER_TRAMPOLINES_SYMBOL: &[u8] = b"WASMER_TRAMPOLINES";

// SystemV says that both x16 and x17 are available as intra-procedural scratch
// registers but Apple's ABI restricts us to use x17.
// ADRP x17, #...        11 00 00 90
// LDR x17, [x17, #...]  31 02 40 f9
// BR x17                20 02 1f d6
const AARCH64_TRAMPOLINE: [u8; 12] = [
    0x11, 0x00, 0x00, 0x90, 0x31, 0x02, 0x40, 0xf9, 0x20, 0x02, 0x1f, 0xd6,
];

// JMP [RIP + ...]   FF 25 00 00 00 00
const X86_64_TRAMPOLINE: [u8; 6] = [0xff, 0x25, 0x00, 0x00, 0x00, 0x00];

fn emit_trampoline(
    obj: &mut Object,
    text: SectionId,
    trampoline_table_symbols: &[SymbolId],
    libcall: LibCall,
    target: &Target,
) {
    let function_name = libcall.to_function_name();
    let libcall_symbol = obj.add_symbol(Symbol {
        name: function_name.as_bytes().to_vec(),
        value: 0,
        size: 0,
        kind: SymbolKind::Text,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(text),
        flags: SymbolFlags::None,
    });

    match target.triple().architecture {
        Architecture::Aarch64(_) => {
            let (reloc1, reloc2) = match obj.format() {
                BinaryFormat::Elf => (
                    RelocationKind::Elf(elf::R_AARCH64_ADR_PREL_PG_HI21),
                    RelocationKind::Elf(elf::R_AARCH64_LDST64_ABS_LO12_NC),
                ),
                BinaryFormat::MachO => (
                    RelocationKind::MachO {
                        value: macho::ARM64_RELOC_PAGE21,
                        relative: true,
                    },
                    RelocationKind::MachO {
                        value: macho::ARM64_RELOC_PAGEOFF12,
                        relative: false,
                    },
                ),
                _ => panic!("Unsupported binary format on AArch64"),
            };
            let offset = obj.add_symbol_data(libcall_symbol, text, &AARCH64_TRAMPOLINE, 4);
            obj.add_relocation(
                text,
                Relocation {
                    offset,
                    size: 32,
                    kind: reloc1,
                    encoding: RelocationEncoding::Generic,
                    symbol: trampoline_table_symbols[libcall as usize],
                    addend: 0,
                },
            )
            .unwrap();
            obj.add_relocation(
                text,
                Relocation {
                    offset: offset + 4,
                    size: 32,
                    kind: reloc2,
                    encoding: RelocationEncoding::Generic,
                    symbol: trampoline_table_symbols[libcall as usize],
                    addend: 0,
                },
            )
            .unwrap();
        }
        Architecture::X86_64 => {
            let offset = obj.add_symbol_data(libcall_symbol, text, &X86_64_TRAMPOLINE, 1);
            obj.add_relocation(
                text,
                Relocation {
                    offset: offset + 2,
                    size: 32,
                    kind: RelocationKind::Relative,
                    encoding: RelocationEncoding::Generic,
                    symbol: trampoline_table_symbols[libcall as usize],
                    // -4 because RIP-relative addressing starts from the end of
                    // the instruction.
                    addend: -4,
                },
            )
            .unwrap();
        }
        arch => panic!("Unsupported architecture: {}", arch),
    };
}

/// Emits the libcall trampolines and table to the object file.
pub fn emit_trampolines(obj: &mut Object, target: &Target) {
    let text = obj.section_id(StandardSection::Text);
    let bss = obj.section_id(StandardSection::UninitializedData);

    let trampoline_table = obj.add_symbol(Symbol {
        name: WASMER_TRAMPOLINES_SYMBOL.to_vec(),
        value: 0,
        size: 0,
        kind: SymbolKind::Data,
        scope: SymbolScope::Dynamic,
        weak: false,
        section: SymbolSection::Section(bss),
        flags: SymbolFlags::None,
    });
    let table_offset =
        obj.add_symbol_bss(trampoline_table, bss, LibCall::VARIANT_COUNT as u64 * 8, 8);

    // Create a symbol for each entry in the table. We could avoid this and use
    // an addend, but this isn't supported in all object formats.
    let mut trampoline_table_symbols = vec![];
    for libcall in LibCall::into_enum_iter() {
        trampoline_table_symbols.push(obj.add_symbol(Symbol {
            name: format!("__WASMER_TRAMPOLINE{}", libcall as usize).into_bytes(),
            value: table_offset + libcall as u64 * 8,
            size: 0,
            kind: SymbolKind::Data,
            scope: SymbolScope::Compilation,
            weak: false,
            section: SymbolSection::Section(bss),
            flags: SymbolFlags::None,
        }));
    }

    for libcall in LibCall::into_enum_iter() {
        emit_trampoline(obj, text, &trampoline_table_symbols, libcall, target);
    }
}

/// Fills in the libcall trampoline table at the given address.
pub unsafe fn fill_trampoline_table(table: *mut usize) {
    for libcall in LibCall::into_enum_iter() {
        *table.add(libcall as usize) = function_pointer(libcall);
    }
}
