use wasmer_compiler::{FunctionAddressMap, FunctionBodyData, InstructionAddressMap, SourceLoc};

pub fn get_function_address_map<'data>(
    instructions: Vec<InstructionAddressMap>,
    data: &FunctionBodyData<'data>,
    body_len: usize,
) -> FunctionAddressMap {
    // Generate source loc for a function start/end to identify boundary within module.
    // It will wrap around if byte code is larger than 4 GB.
    let start_srcloc = SourceLoc::new(data.module_offset as u32);
    let end_srcloc = SourceLoc::new((data.module_offset + data.data.len()) as u32);

    FunctionAddressMap {
        instructions,
        start_srcloc,
        end_srcloc,
        body_offset: 0,
        body_len,
    }
}
