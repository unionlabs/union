//! Implementation of a standard S390x ABI.
//!
//! This machine uses the "vanilla" ABI implementation from abi_impl.rs,
//! however a few details are different from the description there:
//!
//! - On s390x, the caller must provide a "register save area" of 160
//!   bytes to any function it calls.  The called function is free to use
//!   this space for any purpose; usually to save callee-saved GPRs.
//!   (Note that while this area is allocated by the caller, it is counted
//!   as part of the callee's stack frame; in particular, the callee's CFA
//!   is the top of the register save area, not the incoming SP value.)
//!
//! - Overflow arguments are passed on the stack starting immediately
//!   above the register save area.  On s390x, this space is allocated
//!   only once directly in the prologue, using a size large enough to
//!   hold overflow arguments for every call in the function.
//!
//! - On s390x we do not use a frame pointer register; instead, every
//!   element of the stack frame is addressed via (constant) offsets
//!   from the stack pointer.  Note that due to the above (and because
//!   there are no variable-sized stack allocations in cranelift), the
//!   value of the stack pointer register never changes after the
//!   initial allocation in the function prologue.
//!
//! Overall, the stack frame layout on s390x is as follows:
//!
//! ```plain
//!   (high address)
//!
//!                              +---------------------------+
//!                              |          ...              |
//! CFA                  ----->  | stack args                |
//!                              +---------------------------+
//!                              |          ...              |
//!                              | 160 bytes reg save area   |
//! SP at function entry ----->  | (used to save GPRs)       |
//!                              +---------------------------+
//!                              |          ...              |
//!                              | clobbered callee-saves    |
//!                              | (used to save FPRs)       |
//! unwind-frame base     ---->  | (alloc'd by prologue)     |
//!                              +---------------------------+
//!                              |          ...              |
//!                              | spill slots               |
//!                              | (accessed via nominal SP) |
//!                              |          ...              |
//!                              | stack slots               |
//!                              | (accessed via nominal SP) |
//! nominal SP --------------->  | (alloc'd by prologue)     |
//!                              +---------------------------+
//!                              |          ...              |
//!                              | args for call             |
//!                              | outgoing reg save area    |
//! SP during function  ------>  | (alloc'd by prologue)     |
//!                              +---------------------------+
//!
//!   (low address)
//! ```

use crate::ir;
use crate::ir::condcodes::IntCC;
use crate::ir::types;
use crate::ir::MemFlags;
use crate::ir::Type;
use crate::isa;
use crate::isa::s390x::inst::*;
use crate::isa::unwind::UnwindInst;
use crate::machinst::*;
use crate::settings;
use crate::{CodegenError, CodegenResult};
use alloc::boxed::Box;
use alloc::vec::Vec;
use regalloc::{RealReg, Reg, RegClass, Set, Writable};
use smallvec::{smallvec, SmallVec};
use std::convert::TryFrom;

// We use a generic implementation that factors out ABI commonalities.

/// Support for the S390x ABI from the callee side (within a function body).
pub type S390xABICallee = ABICalleeImpl<S390xMachineDeps>;

/// Support for the S390x ABI from the caller side (at a callsite).
pub type S390xABICaller = ABICallerImpl<S390xMachineDeps>;

/// ABI Register usage

fn in_int_reg(ty: Type) -> bool {
    match ty {
        types::I8 | types::I16 | types::I32 | types::I64 | types::R64 => true,
        types::B1 | types::B8 | types::B16 | types::B32 | types::B64 => true,
        _ => false,
    }
}

fn in_flt_reg(ty: Type) -> bool {
    match ty {
        types::F32 | types::F64 => true,
        _ => false,
    }
}

fn get_intreg_for_arg(idx: usize) -> Option<Reg> {
    match idx {
        0 => Some(regs::gpr(2)),
        1 => Some(regs::gpr(3)),
        2 => Some(regs::gpr(4)),
        3 => Some(regs::gpr(5)),
        4 => Some(regs::gpr(6)),
        _ => None,
    }
}

