//! Lowering rules for X64.

// ISLE integration glue.
pub(super) mod isle;

use crate::ir::{types, ExternalName, Inst as IRInst, LibCall, Opcode, Type};
use crate::isa::x64::abi::*;
use crate::isa::x64::inst::args::*;
use crate::isa::x64::inst::*;
use crate::isa::{x64::settings as x64_settings, x64::X64Backend, CallConv};
use crate::machinst::abi::SmallInstVec;
use crate::machinst::lower::*;
use crate::machinst::*;
use crate::result::CodegenResult;
use crate::settings::Flags;
use smallvec::{smallvec, SmallVec};
use target_lexicon::Triple;

//=============================================================================
// Helpers for instruction lowering.

fn is_int_or_ref_ty(ty: Type) -> bool {
    match ty {
        types::I8 | types::I16 | types::I32 | types::I64 | types::R64 => true,
        types::R32 => panic!("shouldn't have 32-bits refs on x64"),
        _ => false,
    }
}

/// Returns whether the given specified `input` is a result produced by an instruction with Opcode
/// `op`.
// TODO investigate failures with checking against the result index.
fn matches_input(ctx: &mut Lower<Inst>, input: InsnInput, op: Opcode) -> Option<IRInst> {
    let inputs = ctx.get_input_as_source_or_const(input.insn, input.input);
    inputs.inst.as_inst().and_then(|(src_inst, _)| {
        let data = ctx.data(src_inst);
        if data.opcode() == op {
            return Some(src_inst);
        }
        None
    })
}

/// Emits instruction(s) to generate the given 64-bit constant value into a newly-allocated
/// temporary register, returning that register.
fn generate_constant(ctx: &mut Lower<Inst>, ty: Type, c: u64) -> ValueRegs<Reg> {
    let from_bits = ty_bits(ty);
    let masked = if from_bits < 64 {
        c & ((1u64 << from_bits) - 1)
    } else {
        c
    };

    let cst_copy = ctx.alloc_tmp(ty);
    for inst in Inst::gen_constant(cst_copy, masked as u128, ty, |ty| {
        ctx.alloc_tmp(ty).only_reg().unwrap()
    })
    .into_iter()
    {
        ctx.emit(inst);
    }
    non_writable_value_regs(cst_copy)
}

/// Put the given input into possibly multiple registers, and mark it as used (side-effect).
fn put_input_in_regs(ctx: &mut Lower<Inst>, spec: InsnInput) -> ValueRegs<Reg> {
    let ty = ctx.input_ty(spec.insn, spec.input);
    let input = ctx.get_input_as_source_or_const(spec.insn, spec.input);

    if let Some(c) = input.constant {
        // Generate constants fresh at each use to minimize long-range register pressure.
        generate_constant(ctx, ty, c)
    } else {
        ctx.put_input_in_regs(spec.insn, spec.input)
    }
}

/// Put the given input into a register, and mark it as used (side-effect).
fn put_input_in_reg(ctx: &mut Lower<Inst>, spec: InsnInput) -> Reg {
    put_input_in_regs(ctx, spec)
        .only_reg()
        .expect("Multi-register value not expected")
}

/// Determines whether a load operation (indicated by `src_insn`) can be merged
/// into the current lowering point. If so, returns the address-base source (as
/// an `InsnInput`) and an offset from that address from which to perform the
/// load.
fn is_mergeable_load(ctx: &mut Lower<Inst>, src_insn: IRInst) -> Option<(InsnInput, i32)> {
    let insn_data = ctx.data(src_insn);
    let inputs = ctx.num_inputs(src_insn);
    if inputs != 1 {
        return None;
    }

    let load_ty = ctx.output_ty(src_insn, 0);
    if ty_bits(load_ty) < 32 {
        // Narrower values are handled by ALU insts that are at least 32 bits
        // wide, which is normally OK as we ignore upper buts; but, if we
        // generate, e.g., a direct-from-memory 32-bit add for a byte value and
        // the byte is the last byte in a page, the extra data that we load is
        // incorrectly accessed. So we only allow loads to merge for
        // 32-bit-and-above widths.
        return None;
    }

    // SIMD instructions can only be load-coalesced when the loaded value comes
    // from an aligned address.
    if load_ty.is_vector() && !insn_data.memflags().map_or(false, |f| f.aligned()) {
        return None;
    }

    // Just testing the opcode is enough, because the width will always match if
    // the type does (and the type should match if the CLIF is properly
    // constructed).
    if insn_data.opcode() == Opcode::Load {
        let offset = insn_data
            .load_store_offset()
            .expect("load should have offset");
        Some((
            InsnInput {
                insn: src_insn,
                input: 0,
            },
            offset,
        ))
    } else {
        None
    }
}

