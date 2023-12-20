//! Lowering rules for S390x.

use crate::ir::Inst as IRInst;
use crate::ir::Opcode;
use crate::isa::s390x::inst::Inst;
use crate::isa::s390x::S390xBackend;
use crate::machinst::{InsnOutput, Lower, LowerBackend, MachLabel};
use crate::CodegenResult;
use smallvec::SmallVec;

pub mod isle;

//=============================================================================
// Lowering-backend trait implementation.

impl LowerBackend for S390xBackend {
    type MInst = Inst;

    fn lower(&self, ctx: &mut Lower<Inst>, ir_inst: IRInst) -> CodegenResult<()> {
        let op = ctx.data(ir_inst).opcode();
        let outputs: SmallVec<[InsnOutput; 2]> = (0..ctx.num_outputs(ir_inst))
            .map(|i| InsnOutput {
                insn: ir_inst,
                output: i,
            })
            .collect();
        let ty = if outputs.len() > 0 {
            Some(ctx.output_ty(ir_inst, 0))
        } else {
            None
        };

        if let Ok(()) = super::lower::isle::lower(
            ctx,
            &self.triple,
            &self.flags,
            &self.isa_flags,
            &outputs,
            ir_inst,
        ) {
            return Ok(());
        }

        match op {
            Opcode::Nop
            | Opcode::Iconst
            | Opcode::F32const
            | Opcode::F64const
            | Opcode::Vconst
            | Opcode::Null
            | Opcode::Isplit
            | Opcode::Iconcat
            | Opcode::Iadd
            | Opcode::IaddIfcout
            | Opcode::Isub
            | Opcode::UaddSat
            | Opcode::SaddSat
            | Opcode::UsubSat
            | Opcode::SsubSat
            | Opcode::IaddPairwise
            | Opcode::Smin
            | Opcode::Umin
            | Opcode::Smax
            | Opcode::Umax
            | Opcode::AvgRound
            | Opcode::Iabs
            | Opcode::Ineg
            | Opcode::Imul
            | Opcode::Umulhi
            | Opcode::Smulhi
            | Opcode::WideningPairwiseDotProductS
            | Opcode::SqmulRoundSat
            | Opcode::Udiv
            | Opcode::Urem
            | Opcode::Sdiv
            | Opcode::Srem
            | Opcode::Ishl
            | Opcode::Ushr
            | Opcode::Sshr
            | Opcode::Rotr
            | Opcode::Rotl
            | Opcode::Ireduce
            | Opcode::Uextend
            | Opcode::Sextend
            | Opcode::Snarrow
            | Opcode::Unarrow
            | Opcode::Uunarrow
            | Opcode::SwidenLow
            | Opcode::SwidenHigh
            | Opcode::UwidenLow
            | Opcode::UwidenHigh
            | Opcode::Bnot
            | Opcode::Band
            | Opcode::Bor
            | Opcode::Bxor
            | Opcode::BandNot
            | Opcode::BorNot
            | Opcode::BxorNot
            | Opcode::Bitselect
            | Opcode::Vselect
            | Opcode::Bmask
            | Opcode::Bitrev
            | Opcode::Bswap
            | Opcode::Clz
            | Opcode::Cls
            | Opcode::Ctz
            | Opcode::Popcnt
            | Opcode::Fadd
            | Opcode::Fsub
            | Opcode::Fmul
            | Opcode::Fdiv
            | Opcode::Fmin
            | Opcode::Fmax
            | Opcode::FminPseudo
            | Opcode::FmaxPseudo
            | Opcode::Sqrt
            | Opcode::Fneg
            | Opcode::Fabs
            | Opcode::Fpromote
            | Opcode::Fdemote
            | Opcode::FvpromoteLow
            | Opcode::Fvdemote
            | Opcode::Ceil
            | Opcode::Floor
            | Opcode::Trunc
            | Opcode::Nearest
            | Opcode::Fma
            | Opcode::Fcopysign
            | Opcode::FcvtFromUint
            | Opcode::FcvtFromSint
            | Opcode::FcvtLowFromSint
            | Opcode::FcvtToUint
            | Opcode::FcvtToSint
            | Opcode::FcvtToUintSat
            | Opcode::FcvtToSintSat
            | Opcode::Splat
            | Opcode::Swizzle
            | Opcode::Shuffle
            | Opcode::Insertlane
            | Opcode::Extractlane
            | Opcode::ScalarToVector
            | Opcode::VhighBits
            | Opcode::Bitcast
            | Opcode::Load
            | Opcode::Uload8
            | Opcode::Sload8
            | Opcode::Uload16
            | Opcode::Sload16
            | Opcode::Uload32
            | Opcode::Sload32
            | Opcode::Uload8x8
            | Opcode::Sload8x8
            | Opcode::Uload16x4
            | Opcode::Sload16x4
            | Opcode::Uload32x2
            | Opcode::Sload32x2
            | Opcode::Store
            | Opcode::Istore8
            | Opcode::Istore16
            | Opcode::Istore32
            | Opcode::AtomicRmw
            | Opcode::AtomicCas
            | Opcode::AtomicLoad
            | Opcode::AtomicStore
            | Opcode::Fence
            | Opcode::Icmp
            | Opcode::Fcmp
            | Opcode::VanyTrue
            | Opcode::VallTrue
            | Opcode::IsNull
            | Opcode::IsInvalid
            | Opcode::Select
            | Opcode::SelectSpectreGuard
            | Opcode::Trap
            | Opcode::ResumableTrap
            | Opcode::Trapz
            | Opcode::Trapnz
            | Opcode::ResumableTrapnz
            | Opcode::Debugtrap
            | Opcode::Call
            | Opcode::CallIndirect
            | Opcode::Return
            | Opcode::StackAddr
            | Opcode::FuncAddr
            | Opcode::SymbolValue
            | Opcode::TlsValue
            | Opcode::GetFramePointer
            | Opcode::GetStackPointer
            | Opcode::GetReturnAddress => {
                unreachable!(
                    "implemented in ISLE: inst = `{}`, type = `{:?}`",
                    ctx.dfg().display_inst(ir_inst),
                    ty
                )
            }

            Opcode::GetPinnedReg
            | Opcode::SetPinnedReg
            | Opcode::Vsplit
            | Opcode::Vconcat
            | Opcode::DynamicStackLoad
            | Opcode::DynamicStackStore
            | Opcode::DynamicStackAddr
            | Opcode::ExtractVector => {
                unreachable!(
                    "TODO: not yet implemented in ISLE: inst = `{}`, type = `{:?}`",
                    ctx.dfg().display_inst(ir_inst),
                    ty
                )
            }

            Opcode::StackLoad | Opcode::StackStore => {
                panic!("Direct stack memory access not supported; should not be used by Wasm");
            }
            Opcode::HeapLoad | Opcode::HeapStore | Opcode::HeapAddr => {
                panic!("heap access instructions should have been removed by legalization!");
            }
            Opcode::TableAddr => {
                panic!("table_addr should have been removed by legalization!");
            }
            Opcode::GlobalValue => {
                panic!("global_value should have been removed by legalization!");
            }
            Opcode::Ifcmp | Opcode::Ffcmp => {
                panic!("Flags opcode should not be encountered.");
            }
            Opcode::Jump | Opcode::Brz | Opcode::Brnz | Opcode::BrTable => {
                panic!("Branch opcode reached non-branch lowering logic!");
            }
            Opcode::IaddImm
            | Opcode::ImulImm
            | Opcode::UdivImm
            | Opcode::SdivImm
            | Opcode::UremImm
            | Opcode::SremImm
            | Opcode::IrsubImm
            | Opcode::IaddCin
            | Opcode::IaddIfcin
            | Opcode::IaddCout
            | Opcode::IaddCarry
            | Opcode::IaddIfcarry
            | Opcode::UaddOverflowTrap
            | Opcode::IsubBin
            | Opcode::IsubIfbin
            | Opcode::IsubBout
            | Opcode::IsubIfbout
            | Opcode::IsubBorrow
            | Opcode::IsubIfborrow
            | Opcode::BandImm
            | Opcode::BorImm
            | Opcode::BxorImm
            | Opcode::RotlImm
            | Opcode::RotrImm
            | Opcode::IshlImm
            | Opcode::UshrImm
            | Opcode::SshrImm
            | Opcode::IcmpImm
            | Opcode::IfcmpImm => {
                panic!("ALU+imm and ALU+carry ops should not appear here!");
            }
        }
    }

    fn lower_branch_group(
        &self,
        ctx: &mut Lower<Inst>,
        branches: &[IRInst],
        targets: &[MachLabel],
    ) -> CodegenResult<()> {
        // A block should end with at most two branches. The first may be a
        // conditional branch; a conditional branch can be followed only by an
        // unconditional branch or fallthrough. Otherwise, if only one branch,
        // it may be an unconditional branch, a fallthrough, a return, or a
        // trap. These conditions are verified by `is_ebb_basic()` during the
        // verifier pass.
        assert!(branches.len() <= 2);
        if branches.len() == 2 {
            let op1 = ctx.data(branches[1]).opcode();
            assert!(op1 == Opcode::Jump);
        }

        // Lower the first branch in ISLE.  This will automatically handle
        // the second branch (if any) by emitting a two-way conditional branch.
        if let Ok(()) = super::lower::isle::lower_branch(
            ctx,
            &self.triple,
            &self.flags,
            &self.isa_flags,
            branches[0],
            targets,
        ) {
            return Ok(());
        }
        unreachable!(
            "implemented in ISLE: branch = `{}`",
            ctx.dfg().display_inst(branches[0]),
        );
    }
}