fn get_fltreg_for_arg(idx: usize) -> Option<Reg> {
    match idx {
        0 => Some(regs::fpr(0)),
        1 => Some(regs::fpr(2)),
        2 => Some(regs::fpr(4)),
        3 => Some(regs::fpr(6)),
        _ => None,
    }
}

fn get_intreg_for_ret(idx: usize) -> Option<Reg> {
    match idx {
        0 => Some(regs::gpr(2)),
        // ABI extension to support multi-value returns:
        1 => Some(regs::gpr(3)),
        2 => Some(regs::gpr(4)),
        3 => Some(regs::gpr(5)),
        _ => None,
    }
}

fn get_fltreg_for_ret(idx: usize) -> Option<Reg> {
    match idx {
        0 => Some(regs::fpr(0)),
        // ABI extension to support multi-value returns:
        1 => Some(regs::fpr(2)),
        2 => Some(regs::fpr(4)),
        3 => Some(regs::fpr(6)),
        _ => None,
    }
}

/// This is the limit for the size of argument and return-value areas on the
/// stack. We place a reasonable limit here to avoid integer overflow issues
/// with 32-bit arithmetic: for now, 128 MB.
static STACK_ARG_RET_SIZE_LIMIT: u64 = 128 * 1024 * 1024;

impl Into<MemArg> for StackAMode {
    fn into(self) -> MemArg {
        match self {
            StackAMode::FPOffset(off, _ty) => MemArg::InitialSPOffset { off },
            StackAMode::NominalSPOffset(off, _ty) => MemArg::NominalSPOffset { off },
            StackAMode::SPOffset(off, _ty) => {
                MemArg::reg_plus_off(stack_reg(), off, MemFlags::trusted())
            }
        }
    }
}

/// S390x-specific ABI behavior. This struct just serves as an implementation
/// point for the trait; it is never actually instantiated.
pub struct S390xMachineDeps;

impl ABIMachineSpec for S390xMachineDeps {
    type I = Inst;

    fn word_bits() -> u32 {
        64
    }

    /// Return required stack alignment in bytes.
    fn stack_align(_call_conv: isa::CallConv) -> u32 {
        8
    }

