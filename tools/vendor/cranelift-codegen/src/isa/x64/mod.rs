//! X86_64-bit Instruction Set Architecture.

use self::inst::EmitInfo;

use super::TargetIsa;
use crate::ir::{condcodes::IntCC, Function, Type};
#[cfg(feature = "unwind")]
use crate::isa::unwind::systemv;
use crate::isa::x64::{inst::regs::create_reg_env_systemv, settings as x64_settings};
use crate::isa::Builder as IsaBuilder;
use crate::machinst::{
    compile, CompiledCode, CompiledCodeStencil, MachTextSectionBuilder, Reg, SigSet,
    TextSectionBuilder, VCode,
};
use crate::result::{CodegenError, CodegenResult};
use crate::settings::{self as shared_settings, Flags};
use alloc::{boxed::Box, vec::Vec};
use core::fmt;
use regalloc2::MachineEnv;
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
    reg_env: MachineEnv,
}

impl X64Backend {
    /// Create a new X64 backend with the given (shared) flags.
    fn new_with_flags(triple: Triple, flags: Flags, x64_flags: x64_settings::Flags) -> Self {
        let reg_env = create_reg_env_systemv(&flags);
        Self {
            triple,
            flags,
            x64_flags,
            reg_env,
        }
    }

    fn compile_vcode(
        &self,
        func: &Function,
    ) -> CodegenResult<(VCode<inst::Inst>, regalloc2::Output)> {
        // This performs lowering to VCode, register-allocates the code, computes
        // block layout and finalizes branches. The result is ready for binary emission.
        let emit_info = EmitInfo::new(self.flags.clone(), self.x64_flags.clone());
        let sigs = SigSet::new::<abi::X64ABIMachineSpec>(func, &self.flags)?;
        let abi = abi::X64Callee::new(&func, self, &self.x64_flags, &sigs)?;
        compile::compile::<Self>(&func, self, abi, emit_info, sigs)
    }
}

impl TargetIsa for X64Backend {
    fn compile_function(
        &self,
        func: &Function,
        want_disasm: bool,
    ) -> CodegenResult<CompiledCodeStencil> {
        let (vcode, regalloc_result) = self.compile_vcode(func)?;

        let emit_result = vcode.emit(
            &regalloc_result,
            want_disasm,
            self.flags.machine_code_cfg_info(),
        );
        let frame_size = emit_result.frame_size;
        let value_labels_ranges = emit_result.value_labels_ranges;
        let buffer = emit_result.buffer.finish();
        let sized_stackslot_offsets = emit_result.sized_stackslot_offsets;
        let dynamic_stackslot_offsets = emit_result.dynamic_stackslot_offsets;

        if let Some(disasm) = emit_result.disasm.as_ref() {
            log::trace!("disassembly:\n{}", disasm);
        }

        Ok(CompiledCodeStencil {
            buffer,
            frame_size,
            disasm: emit_result.disasm,
            value_labels_ranges,
            sized_stackslot_offsets,
            dynamic_stackslot_offsets,
            bb_starts: emit_result.bb_offsets,
            bb_edges: emit_result.bb_edges,
            alignment: emit_result.alignment,
        })
    }

    fn flags(&self) -> &Flags {
        &self.flags
    }

    fn machine_env(&self) -> &MachineEnv {
        &self.reg_env
    }

    fn isa_flags(&self) -> Vec<shared_settings::Value> {
        self.x64_flags.iter().collect()
    }

