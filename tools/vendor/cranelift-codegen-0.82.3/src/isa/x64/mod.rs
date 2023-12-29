//! X86_64-bit Instruction Set Architecture.

use self::inst::EmitInfo;

use super::TargetIsa;
use crate::ir::{condcodes::IntCC, Function};
#[cfg(feature = "unwind")]
use crate::isa::unwind::systemv;
use crate::isa::x64::{inst::regs::create_reg_universe_systemv, settings as x64_settings};
use crate::isa::Builder as IsaBuilder;
use crate::machinst::{
    compile, MachCompileResult, MachTextSectionBuilder, TextSectionBuilder, VCode,
};
use crate::result::{CodegenError, CodegenResult};
use crate::settings::{self as shared_settings, Flags};
use alloc::{boxed::Box, vec::Vec};
use core::fmt;

use regalloc::{PrettyPrint, RealRegUniverse, Reg};
use target_lexicon::Triple;

mod abi;
pub mod encoding;
mod inst;
mod lower;
mod settings;

/// An X64 backend.
pub(crate) struct X64Backend {
    triple: Triple,
    flags: Flags,
    x64_flags: x64_settings::Flags,
    reg_universe: RealRegUniverse,
}

impl X64Backend {
    /// Create a new X64 backend with the given (shared) flags.
    fn new_with_flags(triple: Triple, flags: Flags, x64_flags: x64_settings::Flags) -> Self {
        let reg_universe = create_reg_universe_systemv(&flags);
        Self {
            triple,
            flags,
            x64_flags,
            reg_universe,
        }
    }

    fn compile_vcode(&self, func: &Function, flags: Flags) -> CodegenResult<VCode<inst::Inst>> {
        // This performs lowering to VCode, register-allocates the code, computes
        // block layout and finalizes branches. The result is ready for binary emission.
        let emit_info = EmitInfo::new(flags.clone(), self.x64_flags.clone());
        let abi = Box::new(abi::X64ABICallee::new(&func, flags, self.isa_flags())?);
        compile::compile::<Self>(&func, self, abi, &self.reg_universe, emit_info)
    }
}

impl TargetIsa for X64Backend {
    fn compile_function(
        &self,
        func: &Function,
        want_disasm: bool,
    ) -> CodegenResult<MachCompileResult> {
        let flags = self.flags();
        let vcode = self.compile_vcode(func, flags.clone())?;

        let (buffer, bb_starts, bb_edges) = vcode.emit();
        let buffer = buffer.finish();
        let frame_size = vcode.frame_size();
        let value_labels_ranges = vcode.value_labels_ranges();
        let stackslot_offsets = vcode.stackslot_offsets().clone();

        let disasm = if want_disasm {
            Some(vcode.show_rru(Some(&create_reg_universe_systemv(flags))))
        } else {
            None
        };

        Ok(MachCompileResult {
            buffer,
            frame_size,
            disasm,
            value_labels_ranges,
            stackslot_offsets,
            bb_starts,
            bb_edges,
        })
    }

    fn flags(&self) -> &Flags {
        &self.flags
    }

    fn isa_flags(&self) -> Vec<shared_settings::Value> {
        self.x64_flags.iter().collect()
    }

    fn name(&self) -> &'static str {
        "x64"
    }

    fn triple(&self) -> &Triple {
        &self.triple
    }

    fn unsigned_add_overflow_condition(&self) -> IntCC {
        // Unsigned `<`; this corresponds to the carry flag set on x86, which
        // indicates an add has overflowed.
        IntCC::UnsignedLessThan
    }

    #[cfg(feature = "unwind")]
    fn emit_unwind_info(
        &self,
        result: &MachCompileResult,
        kind: crate::machinst::UnwindInfoKind,
    ) -> CodegenResult<Option<crate::isa::unwind::UnwindInfo>> {
        use crate::isa::unwind::UnwindInfo;
        use crate::machinst::UnwindInfoKind;
        Ok(match kind {
            UnwindInfoKind::SystemV => {
                let mapper = self::inst::unwind::systemv::RegisterMapper;
                Some(UnwindInfo::SystemV(
                    crate::isa::unwind::systemv::create_unwind_info_from_insts(
                        &result.buffer.unwind_info[..],
                        result.buffer.data().len(),
                        &mapper,
                    )?,
                ))
            }
            UnwindInfoKind::Windows => Some(UnwindInfo::WindowsX64(
                crate::isa::unwind::winx64::create_unwind_info_from_insts::<
                    self::inst::unwind::winx64::RegisterMapper,
                >(&result.buffer.unwind_info[..])?,
            )),
            _ => None,
        })
    }

    #[cfg(feature = "unwind")]
    fn create_systemv_cie(&self) -> Option<gimli::write::CommonInformationEntry> {
        Some(inst::unwind::systemv::create_cie())
    }

    #[cfg(feature = "unwind")]
    fn map_regalloc_reg_to_dwarf(&self, reg: Reg) -> Result<u16, systemv::RegisterMappingError> {
        inst::unwind::systemv::map_reg(reg).map(|reg| reg.0)
    }

    fn text_section_builder(&self, num_funcs: u32) -> Box<dyn TextSectionBuilder> {
        Box::new(MachTextSectionBuilder::<inst::Inst>::new(num_funcs))
    }
}