    fn compute_arg_locs(
        call_conv: isa::CallConv,
        _flags: &settings::Flags,
        params: &[ir::AbiParam],
        args_or_rets: ArgsOrRets,
        add_ret_area_ptr: bool,
    ) -> CodegenResult<(Vec<ABIArg>, i64, Option<usize>)> {
        let mut next_gpr = 0;
        let mut next_fpr = 0;
        let mut next_stack: u64 = 0;
        let mut ret = vec![];

        if args_or_rets == ArgsOrRets::Args {
            next_stack = 160;
        }

        for i in 0..params.len() {
            let param = &params[i];

            // Validate "purpose".
            match &param.purpose {
                &ir::ArgumentPurpose::VMContext
                | &ir::ArgumentPurpose::Normal
                | &ir::ArgumentPurpose::StackLimit
                | &ir::ArgumentPurpose::SignatureId => {}
                _ => panic!(
                    "Unsupported argument purpose {:?} in signature: {:?}",
                    param.purpose, params
                ),
            }

            let intreg = in_int_reg(param.value_type);
            let fltreg = in_flt_reg(param.value_type);
            debug_assert!(intreg || fltreg);
            debug_assert!(!(intreg && fltreg));

            let (next_reg, candidate) = if intreg {
                let candidate = match args_or_rets {
                    ArgsOrRets::Args => get_intreg_for_arg(next_gpr),
                    ArgsOrRets::Rets => get_intreg_for_ret(next_gpr),
                };
                (&mut next_gpr, candidate)
            } else {
                let candidate = match args_or_rets {
                    ArgsOrRets::Args => get_fltreg_for_arg(next_fpr),
                    ArgsOrRets::Rets => get_fltreg_for_ret(next_fpr),
                };
                (&mut next_fpr, candidate)
            };

            // In the Wasmtime ABI only the first return value can be in a register.
            let candidate =
                if call_conv.extends_wasmtime() && args_or_rets == ArgsOrRets::Rets && i > 0 {
                    None
                } else {
                    candidate
                };

            if let Some(reg) = candidate {
                ret.push(ABIArg::reg(
                    reg.to_real_reg(),
                    param.value_type,
                    param.extension,
                    param.purpose,
                ));
                *next_reg += 1;
            } else {
                // Compute size. Every argument or return value takes a slot of
                // at least 8 bytes, except for return values in the Wasmtime ABI.
                let size = (ty_bits(param.value_type) / 8) as u64;
                let slot_size = if call_conv.extends_wasmtime() && args_or_rets == ArgsOrRets::Rets
                {
                    size
                } else {
                    std::cmp::max(size, 8)
                };

                // Align the stack slot.
                debug_assert!(slot_size.is_power_of_two());
                next_stack = align_to(next_stack, slot_size);

                // If the type is actually of smaller size (and the argument
                // was not extended), it is passed right-aligned.
                let offset = if size < slot_size && param.extension == ir::ArgumentExtension::None {
                    slot_size - size
                } else {
                    0
                };
                ret.push(ABIArg::stack(
                    (next_stack + offset) as i64,
                    param.value_type,
                    param.extension,
                    param.purpose,
                ));
                next_stack += slot_size;
            }
        }

        next_stack = align_to(next_stack, 8);

        let extra_arg = if add_ret_area_ptr {
            debug_assert!(args_or_rets == ArgsOrRets::Args);
            if let Some(reg) = get_intreg_for_arg(next_gpr) {
                ret.push(ABIArg::reg(
                    reg.to_real_reg(),
                    types::I64,
                    ir::ArgumentExtension::None,
                    ir::ArgumentPurpose::Normal,
                ));
            } else {
                ret.push(ABIArg::stack(
                    next_stack as i64,
                    types::I64,
                    ir::ArgumentExtension::None,
                    ir::ArgumentPurpose::Normal,
                ));
                next_stack += 8;
            }
            Some(ret.len() - 1)
        } else {
            None
        };

        // To avoid overflow issues, limit the arg/return size to something
        // reasonable -- here, 128 MB.
        if next_stack > STACK_ARG_RET_SIZE_LIMIT {
            return Err(CodegenError::ImplLimitExceeded);
        }

        Ok((ret, next_stack as i64, extra_arg))
    }

    fn fp_to_arg_offset(_call_conv: isa::CallConv, _flags: &settings::Flags) -> i64 {
        0
    }

    fn gen_load_stack(mem: StackAMode, into_reg: Writable<Reg>, ty: Type) -> Inst {
        Inst::gen_load(into_reg, mem.into(), ty)
    }

    fn gen_store_stack(mem: StackAMode, from_reg: Reg, ty: Type) -> Inst {
        Inst::gen_store(mem.into(), from_reg, ty)
    }

    fn gen_move(to_reg: Writable<Reg>, from_reg: Reg, ty: Type) -> Inst {
        Inst::gen_move(to_reg, from_reg, ty)
    }

    fn gen_extend(
        to_reg: Writable<Reg>,
        from_reg: Reg,
        signed: bool,
        from_bits: u8,
        to_bits: u8,
    ) -> Inst {
        assert!(from_bits < to_bits);
        Inst::Extend {
            rd: to_reg,
            rn: from_reg,
            signed,
            from_bits,
            to_bits,
        }
    }

    fn gen_ret() -> Inst {
        Inst::Ret { link: gpr(14) }
    }

