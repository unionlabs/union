//! Support for compiling with Cranelift.

use crate::address_map::get_function_address_map;
use crate::config::Cranelift;
#[cfg(feature = "unwind")]
use crate::dwarf::WriterRelocate;
use crate::func_environ::{get_function_name, FuncEnvironment};
use crate::trampoline::{
    make_trampoline_dynamic_function, make_trampoline_function_call, FunctionBuilderContext,
};
use crate::translator::{
    compiled_function_unwind_info, irlibcall_to_libcall, irreloc_to_relocationkind,
    signature_to_cranelift_ir, CraneliftUnwindInfo, FuncTranslator,
};
use cranelift_codegen::ir::{ExternalName, UserFuncName};
use cranelift_codegen::{ir, MachReloc};
use cranelift_codegen::{Context, MachTrap};
#[cfg(feature = "unwind")]
use gimli::write::{Address, EhFrame, FrameTable};
#[cfg(feature = "rayon")]
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::sync::Arc;
use wasmer_compiler::{
    Compiler, FunctionBinaryReader, FunctionBodyData, MiddlewareBinaryReader, ModuleMiddleware,
    ModuleMiddlewareChain, ModuleTranslationState,
};
use wasmer_types::entity::{EntityRef, PrimaryMap};
use wasmer_types::{
    CallingConvention, Compilation, CompileError, CompileModuleInfo, CompiledFunction,
    CompiledFunctionFrameInfo, CompiledFunctionUnwindInfo, Dwarf, FunctionBody, FunctionIndex,
    LocalFunctionIndex, ModuleInfo, Relocation, RelocationTarget, SectionIndex, SignatureIndex,
    Target, TrapCode, TrapInformation,
};

/// A compiler that compiles a WebAssembly module with Cranelift, translating the Wasm to Cranelift IR,
/// optimizing it and then translating to assembly.
pub struct CraneliftCompiler {
    config: Cranelift,
}

impl CraneliftCompiler {
    /// Creates a new Cranelift compiler
    pub fn new(config: Cranelift) -> Self {
        Self { config }
    }

    /// Gets the WebAssembly features for this Compiler
    pub fn config(&self) -> &Cranelift {
        &self.config
    }
}

impl Compiler for CraneliftCompiler {
    fn name(&self) -> &str {
        "cranelift"
    }

    /// Get the middlewares for this compiler
    fn get_middlewares(&self) -> &[Arc<dyn ModuleMiddleware>] {
        &self.config.middlewares
    }

