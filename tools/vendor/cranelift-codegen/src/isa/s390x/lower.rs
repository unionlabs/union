//! Lowering rules for S390x.

use crate::ir::condcodes::IntCC;
use crate::ir::Inst as IRInst;
use crate::ir::{MemFlags, Opcode};
use crate::isa::s390x::abi::*;
use crate::isa::s390x::inst::*;
use crate::isa::s390x::settings as s390x_settings;
use crate::isa::s390x::S390xBackend;
use crate::machinst::lower::*;
use crate::machinst::*;
use crate::settings::Flags;
use crate::CodegenResult;
use regalloc::Reg;
use smallvec::SmallVec;

pub mod isle;

//============================================================================
// Lowering: force instruction input into a register

/// Sign-extend the low `from_bits` bits of `value` to a full u64.
fn sign_extend_to_u64(value: u64, from_bits: u8) -> u64 {
    assert!(from_bits <= 64);
    if from_bits >= 64 {
        value
    } else {
        (((value << (64 - from_bits)) as i64) >> (64 - from_bits)) as u64
    }
}

/// Lower an instruction input to a reg.
fn put_input_in_reg<C: LowerCtx<I = Inst>>(ctx: &mut C, input: InsnInput) -> Reg {
    ctx.put_input_in_regs(input.insn, input.input)
        .only_reg()
        .unwrap()
}

//=============================================================================
// Lowering: comparisons

/// Determines whether this condcode interprets inputs as signed or
/// unsigned.  See the documentation for the `icmp` instruction in
/// cranelift-codegen/meta/src/shared/instructions.rs for further insights
/// into this.
pub fn condcode_is_signed(cc: IntCC) -> bool {
    match cc {
        IntCC::Equal => false,
        IntCC::NotEqual => false,
        IntCC::SignedGreaterThanOrEqual => true,
        IntCC::SignedGreaterThan => true,
        IntCC::SignedLessThanOrEqual => true,
        IntCC::SignedLessThan => true,
        IntCC::UnsignedGreaterThanOrEqual => false,
        IntCC::UnsignedGreaterThan => false,
        IntCC::UnsignedLessThanOrEqual => false,
        IntCC::UnsignedLessThan => false,
        IntCC::Overflow => true,
        IntCC::NotOverflow => true,
    }
}

//============================================================================
// Lowering: main entry point for lowering a instruction