    fn gen_add_imm(into_reg: Writable<Reg>, from_reg: Reg, imm: u32) -> SmallInstVec<Inst> {
        let mut insts = SmallVec::new();
        if let Some(imm) = UImm12::maybe_from_u64(imm as u64) {
            insts.push(Inst::LoadAddr {
                rd: into_reg,
                mem: MemArg::BXD12 {
                    base: from_reg,
                    index: zero_reg(),
                    disp: imm,
                    flags: MemFlags::trusted(),
                },
            });
        } else if let Some(imm) = SImm20::maybe_from_i64(imm as i64) {
            insts.push(Inst::LoadAddr {
                rd: into_reg,
                mem: MemArg::BXD20 {
                    base: from_reg,
                    index: zero_reg(),
                    disp: imm,
                    flags: MemFlags::trusted(),
                },
            });
        } else {
            if from_reg != into_reg.to_reg() {
                insts.push(Inst::mov64(into_reg, from_reg));
            }
            insts.push(Inst::AluRUImm32 {
                alu_op: ALUOp::AddLogical64,
                rd: into_reg,
                imm,
            });
        }
        insts
    }

    fn gen_stack_lower_bound_trap(limit_reg: Reg) -> SmallInstVec<Inst> {
        let mut insts = SmallVec::new();
        insts.push(Inst::CmpTrapRR {
            op: CmpOp::CmpL64,
            rn: stack_reg(),
            rm: limit_reg,
            cond: Cond::from_intcc(IntCC::UnsignedLessThanOrEqual),
            trap_code: ir::TrapCode::StackOverflow,
        });
        insts
    }

    fn gen_epilogue_placeholder() -> Inst {
        Inst::EpiloguePlaceholder
    }

    fn gen_get_stack_addr(mem: StackAMode, into_reg: Writable<Reg>, _ty: Type) -> Inst {
        let mem = mem.into();
        Inst::LoadAddr { rd: into_reg, mem }
    }

    fn get_stacklimit_reg() -> Reg {
        spilltmp_reg()
    }

    fn gen_load_base_offset(into_reg: Writable<Reg>, base: Reg, offset: i32, ty: Type) -> Inst {
        let mem = MemArg::reg_plus_off(base, offset.into(), MemFlags::trusted());
        Inst::gen_load(into_reg, mem, ty)
    }

    fn gen_store_base_offset(base: Reg, offset: i32, from_reg: Reg, ty: Type) -> Inst {
        let mem = MemArg::reg_plus_off(base, offset.into(), MemFlags::trusted());
        Inst::gen_store(mem, from_reg, ty)
    }

    fn gen_sp_reg_adjust(imm: i32) -> SmallInstVec<Inst> {
        if imm == 0 {
            return SmallVec::new();
        }

        let mut insts = SmallVec::new();
        if let Ok(imm) = i16::try_from(imm) {
            insts.push(Inst::AluRSImm16 {
                alu_op: ALUOp::Add64,
                rd: writable_stack_reg(),
                imm,
            });
        } else {
            insts.push(Inst::AluRSImm32 {
                alu_op: ALUOp::Add64,
                rd: writable_stack_reg(),
                imm,
            });
        }
        insts
    }

    fn gen_nominal_sp_adj(offset: i32) -> Inst {
        Inst::VirtualSPOffsetAdj {
            offset: offset.into(),
        }
    }

    fn gen_prologue_frame_setup(_flags: &settings::Flags) -> SmallInstVec<Inst> {
        SmallVec::new()
    }

    fn gen_epilogue_frame_restore(_flags: &settings::Flags) -> SmallInstVec<Inst> {
        SmallVec::new()
    }

    fn gen_probestack(_: u32) -> SmallInstVec<Self::I> {
        // TODO: implement if we ever require stack probes on an s390x host
        // (unlikely unless Lucet is ported)
        smallvec![]
    }

