// This file contains code from external sources.
// Attributions: https://github.com/wasmerio/wasmer/blob/master/ATTRIBUTIONS.md

//! Standalone WebAssembly to Cranelift IR translator.
//!
//! This module defines the `FuncTranslator` type which can translate a single WebAssembly
//! function to Cranelift IR guided by a `FuncEnvironment` which provides information about the
//! WebAssembly module and the runtime environment.

use super::code_translator::{bitcast_arguments, translate_operator, wasm_param_types};
use super::func_environ::{FuncEnvironment, ReturnMode};
use super::func_state::FuncTranslationState;
use super::translation_utils::get_vmctx_value_label;
use cranelift_codegen::entity::EntityRef;
use cranelift_codegen::ir::{self, Block, InstBuilder, ValueLabel};
use cranelift_codegen::timing;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use tracing::info;
use wasmer_compiler::wasmparser;
use wasmer_compiler::{
    wasm_unsupported, wptype_to_type, FunctionBinaryReader, ModuleTranslationState, WasmResult,
};
use wasmer_types::LocalFunctionIndex;

/// WebAssembly to Cranelift IR function translator.
///
/// A `FuncTranslator` is used to translate a binary WebAssembly function into Cranelift IR guided
/// by a `FuncEnvironment` object. A single translator instance can be reused to translate multiple
/// functions which will reduce heap allocation traffic.
pub struct FuncTranslator {
    func_ctx: FunctionBuilderContext,
    state: FuncTranslationState,
}

impl FuncTranslator {
    /// Create a new translator.
    pub fn new() -> Self {
        Self {
            func_ctx: FunctionBuilderContext::new(),
            state: FuncTranslationState::new(),
        }
    }

    /// Translate a binary WebAssembly function.
    ///
    /// The `code` slice contains the binary WebAssembly *function code* as it appears in the code
    /// section of a WebAssembly module, not including the initial size of the function code. The
    /// slice is expected to contain two parts:
    ///
    /// - The declaration of *locals*, and
    /// - The function *body* as an expression.
    ///
    /// See [the WebAssembly specification][wasm].
    ///
    /// [wasm]: https://webassembly.github.io/spec/core/binary/modules.html#code-section
    ///
    /// The Cranelift IR function `func` should be completely empty except for the `func.signature`
    /// and `func.name` fields. The signature may contain special-purpose arguments which are not
    /// regarded as WebAssembly local variables. Any signature arguments marked as
    /// `ArgumentPurpose::Normal` are made accessible as WebAssembly local variables.
    ///
    pub fn translate<FE: FuncEnvironment + ?Sized>(
        &mut self,
        module_translation_state: &ModuleTranslationState,
        reader: &mut dyn FunctionBinaryReader,
        func: &mut ir::Function,
        environ: &mut FE,
        local_function_index: LocalFunctionIndex,
    ) -> WasmResult<()> {
        environ.push_params_on_stack(local_function_index);
        self.translate_from_reader(module_translation_state, reader, func, environ)
    }

    /// Translate a binary WebAssembly function from a `FunctionBinaryReader`.
    pub fn translate_from_reader<FE: FuncEnvironment + ?Sized>(
        &mut self,
        module_translation_state: &ModuleTranslationState,
        reader: &mut dyn FunctionBinaryReader,
        func: &mut ir::Function,
        environ: &mut FE,
    ) -> WasmResult<()> {
        let _tt = timing::wasm_translate_function();
        info!(
            "translate({} bytes, {}{})",
            reader.bytes_remaining(),
            func.name,
            func.signature
        );
        debug_assert_eq!(func.dfg.num_blocks(), 0, "Function must be empty");
        debug_assert_eq!(func.dfg.num_insts(), 0, "Function must be empty");

        // This clears the `FunctionBuilderContext`.
        let mut builder = FunctionBuilder::new(func, &mut self.func_ctx);
        builder.set_srcloc(cur_srcloc(reader));
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block); // This also creates values for the arguments.
        builder.seal_block(entry_block); // Declare all predecessors known.

        // Make sure the entry block is inserted in the layout before we make any callbacks to
        // `environ`. The callback functions may need to insert things in the entry block.
        builder.ensure_inserted_block();

        let num_params = declare_wasm_parameters(&mut builder, entry_block, environ);

        // Set up the translation state with a single pushed control block representing the whole
        // function and its return values.
        let exit_block = builder.create_block();
        builder.append_block_params_for_function_returns(exit_block);
        self.state.initialize(&builder.func.signature, exit_block);

        parse_local_decls(reader, &mut builder, num_params, environ)?;
        parse_function_body(
            module_translation_state,
            reader,
            &mut builder,
            &mut self.state,
            environ,
        )?;

        builder.finalize();
        Ok(())
    }
}

/// Declare local variables for the signature parameters that correspond to WebAssembly locals.
///
/// Return the number of local variables declared.
fn declare_wasm_parameters<FE: FuncEnvironment + ?Sized>(
    builder: &mut FunctionBuilder,
    entry_block: Block,
    environ: &FE,
) -> usize {
    let sig_len = builder.func.signature.params.len();
    let mut next_local = 0;
    for i in 0..sig_len {
        let param_type = builder.func.signature.params[i];
        // There may be additional special-purpose parameters in addition to the normal WebAssembly
        // signature parameters. For example, a `vmctx` pointer.
        if environ.is_wasm_parameter(&builder.func.signature, i) {
            // This is a normal WebAssembly signature parameter, so create a local for it.
            let local = Variable::new(next_local);
            builder.declare_var(local, param_type.value_type);
            next_local += 1;

            let param_value = builder.block_params(entry_block)[i];
            builder.def_var(local, param_value);
        }
        if param_type.purpose == ir::ArgumentPurpose::VMContext {
            let param_value = builder.block_params(entry_block)[i];
            builder.set_val_label(param_value, get_vmctx_value_label());
        }
    }

    next_local
}