    fn dynamic_vector_bytes(&self, _dyn_ty: Type) -> u32 {
        16
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
        result: &CompiledCode,
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

    fn text_section_builder(&self, num_funcs: usize) -> Box<dyn TextSectionBuilder> {
        Box::new(MachTextSectionBuilder::<inst::Inst>::new(num_funcs))
    }

    /// Align functions on x86 to 16 bytes, ensuring that rip-relative loads to SSE registers are
    /// always from aligned memory.
    fn function_alignment(&self) -> u32 {
        16
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
    use crate::ir::{types::*, RelSourceLoc, SourceLoc, UserFuncName, ValueLabel, ValueLabelStart};
    use crate::ir::{AbiParam, Function, InstBuilder, JumpTableData, Signature};
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
        let name = UserFuncName::testcase("test0");
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
                from: RelSourceLoc::new(1),
                label: ValueLabel::new(1),
            }]),
        );
        pos.func.dfg.values_labels.as_mut().unwrap().insert(
            v1,
            crate::ir::ValueLabelAssignments::Starts(vec![ValueLabelStart {
                from: RelSourceLoc::new(2),
                label: ValueLabel::new(1),
            }]),
        );
        pos.func.dfg.values_labels.as_mut().unwrap().insert(
            v2,
            crate::ir::ValueLabelAssignments::Starts(vec![ValueLabelStart {
                from: RelSourceLoc::new(3),
                label: ValueLabel::new(1),
            }]),
        );
        pos.func.dfg.values_labels.as_mut().unwrap().insert(
            v3,
            crate::ir::ValueLabelAssignments::Starts(vec![ValueLabelStart {
                from: RelSourceLoc::new(4),
                label: ValueLabel::new(1),
            }]),
        );
        pos.func.dfg.values_labels.as_mut().unwrap().insert(
            v4,
            crate::ir::ValueLabelAssignments::Starts(vec![ValueLabelStart {
                from: RelSourceLoc::new(5),
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

        // To update this comment, write the golden bytes to a file, and run the following
        // command on it:
        // > objdump -b binary -D <file> -m i386:x86-64 -M intel
        //
        //  0:   55                      push   rbp
        //  1:   48 89 e5                mov    rbp,rsp
        //  4:   48 89 fe                mov    rsi,rdi
        //  7:   81 c6 34 12 00 00       add    esi,0x1234
        //  d:   85 f6                   test   esi,esi
        //  f:   0f 84 1c 00 00 00       je     0x31
        // 15:   49 89 f0                mov    r8,rsi
        // 18:   48 89 f0                mov    rax,rsi
        // 1b:   81 e8 34 12 00 00       sub    eax,0x1234
        // 21:   44 01 c0                add    eax,r8d
        // 24:   85 f6                   test   esi,esi
        // 26:   0f 85 05 00 00 00       jne    0x31
        // 2c:   48 89 ec                mov    rsp,rbp
        // 2f:   5d                      pop    rbp
        // 30:   c3                      ret
        // 31:   49 89 f0                mov    r8,rsi
        // 34:   41 81 c0 34 12 00 00    add    r8d,0x1234
        // 3b:   45 85 c0                test   r8d,r8d
        // 3e:   0f 85 ed ff ff ff       jne    0x31
        // 44:   e9 cf ff ff ff          jmp    0x18

        let golden = vec![
            85, 72, 137, 229, 72, 137, 254, 129, 198, 52, 18, 0, 0, 133, 246, 15, 132, 28, 0, 0, 0,
            73, 137, 240, 72, 137, 240, 129, 232, 52, 18, 0, 0, 68, 1, 192, 133, 246, 15, 133, 5,
            0, 0, 0, 72, 137, 236, 93, 195, 73, 137, 240, 65, 129, 192, 52, 18, 0, 0, 69, 133, 192,
            15, 133, 237, 255, 255, 255, 233, 207, 255, 255, 255,
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

    // Check that br_table lowers properly. We can't test this with an
    // ordinary compile-test because the br_table pseudoinstruction
    // expands during emission.
    #[test]
    fn br_table() {
        let name = UserFuncName::testcase("test0");
        let mut sig = Signature::new(CallConv::SystemV);
        sig.params.push(AbiParam::new(I32));
        sig.returns.push(AbiParam::new(I32));
        let mut func = Function::with_name_signature(name, sig);

        let bb0 = func.dfg.make_block();
        let arg0 = func.dfg.append_block_param(bb0, I32);
        let bb1 = func.dfg.make_block();
        let bb2 = func.dfg.make_block();
        let bb3 = func.dfg.make_block();

        let mut pos = FuncCursor::new(&mut func);

        pos.insert_block(bb0);
        let mut jt_data = JumpTableData::new();
        jt_data.push_entry(bb1);
        jt_data.push_entry(bb2);
        let jt = pos.func.create_jump_table(jt_data);
        pos.ins().br_table(arg0, bb3, jt);

        pos.insert_block(bb1);
        let v1 = pos.ins().iconst(I32, 1);
        pos.ins().return_(&[v1]);

        pos.insert_block(bb2);
        let v2 = pos.ins().iconst(I32, 2);
        pos.ins().return_(&[v2]);

        pos.insert_block(bb3);
        let v3 = pos.ins().iconst(I32, 3);
        pos.ins().return_(&[v3]);

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

        // To update this comment, write the golden bytes to a file, and run the following
        // command on it:
        // > objdump -b binary -D <file> -m i386:x86-64 -M intel
        //
        //  0:   55                      push   rbp
        //  1:   48 89 e5                mov    rbp,rsp
        //  4:   83 ff 02                cmp    edi,0x2
        //  7:   0f 83 27 00 00 00       jae    0x34
        //  d:   44 8b d7                mov    r10d,edi
        // 10:   41 b9 00 00 00 00       mov    r9d,0x0
        // 16:   4d 0f 43 d1             cmovae r10,r9
        // 1a:   4c 8d 0d 0b 00 00 00    lea    r9,[rip+0xb]        # 0x2c
        // 21:   4f 63 54 91 00          movsxd r10,DWORD PTR [r9+r10*4+0x0]
        // 26:   4d 01 d1                add    r9,r10
        // 29:   41 ff e1                jmp    r9
        // 2c:   12 00                   adc    al,BYTE PTR [rax]
        // 2e:   00 00                   add    BYTE PTR [rax],al
        // 30:   1c 00                   sbb    al,0x0
        // 32:   00 00                   add    BYTE PTR [rax],al
        // 34:   b8 03 00 00 00          mov    eax,0x3
        // 39:   48 89 ec                mov    rsp,rbp
        // 3c:   5d                      pop    rbp
        // 3d:   c3                      ret
        // 3e:   b8 01 00 00 00          mov    eax,0x1
        // 43:   48 89 ec                mov    rsp,rbp
        // 46:   5d                      pop    rbp
        // 47:   c3                      ret
        // 48:   b8 02 00 00 00          mov    eax,0x2
        // 4d:   48 89 ec                mov    rsp,rbp
        // 50:   5d                      pop    rbp
        // 51:   c3                      ret

        let golden = vec![
            85, 72, 137, 229, 131, 255, 2, 15, 131, 39, 0, 0, 0, 68, 139, 215, 65, 185, 0, 0, 0, 0,
            77, 15, 67, 209, 76, 141, 13, 11, 0, 0, 0, 79, 99, 84, 145, 0, 77, 1, 209, 65, 255,
            225, 18, 0, 0, 0, 28, 0, 0, 0, 184, 3, 0, 0, 0, 72, 137, 236, 93, 195, 184, 1, 0, 0, 0,
            72, 137, 236, 93, 195, 184, 2, 0, 0, 0, 72, 137, 236, 93, 195,
        ];

        assert_eq!(code, &golden[..]);
    }
}