fn input_to_imm(ctx: &mut Lower<Inst>, spec: InsnInput) -> Option<u64> {
    ctx.get_input_as_source_or_const(spec.insn, spec.input)
        .constant
}

fn emit_vm_call(
    ctx: &mut Lower<Inst>,
    flags: &Flags,
    triple: &Triple,
    libcall: LibCall,
    inputs: &[Reg],
    outputs: &[Writable<Reg>],
) -> CodegenResult<()> {
    let extname = ExternalName::LibCall(libcall);

    let dist = if flags.use_colocated_libcalls() {
        RelocDistance::Near
    } else {
        RelocDistance::Far
    };

    // TODO avoid recreating signatures for every single Libcall function.
    let call_conv = CallConv::for_libcall(flags, CallConv::triple_default(triple));
    let sig = libcall.signature(call_conv);
    let caller_conv = ctx.abi().call_conv(ctx.sigs());

    if !ctx.sigs().have_abi_sig_for_signature(&sig) {
        ctx.sigs_mut()
            .make_abi_sig_from_ir_signature::<X64ABIMachineSpec>(sig.clone(), flags)?;
    }

    let mut abi =
        X64Caller::from_libcall(ctx.sigs(), &sig, &extname, dist, caller_conv, flags.clone())?;

    abi.emit_stack_pre_adjust(ctx);

    assert_eq!(inputs.len(), abi.num_args(ctx.sigs()));

    for (i, input) in inputs.iter().enumerate() {
        for inst in abi.gen_arg(ctx, i, ValueRegs::one(*input)) {
            ctx.emit(inst);
        }
    }

    let mut retval_insts: SmallInstVec<_> = smallvec![];
    for (i, output) in outputs.iter().enumerate() {
        retval_insts.extend(abi.gen_retval(ctx, i, ValueRegs::one(*output)).into_iter());
    }
    abi.emit_call(ctx);
    for inst in retval_insts {
        ctx.emit(inst);
    }
    abi.emit_stack_post_adjust(ctx);

    Ok(())
}

/// Returns whether the given input is a shift by a constant value less or equal than 3.
/// The goal is to embed it within an address mode.
fn matches_small_constant_shift(ctx: &mut Lower<Inst>, spec: InsnInput) -> Option<(InsnInput, u8)> {
    matches_input(ctx, spec, Opcode::Ishl).and_then(|shift| {
        match input_to_imm(
            ctx,
            InsnInput {
                insn: shift,
                input: 1,
            },
        ) {
            Some(shift_amt) if shift_amt <= 3 => Some((
                InsnInput {
                    insn: shift,
                    input: 0,
                },
                shift_amt as u8,
            )),
            _ => None,
        }
    })
}

/// Lowers an instruction to one of the x86 addressing modes.
///
/// Note: the 32-bit offset in Cranelift has to be sign-extended, which maps x86's behavior.
fn lower_to_amode(ctx: &mut Lower<Inst>, spec: InsnInput, offset: i32) -> Amode {
    let flags = ctx
        .memflags(spec.insn)
        .expect("Instruction with amode should have memflags");

    // We now either have an add that we must materialize, or some other input; as well as the
    // final offset.
    if let Some(add) = matches_input(ctx, spec, Opcode::Iadd) {
        debug_assert_eq!(ctx.output_ty(add, 0), types::I64);
        let add_inputs = &[
            InsnInput {
                insn: add,
                input: 0,
            },
            InsnInput {
                insn: add,
                input: 1,
            },
        ];

        // TODO heap_addr legalization generates a uext64 *after* the shift, so these optimizations
        // aren't happening in the wasm case. We could do better, given some range analysis.
        let (base, index, shift) = if let Some((shift_input, shift_amt)) =
            matches_small_constant_shift(ctx, add_inputs[0])
        {
            (
                put_input_in_reg(ctx, add_inputs[1]),
                put_input_in_reg(ctx, shift_input),
                shift_amt,
            )
        } else if let Some((shift_input, shift_amt)) =
            matches_small_constant_shift(ctx, add_inputs[1])
        {
            (
                put_input_in_reg(ctx, add_inputs[0]),
                put_input_in_reg(ctx, shift_input),
                shift_amt,
            )
        } else {
            for i in 0..=1 {
                // Try to pierce through uextend.
                if let Some(uextend) = matches_input(
                    ctx,
                    InsnInput {
                        insn: add,
                        input: i,
                    },
                    Opcode::Uextend,
                ) {
                    if let Some(cst) = ctx.get_input_as_source_or_const(uextend, 0).constant {
                        // Zero the upper bits.
                        let input_size = ctx.input_ty(uextend, 0).bits() as u64;
                        let shift: u64 = 64 - input_size;
                        let uext_cst: u64 = (cst << shift) >> shift;

                        let final_offset = (offset as i64).wrapping_add(uext_cst as i64);
                        if low32_will_sign_extend_to_64(final_offset as u64) {
                            let base = put_input_in_reg(ctx, add_inputs[1 - i]);
                            return Amode::imm_reg(final_offset as u32, base).with_flags(flags);
                        }
                    }
                }

                // If it's a constant, add it directly!
                if let Some(cst) = ctx.get_input_as_source_or_const(add, i).constant {
                    let final_offset = (offset as i64).wrapping_add(cst as i64);
                    if low32_will_sign_extend_to_64(final_offset as u64) {
                        let base = put_input_in_reg(ctx, add_inputs[1 - i]);
                        return Amode::imm_reg(final_offset as u32, base).with_flags(flags);
                    }
                }
            }

            (
                put_input_in_reg(ctx, add_inputs[0]),
                put_input_in_reg(ctx, add_inputs[1]),
                0,
            )
        };

        return Amode::imm_reg_reg_shift(
            offset as u32,
            Gpr::new(base).unwrap(),
            Gpr::new(index).unwrap(),
            shift,
        )
        .with_flags(flags);
    }

    let input = put_input_in_reg(ctx, spec);
    Amode::imm_reg(offset as u32, input).with_flags(flags)
}

