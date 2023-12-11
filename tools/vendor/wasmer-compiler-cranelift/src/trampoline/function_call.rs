// This file contains code from external sources.
// Attributions: https://github.com/wasmerio/wasmer/blob/master/ATTRIBUTIONS.md

//! A trampoline generator for calling Wasm functions easily.
//!
//! That way, you can start calling Wasm functions doing things like:
//! ```ignore
//! let my_func = instance.exports.get("func");
//! my_func.call([1, 2])
//! ```
use crate::translator::{compiled_function_unwind_info, signature_to_cranelift_ir};
use cranelift_codegen::ir;
use cranelift_codegen::ir::InstBuilder;
use cranelift_codegen::isa::TargetIsa;
use cranelift_codegen::Context;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use std::mem;
use wasmer_types::{CompileError, FunctionBody, FunctionType};

/// Create a trampoline for invoking a WebAssembly function.
pub fn make_trampoline_function_call(
    isa: &dyn TargetIsa,
    fn_builder_ctx: &mut FunctionBuilderContext,
    func_type: &FunctionType,
) -> Result<FunctionBody, CompileError> {
    let pointer_type = isa.pointer_type();
    let frontend_config = isa.frontend_config();
    let signature = signature_to_cranelift_ir(func_type, frontend_config);
    let mut wrapper_sig = ir::Signature::new(frontend_config.default_call_conv);

    // Add the callee `vmctx` parameter.
    wrapper_sig.params.push(ir::AbiParam::special(
        pointer_type,
        ir::ArgumentPurpose::VMContext,
    ));

    // Add the `callee_address` parameter.
    wrapper_sig.params.push(ir::AbiParam::new(pointer_type));

    // Add the `values_vec` parameter.
    wrapper_sig.params.push(ir::AbiParam::new(pointer_type));

    let mut context = Context::new();
    context.func = ir::Function::with_name_signature(ir::UserFuncName::user(0, 0), wrapper_sig);

    let value_size = mem::size_of::<u128>();
    {
        let mut builder = FunctionBuilder::new(&mut context.func, fn_builder_ctx);
        let block0 = builder.create_block();

        builder.append_block_params_for_function_params(block0);
        builder.switch_to_block(block0);
        builder.seal_block(block0);

        let (vmctx_ptr_val, callee_value, values_vec_ptr_val) = {
            let params = builder.func.dfg.block_params(block0);
            (params[0], params[1], params[2])
        };

        // Load the argument values out of `values_vec`.
        let mflags = ir::MemFlags::trusted();
        let callee_args = signature
            .params
            .iter()
            .enumerate()
            .map(|(i, r)| {
                match i {
                    0 => vmctx_ptr_val,
                    _ =>
                    // i - 1 because vmctx is not passed through `values_vec`.
                    {
                        builder.ins().load(
                            r.value_type,
                            mflags,
                            values_vec_ptr_val,
                            ((i - 1) * value_size) as i32,
                        )
                    }
                }
            })
            .collect::<Vec<_>>();

        let new_sig = builder.import_signature(signature);

        let call = builder
            .ins()
            .call_indirect(new_sig, callee_value, &callee_args);

        let results = builder.func.dfg.inst_results(call).to_vec();

        // Store the return values into `values_vec`.
        let mflags = ir::MemFlags::trusted();
        for (i, r) in results.iter().enumerate() {
            builder
                .ins()
                .store(mflags, *r, values_vec_ptr_val, (i * value_size) as i32);
        }

        builder.ins().return_(&[]);
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
