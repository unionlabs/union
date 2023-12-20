// This file contains code from external sources.
// Attributions: https://github.com/wasmerio/wasmer/blob/master/ATTRIBUTIONS.md

use cranelift_codegen::Context;
use cranelift_codegen::MachSrcLoc;
use std::ops::Range;
use wasmer_types::{FunctionAddressMap, InstructionAddressMap, SourceLoc};

pub fn get_function_address_map(
    context: &Context,
    range: Range<usize>,
    body_len: usize,
) -> FunctionAddressMap {
    let mut instructions = Vec::new();

    // New-style backend: we have a `MachCompileResult` that will give us `MachSrcLoc` mapping
    // tuples.
    let mcr = context.compiled_code().unwrap();
    for &MachSrcLoc { start, end, loc } in mcr.buffer.get_srclocs_sorted() {
        instructions.push(InstructionAddressMap {
            srcloc: SourceLoc::new(loc.bits()),
            code_offset: start as usize,
            code_len: (end - start) as usize,
        });
    }

    // Generate artificial srcloc for function start/end to identify boundary
    // within module. Similar to FuncTranslator::cur_srcloc(): it will wrap around
    // if byte code is larger than 4 GB.
    let start_srcloc = SourceLoc::new(range.start as u32);
    let end_srcloc = SourceLoc::new(range.end as u32);

    FunctionAddressMap {
        instructions,
        start_srcloc,
        end_srcloc,
        body_offset: 0,
        body_len,
    }
}
