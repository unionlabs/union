//! Implementation of a standard S390x ABI.
//!
//! This machine uses the "vanilla" ABI implementation from abi.rs,
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
//! - If we are asked to "preserve frame pointers" to enable stack
//!   unwinding, we use the stack backchain feature instead, which
//!   is documented by the s390x ELF ABI, but marked as optional.
//!   This ensures that at all times during execution of a function,
//!   the lowest word on the stack (part of the register save area)
//!   holds a copy of the stack pointer at function entry.
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
//!                              | (used to save GPRs)       |
//! SP at function entry ----->  | (incl. caller's backchain)|
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
//!                              | (alloc'd by prologue)     |
//! SP during function  ------>  | (incl. callee's backchain)|
//!                              +---------------------------+
//!
//!   (low address)
//! ```

use crate::ir;
use crate::ir::condcodes::IntCC;
use crate::ir::types;
use crate::ir::MemFlags;
use crate::ir::Signature;
use crate::ir::Type;
use crate::isa;
use crate::isa::s390x::{inst::*, settings as s390x_settings};
use crate::isa::unwind::UnwindInst;
use crate::machinst::*;
use crate::machinst::{RealReg, Reg, RegClass, Writable};
use crate::settings;
use crate::{CodegenError, CodegenResult};
use alloc::vec::Vec;
use regalloc2::{PReg, PRegSet};
use smallvec::{smallvec, SmallVec};
use std::convert::TryFrom;

// We use a generic implementation that factors out ABI commonalities.

/// Support for the S390x ABI from the callee side (within a function body).
pub type S390xCallee = Callee<S390xMachineDeps>;

/// ABI Register usage

fn in_int_reg(ty: Type) -> bool {
    match ty {
        types::I8 | types::I16 | types::I32 | types::I64 | types::R64 => true,
        _ => false,
    }
}

fn in_flt_reg(ty: Type) -> bool {
    match ty {
        types::F32 | types::F64 => true,
        _ => false,
    }
}

fn in_vec_reg(ty: Type) -> bool {
    ty.is_vector() && ty.bits() == 128
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
        0 => Some(regs::vr(0)),
        1 => Some(regs::vr(2)),
        2 => Some(regs::vr(4)),
        3 => Some(regs::vr(6)),
        _ => None,
    }
}

fn get_vecreg_for_arg(idx: usize) -> Option<Reg> {
    match idx {
        0 => Some(regs::vr(24)),
        1 => Some(regs::vr(25)),
        2 => Some(regs::vr(26)),
        3 => Some(regs::vr(27)),
        4 => Some(regs::vr(28)),
        5 => Some(regs::vr(29)),
        6 => Some(regs::vr(30)),
        7 => Some(regs::vr(31)),
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
        0 => Some(regs::vr(0)),
        // ABI extension to support multi-value returns:
        1 => Some(regs::vr(2)),
        2 => Some(regs::vr(4)),
        3 => Some(regs::vr(6)),
        _ => None,
    }
}

fn get_vecreg_for_ret(idx: usize) -> Option<Reg> {
    match idx {
        0 => Some(regs::vr(24)),
        // ABI extension to support multi-value returns:
        1 => Some(regs::vr(25)),
        2 => Some(regs::vr(26)),
        3 => Some(regs::vr(27)),
        4 => Some(regs::vr(28)),
        5 => Some(regs::vr(29)),
        6 => Some(regs::vr(30)),
        7 => Some(regs::vr(31)),
        _ => None,
    }
}

/// This is the limit for the size of argument and return-value areas on the
/// stack. We place a reasonable limit here to avoid integer overflow issues
/// with 32-bit arithmetic: for now, 128 MB.
static STACK_ARG_RET_SIZE_LIMIT: u64 = 128 * 1024 * 1024;

/// The size of the register save area
pub static REG_SAVE_AREA_SIZE: u32 = 160;

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

impl IsaFlags for s390x_settings::Flags {}

impl ABIMachineSpec for S390xMachineDeps {
    type I = Inst;

    type F = s390x_settings::Flags;

    fn word_bits() -> u32 {
        64
    }

    /// Return required stack alignment in bytes.
    fn stack_align(_call_conv: isa::CallConv) -> u32 {
        8
    }