/// Parse the local variable declarations that precede the function body.
///
/// Declare local variables, starting from `num_params`.
fn parse_local_decls<FE: FuncEnvironment + ?Sized>(
    reader: &mut dyn FunctionBinaryReader,
    builder: &mut FunctionBuilder,
    num_params: usize,
    environ: &mut FE,
) -> WasmResult<()> {
    let mut next_local = num_params;
    let local_count = reader.read_local_count()?;

    for _ in 0..local_count {
        builder.set_srcloc(cur_srcloc(reader));
        let (count, ty) = reader.read_local_decl()?;
        declare_locals(builder, count, ty, &mut next_local, environ)?;
    }

    Ok(())
}

/// Declare `count` local variables of the same type, starting from `next_local`.
///
/// Fail if the type is not valid for a local.
fn declare_locals<FE: FuncEnvironment + ?Sized>(
    builder: &mut FunctionBuilder,
    count: u32,
    wasm_type: wasmparser::Type,
    next_local: &mut usize,
    environ: &mut FE,
) -> WasmResult<()> {
    // All locals are initialized to 0.
    use wasmparser::Type::*;
    let zeroval = match wasm_type {
        I32 => builder.ins().iconst(ir::types::I32, 0),
        I64 => builder.ins().iconst(ir::types::I64, 0),
        F32 => builder.ins().f32const(ir::immediates::Ieee32::with_bits(0)),
        F64 => builder.ins().f64const(ir::immediates::Ieee64::with_bits(0)),
        V128 => {
            let constant_handle = builder.func.dfg.constants.insert([0; 16].to_vec().into());
            builder.ins().vconst(ir::types::I8X16, constant_handle)
        }
        ExternRef => builder.ins().null(environ.reference_type()),
        FuncRef => builder.ins().null(environ.reference_type()),
        ty => return Err(wasm_unsupported!("unsupported local type {:?}", ty)),
    };

    let wasmer_ty = wptype_to_type(wasm_type).unwrap();
    let ty = builder.func.dfg.value_type(zeroval);
    for _ in 0..count {
        let local = Variable::new(*next_local);
        builder.declare_var(local, ty);
        builder.def_var(local, zeroval);
        builder.set_val_label(zeroval, ValueLabel::new(*next_local));
        environ.push_local_decl_on_stack(wasmer_ty);
        *next_local += 1;
    }
    Ok(())
}

/// Parse the function body in `reader`.
///
/// This assumes that the local variable declarations have already been parsed and function
/// arguments and locals are declared in the builder.
fn parse_function_body<FE: FuncEnvironment + ?Sized>(
    module_translation_state: &ModuleTranslationState,
    reader: &mut dyn FunctionBinaryReader,
    builder: &mut FunctionBuilder,
    state: &mut FuncTranslationState,
    environ: &mut FE,
) -> WasmResult<()> {
    // The control stack is initialized with a single block representing the whole function.
    debug_assert_eq!(state.control_stack.len(), 1, "State not initialized");

    // Keep going until the final `End` operator which pops the outermost block.
    while !state.control_stack.is_empty() {
        builder.set_srcloc(cur_srcloc(reader));
        let op = reader.read_operator()?;
        environ.before_translate_operator(&op, builder, state)?;
        translate_operator(module_translation_state, &op, builder, state, environ)?;
        environ.after_translate_operator(&op, builder, state)?;
    }

    // When returning we drop all values in locals and on the stack.

    // The final `End` operator left us in the exit block where we need to manually add a return
    // instruction.
    //
    // If the exit block is unreachable, it may not have the correct arguments, so we would
    // generate a return instruction that doesn't match the signature.
    if state.reachable {
        debug_assert!(builder.is_pristine());
        if !builder.is_unreachable() {
            environ.translate_drop_locals(builder)?;

            let _num_elems_to_drop = state.stack.len() - builder.func.signature.returns.len();
            // drop elements on the stack that we're not returning
            /*for val in state
                .stack
                .iter()
                .zip(state.metadata_stack.iter())
                .take(num_elems_to_drop)
                .filter(|(_, metadata)| metadata.ref_counted)
                .map(|(val, _)| val)
            {
                environ.translate_externref_dec(builder.cursor(), *val)?;
            }*/

            // TODO: look into what `state.reachable` check above does as well as `!builder.is_unreachable`, do we need that too for ref counting?

            match environ.return_mode() {
                ReturnMode::NormalReturns => {
                    let return_types = wasm_param_types(&builder.func.signature.returns, |i| {
                        environ.is_wasm_return(&builder.func.signature, i)
                    });
                    bitcast_arguments(&mut state.stack, &return_types, builder);
                    builder.ins().return_(&state.stack)
                }
                ReturnMode::FallthroughReturn => builder.ins().fallthrough_return(&state.stack),
            };
        }
    }

    // Discard any remaining values on the stack. Either we just returned them,
    // or the end of the function is unreachable.
    state.stack.clear();
    //state.metadata_stack.clear();

    debug_assert!(reader.eof());

    Ok(())
}

/// Get the current source location from a reader.
fn cur_srcloc(reader: &dyn FunctionBinaryReader) -> ir::SourceLoc {
    // We record source locations as byte code offsets relative to the beginning of the file.
    // This will wrap around if byte code is larger than 4 GB.
    ir::SourceLoc::new(reader.original_position() as u32)
}
