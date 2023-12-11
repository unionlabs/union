// This file contains code from external sources.
// Attributions: https://github.com/wasmerio/wasmer/blob/master/ATTRIBUTIONS.md

//! A trampoline generator for calling dynamic host functions from Wasm.

use crate::translator::{compiled_function_unwind_info, signature_to_cranelift_ir};
use cranelift_codegen::ir;
use cranelift_codegen::ir::{
    Function, InstBuilder, MemFlags, StackSlotData, StackSlotKind, UserFuncName,
};
use cranelift_codegen::isa::TargetIsa;
use cranelift_codegen::Context;
use std::cmp;
use std::mem;

use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use wasmer_types::{CompileError, FunctionBody, FunctionType, VMOffsets};

/// Create a trampoline for invoking a WebAssembly function.
pub fn make_trampoline_dynamic_function(
    isa: &dyn TargetIsa,
    offsets: &VMOffsets,
    fn_builder_ctx: &mut FunctionBuilderContext,
    func_type: &FunctionType,
) -> Result<FunctionBody, CompileError> {
    let pointer_type = isa.pointer_type();
    let frontend_config = isa.frontend_config();
    let signature = signature_to_cranelift_ir(func_type, frontend_config);
    let mut stub_sig = ir::Signature::new(frontend_config.default_call_conv);
    // Add the caller `vmctx` parameter.
    stub_sig.params.push(ir::AbiParam::special(
        pointer_type,
        ir::ArgumentPurpose::VMContext,
    ));

    // Add the `values_vec` parameter.
    stub_sig.params.push(ir::AbiParam::new(pointer_type));

    // Compute the size of the values vector. The vmctx and caller vmctx are passed separately.
    let value_size = mem::size_of::<u128>();
    let values_vec_len =
        (value_size * cmp::max(signature.params.len() - 1, signature.returns.len())) as u32;

    let mut context = Context::new();
    context.func = Function::with_name_signature(UserFuncName::user(0, 0), signature.clone());

    let ss = context.func.create_sized_stack_slot(StackSlotData::new(
        StackSlotKind::ExplicitSlot,
        values_vec_len,
    ));

    {
        let mut builder = FunctionBuilder::new(&mut context.func, fn_builder_ctx);
        let block0 = builder.create_block();

        builder.append_block_params_for_function_params(block0);
        builder.switch_to_block(block0);
        builder.seal_block(block0);

        let values_vec_ptr_val = builder.ins().stack_addr(pointer_type, ss, 0);
        let mflags = MemFlags::trusted();
        // We only get the non-vmctx arguments
        for i in 1..signature.params.len() {
            let val = builder.func.dfg.block_params(block0)[i];
            builder.ins().store(
                mflags,
                val,
                values_vec_ptr_val,
                ((i - 1) * value_size) as i32,
            );
        }

        let block_params = builder.func.dfg.block_params(block0);
        let vmctx_ptr_val = block_params[0];
        let callee_args = vec![vmctx_ptr_val, values_vec_ptr_val];

        let new_sig = builder.import_signature(stub_sig);

        let mem_flags = ir::MemFlags::trusted();
        let callee_value = builder.ins().load(
            pointer_type,
            mem_flags,
            vmctx_ptr_val,
            offsets.vmdynamicfunction_import_context_address() as i32,
        );

        builder
            .ins()
            .call_indirect(new_sig, callee_value, &callee_args);

        let mflags = MemFlags::trusted();
        let mut results = Vec::new();
        for (i, r) in signature.returns.iter().enumerate() {
            let load = builder.ins().load(
                r.value_type,
                mflags,
                values_vec_ptr_val,
                (i * value_size) as i32,
            );
            results.push(load);
        }
        builder.ins().return_(&results);
        builder.finalize()
    }

    let mut code_buf = Vec::new();
    context
        .compile_and_emit(isa, &mut code_buf)
        .map_err(|error| CompileError::Codegen(error.inner.to_string()))?;

    let unwind_info = compiled_function_unwind_info(isa, &context)?.maybe_into_to_windows_unwind();

    Ok(FunctionBody {
        body: code_buf,
        unwind_info,
    })
}
