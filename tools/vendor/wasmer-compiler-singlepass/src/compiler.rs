//! Support for compiling with Singlepass.
// Allow unused imports while developing.
#![allow(unused_imports, dead_code)]

use crate::codegen::FuncGen;
use crate::config::Singlepass;
#[cfg(feature = "unwind")]
use crate::dwarf::WriterRelocate;
use crate::machine::Machine;
use crate::machine::{
    gen_import_call_trampoline, gen_std_dynamic_import_trampoline, gen_std_trampoline,
};
use crate::machine_arm64::MachineARM64;
use crate::machine_x64::MachineX86_64;
#[cfg(feature = "unwind")]
use crate::unwind::{create_systemv_cie, UnwindFrame};
use enumset::EnumSet;
#[cfg(feature = "unwind")]
use gimli::write::{EhFrame, FrameTable};
#[cfg(feature = "rayon")]
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::sync::Arc;
use wasmer_compiler::{
    Compiler, CompilerConfig, FunctionBinaryReader, FunctionBodyData, MiddlewareBinaryReader,
    ModuleMiddleware, ModuleMiddlewareChain, ModuleTranslationState,
};
use wasmer_types::entity::{EntityRef, PrimaryMap};
use wasmer_types::{
    Architecture, CallingConvention, Compilation, CompileError, CompileModuleInfo,
    CompiledFunction, CpuFeature, Dwarf, FunctionBody, FunctionIndex, FunctionType,
    LocalFunctionIndex, MemoryIndex, ModuleInfo, OperatingSystem, SectionIndex, TableIndex, Target,
    TrapCode, TrapInformation, VMOffsets,
};

/// A compiler that compiles a WebAssembly module with Singlepass.
/// It does the compilation in one pass
pub struct SinglepassCompiler {
    config: Singlepass,
}

impl SinglepassCompiler {
    /// Creates a new Singlepass compiler
    pub fn new(config: Singlepass) -> Self {
        Self { config }
    }

    /// Gets the config for this Compiler
    fn config(&self) -> &Singlepass {
        &self.config
    }
}

impl Compiler for SinglepassCompiler {
    fn name(&self) -> &str {
        "singlepass"
    }

    /// Get the middlewares for this compiler
    fn get_middlewares(&self) -> &[Arc<dyn ModuleMiddleware>] {
        &self.config.middlewares
    }

