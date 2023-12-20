//! ARM 64-bit Instruction Set Architecture.

use crate::ir::condcodes::IntCC;
use crate::ir::{Function, Type};
use crate::isa::aarch64::settings as aarch64_settings;
#[cfg(feature = "unwind")]
use crate::isa::unwind::systemv;
use crate::isa::{Builder as IsaBuilder, TargetIsa};
use crate::machinst::{
    compile, CompiledCode, CompiledCodeStencil, MachTextSectionBuilder, Reg, SigSet,
    TextSectionBuilder, VCode,
};
use crate::result::CodegenResult;
use crate::settings as shared_settings;
use alloc::{boxed::Box, vec::Vec};
use core::fmt;
use regalloc2::MachineEnv;
use target_lexicon::{Aarch64Architecture, Architecture, OperatingSystem, Triple};

// New backend:
mod abi;
pub(crate) mod inst;
mod lower;
mod lower_inst;
mod settings;

use inst::create_reg_env;

use self::inst::EmitInfo;

/// An AArch64 backend.
pub struct AArch64Backend {
    triple: Triple,
    flags: shared_settings::Flags,
    isa_flags: aarch64_settings::Flags,
    machine_env: MachineEnv,
}

impl AArch64Backend {
    /// Create a new AArch64 backend with the given (shared) flags.
    pub fn new_with_flags(
        triple: Triple,
        flags: shared_settings::Flags,
        isa_flags: aarch64_settings::Flags,
    ) -> AArch64Backend {
        let machine_env = create_reg_env(&flags);
        AArch64Backend {
            triple,
            flags,
            isa_flags,
            machine_env,
        }
    }

    /// This performs lowering to VCode, register-allocates the code, computes block layout and
    /// finalizes branches. The result is ready for binary emission.
    fn compile_vcode(
        &self,
        func: &Function,
    ) -> CodegenResult<(VCode<inst::Inst>, regalloc2::Output)> {
        let emit_info = EmitInfo::new(self.flags.clone());
        let sigs = SigSet::new::<abi::AArch64MachineDeps>(func, &self.flags)?;
        let abi = abi::AArch64Callee::new(func, self, &self.isa_flags, &sigs)?;
        compile::compile::<AArch64Backend>(func, self, abi, emit_info, sigs)
    }
}

impl TargetIsa for AArch64Backend {
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
            log::debug!("disassembly:\n{}", disasm);
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

    fn name(&self) -> &'static str {
        "aarch64"
    }

    fn triple(&self) -> &Triple {
        &self.triple
    }

    fn flags(&self) -> &shared_settings::Flags {
        &self.flags
    }

    fn machine_env(&self) -> &MachineEnv {
        &self.machine_env
    }

    fn isa_flags(&self) -> Vec<shared_settings::Value> {
        self.isa_flags.iter().collect()
    }

    fn is_branch_protection_enabled(&self) -> bool {
        self.isa_flags.use_bti()
    }

    fn dynamic_vector_bytes(&self, _dyn_ty: Type) -> u32 {
        16
    }

    fn unsigned_add_overflow_condition(&self) -> IntCC {
        // Unsigned `>=`; this corresponds to the carry flag set on aarch64, which happens on
        // overflow of an add.
        IntCC::UnsignedGreaterThanOrEqual
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
            UnwindInfoKind::Windows => {
                // TODO: support Windows unwind info on AArch64
                None
            }
            _ => None,
        })
    }

    #[cfg(feature = "unwind")]
    fn create_systemv_cie(&self) -> Option<gimli::write::CommonInformationEntry> {
        let is_apple_os = match self.triple.operating_system {
            OperatingSystem::Darwin
            | OperatingSystem::Ios
            | OperatingSystem::MacOSX { .. }
            | OperatingSystem::Tvos => true,
            _ => false,
        };

        if self.isa_flags.sign_return_address()
            && self.isa_flags.sign_return_address_with_bkey()
            && !is_apple_os
        {
            unimplemented!("Specifying that the B key is used with pointer authentication instructions in the CIE is not implemented.");
        }

        Some(inst::unwind::systemv::create_cie())
    }

    fn text_section_builder(&self, num_funcs: usize) -> Box<dyn TextSectionBuilder> {
        Box::new(MachTextSectionBuilder::<inst::Inst>::new(num_funcs))
    }

    #[cfg(feature = "unwind")]
    fn map_regalloc_reg_to_dwarf(&self, reg: Reg) -> Result<u16, systemv::RegisterMappingError> {
        inst::unwind::systemv::map_reg(reg).map(|reg| reg.0)
    }

    fn function_alignment(&self) -> u32 {
        // We use 32-byte alignment for performance reasons, but for correctness we would only need
        // 4-byte alignment.
        32
    }
}