    // Returns stack bytes used as well as instructions. Does not adjust
    // nominal SP offset; abi_impl generic code will do that.
    fn gen_clobber_save(
        _call_conv: isa::CallConv,
        _setup_frame: bool,
        flags: &settings::Flags,
        clobbered_callee_saves: &Vec<Writable<RealReg>>,
        fixed_frame_storage_size: u32,
        outgoing_args_size: u32,
    ) -> (u64, SmallVec<[Inst; 16]>) {
        let mut insts = SmallVec::new();
        let mut clobbered_fpr = vec![];
        let mut clobbered_gpr = vec![];

        for &reg in clobbered_callee_saves.iter() {
            match reg.to_reg().get_class() {
                RegClass::I64 => clobbered_gpr.push(reg),
                RegClass::F64 => clobbered_fpr.push(reg),
                class => panic!("Unexpected RegClass: {:?}", class),
            }
        }

        let mut first_clobbered_gpr = 16;
        for reg in clobbered_gpr {
            let enc = reg.to_reg().get_hw_encoding();
            if enc < first_clobbered_gpr {
                first_clobbered_gpr = enc;
            }
        }
        let clobber_size = clobbered_fpr.len() * 8;
        if flags.unwind_info() {
            insts.push(Inst::Unwind {
                inst: UnwindInst::DefineNewFrame {
                    offset_upward_to_caller_sp: 160,
                    offset_downward_to_clobbers: clobber_size as u32,
                },
            });
        }

        // Use STMG to save clobbered GPRs into save area.
        if first_clobbered_gpr < 16 {
            let offset = 8 * first_clobbered_gpr as i64;
            insts.push(Inst::StoreMultiple64 {
                rt: gpr(first_clobbered_gpr as u8),
                rt2: gpr(15),
                mem: MemArg::reg_plus_off(stack_reg(), offset, MemFlags::trusted()),
            });
        }
        if flags.unwind_info() {
            for i in first_clobbered_gpr..16 {
                insts.push(Inst::Unwind {
                    inst: UnwindInst::SaveReg {
                        clobber_offset: clobber_size as u32 + (i * 8) as u32,
                        reg: gpr(i as u8).to_real_reg(),
                    },
                });
            }
        }

        // Decrement stack pointer.
        let stack_size =
            outgoing_args_size as i32 + clobber_size as i32 + fixed_frame_storage_size as i32;
        insts.extend(Self::gen_sp_reg_adjust(-stack_size));
        if flags.unwind_info() {
            insts.push(Inst::Unwind {
                inst: UnwindInst::StackAlloc {
                    size: stack_size as u32,
                },
            });
        }

        let sp_adj = outgoing_args_size as i32;
        if sp_adj > 0 {
            insts.push(Self::gen_nominal_sp_adj(sp_adj));
        }

        // Save FPRs.
        for (i, reg) in clobbered_fpr.iter().enumerate() {
            insts.push(Inst::FpuStore64 {
                rd: reg.to_reg().to_reg(),
                mem: MemArg::reg_plus_off(
                    stack_reg(),
                    (i * 8) as i64 + outgoing_args_size as i64 + fixed_frame_storage_size as i64,
                    MemFlags::trusted(),
                ),
            });
            if flags.unwind_info() {
                insts.push(Inst::Unwind {
                    inst: UnwindInst::SaveReg {
                        clobber_offset: (i * 8) as u32,
                        reg: reg.to_reg(),
                    },
                });
            }
        }

        (clobber_size as u64, insts)
    }