//=============================================================================
// Top-level instruction lowering entry point, for one instruction.

/// Actually codegen an instruction's results into registers.
fn lower_insn_to_regs(
    ctx: &mut Lower<Inst>,
    insn: IRInst,
    flags: &Flags,
    isa_flags: &x64_settings::Flags,
    triple: &Triple,
) -> CodegenResult<()> {
    let outputs: SmallVec<[InsnOutput; 2]> = (0..ctx.num_outputs(insn))
        .map(|i| InsnOutput { insn, output: i })
        .collect();

    if let Ok(()) = isle::lower(ctx, triple, flags, isa_flags, &outputs, insn) {
        return Ok(());
    }

    let op = ctx.data(insn).opcode();
    match op {
        Opcode::Iconst
        | Opcode::F32const
        | Opcode::F64const
        | Opcode::Null
        | Opcode::Iadd
        | Opcode::IaddIfcout
        | Opcode::SaddSat
        | Opcode::UaddSat
        | Opcode::Isub
        | Opcode::SsubSat
        | Opcode::UsubSat
        | Opcode::AvgRound
        | Opcode::Band
        | Opcode::Bor
        | Opcode::Bxor
        | Opcode::Imul
        | Opcode::BandNot
        | Opcode::Iabs
        | Opcode::Smax
        | Opcode::Umax
        | Opcode::Smin
        | Opcode::Umin
        | Opcode::Bnot
        | Opcode::Bitselect
        | Opcode::Vselect
        | Opcode::Ushr
        | Opcode::Sshr
        | Opcode::Ishl
        | Opcode::Rotl
        | Opcode::Rotr
        | Opcode::Ineg
        | Opcode::Trap
        | Opcode::ResumableTrap
        | Opcode::Clz
        | Opcode::Ctz
        | Opcode::Popcnt
        | Opcode::Bitrev
        | Opcode::Bswap
        | Opcode::IsNull
        | Opcode::IsInvalid
        | Opcode::Uextend
        | Opcode::Sextend
        | Opcode::Ireduce
        | Opcode::Debugtrap
        | Opcode::WideningPairwiseDotProductS
        | Opcode::Fadd
        | Opcode::Fsub
        | Opcode::Fmul
        | Opcode::Fdiv
        | Opcode::Fmin
        | Opcode::Fmax
        | Opcode::FminPseudo
        | Opcode::FmaxPseudo
        | Opcode::Sqrt
        | Opcode::Fpromote
        | Opcode::FvpromoteLow
        | Opcode::Fdemote
        | Opcode::Fvdemote
        | Opcode::Fma
        | Opcode::Icmp
        | Opcode::Fcmp
        | Opcode::Load
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
        | Opcode::Uload32x2
        | Opcode::Store
        | Opcode::Istore8
        | Opcode::Istore16
        | Opcode::Istore32
        | Opcode::AtomicRmw
        | Opcode::AtomicCas
        | Opcode::AtomicLoad
        | Opcode::AtomicStore
        | Opcode::Fence
        | Opcode::FuncAddr
        | Opcode::SymbolValue
        | Opcode::Return
        | Opcode::Call
        | Opcode::CallIndirect
        | Opcode::GetFramePointer
        | Opcode::GetStackPointer
        | Opcode::GetReturnAddress
        | Opcode::Select
        | Opcode::SelectSpectreGuard
        | Opcode::FcvtFromSint
        | Opcode::FcvtLowFromSint
        | Opcode::FcvtFromUint
        | Opcode::FcvtToUint
        | Opcode::FcvtToSint
        | Opcode::FcvtToUintSat
        | Opcode::FcvtToSintSat
        | Opcode::IaddPairwise
        | Opcode::UwidenHigh
        | Opcode::UwidenLow
        | Opcode::SwidenHigh
        | Opcode::SwidenLow
        | Opcode::Snarrow
        | Opcode::Unarrow
        | Opcode::Bitcast
        | Opcode::Fabs
        | Opcode::Fneg
        | Opcode::Fcopysign
        | Opcode::Ceil
        | Opcode::Floor
        | Opcode::Nearest
        | Opcode::Trunc
        | Opcode::StackAddr
        | Opcode::Udiv
        | Opcode::Urem
        | Opcode::Sdiv
        | Opcode::Srem
        | Opcode::Umulhi
        | Opcode::Smulhi
        | Opcode::GetPinnedReg
        | Opcode::SetPinnedReg
        | Opcode::Vconst
        | Opcode::Insertlane
        | Opcode::Shuffle
        | Opcode::Swizzle
        | Opcode::Extractlane
        | Opcode::ScalarToVector
        | Opcode::Splat
        | Opcode::VanyTrue
        | Opcode::VallTrue
        | Opcode::VhighBits
        | Opcode::Iconcat
        | Opcode::Isplit
        | Opcode::TlsValue
        | Opcode::SqmulRoundSat
        | Opcode::Uunarrow
        | Opcode::Nop
        | Opcode::Bmask => {
            let ty = if outputs.len() > 0 {
                Some(ctx.output_ty(insn, 0))
            } else {
                None
            };

            unreachable!(
                "implemented in ISLE: inst = `{}`, type = `{:?}`",
                ctx.dfg().display_inst(insn),
                ty
            )
        }

        Opcode::DynamicStackAddr => unimplemented!("DynamicStackAddr"),

        // Unimplemented opcodes below. These are not currently used by Wasm
        // lowering or other known embeddings, but should be either supported or
        // removed eventually
        Opcode::ExtractVector => {
            unimplemented!("ExtractVector not supported");
        }

        Opcode::Cls => unimplemented!("Cls not supported"),

        Opcode::BorNot | Opcode::BxorNot => {
            unimplemented!("or-not / xor-not opcodes not implemented");
        }

        Opcode::Vsplit | Opcode::Vconcat => {
            unimplemented!("Vector split/concat ops not implemented.");
        }

        // Opcodes that should be removed by legalization. These should
        // eventually be removed if/when we replace in-situ legalization with
        // something better.
        Opcode::Ifcmp | Opcode::Ffcmp => {
            panic!("Should never reach ifcmp/ffcmp as isel root!");
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
        | Opcode::UaddOverflowTrap
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

        Opcode::StackLoad
        | Opcode::StackStore
        | Opcode::DynamicStackStore
        | Opcode::DynamicStackLoad => {
            panic!("Direct stack memory access not supported; should have been legalized");
        }

        Opcode::GlobalValue => {
            panic!("global_value should have been removed by legalization!");
        }

        Opcode::HeapLoad | Opcode::HeapStore | Opcode::HeapAddr => {
            panic!("heap access instructions should have been removed by legalization!");
        }

        Opcode::TableAddr => {
            panic!("table_addr should have been removed by legalization!");
        }

        Opcode::Trapz | Opcode::Trapnz | Opcode::ResumableTrapnz => {
            panic!("trapz / trapnz / resumable_trapnz should have been removed by legalization!");
        }

        Opcode::Jump | Opcode::Brz | Opcode::Brnz | Opcode::BrTable => {
            panic!("Branch opcode reached non-branch lowering logic!");
        }
    }
}

//=============================================================================
// Lowering-backend trait implementation.

impl LowerBackend for X64Backend {
    type MInst = Inst;

    fn lower(&self, ctx: &mut Lower<Inst>, ir_inst: IRInst) -> CodegenResult<()> {
        lower_insn_to_regs(ctx, ir_inst, &self.flags, &self.x64_flags, &self.triple)
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

        if let Ok(()) = isle::lower_branch(
            ctx,
            &self.triple,
            &self.flags,
            &self.x64_flags,
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

    fn maybe_pinned_reg(&self) -> Option<Reg> {
        Some(regs::pinned_reg())
    }
}