impl fmt::Display for X64Backend {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MachBackend")
            .field("name", &self.name())
            .field("triple", &self.triple())
            .field("flags", &format!("{}", self.flags()))
            .finish()
    }
}

/// Create a new `isa::Builder`.
pub(crate) fn isa_builder(triple: Triple) -> IsaBuilder {
    IsaBuilder {
        triple,
        setup: x64_settings::builder(),
        constructor: isa_constructor,
    }
}

fn isa_constructor(
    triple: Triple,
    shared_flags: Flags,
    builder: shared_settings::Builder,
) -> CodegenResult<Box<dyn TargetIsa>> {
    let isa_flags = x64_settings::Flags::new(&shared_flags, builder);

    // Check for compatibility between flags and ISA level
    // requested. In particular, SIMD support requires SSE4.2.
    if shared_flags.enable_simd() {
        if !isa_flags.has_sse3()
            || !isa_flags.has_ssse3()
            || !isa_flags.has_sse41()
            || !isa_flags.has_sse42()
        {
            return Err(CodegenError::Unsupported(
                "SIMD support requires SSE3, SSSE3, SSE4.1, and SSE4.2 on x86_64.".into(),
            ));
        }
    }

    let backend = X64Backend::new_with_flags(triple, shared_flags, isa_flags);
    Ok(Box::new(backend))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cursor::{Cursor, FuncCursor};
    use crate::ir::{types::*, SourceLoc, ValueLabel, ValueLabelStart};
    use crate::ir::{AbiParam, ExternalName, Function, InstBuilder, Signature};
    use crate::isa::CallConv;
    use crate::settings;
    use crate::settings::Configurable;
    use core::str::FromStr;
    use cranelift_entity::EntityRef;
    use target_lexicon::Triple;

    /// We have to test cold blocks by observing final machine code,
    /// rather than VCode, because the VCode orders blocks in lowering
    /// order, not emission order. (The exact difference between the
    /// two is that cold blocks are sunk in the latter.) We might as
    /// well do the test here, where we have a backend to use.
    #[test]
    fn test_cold_blocks() {
        let name = ExternalName::testcase("test0");
        let mut sig = Signature::new(CallConv::SystemV);
        sig.params.push(AbiParam::new(I32));
        sig.returns.push(AbiParam::new(I32));
        let mut func = Function::with_name_signature(name, sig);
        // Add debug info: this tests the debug machinery wrt cold
        // blocks as well.
        func.dfg.collect_debug_info();

        let bb0 = func.dfg.make_block();
        let arg0 = func.dfg.append_block_param(bb0, I32);
        let bb1 = func.dfg.make_block();
        let bb2 = func.dfg.make_block();
        let bb3 = func.dfg.make_block();
        let bb1_param = func.dfg.append_block_param(bb1, I32);
        let bb3_param = func.dfg.append_block_param(bb3, I32);

        let mut pos = FuncCursor::new(&mut func);

        pos.insert_block(bb0);
        pos.set_srcloc(SourceLoc::new(1));
        let v0 = pos.ins().iconst(I32, 0x1234);
        pos.set_srcloc(SourceLoc::new(2));
        let v1 = pos.ins().iadd(arg0, v0);
        pos.ins().brnz(v1, bb1, &[v1]);
        pos.ins().jump(bb2, &[]);

        pos.insert_block(bb1);
        pos.set_srcloc(SourceLoc::new(3));
        let v2 = pos.ins().isub(v1, v0);
        pos.set_srcloc(SourceLoc::new(4));
        let v3 = pos.ins().iadd(v2, bb1_param);
        pos.ins().brnz(v1, bb2, &[]);
        pos.ins().jump(bb3, &[v3]);

        pos.func.layout.set_cold(bb2);
        pos.insert_block(bb2);
        pos.set_srcloc(SourceLoc::new(5));
        let v4 = pos.ins().iadd(v1, v0);
        pos.ins().brnz(v4, bb2, &[]);
        pos.ins().jump(bb1, &[v4]);

        pos.insert_block(bb3);
        pos.set_srcloc(SourceLoc::new(6));
        pos.ins().return_(&[bb3_param]);

        // Create some debug info. Make one label that follows all the
        // values around. Note that this is usually done via an API on
        // the FunctionBuilder, but that's in cranelift_frontend
        // (i.e., a higher level of the crate DAG) so we have to build
        // it manually here.
        pos.func.dfg.values_labels.as_mut().unwrap().insert(
            v0,
            crate::ir::ValueLabelAssignments::Starts(vec![ValueLabelStart {
                from: SourceLoc::new(1),
                label: ValueLabel::new(1),
            }]),
        );
        pos.func.dfg.values_labels.as_mut().unwrap().insert(
            v1,
            crate::ir::ValueLabelAssignments::Starts(vec![ValueLabelStart {
                from: SourceLoc::new(2),
                label: ValueLabel::new(1),
            }]),
        );
        pos.func.dfg.values_labels.as_mut().unwrap().insert(
            v2,
            crate::ir::ValueLabelAssignments::Starts(vec![ValueLabelStart {
                from: SourceLoc::new(3),
                label: ValueLabel::new(1),
            }]),
        );
        pos.func.dfg.values_labels.as_mut().unwrap().insert(
            v3,
            crate::ir::ValueLabelAssignments::Starts(vec![ValueLabelStart {
                from: SourceLoc::new(4),
                label: ValueLabel::new(1),
            }]),
        );
        pos.func.dfg.values_labels.as_mut().unwrap().insert(
            v4,
            crate::ir::ValueLabelAssignments::Starts(vec![ValueLabelStart {
                from: SourceLoc::new(5),
                label: ValueLabel::new(1),
            }]),
        );

        let mut shared_flags_builder = settings::builder();
        shared_flags_builder.set("opt_level", "none").unwrap();
        shared_flags_builder.set("enable_verifier", "true").unwrap();
        let shared_flags = settings::Flags::new(shared_flags_builder);
        let isa_flags = x64_settings::Flags::new(&shared_flags, x64_settings::builder());
        let backend = X64Backend::new_with_flags(
            Triple::from_str("x86_64").unwrap(),
            shared_flags,
            isa_flags,
        );
        let result = backend
            .compile_function(&mut func, /* want_disasm = */ false)
            .unwrap();
        let code = result.buffer.data();

        // 00000000  55                push rbp
        // 00000001  4889E5            mov rbp,rsp
        // 00000004  4889FE            mov rsi,rdi
        // 00000007  81C634120000      add esi,0x1234
        // 0000000D  85F6              test esi,esi
        // 0000000F  0F841B000000      jz near 0x30
        // 00000015  4889F7            mov rdi,rsi
        // 00000018  4889F0            mov rax,rsi
        // 0000001B  81E834120000      sub eax,0x1234
        // 00000021  01F8              add eax,edi
        // 00000023  85F6              test esi,esi
        // 00000025  0F8505000000      jnz near 0x30
        // 0000002B  4889EC            mov rsp,rbp
        // 0000002E  5D                pop rbp
        // 0000002F  C3                ret
        // 00000030  4889F7            mov rdi,rsi    <--- cold block
        // 00000033  81C734120000      add edi,0x1234
        // 00000039  85FF              test edi,edi
        // 0000003B  0F85EFFFFFFF      jnz near 0x30
        // 00000041  E9D2FFFFFF        jmp 0x18

        let golden = vec![
            85, 72, 137, 229, 72, 137, 254, 129, 198, 52, 18, 0, 0, 133, 246, 15, 132, 27, 0, 0, 0,
            72, 137, 247, 72, 137, 240, 129, 232, 52, 18, 0, 0, 1, 248, 133, 246, 15, 133, 5, 0, 0,
            0, 72, 137, 236, 93, 195, 72, 137, 247, 129, 199, 52, 18, 0, 0, 133, 255, 15, 133, 239,
            255, 255, 255, 233, 210, 255, 255, 255,
        ];

        assert_eq!(code, &golden[..]);
    }

    // Check that feature tests for SIMD work correctly.
    #[test]
    fn simd_required_features() {
        let mut shared_flags_builder = settings::builder();
        shared_flags_builder.set("enable_simd", "true").unwrap();
        let shared_flags = settings::Flags::new(shared_flags_builder);
        let mut isa_builder = crate::isa::lookup_by_name("x86_64").unwrap();
        isa_builder.set("has_sse3", "false").unwrap();
        isa_builder.set("has_ssse3", "false").unwrap();
        isa_builder.set("has_sse41", "false").unwrap();
        isa_builder.set("has_sse42", "false").unwrap();
        assert!(matches!(
            isa_builder.finish(shared_flags),
            Err(CodegenError::Unsupported(_)),
        ));
    }
}
