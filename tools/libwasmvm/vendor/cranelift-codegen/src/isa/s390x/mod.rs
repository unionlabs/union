//! IBM Z 64-bit Instruction Set Architecture.

use crate::ir::condcodes::IntCC;
use crate::ir::Function;
use crate::isa::s390x::settings as s390x_settings;
#[cfg(feature = "unwind")]
use crate::isa::unwind::systemv::RegisterMappingError;
use crate::isa::{Builder as IsaBuilder, TargetIsa};
use crate::machinst::{
    compile, MachCompileResult, MachTextSectionBuilder, TextSectionBuilder, VCode,
};
use crate::result::CodegenResult;
use crate::settings as shared_settings;

use alloc::{boxed::Box, vec::Vec};
use core::fmt;

use regalloc::{PrettyPrint, RealRegUniverse, Reg};
use target_lexicon::{Architecture, Triple};

// New backend:
mod abi;
pub(crate) mod inst;
mod lower;
mod settings;

use inst::create_reg_universe;

use self::inst::EmitInfo;

/// A IBM Z backend.
pub struct S390xBackend {
    triple: Triple,
    flags: shared_settings::Flags,
    isa_flags: s390x_settings::Flags,
    reg_universe: RealRegUniverse,
}

impl S390xBackend {
    /// Create a new IBM Z backend with the given (shared) flags.
    pub fn new_with_flags(
        triple: Triple,
        flags: shared_settings::Flags,
        isa_flags: s390x_settings::Flags,
    ) -> S390xBackend {
        let reg_universe = create_reg_universe(&flags);
        S390xBackend {
            triple,
            flags,
            isa_flags,
            reg_universe,
        }
    }

    /// This performs lowering to VCode, register-allocates the code, computes block layout and
    /// finalizes branches. The result is ready for binary emission.
    fn compile_vcode(
        &self,
        func: &Function,
        flags: shared_settings::Flags,
    ) -> CodegenResult<VCode<inst::Inst>> {
        let emit_info = EmitInfo::new(flags.clone(), self.isa_flags.clone());
        let abi = Box::new(abi::S390xABICallee::new(func, flags, self.isa_flags())?);
        compile::compile::<S390xBackend>(func, self, abi, &self.reg_universe, emit_info)
    }
}

impl TargetIsa for S390xBackend {
    fn compile_function(
        &self,
        func: &Function,
        want_disasm: bool,
    ) -> CodegenResult<MachCompileResult> {
        let flags = self.flags();
        let vcode = self.compile_vcode(func, flags.clone())?;
        let (buffer, bb_starts, bb_edges) = vcode.emit();
        let frame_size = vcode.frame_size();
        let value_labels_ranges = vcode.value_labels_ranges();
        let stackslot_offsets = vcode.stackslot_offsets().clone();

        let disasm = if want_disasm {
            Some(vcode.show_rru(Some(&create_reg_universe(flags))))
        } else {
            None
        };

        let buffer = buffer.finish();

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

    fn name(&self) -> &'static str {
        "s390x"
    }

    fn triple(&self) -> &Triple {
        &self.triple
    }

    fn flags(&self) -> &shared_settings::Flags {
        &self.flags
    }

    fn isa_flags(&self) -> Vec<shared_settings::Value> {
        self.isa_flags.iter().collect()
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

    fn text_section_builder(&self, num_funcs: u32) -> Box<dyn TextSectionBuilder> {
        Box::new(MachTextSectionBuilder::<inst::Inst>::new(num_funcs))
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
    use crate::ir::{AbiParam, ExternalName, Function, InstBuilder, Signature};
    use crate::isa::CallConv;
    use crate::settings;
    use crate::settings::Configurable;
    use core::str::FromStr;
    use target_lexicon::Triple;

    #[test]
    fn test_compile_function() {
        let name = ExternalName::testcase("test0");
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
        let name = ExternalName::testcase("test0");
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

        // ahi %r2, 4660
        // chi %r2, 0
        // jglh label1 ; jg label2
        // jg label6
        // jg label3
        // ahik %r3, %r2, 4660
        // chi %r3, 0
        // jglh label4 ; jg label5
        // jg label3
        // jg label6
        // chi %r2, 0
        // jglh label7 ; jg label8
        // jg label3
        // ahi %r2, -4660
        // br %r14
        let golden = vec![
            167, 42, 18, 52, 167, 46, 0, 0, 192, 100, 0, 0, 0, 11, 236, 50, 18, 52, 0, 216, 167,
            62, 0, 0, 192, 100, 255, 255, 255, 251, 167, 46, 0, 0, 192, 100, 255, 255, 255, 246,
            167, 42, 237, 204, 7, 254,
        ];

        assert_eq!(code, &golden[..]);
    }
}