fn lower_insn_to_regs<C: LowerCtx<I = Inst>>(
    ctx: &mut C,
    insn: IRInst,
    flags: &Flags,
    isa_flags: &s390x_settings::Flags,
) -> CodegenResult<()> {
    let op = ctx.data(insn).opcode();
    let inputs: SmallVec<[InsnInput; 4]> = (0..ctx.num_inputs(insn))
        .map(|i| InsnInput { insn, input: i })
        .collect();
    let outputs: SmallVec<[InsnOutput; 2]> = (0..ctx.num_outputs(insn))
        .map(|i| InsnOutput { insn, output: i })
        .collect();
    let ty = if outputs.len() > 0 {
        Some(ctx.output_ty(insn, 0))
    } else {
        None
    };

    if let Ok(()) = super::lower::isle::lower(ctx, flags, isa_flags, &outputs, insn) {
        return Ok(());
    }

    let implemented_in_isle = || {
        unreachable!(
            "implemented in ISLE: inst = `{}`, type = `{:?}`",
            ctx.dfg().display_inst(insn),
            ty
        );
    };

    match op {
        Opcode::Nop
        | Opcode::Copy
        | Opcode::Iconst
        | Opcode::Bconst
        | Opcode::F32const
        | Opcode::F64const
        | Opcode::Null
        | Opcode::Iadd
        | Opcode::IaddIfcout
        | Opcode::Isub
        | Opcode::Iabs
        | Opcode::Ineg
        | Opcode::Imul
        | Opcode::Umulhi
        | Opcode::Smulhi
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
        | Opcode::Bnot
        | Opcode::Band
        | Opcode::Bor
        | Opcode::Bxor
        | Opcode::BandNot
        | Opcode::BorNot
        | Opcode::BxorNot
        | Opcode::Bitselect
        | Opcode::Breduce
        | Opcode::Bextend
        | Opcode::Bmask
        | Opcode::Bint
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
        | Opcode::Sqrt
        | Opcode::Fneg
        | Opcode::Fabs
        | Opcode::Fpromote
        | Opcode::Fdemote
        | Opcode::Ceil
        | Opcode::Floor
        | Opcode::Trunc
        | Opcode::Nearest
        | Opcode::Fma
        | Opcode::Fcopysign
        | Opcode::FcvtFromUint
        | Opcode::FcvtFromSint
        | Opcode::FcvtToUint
        | Opcode::FcvtToSint
        | Opcode::FcvtToUintSat
        | Opcode::FcvtToSintSat
        | Opcode::Bitcast
        | Opcode::Load
        | Opcode::Uload8
        | Opcode::Sload8
        | Opcode::Uload16
        | Opcode::Sload16
        | Opcode::Uload32
        | Opcode::Sload32
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
        | Opcode::IsNull
        | Opcode::IsInvalid
        | Opcode::Select
        | Opcode::SelectifSpectreGuard
        | Opcode::Trap
        | Opcode::ResumableTrap
        | Opcode::Trapz
        | Opcode::Trapnz
        | Opcode::ResumableTrapnz
        | Opcode::Trapif
        | Opcode::Debugtrap
        | Opcode::StackAddr
        | Opcode::FuncAddr
        | Opcode::SymbolValue => implemented_in_isle(),

        Opcode::UaddSat | Opcode::SaddSat => unimplemented!(),
        Opcode::UsubSat | Opcode::SsubSat => unimplemented!(),

        Opcode::Bitrev => unimplemented!(),

        Opcode::FcvtLowFromSint => unimplemented!("FcvtLowFromSint"),

        Opcode::StackLoad | Opcode::StackStore => {
            panic!("Direct stack memory access not supported; should not be used by Wasm");
        }

        Opcode::ConstAddr => unimplemented!(),

        Opcode::HeapAddr => {
            panic!("heap_addr should have been removed by legalization!");
        }

        Opcode::TableAddr => {
            panic!("table_addr should have been removed by legalization!");
        }

        Opcode::GlobalValue => {
            panic!("global_value should have been removed by legalization!");
        }

        Opcode::TlsValue => {
            unimplemented!("Thread-local storage support not implemented!");
        }

        Opcode::GetPinnedReg | Opcode::SetPinnedReg => {
            unimplemented!("Pinned register support not implemented!");
        }

        Opcode::Call | Opcode::CallIndirect => {
            let caller_conv = ctx.abi().call_conv();
            let (mut abi, inputs) = match op {
                Opcode::Call => {
                    let (extname, dist) = ctx.call_target(insn).unwrap();
                    let extname = extname.clone();
                    let sig = ctx.call_sig(insn).unwrap();
                    assert!(inputs.len() == sig.params.len());
                    assert!(outputs.len() == sig.returns.len());
                    (
                        S390xABICaller::from_func(sig, &extname, dist, caller_conv, flags)?,
                        &inputs[..],
                    )
                }
                Opcode::CallIndirect => {
                    let ptr = put_input_in_reg(ctx, inputs[0]);
                    let sig = ctx.call_sig(insn).unwrap();
                    assert!(inputs.len() - 1 == sig.params.len());
                    assert!(outputs.len() == sig.returns.len());
                    (
                        S390xABICaller::from_ptr(sig, ptr, op, caller_conv, flags)?,
                        &inputs[1..],
                    )
                }
                _ => unreachable!(),
            };

            assert!(inputs.len() == abi.num_args());
            for (i, input) in inputs.iter().enumerate() {
                let arg_reg = put_input_in_reg(ctx, *input);
                abi.emit_copy_regs_to_arg(ctx, i, ValueRegs::one(arg_reg));
            }
            abi.emit_call(ctx);
            for (i, output) in outputs.iter().enumerate() {
                let retval_reg = get_output_reg(ctx, *output).only_reg().unwrap();
                abi.emit_copy_retval_to_regs(ctx, i, ValueRegs::one(retval_reg));
            }
            abi.accumulate_outgoing_args_size(ctx);
        }

        Opcode::FallthroughReturn | Opcode::Return => {
            for (i, input) in inputs.iter().enumerate() {
                let reg = put_input_in_reg(ctx, *input);
                let retval_reg = ctx.retval(i).only_reg().unwrap();
                let ty = ctx.input_ty(insn, i);
                ctx.emit(Inst::gen_move(retval_reg, reg, ty));
            }
            // N.B.: the Ret itself is generated by the ABI.
        }

        Opcode::RawBitcast
        | Opcode::Splat
        | Opcode::Swizzle
        | Opcode::Insertlane
        | Opcode::Extractlane
        | Opcode::Imin
        | Opcode::Umin
        | Opcode::Imax
        | Opcode::Umax
        | Opcode::AvgRound
        | Opcode::FminPseudo
        | Opcode::FmaxPseudo
        | Opcode::Uload8x8
        | Opcode::Uload8x8Complex
        | Opcode::Sload8x8
        | Opcode::Sload8x8Complex
        | Opcode::Uload16x4
        | Opcode::Uload16x4Complex
        | Opcode::Sload16x4
        | Opcode::Sload16x4Complex
        | Opcode::Uload32x2
        | Opcode::Uload32x2Complex
        | Opcode::Sload32x2
        | Opcode::Sload32x2Complex
        | Opcode::Vconst
        | Opcode::Shuffle
        | Opcode::Vsplit
        | Opcode::Vconcat
        | Opcode::Vselect
        | Opcode::VanyTrue
        | Opcode::VallTrue
        | Opcode::VhighBits
        | Opcode::ScalarToVector
        | Opcode::Snarrow
        | Opcode::Unarrow
        | Opcode::Uunarrow
        | Opcode::SwidenLow
        | Opcode::SwidenHigh
        | Opcode::UwidenLow
        | Opcode::UwidenHigh
        | Opcode::WideningPairwiseDotProductS
        | Opcode::SqmulRoundSat
        | Opcode::FvpromoteLow
        | Opcode::Fvdemote
        | Opcode::IaddPairwise => {
            // TODO
            unimplemented!("Vector ops not implemented.");
        }

        Opcode::Isplit | Opcode::Iconcat => unimplemented!("Wide integer ops not implemented."),

        Opcode::IfcmpSp => {
            panic!("Unused opcode should not be encountered.");
        }

        Opcode::LoadComplex
        | Opcode::Uload8Complex
        | Opcode::Sload8Complex
        | Opcode::Uload16Complex
        | Opcode::Sload16Complex
        | Opcode::Uload32Complex
        | Opcode::Sload32Complex
        | Opcode::StoreComplex
        | Opcode::Istore8Complex
        | Opcode::Istore16Complex
        | Opcode::Istore32Complex => {
            panic!("Load/store complex opcode should not be encountered.");
        }

        Opcode::Ifcmp
        | Opcode::Ffcmp
        | Opcode::Trapff
        | Opcode::Trueif
        | Opcode::Trueff
        | Opcode::Selectif => {
            panic!("Flags opcode should not be encountered.");
        }

        Opcode::Jump
        | Opcode::Brz
        | Opcode::Brnz
        | Opcode::BrIcmp
        | Opcode::Brif
        | Opcode::Brff
        | Opcode::BrTable => {
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

    Ok(())
}

//=============================================================================
// Lowering-backend trait implementation.

impl LowerBackend for S390xBackend {
    type MInst = Inst;

    fn lower<C: LowerCtx<I = Inst>>(&self, ctx: &mut C, ir_inst: IRInst) -> CodegenResult<()> {
        lower_insn_to_regs(ctx, ir_inst, &self.flags, &self.isa_flags)
    }

    fn lower_branch_group<C: LowerCtx<I = Inst>>(
        &self,
        ctx: &mut C,
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
