//! Lower a single Cranelift instruction into vcode.

use crate::ir::Inst as IRInst;
use crate::ir::Opcode;
use crate::isa::aarch64::inst::*;
use crate::isa::aarch64::settings as aarch64_settings;
use crate::machinst::lower::*;
use crate::machinst::*;
use crate::settings::Flags;
use crate::{CodegenError, CodegenResult};
use target_lexicon::Triple;

/// Actually codegen an instruction's results into registers.
pub(crate) fn lower_insn_to_regs(
    ctx: &mut Lower<Inst>,
    insn: IRInst,
    triple: &Triple,
    flags: &Flags,
    isa_flags: &aarch64_settings::Flags,
) -> CodegenResult<()> {
    let op = ctx.data(insn).opcode();
    let outputs = insn_outputs(ctx, insn);
    let ty = if outputs.len() > 0 {
        Some(ctx.output_ty(insn, 0))
    } else {
        None
    };

    if let Ok(()) = super::lower::isle::lower(ctx, triple, flags, isa_flags, &outputs, insn) {
        return Ok(());
    }

    let implemented_in_isle = |ctx: &mut Lower<Inst>| -> ! {
        unreachable!(
            "implemented in ISLE: inst = `{}`, type = `{:?}`",
            ctx.dfg().display_inst(insn),
            ty
        );
    };

    match op {
        Opcode::Iconst | Opcode::Null => implemented_in_isle(ctx),

        Opcode::F32const => {
            let rd = get_output_reg(ctx, outputs[0]).only_reg().unwrap();
            let val = ctx.get_constant(insn).unwrap();
            for inst in
                Inst::load_fp_constant32(rd, val as u32, |ty| ctx.alloc_tmp(ty).only_reg().unwrap())
            {
                ctx.emit(inst);
            }
        }

        Opcode::F64const => {
            let rd = get_output_reg(ctx, outputs[0]).only_reg().unwrap();
            let val = ctx.get_constant(insn).unwrap();
            for inst in
                Inst::load_fp_constant64(rd, val, |ty| ctx.alloc_tmp(ty).only_reg().unwrap())
            {
                ctx.emit(inst);
            }
        }

        Opcode::GetFramePointer | Opcode::GetStackPointer | Opcode::GetReturnAddress => {
            implemented_in_isle(ctx)
        }

        Opcode::Iadd => implemented_in_isle(ctx),
        Opcode::Isub => implemented_in_isle(ctx),
        Opcode::UaddSat | Opcode::SaddSat | Opcode::UsubSat | Opcode::SsubSat => {
            implemented_in_isle(ctx)
        }

        Opcode::Ineg => implemented_in_isle(ctx),

        Opcode::Imul => implemented_in_isle(ctx),

        Opcode::Umulhi | Opcode::Smulhi => implemented_in_isle(ctx),

        Opcode::Udiv | Opcode::Sdiv | Opcode::Urem | Opcode::Srem => implemented_in_isle(ctx),

        Opcode::Uextend | Opcode::Sextend => implemented_in_isle(ctx),

        Opcode::Bnot => implemented_in_isle(ctx),

        Opcode::Band
        | Opcode::Bor
        | Opcode::Bxor
        | Opcode::BandNot
        | Opcode::BorNot
        | Opcode::BxorNot => implemented_in_isle(ctx),

        Opcode::Ishl | Opcode::Ushr | Opcode::Sshr => implemented_in_isle(ctx),

        Opcode::Rotr | Opcode::Rotl => implemented_in_isle(ctx),

        Opcode::Bitrev | Opcode::Clz | Opcode::Cls | Opcode::Ctz => implemented_in_isle(ctx),

        Opcode::Bswap => implemented_in_isle(ctx),

        Opcode::Popcnt => implemented_in_isle(ctx),

        Opcode::Load
        | Opcode::Uload8
        | Opcode::Sload8
        | Opcode::Uload16
        | Opcode::Sload16
        | Opcode::Uload32
        | Opcode::Sload32
        | Opcode::Sload8x8
        | Opcode::Uload8x8
        | Opcode::Sload16x4
        | Opcode::Uload16x4
        | Opcode::Sload32x2
        | Opcode::Uload32x2 => implemented_in_isle(ctx),

        Opcode::Store | Opcode::Istore8 | Opcode::Istore16 | Opcode::Istore32 => {
            implemented_in_isle(ctx)
        }

        Opcode::StackAddr => implemented_in_isle(ctx),

        Opcode::DynamicStackAddr => implemented_in_isle(ctx),

        Opcode::AtomicRmw => implemented_in_isle(ctx),

        Opcode::AtomicCas => implemented_in_isle(ctx),

        Opcode::AtomicLoad => implemented_in_isle(ctx),

        Opcode::AtomicStore => implemented_in_isle(ctx),

        Opcode::Fence => implemented_in_isle(ctx),

        Opcode::StackLoad
        | Opcode::StackStore
        | Opcode::DynamicStackStore
        | Opcode::DynamicStackLoad => {
            panic!("Direct stack memory access not supported; should not be used by Wasm");
        }

        Opcode::HeapLoad | Opcode::HeapStore | Opcode::HeapAddr => {
            panic!("heap access instructions should have been removed by legalization!");
        }

        Opcode::TableAddr => {
            panic!("table_addr should have been removed by legalization!");
        }

        Opcode::Nop => {
            // Nothing.
        }

        Opcode::Select => implemented_in_isle(ctx),

        Opcode::SelectSpectreGuard => implemented_in_isle(ctx),

        Opcode::Bitselect | Opcode::Vselect => implemented_in_isle(ctx),

        Opcode::IsNull | Opcode::IsInvalid => implemented_in_isle(ctx),

        Opcode::Ireduce => implemented_in_isle(ctx),

        Opcode::Bmask => implemented_in_isle(ctx),

        Opcode::Bitcast => implemented_in_isle(ctx),

        Opcode::Return => implemented_in_isle(ctx),

        Opcode::Ifcmp | Opcode::Ffcmp => {
            // An Ifcmp/Ffcmp must always be seen as a use of a brif/brff or trueif/trueff
            // instruction. This will always be the case as long as the IR uses an Ifcmp/Ffcmp from
            // the same block, or a dominating block. In other words, it cannot pass through a BB
            // param (phi). The flags pass of the verifier will ensure this.
            panic!("Should never reach ifcmp as isel root!");
        }

        Opcode::Icmp => implemented_in_isle(ctx),

        Opcode::Fcmp => implemented_in_isle(ctx),

        Opcode::Debugtrap => implemented_in_isle(ctx),

        Opcode::Trap | Opcode::ResumableTrap => implemented_in_isle(ctx),

        Opcode::Trapz | Opcode::Trapnz | Opcode::ResumableTrapnz => {
            panic!("trapz / trapnz / resumable_trapnz should have been removed by legalization!");
        }

        Opcode::FuncAddr => implemented_in_isle(ctx),

        Opcode::GlobalValue => {
            panic!("global_value should have been removed by legalization!");
        }

        Opcode::SymbolValue => implemented_in_isle(ctx),

        Opcode::Call | Opcode::CallIndirect => implemented_in_isle(ctx),

        Opcode::GetPinnedReg | Opcode::SetPinnedReg => implemented_in_isle(ctx),

        Opcode::Jump | Opcode::Brz | Opcode::Brnz | Opcode::BrTable => {
            panic!("Branch opcode reached non-branch lowering logic!");
        }

        Opcode::Vconst => implemented_in_isle(ctx),

        Opcode::Extractlane => implemented_in_isle(ctx),

        Opcode::Insertlane => implemented_in_isle(ctx),

        Opcode::Splat => implemented_in_isle(ctx),

        Opcode::ScalarToVector => implemented_in_isle(ctx),

        Opcode::VallTrue | Opcode::VanyTrue => implemented_in_isle(ctx),

        Opcode::VhighBits => implemented_in_isle(ctx),

        Opcode::Shuffle => implemented_in_isle(ctx),

        Opcode::Swizzle => implemented_in_isle(ctx),

        Opcode::Isplit => implemented_in_isle(ctx),

        Opcode::Iconcat => implemented_in_isle(ctx),

        Opcode::Smax | Opcode::Umax | Opcode::Umin | Opcode::Smin => implemented_in_isle(ctx),

        Opcode::IaddPairwise => implemented_in_isle(ctx),

        Opcode::WideningPairwiseDotProductS => implemented_in_isle(ctx),

        Opcode::Fadd | Opcode::Fsub | Opcode::Fmul | Opcode::Fdiv | Opcode::Fmin | Opcode::Fmax => {
            implemented_in_isle(ctx)
        }

        Opcode::FminPseudo | Opcode::FmaxPseudo => implemented_in_isle(ctx),

        Opcode::Sqrt | Opcode::Fneg | Opcode::Fabs | Opcode::Fpromote | Opcode::Fdemote => {
            implemented_in_isle(ctx)
        }

        Opcode::Ceil | Opcode::Floor | Opcode::Trunc | Opcode::Nearest => implemented_in_isle(ctx),

        Opcode::Fma => implemented_in_isle(ctx),

        Opcode::Fcopysign => implemented_in_isle(ctx),

        Opcode::FcvtToUint | Opcode::FcvtToSint => implemented_in_isle(ctx),

        Opcode::FcvtFromUint | Opcode::FcvtFromSint => implemented_in_isle(ctx),

        Opcode::FcvtToUintSat | Opcode::FcvtToSintSat => implemented_in_isle(ctx),

        Opcode::IaddIfcout => implemented_in_isle(ctx),

        Opcode::UaddOverflowTrap => implemented_in_isle(ctx),

        Opcode::IaddCout => implemented_in_isle(ctx),

        Opcode::IaddImm
        | Opcode::ImulImm
        | Opcode::UdivImm
        | Opcode::SdivImm
        | Opcode::UremImm
        | Opcode::SremImm
        | Opcode::IrsubImm
        | Opcode::IaddCin
        | Opcode::IaddIfcin
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

        Opcode::Iabs => implemented_in_isle(ctx),
        Opcode::AvgRound => implemented_in_isle(ctx),

        Opcode::Snarrow | Opcode::Unarrow | Opcode::Uunarrow => implemented_in_isle(ctx),

        Opcode::SwidenLow | Opcode::SwidenHigh | Opcode::UwidenLow | Opcode::UwidenHigh => {
            implemented_in_isle(ctx)
        }

        Opcode::TlsValue => implemented_in_isle(ctx),

        Opcode::SqmulRoundSat => implemented_in_isle(ctx),

        Opcode::FcvtLowFromSint => implemented_in_isle(ctx),

        Opcode::FvpromoteLow => implemented_in_isle(ctx),

        Opcode::Fvdemote => implemented_in_isle(ctx),

        Opcode::ExtractVector => implemented_in_isle(ctx),

        Opcode::Vconcat | Opcode::Vsplit => {
            return Err(CodegenError::Unsupported(format!(
                "Unimplemented lowering: {}",
                op
            )));
        }
    }

    Ok(())
}
