//! IBM Z 64-bit Instruction Set Architecture.

use crate::ir::condcodes::IntCC;
use crate::ir::{Function, Type};
use crate::isa::s390x::settings as s390x_settings;
#[cfg(feature = "unwind")]
use crate::isa::unwind::systemv::RegisterMappingError;
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
use target_lexicon::{Architecture, Triple};

// New backend:
mod abi;
pub(crate) mod inst;
mod lower;
mod settings;

use inst::create_machine_env;

use self::inst::EmitInfo;

/// A IBM Z backend.
pub struct S390xBackend {
    triple: Triple,
    flags: shared_settings::Flags,
    isa_flags: s390x_settings::Flags,
    machine_env: MachineEnv,
}

impl S390xBackend {
    /// Create a new IBM Z backend with the given (shared) flags.
    pub fn new_with_flags(
        triple: Triple,
        flags: shared_settings::Flags,
        isa_flags: s390x_settings::Flags,
    ) -> S390xBackend {
        let machine_env = create_machine_env(&flags);
        S390xBackend {
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
        let emit_info = EmitInfo::new(self.isa_flags.clone());
        let sigs = SigSet::new::<abi::S390xMachineDeps>(func, &self.flags)?;
        let abi = abi::S390xCallee::new(func, self, &self.isa_flags, &sigs)?;
        compile::compile::<S390xBackend>(func, self, abi, emit_info, sigs)
    }
}

impl TargetIsa for S390xBackend {
    fn compile_function(
        &self,
        func: &Function,
        want_disasm: bool,
    ) -> CodegenResult<CompiledCodeStencil> {
        let flags = self.flags();
        let (vcode, regalloc_result) = self.compile_vcode(func)?;

        let emit_result = vcode.emit(&regalloc_result, want_disasm, flags.machine_code_cfg_info());
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
        "s390x"
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

    fn dynamic_vector_bytes(&self, _dyn_ty: Type) -> u32 {
        16
    }

    fn unsigned_add_overflow_condition(&self) -> IntCC {
        // The ADD LOGICAL family of instructions set the condition code
        // differently from normal comparisons, in a way that cannot be
        // represented by any of the standard IntCC values.  So we use a
        // dummy value here, which gets remapped to the correct condition
        // code mask during lowering.
        IntCC::UnsignedGreaterThan
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
            _ => None,
        })
    }

    #[cfg(feature = "unwind")]
    fn create_systemv_cie(&self) -> Option<gimli::write::CommonInformationEntry> {
        Some(inst::unwind::systemv::create_cie())
    }

    #[cfg(feature = "unwind")]
    fn map_regalloc_reg_to_dwarf(&self, reg: Reg) -> Result<u16, RegisterMappingError> {
        inst::unwind::systemv::map_reg(reg).map(|reg| reg.0)
    }

    fn text_section_builder(&self, num_funcs: usize) -> Box<dyn TextSectionBuilder> {
        Box::new(MachTextSectionBuilder::<inst::Inst>::new(num_funcs))
    }

    fn function_alignment(&self) -> u32 {
        4
    }
}

impl fmt::Display for S390xBackend {
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
    assert!(triple.architecture == Architecture::S390x);
    IsaBuilder {
        triple,
        setup: s390x_settings::builder(),
        constructor: |triple, shared_flags, builder| {
            let isa_flags = s390x_settings::Flags::new(&shared_flags, builder);
            let backend = S390xBackend::new_with_flags(triple, shared_flags, isa_flags);
            Ok(Box::new(backend))
        },
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cursor::{Cursor, FuncCursor};
    use crate::ir::types::*;
    use crate::ir::UserFuncName;
    use crate::ir::{AbiParam, Function, InstBuilder, Signature};
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
        let isa_flags = s390x_settings::Flags::new(&shared_flags, s390x_settings::builder());
        let backend = S390xBackend::new_with_flags(
            Triple::from_str("s390x").unwrap(),
            shared_flags,
            isa_flags,
        );
        let result = backend
            .compile_function(&mut func, /* want_disasm = */ false)
            .unwrap();
        let code = result.buffer.data();

        // ahi %r2, 0x1234
        // br %r14
        let golden = vec![0xa7, 0x2a, 0x12, 0x34, 0x07, 0xfe];

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
        let isa_flags = s390x_settings::Flags::new(&shared_flags, s390x_settings::builder());
        let backend = S390xBackend::new_with_flags(
            Triple::from_str("s390x").unwrap(),
            shared_flags,
            isa_flags,
        );
        let result = backend
            .compile_function(&mut func, /* want_disasm = */ false)
            .unwrap();
        let code = result.buffer.data();

        // FIXME: the branching logic should be optimized more

        // To update this comment, write the golden bytes to a file, and run the following command
        // on it to update:
        // > s390x-linux-gnu-objdump -b binary -D <file> -m s390
        //
        //  0:   a7 2a 12 34             ahi     %r2,4660
        //  4:   a7 2e 00 00             chi     %r2,0
        //  8:   c0 64 00 00 00 0b       jglh    0x1e
        //  e:   ec 32 12 34 00 d8       ahik    %r3,%r2,4660
        // 14:   a7 3e 00 00             chi     %r3,0
        // 18:   c0 64 ff ff ff fb       jglh    0xe
        // 1e:   a7 2e 00 00             chi     %r2,0
        // 22:   c0 64 ff ff ff f6       jglh    0xe
        // 28:   a7 2a ed cc             ahi     %r2,-4660
        // 2c:   07 fe                   br      %r14

        let golden = vec![
            167, 42, 18, 52, 167, 46, 0, 0, 192, 100, 0, 0, 0, 11, 236, 50, 18, 52, 0, 216, 167,
            62, 0, 0, 192, 100, 255, 255, 255, 251, 167, 46, 0, 0, 192, 100, 255, 255, 255, 246,
            167, 42, 237, 204, 7, 254,
        ];

        assert_eq!(code, &golden[..]);
    }
}