    fn compute_arg_locs<'a, I>(
        call_conv: isa::CallConv,
        _flags: &settings::Flags,
        params: I,
        args_or_rets: ArgsOrRets,
        add_ret_area_ptr: bool,
        mut args: ArgsAccumulator<'_>,
    ) -> CodegenResult<(i64, Option<usize>)>
    where
        I: IntoIterator<Item = &'a ir::AbiParam>,
    {
        let mut next_gpr = 0;
        let mut next_fpr = 0;
        let mut next_vr = 0;
        let mut next_stack: u64 = 0;

        if args_or_rets == ArgsOrRets::Args {
            next_stack = REG_SAVE_AREA_SIZE as u64;
        }

        // In the SystemV ABI, the return area pointer is the first argument,
        // so we need to leave room for it if required.  (In the Wasmtime ABI,
        // the return area pointer is the last argument and is handled below.)
        if add_ret_area_ptr && !call_conv.extends_wasmtime() {
            next_gpr += 1;
        }

        for (i, mut param) in params.into_iter().copied().enumerate() {
            let intreg = in_int_reg(param.value_type);
            let fltreg = in_flt_reg(param.value_type);
            let vecreg = in_vec_reg(param.value_type);
            debug_assert!(intreg as i32 + fltreg as i32 + vecreg as i32 <= 1);

            let (next_reg, candidate, implicit_ref) = if intreg {
                let candidate = match args_or_rets {
                    ArgsOrRets::Args => get_intreg_for_arg(next_gpr),
                    ArgsOrRets::Rets => get_intreg_for_ret(next_gpr),
                };
                (&mut next_gpr, candidate, None)
            } else if fltreg {
                let candidate = match args_or_rets {
                    ArgsOrRets::Args => get_fltreg_for_arg(next_fpr),
                    ArgsOrRets::Rets => get_fltreg_for_ret(next_fpr),
                };
                (&mut next_fpr, candidate, None)
            } else if vecreg {
                let candidate = match args_or_rets {
                    ArgsOrRets::Args => get_vecreg_for_arg(next_vr),
                    ArgsOrRets::Rets => get_vecreg_for_ret(next_vr),
                };
                (&mut next_vr, candidate, None)
            } else if call_conv.extends_wasmtime() {
                panic!("i128 args/return values not supported in the Wasmtime ABI");
            } else {
                assert!(param.extension == ir::ArgumentExtension::None);
                // We must pass this by implicit reference.
                if args_or_rets == ArgsOrRets::Rets {
                    // For return values, just force them to memory.
                    (&mut next_gpr, None, None)
                } else {
                    // For arguments, implicitly convert to pointer type.
                    let implicit_ref = Some(param.value_type);
                    param = ir::AbiParam::new(types::I64);
                    let candidate = get_intreg_for_arg(next_gpr);
                    (&mut next_gpr, candidate, implicit_ref)
                }
            };

            // In the Wasmtime ABI only the first return value can be in a register.
            let candidate =
                if call_conv.extends_wasmtime() && args_or_rets == ArgsOrRets::Rets && i > 0 {
                    None
                } else {
                    candidate
                };

            let slot = if let Some(reg) = candidate {
                *next_reg += 1;
                ABIArgSlot::Reg {
                    reg: reg.to_real_reg().unwrap(),
                    ty: param.value_type,
                    extension: param.extension,
                }
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
                let slot_align = std::cmp::min(slot_size, 8);
                next_stack = align_to(next_stack, slot_align);

                // If the type is actually of smaller size (and the argument
                // was not extended), it is passed right-aligned.
                let offset = if size < slot_size && param.extension == ir::ArgumentExtension::None {
                    slot_size - size
                } else {
                    0
                };
                let offset = (next_stack + offset) as i64;
                next_stack += slot_size;
                ABIArgSlot::Stack {
                    offset,
                    ty: param.value_type,
                    extension: param.extension,
                }
            };

            if let ir::ArgumentPurpose::StructArgument(size) = param.purpose {
                assert!(size % 8 == 0, "StructArgument size is not properly aligned");
                args.push(ABIArg::StructArg {
                    pointer: Some(slot),
                    offset: 0,
                    size: size as u64,
                    purpose: param.purpose,
                });
            } else if let Some(ty) = implicit_ref {
                assert!(
                    (ty_bits(ty) / 8) % 8 == 0,
                    "implicit argument size is not properly aligned"
                );
                args.push(ABIArg::ImplicitPtrArg {
                    pointer: slot,
                    offset: 0,
                    ty,
                    purpose: param.purpose,
                });
            } else {
                args.push(ABIArg::Slots {
                    slots: smallvec![slot],
                    purpose: param.purpose,
                });
            }
        }

        next_stack = align_to(next_stack, 8);

        let extra_arg = if add_ret_area_ptr {
            debug_assert!(args_or_rets == ArgsOrRets::Args);
            // The return pointer is passed either as first argument
            // (in the SystemV ABI) or as last argument (Wasmtime ABI).
            let next_gpr = if call_conv.extends_wasmtime() {
                next_gpr
            } else {
                0
            };
            if let Some(reg) = get_intreg_for_arg(next_gpr) {
                args.push(ABIArg::reg(
                    reg.to_real_reg().unwrap(),
                    types::I64,
                    ir::ArgumentExtension::None,
                    ir::ArgumentPurpose::Normal,
                ));
            } else {
                args.push(ABIArg::stack(
                    next_stack as i64,
                    types::I64,
                    ir::ArgumentExtension::None,
                    ir::ArgumentPurpose::Normal,
                ));
                next_stack += 8;
            }
            Some(args.args().len() - 1)
        } else {
            None
        };

        // After all arguments are in their well-defined location,
        // allocate buffers for all StructArg or ImplicitPtrArg arguments.
        for arg in args.args_mut() {
            match arg {
                ABIArg::StructArg { offset, size, .. } => {
                    *offset = next_stack as i64;
                    next_stack += *size;
                }
                ABIArg::ImplicitPtrArg { offset, ty, .. } => {
                    *offset = next_stack as i64;
                    next_stack += (ty_bits(*ty) / 8) as u64;
                }
                _ => {}
            }
        }

        // To avoid overflow issues, limit the arg/return size to something
        // reasonable -- here, 128 MB.
        if next_stack > STACK_ARG_RET_SIZE_LIMIT {
            return Err(CodegenError::ImplLimitExceeded);
        }

        Ok((next_stack as i64, extra_arg))
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

    fn gen_args(_isa_flags: &s390x_settings::Flags, args: Vec<ArgPair>) -> Inst {
        Inst::Args { args }
    }

    fn gen_ret(_setup_frame: bool, _isa_flags: &s390x_settings::Flags, rets: Vec<RetPair>) -> Inst {
        Inst::Ret {
            link: gpr(14),
            rets,
        }
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
                ri: into_reg.to_reg(),
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
                ri: stack_reg(),
                imm,
            });
        } else {
            insts.push(Inst::AluRSImm32 {
                alu_op: ALUOp::Add64,
                rd: writable_stack_reg(),
                ri: stack_reg(),
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

    fn gen_probestack(_insts: &mut SmallInstVec<Self::I>, _: u32) {
        // TODO: implement if we ever require stack probes on an s390x host
        // (unlikely unless Lucet is ported)
        unimplemented!("Stack probing is unimplemented on S390x");
    }

    fn gen_inline_probestack(
        _insts: &mut SmallInstVec<Self::I>,
        _frame_size: u32,
        _guard_size: u32,
    ) {
        unimplemented!("Inline stack probing is unimplemented on S390x");
    }

    // Returns stack bytes used as well as instructions. Does not adjust
    // nominal SP offset; abi generic code will do that.
    fn gen_clobber_save(
        _call_conv: isa::CallConv,
        _setup_frame: bool,
        flags: &settings::Flags,
        clobbered_callee_saves: &[Writable<RealReg>],
        fixed_frame_storage_size: u32,
        mut outgoing_args_size: u32,
    ) -> (u64, SmallVec<[Inst; 16]>) {
        let mut insts = SmallVec::new();

        // Collect clobbered registers.
        let (first_clobbered_gpr, clobbered_fpr) =
            get_clobbered_gpr_fpr(flags, clobbered_callee_saves, &mut outgoing_args_size);
        let clobber_size = clobbered_fpr.len() * 8;
        if flags.unwind_info() {
            insts.push(Inst::Unwind {
                inst: UnwindInst::DefineNewFrame {
                    offset_upward_to_caller_sp: REG_SAVE_AREA_SIZE,
                    offset_downward_to_clobbers: clobber_size as u32,
                },
            });
        }

        // Use STMG to save clobbered GPRs into save area.
        if first_clobbered_gpr < 16 {
            let offset = 8 * first_clobbered_gpr as i64;
            insts.push(Inst::StoreMultiple64 {
                rt: gpr(first_clobbered_gpr),
                rt2: gpr(15),
                mem: MemArg::reg_plus_off(stack_reg(), offset, MemFlags::trusted()),
            });
        }
        if flags.unwind_info() {
            for i in first_clobbered_gpr..16 {
                insts.push(Inst::Unwind {
                    inst: UnwindInst::SaveReg {
                        clobber_offset: clobber_size as u32 + (i * 8) as u32,
                        reg: gpr(i).to_real_reg().unwrap(),
                    },
                });
            }
        }

        // Save current stack pointer value if we need to write the backchain.
        if flags.preserve_frame_pointers() {
            insts.push(Inst::mov64(writable_gpr(1), stack_reg()));
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

        // Write the stack backchain if requested, using the value saved above.
        if flags.preserve_frame_pointers() {
            insts.push(Inst::Store64 {
                rd: gpr(1),
                mem: MemArg::reg_plus_off(stack_reg(), 0, MemFlags::trusted()),
            });
        }

        // Save FPRs.
        for (i, reg) in clobbered_fpr.iter().enumerate() {
            insts.push(Inst::VecStoreLane {
                size: 64,
                rd: reg.to_reg().into(),
                mem: MemArg::reg_plus_off(
                    stack_reg(),
                    (i * 8) as i64 + outgoing_args_size as i64 + fixed_frame_storage_size as i64,
                    MemFlags::trusted(),
                ),
                lane_imm: 0,
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
        sig: &Signature,
        flags: &settings::Flags,
        clobbers: &[Writable<RealReg>],
        fixed_frame_storage_size: u32,
        mut outgoing_args_size: u32,
    ) -> SmallVec<[Inst; 16]> {
        let mut insts = SmallVec::new();
        let clobbered_callee_saves =
            Self::get_clobbered_callee_saves(call_conv, flags, sig, clobbers);

        // Collect clobbered registers.
        let (first_clobbered_gpr, clobbered_fpr) =
            get_clobbered_gpr_fpr(flags, &clobbered_callee_saves, &mut outgoing_args_size);
        let clobber_size = clobbered_fpr.len() * 8;

        // Restore FPRs.
        for (i, reg) in clobbered_fpr.iter().enumerate() {
            insts.push(Inst::VecLoadLaneUndef {
                size: 64,
                rd: Writable::from_reg(reg.to_reg().into()),
                mem: MemArg::reg_plus_off(
                    stack_reg(),
                    (i * 8) as i64 + outgoing_args_size as i64 + fixed_frame_storage_size as i64,
                    MemFlags::trusted(),
                ),
                lane_imm: 0,
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
                rt: writable_gpr(first_clobbered_gpr),
                rt2: writable_gpr(15),
                mem: MemArg::reg_plus_off(stack_reg(), offset, MemFlags::trusted()),
            });
        }

        insts
    }

    fn gen_call(
        _dest: &CallDest,
        _uses: CallArgList,
        _defs: CallRetList,
        _clobbers: PRegSet,
        _opcode: ir::Opcode,
        _tmp: Writable<Reg>,
        _callee_conv: isa::CallConv,
        _caller_conv: isa::CallConv,
    ) -> SmallVec<[Inst; 2]> {
        unreachable!();
    }

    fn gen_memcpy<F: FnMut(Type) -> Writable<Reg>>(
        _call_conv: isa::CallConv,
        _dst: Reg,
        _src: Reg,
        _size: usize,
        _alloc: F,
    ) -> SmallVec<[Self::I; 8]> {
        unimplemented!("StructArgs not implemented for S390X yet");
    }

    fn get_number_of_spillslots_for_value(rc: RegClass, _vector_scale: u32) -> u32 {
        // We allocate in terms of 8-byte slots.
        match rc {
            RegClass::Int => 1,
            RegClass::Float => 2,
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

    fn get_regs_clobbered_by_call(_call_conv_of_callee: isa::CallConv) -> PRegSet {
        CLOBBERS
    }

    fn get_ext_mode(
        _call_conv: isa::CallConv,
        specified: ir::ArgumentExtension,
    ) -> ir::ArgumentExtension {
        specified
    }

    fn get_clobbered_callee_saves(
        call_conv: isa::CallConv,
        flags: &settings::Flags,
        _sig: &Signature,
        regs: &[Writable<RealReg>],
    ) -> Vec<Writable<RealReg>> {
        assert!(
            !flags.enable_pinned_reg(),
            "Pinned register not supported on s390x"
        );

        let mut regs: Vec<Writable<RealReg>> = regs
            .iter()
            .cloned()
            .filter(|r| is_reg_saved_in_prologue(call_conv, r.to_reg()))
            .collect();

        // Sort registers for deterministic code output. We can do an unstable
        // sort because the registers will be unique (there are no dups).
        regs.sort_unstable_by_key(|r| PReg::from(r.to_reg()).index());
        regs
    }

    fn is_frame_setup_needed(
        _is_leaf: bool,
        _stack_args_size: u32,
        _num_clobbered_callee_saves: usize,
        _frame_storage_size: u32,
    ) -> bool {
        // The call frame set-up is handled by gen_clobber_save().
        false
    }
}

fn is_reg_saved_in_prologue(_call_conv: isa::CallConv, r: RealReg) -> bool {
    match r.class() {
        RegClass::Int => {
            // r6 - r15 inclusive are callee-saves.
            r.hw_enc() >= 6 && r.hw_enc() <= 15
        }
        RegClass::Float => {
            // f8 - f15 inclusive are callee-saves.
            r.hw_enc() >= 8 && r.hw_enc() <= 15
        }
    }
}

fn get_clobbered_gpr_fpr(
    flags: &settings::Flags,
    clobbered_callee_saves: &[Writable<RealReg>],
    outgoing_args_size: &mut u32,
) -> (u8, SmallVec<[Writable<RealReg>; 8]>) {
    // Collect clobbered registers.  Note we save/restore GPR always as
    // a block of registers using LOAD MULTIPLE / STORE MULTIPLE, starting
    // with the clobbered GPR with the lowest number up to %r15.  We
    // return the number of that first GPR (or 16 if none is to be saved).
    let mut clobbered_fpr = SmallVec::new();
    let mut first_clobbered_gpr = 16;

    // If the front end asks to preserve frame pointers (which we do not
    // really have in the s390x ABI), we use the stack backchain instead.
    // For this to work in all cases, we must allocate a stack frame with
    // at least the outgoing register save area even in leaf functions.
    // Update out caller's outgoing_args_size to reflect this.
    if flags.preserve_frame_pointers() {
        if *outgoing_args_size < REG_SAVE_AREA_SIZE {
            *outgoing_args_size = REG_SAVE_AREA_SIZE;
        }
    }

    // We need to save/restore the link register in non-leaf functions.
    // This is not included in the clobber list because we have excluded
    // call instructions via the is_included_in_clobbers callback.
    // We also want to enforce saving the link register in leaf functions
    // for stack unwinding, if we're asked to preserve frame pointers.
    if *outgoing_args_size > 0 {
        first_clobbered_gpr = 14;
    }

    for &reg in clobbered_callee_saves.iter() {
        match reg.to_reg().class() {
            RegClass::Int => {
                let enc = reg.to_reg().hw_enc();
                if enc < first_clobbered_gpr {
                    first_clobbered_gpr = enc;
                }
            }
            RegClass::Float => clobbered_fpr.push(reg),
        }
    }

    (first_clobbered_gpr, clobbered_fpr)
}

const fn clobbers() -> PRegSet {
    PRegSet::empty()
        .with(gpr_preg(0))
        .with(gpr_preg(1))
        .with(gpr_preg(2))
        .with(gpr_preg(3))
        .with(gpr_preg(4))
        .with(gpr_preg(5))
        // v0 - v7 inclusive and v16 - v31 inclusive are
        // caller-saves. The upper 64 bits of v8 - v15 inclusive are
        // also caller-saves.  However, because we cannot currently
        // represent partial registers to regalloc2, we indicate here
        // that every vector register is caller-save. Because this
        // function is used at *callsites*, approximating in this
        // direction (save more than necessary) is conservative and
        // thus safe.
        //
        // Note that we exclude clobbers from a call instruction when
        // a call instruction's callee has the same ABI as the caller
        // (the current function body); this is safe (anything
        // clobbered by callee can be clobbered by caller as well) and
        // avoids unnecessary saves of v8-v15 in the prologue even
        // though we include them as defs here.
        .with(vr_preg(0))
        .with(vr_preg(1))
        .with(vr_preg(2))
        .with(vr_preg(3))
        .with(vr_preg(4))
        .with(vr_preg(5))
        .with(vr_preg(6))
        .with(vr_preg(7))
        .with(vr_preg(8))
        .with(vr_preg(9))
        .with(vr_preg(10))
        .with(vr_preg(11))
        .with(vr_preg(12))
        .with(vr_preg(13))
        .with(vr_preg(14))
        .with(vr_preg(15))
        .with(vr_preg(16))
        .with(vr_preg(17))
        .with(vr_preg(18))
        .with(vr_preg(19))
        .with(vr_preg(20))
        .with(vr_preg(21))
        .with(vr_preg(22))
        .with(vr_preg(23))
        .with(vr_preg(24))
        .with(vr_preg(25))
        .with(vr_preg(26))
        .with(vr_preg(27))
        .with(vr_preg(28))
        .with(vr_preg(29))
        .with(vr_preg(30))
        .with(vr_preg(31))
}

const CLOBBERS: PRegSet = clobbers();