    /// Compile the module using Singlepass, producing a compilation result with
    /// associated relocations.
    fn compile_module(
        &self,
        target: &Target,
        compile_info: &CompileModuleInfo,
        _module_translation: &ModuleTranslationState,
        function_body_inputs: PrimaryMap<LocalFunctionIndex, FunctionBodyData<'_>>,
    ) -> Result<Compilation, CompileError> {
        match target.triple().architecture {
            Architecture::X86_64 => {}
            Architecture::Aarch64(_) => {}
            _ => {
                return Err(CompileError::UnsupportedTarget(
                    target.triple().architecture.to_string(),
                ))
            }
        }

        let calling_convention = match target.triple().default_calling_convention() {
            Ok(CallingConvention::WindowsFastcall) => CallingConvention::WindowsFastcall,
            Ok(CallingConvention::SystemV) => CallingConvention::SystemV,
            Ok(CallingConvention::AppleAarch64) => CallingConvention::AppleAarch64,
            _ => {
                return Err(CompileError::UnsupportedTarget(
                    "Unsupported Calling convention for Singlepass compiler".to_string(),
                ))
            }
        };

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
                    match create_systemv_cie(target.triple().architecture) {
                        Some(cie) => {
                            let mut dwarf_frametable = FrameTable::default();
                            let cie_id = dwarf_frametable.add_cie(cie);
                            Some((dwarf_frametable, cie_id))
                        }
                        None => None,
                    }
                }
                _ => None,
            }
        };

        let memory_styles = &compile_info.memory_styles;
        let table_styles = &compile_info.table_styles;
        let vmoffsets = VMOffsets::new(8, &compile_info.module);
        let module = &compile_info.module;
        let mut custom_sections: PrimaryMap<SectionIndex, _> = (0..module.num_imported_functions)
            .map(FunctionIndex::new)
            .collect::<Vec<_>>()
            .into_par_iter_if_rayon()
            .map(|i| {
                gen_import_call_trampoline(
                    &vmoffsets,
                    i,
                    &module.signatures[module.functions[i]],
                    target,
                    calling_convention,
                )
                .unwrap()
            })
            .collect::<Vec<_>>()
            .into_iter()
            .collect();
        let (functions, fdes): (Vec<CompiledFunction>, Vec<_>) = function_body_inputs
            .iter()
            .collect::<Vec<(LocalFunctionIndex, &FunctionBodyData<'_>)>>()
            .into_par_iter_if_rayon()
            .map(|(i, input)| {
                let middleware_chain = self
                    .config
                    .middlewares
                    .generate_function_middleware_chain(i);
                let mut reader =
                    MiddlewareBinaryReader::new_with_offset(input.data, input.module_offset);
                reader.set_middleware_chain(middleware_chain);

                // This local list excludes arguments.
                let mut locals = vec![];
                let num_locals = reader.read_local_count()?;
                for _ in 0..num_locals {
                    let (count, ty) = reader.read_local_decl()?;
                    for _ in 0..count {
                        locals.push(ty);
                    }
                }

                match target.triple().architecture {
                    Architecture::X86_64 => {
                        let machine = MachineX86_64::new(Some(target.clone()))?;
                        let mut generator = FuncGen::new(
                            module,
                            &self.config,
                            &vmoffsets,
                            memory_styles,
                            table_styles,
                            i,
                            &locals,
                            machine,
                            calling_convention,
                        )?;
                        while generator.has_control_frames() {
                            generator.set_srcloc(reader.original_position() as u32);
                            let op = reader.read_operator()?;
                            generator.feed_operator(op)?;
                        }

                        generator.finalize(input)
                    }
                    Architecture::Aarch64(_) => {
                        let machine = MachineARM64::new();
                        let mut generator = FuncGen::new(
                            module,
                            &self.config,
                            &vmoffsets,
                            memory_styles,
                            table_styles,
                            i,
                            &locals,
                            machine,
                            calling_convention,
                        )?;
                        while generator.has_control_frames() {
                            generator.set_srcloc(reader.original_position() as u32);
                            let op = reader.read_operator()?;
                            generator.feed_operator(op)?;
                        }

                        generator.finalize(input)
                    }
                    _ => unimplemented!(),
                }
            })
            .collect::<Result<Vec<_>, CompileError>>()?
            .into_iter()
            .unzip();

        let function_call_trampolines = module
            .signatures
            .values()
            .collect::<Vec<_>>()
            .into_par_iter_if_rayon()
            .map(|func_type| gen_std_trampoline(func_type, target, calling_convention).unwrap())
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<PrimaryMap<_, _>>();

        let dynamic_function_trampolines = module
            .imported_function_types()
            .collect::<Vec<_>>()
            .into_par_iter_if_rayon()
            .map(|func_type| {
                gen_std_dynamic_import_trampoline(
                    &vmoffsets,
                    &func_type,
                    target,
                    calling_convention,
                )
                .unwrap()
            })
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<PrimaryMap<FunctionIndex, FunctionBody>>();

        #[cfg(feature = "unwind")]
        let dwarf = if let Some((mut dwarf_frametable, cie_id)) = dwarf_frametable {
            for fde in fdes.into_iter().flatten() {
                match fde {
                    UnwindFrame::SystemV(fde) => dwarf_frametable.add_fde(cie_id, fde),
                }
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

        Ok(Compilation {
            functions: functions.into_iter().collect(),
            custom_sections,
            function_call_trampolines,
            dynamic_function_trampolines,
            debug: dwarf,
        })
    }

    fn get_cpu_features_used(&self, cpu_features: &EnumSet<CpuFeature>) -> EnumSet<CpuFeature> {
        let used = CpuFeature::AVX | CpuFeature::SSE42 | CpuFeature::LZCNT | CpuFeature::BMI1;
        cpu_features.intersection(used)
    }
}

trait IntoParIterIfRayon {
    type Output;
    fn into_par_iter_if_rayon(self) -> Self::Output;
}

impl<T: Send> IntoParIterIfRayon for Vec<T> {
    #[cfg(not(feature = "rayon"))]
    type Output = std::vec::IntoIter<T>;
    #[cfg(feature = "rayon")]
    type Output = rayon::vec::IntoIter<T>;

    fn into_par_iter_if_rayon(self) -> Self::Output {
        #[cfg(not(feature = "rayon"))]
        return self.into_iter();
        #[cfg(feature = "rayon")]
        return self.into_par_iter();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use target_lexicon::triple;
    use wasmer_compiler::Features;
    use wasmer_types::{CpuFeature, MemoryStyle, TableStyle, Triple};

    fn dummy_compilation_ingredients<'a>() -> (
        CompileModuleInfo,
        ModuleTranslationState,
        PrimaryMap<LocalFunctionIndex, FunctionBodyData<'a>>,
    ) {
        let compile_info = CompileModuleInfo {
            features: Features::new(),
            module: Arc::new(ModuleInfo::new()),
            memory_styles: PrimaryMap::<MemoryIndex, MemoryStyle>::new(),
            table_styles: PrimaryMap::<TableIndex, TableStyle>::new(),
        };
        let module_translation = ModuleTranslationState::new();
        let function_body_inputs = PrimaryMap::<LocalFunctionIndex, FunctionBodyData<'_>>::new();
        (compile_info, module_translation, function_body_inputs)
    }

    #[test]
    fn errors_for_unsupported_targets() {
        let compiler = SinglepassCompiler::new(Singlepass::default());

        // Compile for 32bit Linux
        let linux32 = Target::new(triple!("i686-unknown-linux-gnu"), CpuFeature::for_host());
        let (mut info, translation, inputs) = dummy_compilation_ingredients();
        let result = compiler.compile_module(&linux32, &mut info, &translation, inputs);
        match result.unwrap_err() {
            CompileError::UnsupportedTarget(name) => assert_eq!(name, "i686"),
            error => panic!("Unexpected error: {:?}", error),
        };

        // Compile for win32
        let win32 = Target::new(triple!("i686-pc-windows-gnu"), CpuFeature::for_host());
        let (mut info, translation, inputs) = dummy_compilation_ingredients();
        let result = compiler.compile_module(&win32, &mut info, &translation, inputs);
        match result.unwrap_err() {
            CompileError::UnsupportedTarget(name) => assert_eq!(name, "i686"), // Windows should be checked before architecture
            error => panic!("Unexpected error: {:?}", error),
        };
    }

    #[test]
    fn errors_for_unsuported_cpufeatures() {
        let compiler = SinglepassCompiler::new(Singlepass::default());
        let mut features =
            CpuFeature::AVX | CpuFeature::SSE42 | CpuFeature::LZCNT | CpuFeature::BMI1;
        // simple test
        assert!(compiler
            .get_cpu_features_used(&features)
            .is_subset(CpuFeature::AVX | CpuFeature::SSE42 | CpuFeature::LZCNT | CpuFeature::BMI1));
        // check that an AVX build don't work on SSE4.2 only host
        assert!(!compiler
            .get_cpu_features_used(&features)
            .is_subset(CpuFeature::SSE42 | CpuFeature::LZCNT | CpuFeature::BMI1));
        // check that having a host with AVX512 doesn't change anything
        features.insert_all(CpuFeature::AVX512DQ | CpuFeature::AVX512F);
        assert!(compiler
            .get_cpu_features_used(&features)
            .is_subset(CpuFeature::AVX | CpuFeature::SSE42 | CpuFeature::LZCNT | CpuFeature::BMI1));
    }
}