    fn gen_clobber_restore(
        call_conv: isa::CallConv,
        _: &settings::Flags,
        clobbers: &Set<Writable<RealReg>>,
        fixed_frame_storage_size: u32,
        outgoing_args_size: u32,
    ) -> SmallVec<[Inst; 16]> {
        let mut insts = SmallVec::new();

        // Collect clobbered registers.
        let (clobbered_gpr, clobbered_fpr) = get_regs_saved_in_prologue(call_conv, clobbers);
        let mut first_clobbered_gpr = 16;
        for reg in clobbered_gpr {
            let enc = reg.to_reg().get_hw_encoding();
            if enc < first_clobbered_gpr {
                first_clobbered_gpr = enc;
            }
        }
        let clobber_size = clobbered_fpr.len() * 8;

        // Restore FPRs.
        for (i, reg) in clobbered_fpr.iter().enumerate() {
            insts.push(Inst::FpuLoad64 {
                rd: Writable::from_reg(reg.to_reg().to_reg()),
                mem: MemArg::reg_plus_off(
                    stack_reg(),
                    (i * 8) as i64 + outgoing_args_size as i64 + fixed_frame_storage_size as i64,
                    MemFlags::trusted(),
                ),
            });
        }

        // Increment stack pointer unless it will be restored implicitly.
        let stack_size =
            outgoing_args_size as i32 + clobber_size as i32 + fixed_frame_storage_size as i32;
        let implicit_sp_restore = first_clobbered_gpr < 16
            && SImm20::maybe_from_i64(8 * first_clobbered_gpr as i64 + stack_size as i64).is_some();
        if !implicit_sp_restore {
            insts.extend(Self::gen_sp_reg_adjust(stack_size));
        }

        // Use LMG to restore clobbered GPRs from save area.
        if first_clobbered_gpr < 16 {
            let mut offset = 8 * first_clobbered_gpr as i64;
            if implicit_sp_restore {
                offset += stack_size as i64;
            }
            insts.push(Inst::LoadMultiple64 {
                rt: writable_gpr(first_clobbered_gpr as u8),
                rt2: writable_gpr(15),
                mem: MemArg::reg_plus_off(stack_reg(), offset, MemFlags::trusted()),
            });
        }

        insts
    }

    fn gen_call(
        dest: &CallDest,
        uses: Vec<Reg>,
        defs: Vec<Writable<Reg>>,
        opcode: ir::Opcode,
        tmp: Writable<Reg>,
        _callee_conv: isa::CallConv,
        _caller_conv: isa::CallConv,
    ) -> SmallVec<[(InstIsSafepoint, Inst); 2]> {
        let mut insts = SmallVec::new();
        match &dest {
            &CallDest::ExtName(ref name, RelocDistance::Near) => insts.push((
                InstIsSafepoint::Yes,
                Inst::Call {
                    link: writable_gpr(14),
                    info: Box::new(CallInfo {
                        dest: name.clone(),
                        uses,
                        defs,
                        opcode,
                    }),
                },
            )),
            &CallDest::ExtName(ref name, RelocDistance::Far) => {
                insts.push((
                    InstIsSafepoint::No,
                    Inst::LoadExtNameFar {
                        rd: tmp,
                        name: Box::new(name.clone()),
                        offset: 0,
                    },
                ));
                insts.push((
                    InstIsSafepoint::Yes,
                    Inst::CallInd {
                        link: writable_gpr(14),
                        info: Box::new(CallIndInfo {
                            rn: tmp.to_reg(),
                            uses,
                            defs,
                            opcode,
                        }),
                    },
                ));
            }
            &CallDest::Reg(reg) => insts.push((
                InstIsSafepoint::Yes,
                Inst::CallInd {
                    link: writable_gpr(14),
                    info: Box::new(CallIndInfo {
                        rn: *reg,
                        uses,
                        defs,
                        opcode,
                    }),
                },
            )),
        }

        insts
    }

    fn gen_memcpy(
        _call_conv: isa::CallConv,
        _dst: Reg,
        _src: Reg,
        _size: usize,
    ) -> SmallVec<[Self::I; 8]> {
        unimplemented!("StructArgs not implemented for S390X yet");
    }

    fn get_number_of_spillslots_for_value(rc: RegClass) -> u32 {
        // We allocate in terms of 8-byte slots.
        match rc {
            RegClass::I64 => 1,
            RegClass::F64 => 1,
            _ => panic!("Unexpected register class!"),
        }
    }