    /// Compile the module using Cranelift, producing a compilation result with
    /// associated relocations.
    fn compile_module(
        &self,
        target: &Target,
        compile_info: &CompileModuleInfo,
        module_translation_state: &ModuleTranslationState,
        function_body_inputs: PrimaryMap<LocalFunctionIndex, FunctionBodyData<'_>>,
    ) -> Result<Compilation, CompileError> {
        let isa = self
            .config()
            .isa(target)
            .map_err(|error| CompileError::Codegen(error.to_string()))?;
        let frontend_config = isa.frontend_config();
        let memory_styles = &compile_info.memory_styles;
        let table_styles = &compile_info.table_styles;
        let module = &compile_info.module;
        let signatures = module
            .signatures
            .iter()
            .map(|(_sig_index, func_type)| signature_to_cranelift_ir(func_type, frontend_config))
            .collect::<PrimaryMap<SignatureIndex, ir::Signature>>();

        // Generate the frametable
        #[cfg(feature = "unwind")]
        let dwarf_frametable = if function_body_inputs.is_empty() {
            // If we have no function body inputs, we don't need to
            // construct the `FrameTable`. Constructing it, with empty
            // FDEs will cause some issues in Linux.
            None
        } else {
            match target.triple().default_calling_convention() {
                Ok(CallingConvention::SystemV) => {
                    match isa.create_systemv_cie() {
                        Some(cie) => {
                            let mut dwarf_frametable = FrameTable::default();
                            let cie_id = dwarf_frametable.add_cie(cie);
                            Some((dwarf_frametable, cie_id))
                        }
                        // Even though we are in a SystemV system, Cranelift doesn't support it
                        None => None,
                    }
                }
                _ => None,
            }
        };

        let mut custom_sections = PrimaryMap::new();

        #[cfg(not(feature = "rayon"))]
        let mut func_translator = FuncTranslator::new();
        #[cfg(not(feature = "rayon"))]
        let (functions, fdes): (Vec<CompiledFunction>, Vec<_>) = function_body_inputs
            .iter()
            .collect::<Vec<(LocalFunctionIndex, &FunctionBodyData<'_>)>>()
            .into_iter()
            .map(|(i, input)| {
                let func_index = module.func_index(i);
                let mut context = Context::new();
                let mut func_env = FuncEnvironment::new(
                    isa.frontend_config(),
                    module,
                    &signatures,
                    &memory_styles,
                    &table_styles,
                );
                context.func.name = match get_function_name(func_index) {
                    ExternalName::User(nameref) => {
                        if context.func.params.user_named_funcs().is_valid(nameref) {
                            let name = &context.func.params.user_named_funcs()[nameref];
                            UserFuncName::User(name.clone())
                        } else {
                            UserFuncName::default()
                        }
                    }
                    ExternalName::TestCase(testcase) => UserFuncName::Testcase(testcase),
                    _ => UserFuncName::default(),
                };
                context.func.signature = signatures[module.functions[func_index]].clone();
                // if generate_debug_info {
                //     context.func.collect_debug_info();
                // }
                let mut reader =
                    MiddlewareBinaryReader::new_with_offset(input.data, input.module_offset);
                reader.set_middleware_chain(
                    self.config
                        .middlewares
                        .generate_function_middleware_chain(i),
                );

                func_translator.translate(
                    module_translation_state,
                    &mut reader,
                    &mut context.func,
                    &mut func_env,
                    i,
                )?;

                let mut code_buf: Vec<u8> = Vec::new();
                context
                    .compile_and_emit(&*isa, &mut code_buf)
                    .map_err(|error| CompileError::Codegen(error.inner.to_string()))?;

                let result = context.compiled_code().unwrap();
                let func_relocs = result
                    .buffer
                    .relocs()
                    .into_iter()
                    .map(|r| mach_reloc_to_reloc(module, r))
                    .collect::<Vec<_>>();

                let traps = result
                    .buffer
                    .traps()
                    .into_iter()
                    .map(mach_trap_to_trap)
                    .collect::<Vec<_>>();

                let (unwind_info, fde) = match compiled_function_unwind_info(&*isa, &context)? {
                    #[cfg(feature = "unwind")]
                    CraneliftUnwindInfo::Fde(fde) => {
                        if dwarf_frametable.is_some() {
                            let fde = fde.to_fde(Address::Symbol {
                                // The symbol is the kind of relocation.
                                // "0" is used for functions
                                symbol: WriterRelocate::FUNCTION_SYMBOL,
                                // We use the addend as a way to specify the
                                // function index
                                addend: i.index() as _,
                            });
                            // The unwind information is inserted into the dwarf section
                            (Some(CompiledFunctionUnwindInfo::Dwarf), Some(fde))
                        } else {
                            (None, None)
                        }
                    }
                    #[cfg(feature = "unwind")]
                    other => (other.maybe_into_to_windows_unwind(), None),

                    // This is a bit hacky, but necessary since gimli is not
                    // available when the "unwind" feature is disabled.
                    #[cfg(not(feature = "unwind"))]
                    other => (other.maybe_into_to_windows_unwind(), None::<()>),
                };

                let range = reader.range();
                let address_map = get_function_address_map(&context, range, code_buf.len());

                Ok((
                    CompiledFunction {
                        body: FunctionBody {
                            body: code_buf,
                            unwind_info,
                        },
                        relocations: func_relocs,
                        frame_info: CompiledFunctionFrameInfo { address_map, traps },
                    },
                    fde,
                ))
            })
            .collect::<Result<Vec<_>, CompileError>>()?
            .into_iter()
            .unzip();
        #[cfg(feature = "rayon")]
        let (functions, fdes): (Vec<CompiledFunction>, Vec<_>) = function_body_inputs
            .iter()
            .collect::<Vec<(LocalFunctionIndex, &FunctionBodyData<'_>)>>()
            .par_iter()
            .map_init(FuncTranslator::new, |func_translator, (i, input)| {
                let func_index = module.func_index(*i);
                let mut context = Context::new();
                let mut func_env = FuncEnvironment::new(
                    isa.frontend_config(),
                    module,
                    &signatures,
                    memory_styles,
                    table_styles,
                );
                context.func.name = match get_function_name(func_index) {
                    ExternalName::User(nameref) => {
                        if context.func.params.user_named_funcs().is_valid(nameref) {
                            let name = &context.func.params.user_named_funcs()[nameref];
                            UserFuncName::User(name.clone())
                        } else {
                            UserFuncName::default()
                        }
                    }
                    ExternalName::TestCase(testcase) => UserFuncName::Testcase(testcase),
                    _ => UserFuncName::default(),
                };
                context.func.signature = signatures[module.functions[func_index]].clone();
                // if generate_debug_info {
                //     context.func.collect_debug_info();
                // }
                let mut reader =
                    MiddlewareBinaryReader::new_with_offset(input.data, input.module_offset);
                reader.set_middleware_chain(
                    self.config
                        .middlewares
                        .generate_function_middleware_chain(*i),
                );

                func_translator.translate(
                    module_translation_state,
                    &mut reader,
                    &mut context.func,
                    &mut func_env,
                    *i,
                )?;

                let mut code_buf: Vec<u8> = Vec::new();
                context
                    .compile_and_emit(&*isa, &mut code_buf)
                    .map_err(|error| CompileError::Codegen(error.inner.to_string()))?;

                let result = context.compiled_code().unwrap();
                let func_relocs = result
                    .buffer
                    .relocs()
                    .iter()
                    .map(|r| mach_reloc_to_reloc(module, r))
                    .collect::<Vec<_>>();

                let traps = result
                    .buffer
                    .traps()
                    .iter()
                    .map(mach_trap_to_trap)
                    .collect::<Vec<_>>();

                let (unwind_info, fde) = match compiled_function_unwind_info(&*isa, &context)? {
                    #[cfg(feature = "unwind")]
                    CraneliftUnwindInfo::Fde(fde) => {
                        if dwarf_frametable.is_some() {
                            let fde = fde.to_fde(Address::Symbol {
                                // The symbol is the kind of relocation.
                                // "0" is used for functions
                                symbol: WriterRelocate::FUNCTION_SYMBOL,
                                // We use the addend as a way to specify the
                                // function index
                                addend: i.index() as _,
                            });
                            // The unwind information is inserted into the dwarf section
                            (Some(CompiledFunctionUnwindInfo::Dwarf), Some(fde))
                        } else {
                            (None, None)
                        }
                    }
                    #[cfg(feature = "unwind")]
                    other => (other.maybe_into_to_windows_unwind(), None),

                    // This is a bit hacky, but necessary since gimli is not
                    // available when the "unwind" feature is disabled.
                    #[cfg(not(feature = "unwind"))]
                    other => (other.maybe_into_to_windows_unwind(), None::<()>),
                };

                let range = reader.range();
                let address_map = get_function_address_map(&context, range, code_buf.len());

                Ok((
                    CompiledFunction {
                        body: FunctionBody {
                            body: code_buf,
                            unwind_info,
                        },
                        relocations: func_relocs,
                        frame_info: CompiledFunctionFrameInfo { address_map, traps },
                    },
                    fde,
                ))
            })
            .collect::<Result<Vec<_>, CompileError>>()?
            .into_iter()
            .unzip();

        #[cfg(feature = "unwind")]
        let dwarf = if let Some((mut dwarf_frametable, cie_id)) = dwarf_frametable {
            for fde in fdes.into_iter().flatten() {
                dwarf_frametable.add_fde(cie_id, fde);
            }
            let mut eh_frame = EhFrame(WriterRelocate::new(target.triple().endianness().ok()));
            dwarf_frametable.write_eh_frame(&mut eh_frame).unwrap();

            let eh_frame_section = eh_frame.0.into_section();
            custom_sections.push(eh_frame_section);
            Some(Dwarf::new(SectionIndex::new(custom_sections.len() - 1)))
        } else {
            None
        };
        #[cfg(not(feature = "unwind"))]
        let dwarf = None;

        // function call trampolines (only for local functions, by signature)
        #[cfg(not(feature = "rayon"))]
        let mut cx = FunctionBuilderContext::new();
        #[cfg(not(feature = "rayon"))]
        let function_call_trampolines = module
            .signatures
            .values()
            .collect::<Vec<_>>()
            .into_iter()
            .map(|sig| make_trampoline_function_call(&*isa, &mut cx, sig))
            .collect::<Result<Vec<FunctionBody>, CompileError>>()?
            .into_iter()
            .collect::<PrimaryMap<SignatureIndex, FunctionBody>>();
        #[cfg(feature = "rayon")]
        let function_call_trampolines = module
            .signatures
            .values()
            .collect::<Vec<_>>()
            .par_iter()
            .map_init(FunctionBuilderContext::new, |cx, sig| {
                make_trampoline_function_call(&*isa, cx, sig)
            })
            .collect::<Result<Vec<FunctionBody>, CompileError>>()?
            .into_iter()
            .collect::<PrimaryMap<SignatureIndex, FunctionBody>>();

        use wasmer_types::VMOffsets;
        let offsets = VMOffsets::new_for_trampolines(frontend_config.pointer_bytes());
        // dynamic function trampolines (only for imported functions)
        #[cfg(not(feature = "rayon"))]
        let mut cx = FunctionBuilderContext::new();
        #[cfg(not(feature = "rayon"))]
        let dynamic_function_trampolines = module
            .imported_function_types()
            .collect::<Vec<_>>()
            .into_iter()
            .map(|func_type| make_trampoline_dynamic_function(&*isa, &offsets, &mut cx, &func_type))
            .collect::<Result<Vec<_>, CompileError>>()?
            .into_iter()
            .collect::<PrimaryMap<FunctionIndex, FunctionBody>>();
        #[cfg(feature = "rayon")]
        let dynamic_function_trampolines = module
            .imported_function_types()
            .collect::<Vec<_>>()
            .par_iter()
            .map_init(FunctionBuilderContext::new, |cx, func_type| {
                make_trampoline_dynamic_function(&*isa, &offsets, cx, func_type)
            })
            .collect::<Result<Vec<_>, CompileError>>()?
            .into_iter()
            .collect::<PrimaryMap<FunctionIndex, FunctionBody>>();

        Ok(Compilation {
            functions: functions.into_iter().collect(),
            custom_sections,
            function_call_trampolines,
            dynamic_function_trampolines,
            debug: dwarf,
        })
    }
}

fn mach_reloc_to_reloc(module: &ModuleInfo, reloc: &MachReloc) -> Relocation {
    let &MachReloc {
        offset,
        kind,
        ref name,
        addend,
    } = reloc;
    let reloc_target = if let ExternalName::User(extname_ref) = *name {
        //debug_assert_eq!(namespace, 0);
        RelocationTarget::LocalFunc(
            module
                .local_func_index(FunctionIndex::from_u32(extname_ref.as_u32()))
                .expect("The provided function should be local"),
        )
    } else if let ExternalName::LibCall(libcall) = *name {
        RelocationTarget::LibCall(irlibcall_to_libcall(libcall))
    } else {
        panic!("unrecognized external name")
    };
    Relocation {
        kind: irreloc_to_relocationkind(kind),
        reloc_target,
        offset,
        addend,
    }
}

fn mach_trap_to_trap(trap: &MachTrap) -> TrapInformation {
    let &MachTrap { offset, code } = trap;
    TrapInformation {
        code_offset: offset,
        trap_code: translate_ir_trapcode(code),
    }
}

/// Translates the Cranelift IR TrapCode into generic Trap Code
fn translate_ir_trapcode(trap: ir::TrapCode) -> TrapCode {
    match trap {
        ir::TrapCode::StackOverflow => TrapCode::StackOverflow,
        ir::TrapCode::HeapOutOfBounds => TrapCode::HeapAccessOutOfBounds,
        ir::TrapCode::HeapMisaligned => TrapCode::UnalignedAtomic,
        ir::TrapCode::TableOutOfBounds => TrapCode::TableAccessOutOfBounds,
        ir::TrapCode::IndirectCallToNull => TrapCode::IndirectCallToNull,
        ir::TrapCode::BadSignature => TrapCode::BadSignature,
        ir::TrapCode::IntegerOverflow => TrapCode::IntegerOverflow,
        ir::TrapCode::IntegerDivisionByZero => TrapCode::IntegerDivisionByZero,
        ir::TrapCode::BadConversionToInteger => TrapCode::BadConversionToInteger,
        ir::TrapCode::UnreachableCodeReached => TrapCode::UnreachableCodeReached,
        ir::TrapCode::Interrupt => unimplemented!("Interrupts not supported"),
        ir::TrapCode::User(_user_code) => unimplemented!("User trap code not supported"),
        // ir::TrapCode::Interrupt => TrapCode::Interrupt,
        // ir::TrapCode::User(user_code) => TrapCode::User(user_code),
    }
}