impl fmt::Display for AArch64Backend {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MachBackend")
            .field("name", &self.name())
            .field("triple", &self.triple())
            .field("flags", &format!("{}", self.flags()))
            .finish()
    }
}

/// Create a new `isa::Builder`.
pub fn isa_builder(triple: Triple) -> IsaBuilder {
    assert!(triple.architecture == Architecture::Aarch64(Aarch64Architecture::Aarch64));
    IsaBuilder {
        triple,
        setup: aarch64_settings::builder(),
        constructor: |triple, shared_flags, builder| {
            let isa_flags = aarch64_settings::Flags::new(&shared_flags, builder);
            let backend = AArch64Backend::new_with_flags(triple, shared_flags, isa_flags);
            Ok(Box::new(backend))
        },
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cursor::{Cursor, FuncCursor};
    use crate::ir::types::*;
    use crate::ir::{AbiParam, Function, InstBuilder, JumpTableData, Signature, UserFuncName};
    use crate::isa::CallConv;
    use crate::settings;
    use crate::settings::Configurable;
    use core::str::FromStr;
    use target_lexicon::Triple;

    #[test]
    fn test_compile_function() {
        let name = UserFuncName::testcase("test0");
        let mut sig = Signature::new(CallConv::SystemV);
        sig.params.push(AbiParam::new(I32));
        sig.returns.push(AbiParam::new(I32));
        let mut func = Function::with_name_signature(name, sig);

        let bb0 = func.dfg.make_block();
        let arg0 = func.dfg.append_block_param(bb0, I32);

        let mut pos = FuncCursor::new(&mut func);
        pos.insert_block(bb0);
        let v0 = pos.ins().iconst(I32, 0x1234);
        let v1 = pos.ins().iadd(arg0, v0);
        pos.ins().return_(&[v1]);

        let mut shared_flags_builder = settings::builder();
        shared_flags_builder.set("opt_level", "none").unwrap();
        let shared_flags = settings::Flags::new(shared_flags_builder);
        let isa_flags = aarch64_settings::Flags::new(&shared_flags, aarch64_settings::builder());
        let backend = AArch64Backend::new_with_flags(
            Triple::from_str("aarch64").unwrap(),
            shared_flags,
            isa_flags,
        );
        let buffer = backend.compile_function(&mut func, false).unwrap().buffer;
        let code = buffer.data();

        // To update this comment, write the golden bytes to a file, and run the following command
        // on it to update:
        // > aarch64-linux-gnu-objdump -b binary -D <file> -m aarch64
        //
        // 0:   d2824682        mov     x2, #0x1234                     // #4660
        // 4:   0b020000        add     w0, w0, w2
        // 8:   d65f03c0        ret

        let golden = vec![130, 70, 130, 210, 0, 0, 2, 11, 192, 3, 95, 214];

        assert_eq!(code, &golden[..]);
    }

    #[test]
    fn test_branch_lowering() {
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
        let v0 = pos.ins().iconst(I32, 0x1234);
        let v1 = pos.ins().iadd(arg0, v0);
        pos.ins().brnz(v1, bb1, &[]);
        pos.ins().jump(bb2, &[]);
        pos.insert_block(bb1);
        pos.ins().brnz(v1, bb2, &[]);
        pos.ins().jump(bb3, &[]);
        pos.insert_block(bb2);
        let v2 = pos.ins().iadd(v1, v0);
        pos.ins().brnz(v2, bb2, &[]);
        pos.ins().jump(bb1, &[]);
        pos.insert_block(bb3);
        let v3 = pos.ins().isub(v1, v0);
        pos.ins().return_(&[v3]);

        let mut shared_flags_builder = settings::builder();
        shared_flags_builder.set("opt_level", "none").unwrap();
        let shared_flags = settings::Flags::new(shared_flags_builder);
        let isa_flags = aarch64_settings::Flags::new(&shared_flags, aarch64_settings::builder());
        let backend = AArch64Backend::new_with_flags(
            Triple::from_str("aarch64").unwrap(),
            shared_flags,
            isa_flags,
        );
        let result = backend
            .compile_function(&mut func, /* want_disasm = */ false)
            .unwrap();
        let code = result.buffer.data();

        // To update this comment, write the golden bytes to a file, and run the following command
        // on it to update:
        // > aarch64-linux-gnu-objdump -b binary -D <file> -m aarch64
        //
        //   0:   d2824689        mov     x9, #0x1234                     // #4660
        //   4:   0b09000b        add     w11, w0, w9
        //   8:   2a0b03ea        mov     w10, w11
        //   c:   b50000aa        cbnz    x10, 0x20
        //  10:   d282468c        mov     x12, #0x1234                    // #4660
        //  14:   0b0c016e        add     w14, w11, w12
        //  18:   2a0e03ed        mov     w13, w14
        //  1c:   b5ffffad        cbnz    x13, 0x10
        //  20:   2a0b03e0        mov     w0, w11
        //  24:   b5ffff60        cbnz    x0, 0x10
        //  28:   d2824681        mov     x1, #0x1234                     // #4660
        //  2c:   4b010160        sub     w0, w11, w1
        //  30:   d65f03c0        ret

        let golden = vec![
            137, 70, 130, 210, 11, 0, 9, 11, 234, 3, 11, 42, 170, 0, 0, 181, 140, 70, 130, 210,
            110, 1, 12, 11, 237, 3, 14, 42, 173, 255, 255, 181, 224, 3, 11, 42, 96, 255, 255, 181,
            129, 70, 130, 210, 96, 1, 1, 75, 192, 3, 95, 214,
        ];

        assert_eq!(code, &golden[..]);
    }

    #[test]
    fn test_br_table() {
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
        let isa_flags = aarch64_settings::Flags::new(&shared_flags, aarch64_settings::builder());
        let backend = AArch64Backend::new_with_flags(
            Triple::from_str("aarch64").unwrap(),
            shared_flags,
            isa_flags,
        );
        let result = backend
            .compile_function(&mut func, /* want_disasm = */ false)
            .unwrap();
        let code = result.buffer.data();

        // To update this comment, write the golden bytes to a file, and run the following command
        // on it to update:
        // > aarch64-linux-gnu-objdump -b binary -D <file> -m aarch64
        //
        //   0:   7100081f        cmp     w0, #0x2
        //   4:   54000122        b.cs    0x28  // b.hs, b.nlast
        //   8:   9a8023e8        csel    x8, xzr, x0, cs  // cs = hs, nlast
        //   c:   d503229f        csdb
        //  10:   10000087        adr     x7, 0x20
        //  14:   b8a858e8        ldrsw   x8, [x7, w8, uxtw #2]
        //  18:   8b0800e7        add     x7, x7, x8
        //  1c:   d61f00e0        br      x7
        //  20:   00000010        udf     #16
        //  24:   00000018        udf     #24
        //  28:   d2800060        mov     x0, #0x3                        // #3
        //  2c:   d65f03c0        ret
        //  30:   d2800020        mov     x0, #0x1                        // #1
        //  34:   d65f03c0        ret
        //  38:   d2800040        mov     x0, #0x2                        // #2
        //  3c:   d65f03c0        ret

        let golden = vec![
            31, 8, 0, 113, 34, 1, 0, 84, 232, 35, 128, 154, 159, 34, 3, 213, 135, 0, 0, 16, 232,
            88, 168, 184, 231, 0, 8, 139, 224, 0, 31, 214, 16, 0, 0, 0, 24, 0, 0, 0, 96, 0, 128,
            210, 192, 3, 95, 214, 32, 0, 128, 210, 192, 3, 95, 214, 64, 0, 128, 210, 192, 3, 95,
            214,
        ];

        assert_eq!(code, &golden[..]);
    }
}