    /// Get the current virtual-SP offset from an instruction-emission state.
    fn get_virtual_sp_offset_from_state(s: &EmitState) -> i64 {
        s.virtual_sp_offset
    }

    /// Get the nominal-SP-to-FP offset from an instruction-emission state.
    fn get_nominal_sp_to_fp(s: &EmitState) -> i64 {
        s.initial_sp_offset
    }

    fn get_regs_clobbered_by_call(call_conv_of_callee: isa::CallConv) -> Vec<Writable<Reg>> {
        let mut caller_saved = Vec::new();
        for i in 0..15 {
            let x = writable_gpr(i);
            if is_reg_clobbered_by_call(call_conv_of_callee, x.to_reg().to_real_reg()) {
                caller_saved.push(x);
            }
        }
        for i in 0..15 {
            let v = writable_fpr(i);
            if is_reg_clobbered_by_call(call_conv_of_callee, v.to_reg().to_real_reg()) {
                caller_saved.push(v);
            }
        }
        caller_saved
    }

    fn get_ext_mode(
        _call_conv: isa::CallConv,
        specified: ir::ArgumentExtension,
    ) -> ir::ArgumentExtension {
        specified
    }

    fn get_clobbered_callee_saves(
        call_conv: isa::CallConv,
        regs: &Set<Writable<RealReg>>,
    ) -> Vec<Writable<RealReg>> {
        let mut regs: Vec<Writable<RealReg>> = regs
            .iter()
            .cloned()
            .filter(|r| is_reg_saved_in_prologue(call_conv, r.to_reg()))
            .collect();

        // Sort registers for deterministic code output. We can do an unstable
        // sort because the registers will be unique (there are no dups).
        regs.sort_unstable_by_key(|r| r.to_reg().get_index());
        regs
    }

    fn is_frame_setup_needed(
        _is_leaf: bool,
        _stack_args_size: u32,
        _num_clobbered_callee_saves: usize,
        _fixed_frame_storage_size: u32,
    ) -> bool {
        // The call frame set-up is handled by gen_clobber_save().
        false
    }
}

fn is_reg_saved_in_prologue(_call_conv: isa::CallConv, r: RealReg) -> bool {
    match r.get_class() {
        RegClass::I64 => {
            // r6 - r15 inclusive are callee-saves.
            r.get_hw_encoding() >= 6 && r.get_hw_encoding() <= 15
        }
        RegClass::F64 => {
            // f8 - f15 inclusive are callee-saves.
            r.get_hw_encoding() >= 8 && r.get_hw_encoding() <= 15
        }
        _ => panic!("Unexpected RegClass"),
    }
}

fn get_regs_saved_in_prologue(
    call_conv: isa::CallConv,
    regs: &Set<Writable<RealReg>>,
) -> (Vec<Writable<RealReg>>, Vec<Writable<RealReg>>) {
    let mut int_saves = vec![];
    let mut fpr_saves = vec![];
    for &reg in regs.iter() {
        if is_reg_saved_in_prologue(call_conv, reg.to_reg()) {
            match reg.to_reg().get_class() {
                RegClass::I64 => int_saves.push(reg),
                RegClass::F64 => fpr_saves.push(reg),
                _ => panic!("Unexpected RegClass"),
            }
        }
    }
    // Sort registers for deterministic code output.
    int_saves.sort_by_key(|r| r.to_reg().get_index());
    fpr_saves.sort_by_key(|r| r.to_reg().get_index());
    (int_saves, fpr_saves)
}

fn is_reg_clobbered_by_call(_call_conv: isa::CallConv, r: RealReg) -> bool {
    match r.get_class() {
        RegClass::I64 => {
            // r0 - r5 inclusive are caller-saves.
            r.get_hw_encoding() <= 5
        }
        RegClass::F64 => {
            // f0 - f7 inclusive are caller-saves.
            r.get_hw_encoding() <= 7
        }
        _ => panic!("Unexpected RegClass"),
    }
}
