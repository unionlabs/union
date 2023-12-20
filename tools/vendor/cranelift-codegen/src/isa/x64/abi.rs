//! Implementation of the standard x64 ABI.

use crate::ir::{self, types, LibCall, MemFlags, Opcode, Signature, TrapCode, Type};
use crate::ir::{types::*, ExternalName};
use crate::isa;
use crate::isa::{unwind::UnwindInst, x64::inst::*, x64::settings as x64_settings, CallConv};
use crate::machinst::abi::*;
use crate::machinst::*;
use crate::settings;
use crate::{CodegenError, CodegenResult};
use alloc::boxed::Box;
use alloc::vec::Vec;
use args::*;
use regalloc2::{PRegSet, VReg};
use smallvec::{smallvec, SmallVec};
use std::convert::TryFrom;

/// This is the limit for the size of argument and return-value areas on the
/// stack. We place a reasonable limit here to avoid integer overflow issues
/// with 32-bit arithmetic: for now, 128 MB.
static STACK_ARG_RET_SIZE_LIMIT: u64 = 128 * 1024 * 1024;

/// Support for the x64 ABI from the callee side (within a function body).
pub(crate) type X64Callee = Callee<X64ABIMachineSpec>;

/// Support for the x64 ABI from the caller side (at a callsite).
pub(crate) type X64Caller = Caller<X64ABIMachineSpec>;

/// Implementation of ABI primitives for x64.
pub struct X64ABIMachineSpec;

impl X64ABIMachineSpec {
    fn gen_probestack_unroll(insts: &mut SmallInstVec<Inst>, guard_size: u32, probe_count: u32) {
        insts.reserve(probe_count as usize);
        for i in 0..probe_count {
            let offset = (guard_size * (i + 1)) as i64;

            // TODO: It would be nice if we could store the imm 0, but we don't have insts for those
            // so store the stack pointer. Any register will do, since the stack is undefined at this point
            insts.push(Self::gen_store_stack(
                StackAMode::SPOffset(-offset, I8),
                regs::rsp(),
                I32,
            ));
        }
    }
    fn gen_probestack_loop(insts: &mut SmallInstVec<Inst>, frame_size: u32, guard_size: u32) {
        // We have to use a caller saved register since clobbering only happens
        // after stack probing.
        //
        // R11 is caller saved on both Fastcall and SystemV, and not used for argument
        // passing, so it's pretty much free. It is also not used by the stacklimit mechanism.
        let tmp = regs::r11();
        debug_assert!({
            let real_reg = tmp.to_real_reg().unwrap();
            !is_callee_save_systemv(real_reg, false) && !is_callee_save_fastcall(real_reg, false)
        });

        insts.push(Inst::StackProbeLoop {
            tmp: Writable::from_reg(tmp),
            frame_size,
            guard_size,
        });
    }
}

impl IsaFlags for x64_settings::Flags {}

impl ABIMachineSpec for X64ABIMachineSpec {
    type I = Inst;

    type F = x64_settings::Flags;

    fn word_bits() -> u32 {
        64
    }

    /// Return required stack alignment in bytes.
    fn stack_align(_call_conv: isa::CallConv) -> u32 {
        16
    }

    fn compute_arg_locs<'a, I>(
        call_conv: isa::CallConv,
        flags: &settings::Flags,
        params: I,
        args_or_rets: ArgsOrRets,
        add_ret_area_ptr: bool,
        mut args: ArgsAccumulator<'_>,
    ) -> CodegenResult<(i64, Option<usize>)>
    where
        I: IntoIterator<Item = &'a ir::AbiParam>,
    {
        let is_fastcall = call_conv.extends_windows_fastcall();

        let mut next_gpr = 0;
        let mut next_vreg = 0;
        let mut next_stack: u64 = 0;
        let mut next_param_idx = 0; // Fastcall cares about overall param index

        if args_or_rets == ArgsOrRets::Args && is_fastcall {
            // Fastcall always reserves 32 bytes of shadow space corresponding to
            // the four initial in-arg parameters.
            //
            // (See:
            // https://docs.microsoft.com/en-us/cpp/build/x64-calling-convention?view=msvc-160)
            next_stack = 32;
        }

        for param in params {
            if let ir::ArgumentPurpose::StructArgument(size) = param.purpose {
                let offset = next_stack as i64;
                let size = size as u64;
                assert!(size % 8 == 0, "StructArgument size is not properly aligned");
                next_stack += size;
                args.push(ABIArg::StructArg {
                    pointer: None,
                    offset,
                    size,
                    purpose: param.purpose,
                });
                continue;
            }

            // Find regclass(es) of the register(s) used to store a value of this type.
            let (rcs, reg_tys) = Inst::rc_for_type(param.value_type)?;

            // Now assign ABIArgSlots for each register-sized part.
            //
            // Note that the handling of `i128` values is unique here:
            //
            // - If `enable_llvm_abi_extensions` is set in the flags, each
            //   `i128` is split into two `i64`s and assigned exactly as if it
            //   were two consecutive 64-bit args. This is consistent with LLVM's
            //   behavior, and is needed for some uses of Cranelift (e.g., the
            //   rustc backend).
            //
            // - Otherwise, both SysV and Fastcall specify behavior (use of
            //   vector register, a register pair, or passing by reference
            //   depending on the case), but for simplicity, we will just panic if
            //   an i128 type appears in a signature and the LLVM extensions flag
            //   is not set.
            //
            // For examples of how rustc compiles i128 args and return values on
            // both SysV and Fastcall platforms, see:
            // https://godbolt.org/z/PhG3ob

            if param.value_type.bits() > 64
                && !param.value_type.is_vector()
                && !flags.enable_llvm_abi_extensions()
            {
                panic!(
                    "i128 args/return values not supported unless LLVM ABI extensions are enabled"
                );
            }

            let mut slots = ABIArgSlotVec::new();
            for (rc, reg_ty) in rcs.iter().zip(reg_tys.iter()) {
                let intreg = *rc == RegClass::Int;
                let nextreg = if intreg {
                    match args_or_rets {
                        ArgsOrRets::Args => {
                            get_intreg_for_arg(&call_conv, next_gpr, next_param_idx)
                        }
                        ArgsOrRets::Rets => {
                            get_intreg_for_retval(&call_conv, next_gpr, next_param_idx)
                        }
                    }
                } else {
                    match args_or_rets {
                        ArgsOrRets::Args => {
                            get_fltreg_for_arg(&call_conv, next_vreg, next_param_idx)
                        }
                        ArgsOrRets::Rets => {
                            get_fltreg_for_retval(&call_conv, next_vreg, next_param_idx)
                        }
                    }
                };
                next_param_idx += 1;
                if let Some(reg) = nextreg {
                    if intreg {
                        next_gpr += 1;
                    } else {
                        next_vreg += 1;
                    }
                    slots.push(ABIArgSlot::Reg {
                        reg: reg.to_real_reg().unwrap(),
                        ty: *reg_ty,
                        extension: param.extension,
                    });
                } else {
                    // Compute size. For the wasmtime ABI it differs from native
                    // ABIs in how multiple values are returned, so we take a
                    // leaf out of arm64's book by not rounding everything up to
                    // 8 bytes. For all ABI arguments, and other ABI returns,
                    // though, each slot takes a minimum of 8 bytes.
                    //
                    // Note that in all cases 16-byte stack alignment happens
                    // separately after all args.
                    let size = (reg_ty.bits() / 8) as u64;
                    let size = if args_or_rets == ArgsOrRets::Rets && call_conv.extends_wasmtime() {
                        size
                    } else {
                        std::cmp::max(size, 8)
                    };
                    // Align.
                    debug_assert!(size.is_power_of_two());
                    next_stack = align_to(next_stack, size);
                    slots.push(ABIArgSlot::Stack {
                        offset: next_stack as i64,
                        ty: *reg_ty,
                        extension: param.extension,
                    });
                    next_stack += size;
                }
            }

            args.push(ABIArg::Slots {
                slots,
                purpose: param.purpose,
            });
        }

        let extra_arg = if add_ret_area_ptr {
            debug_assert!(args_or_rets == ArgsOrRets::Args);
            if let Some(reg) = get_intreg_for_arg(&call_conv, next_gpr, next_param_idx) {
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

        next_stack = align_to(next_stack, 16);

        // To avoid overflow issues, limit the arg/return size to something reasonable.
        if next_stack > STACK_ARG_RET_SIZE_LIMIT {
            return Err(CodegenError::ImplLimitExceeded);
        }

        Ok((next_stack as i64, extra_arg))
    }

    fn fp_to_arg_offset(_call_conv: isa::CallConv, _flags: &settings::Flags) -> i64 {
        16 // frame pointer + return address.
    }

    fn gen_load_stack(mem: StackAMode, into_reg: Writable<Reg>, ty: Type) -> Self::I {
        // For integer-typed values, we always load a full 64 bits (and we always spill a full 64
        // bits as well -- see `Inst::store()`).
        let ty = match ty {
            types::I8 | types::I16 | types::I32 => types::I64,
            _ => ty,
        };
        Inst::load(ty, mem, into_reg, ExtKind::None)
    }

    fn gen_store_stack(mem: StackAMode, from_reg: Reg, ty: Type) -> Self::I {
        Inst::store(ty, from_reg, mem)
    }

    fn gen_move(to_reg: Writable<Reg>, from_reg: Reg, ty: Type) -> Self::I {
        Inst::gen_move(to_reg, from_reg, ty)
    }

    /// Generate an integer-extend operation.
    fn gen_extend(
        to_reg: Writable<Reg>,
        from_reg: Reg,
        is_signed: bool,
        from_bits: u8,
        to_bits: u8,
    ) -> Self::I {
        let ext_mode = ExtMode::new(from_bits as u16, to_bits as u16)
            .unwrap_or_else(|| panic!("invalid extension: {} -> {}", from_bits, to_bits));
        if is_signed {
            Inst::movsx_rm_r(ext_mode, RegMem::reg(from_reg), to_reg)
        } else {
            Inst::movzx_rm_r(ext_mode, RegMem::reg(from_reg), to_reg)
        }
    }

    fn gen_args(_isa_flags: &x64_settings::Flags, args: Vec<ArgPair>) -> Inst {
        Inst::Args { args }
    }

    fn gen_ret(
        _setup_frame: bool,
        _isa_flags: &x64_settings::Flags,
        rets: Vec<RetPair>,
    ) -> Self::I {
        Inst::ret(rets)
    }

    fn gen_add_imm(into_reg: Writable<Reg>, from_reg: Reg, imm: u32) -> SmallInstVec<Self::I> {
        let mut ret = SmallVec::new();
        if from_reg != into_reg.to_reg() {
            ret.push(Inst::gen_move(into_reg, from_reg, I64));
        }
        ret.push(Inst::alu_rmi_r(
            OperandSize::Size64,
            AluRmiROpcode::Add,
            RegMemImm::imm(imm),
            into_reg,
        ));
        ret
    }

    fn gen_stack_lower_bound_trap(limit_reg: Reg) -> SmallInstVec<Self::I> {
        smallvec![
            Inst::cmp_rmi_r(OperandSize::Size64, RegMemImm::reg(regs::rsp()), limit_reg),
            Inst::TrapIf {
                // NBE == "> unsigned"; args above are reversed; this tests limit_reg > rsp.
                cc: CC::NBE,
                trap_code: TrapCode::StackOverflow,
            },
        ]
    }

    fn gen_get_stack_addr(mem: StackAMode, into_reg: Writable<Reg>, _ty: Type) -> Self::I {
        let mem: SyntheticAmode = mem.into();
        Inst::lea(mem, into_reg)
    }

    fn get_stacklimit_reg() -> Reg {
        debug_assert!(!is_callee_save_systemv(
            regs::r10().to_real_reg().unwrap(),
            false
        ));

        // As per comment on trait definition, we must return a caller-save
        // register here.
        regs::r10()
    }

    fn gen_load_base_offset(into_reg: Writable<Reg>, base: Reg, offset: i32, ty: Type) -> Self::I {
        // Only ever used for I64s; if that changes, see if the ExtKind below needs to be changed.
        assert_eq!(ty, I64);
        let simm32 = offset as u32;
        let mem = Amode::imm_reg(simm32, base);
        Inst::load(ty, mem, into_reg, ExtKind::None)
    }

    fn gen_store_base_offset(base: Reg, offset: i32, from_reg: Reg, ty: Type) -> Self::I {
        let simm32 = offset as u32;
        let mem = Amode::imm_reg(simm32, base);
        Inst::store(ty, from_reg, mem)
    }

    fn gen_sp_reg_adjust(amount: i32) -> SmallInstVec<Self::I> {
        let (alu_op, amount) = if amount >= 0 {
            (AluRmiROpcode::Add, amount)
        } else {
            (AluRmiROpcode::Sub, -amount)
        };

        let amount = amount as u32;

        smallvec![Inst::alu_rmi_r(
            OperandSize::Size64,
            alu_op,
            RegMemImm::imm(amount),
            Writable::from_reg(regs::rsp()),
        )]
    }

    fn gen_nominal_sp_adj(offset: i32) -> Self::I {
        Inst::VirtualSPOffsetAdj {
            offset: offset as i64,
        }
    }

    fn gen_prologue_frame_setup(flags: &settings::Flags) -> SmallInstVec<Self::I> {
        let r_rsp = regs::rsp();
        let r_rbp = regs::rbp();
        let w_rbp = Writable::from_reg(r_rbp);
        let mut insts = SmallVec::new();
        // `push %rbp`
        // RSP before the call will be 0 % 16.  So here, it is 8 % 16.
        insts.push(Inst::push64(RegMemImm::reg(r_rbp)));

        if flags.unwind_info() {
            insts.push(Inst::Unwind {
                inst: UnwindInst::PushFrameRegs {
                    offset_upward_to_caller_sp: 16, // RBP, return address
                },
            });
        }

        // `mov %rsp, %rbp`
        // RSP is now 0 % 16
        insts.push(Inst::mov_r_r(OperandSize::Size64, r_rsp, w_rbp));
        insts
    }

    fn gen_epilogue_frame_restore(_: &settings::Flags) -> SmallInstVec<Self::I> {
        let mut insts = SmallVec::new();
        // `mov %rbp, %rsp`
        insts.push(Inst::mov_r_r(
            OperandSize::Size64,
            regs::rbp(),
            Writable::from_reg(regs::rsp()),
        ));
        // `pop %rbp`
        insts.push(Inst::pop64(Writable::from_reg(regs::rbp())));
        insts
    }

    fn gen_probestack(insts: &mut SmallInstVec<Self::I>, frame_size: u32) {
        insts.push(Inst::imm(
            OperandSize::Size32,
            frame_size as u64,
            Writable::from_reg(regs::rax()),
        ));
        insts.push(Inst::CallKnown {
            dest: ExternalName::LibCall(LibCall::Probestack),
            info: Box::new(CallInfo {
                // No need to include arg here: we are post-regalloc
                // so no constraints will be seen anyway.
                uses: smallvec![],
                defs: smallvec![],
                clobbers: PRegSet::empty(),
                opcode: Opcode::Call,
            }),
        });
    }

    fn gen_inline_probestack(insts: &mut SmallInstVec<Self::I>, frame_size: u32, guard_size: u32) {
        // Unroll at most n consecutive probes, before falling back to using a loop
        //
        // This was number was picked because the loop version is 38 bytes long. We can fit
        // 5 inline probes in that space, so unroll if its beneficial in terms of code size.
        const PROBE_MAX_UNROLL: u32 = 5;

        // Number of probes that we need to perform
        let probe_count = align_to(frame_size, guard_size) / guard_size;

        if probe_count <= PROBE_MAX_UNROLL {
            Self::gen_probestack_unroll(insts, guard_size, probe_count)
        } else {
            Self::gen_probestack_loop(insts, frame_size, guard_size)
        }
    }

    fn gen_clobber_save(
        _call_conv: isa::CallConv,
        setup_frame: bool,
        flags: &settings::Flags,
        clobbered_callee_saves: &[Writable<RealReg>],
        fixed_frame_storage_size: u32,
        _outgoing_args_size: u32,
    ) -> (u64, SmallVec<[Self::I; 16]>) {
        let mut insts = SmallVec::new();
        let clobbered_size = compute_clobber_size(&clobbered_callee_saves);

        if flags.unwind_info() && setup_frame {
            // Emit unwind info: start the frame. The frame (from unwind
            // consumers' point of view) starts at clobbbers, just below
            // the FP and return address. Spill slots and stack slots are
            // part of our actual frame but do not concern the unwinder.
            insts.push(Inst::Unwind {
                inst: UnwindInst::DefineNewFrame {
                    offset_downward_to_clobbers: clobbered_size,
                    offset_upward_to_caller_sp: 16, // RBP, return address
                },
            });
        }

        // Adjust the stack pointer downward for clobbers and the function fixed
        // frame (spillslots and storage slots).
        let stack_size = fixed_frame_storage_size + clobbered_size;
        if stack_size > 0 {
            insts.push(Inst::alu_rmi_r(
                OperandSize::Size64,
                AluRmiROpcode::Sub,
                RegMemImm::imm(stack_size),
                Writable::from_reg(regs::rsp()),
            ));
        }
        // Store each clobbered register in order at offsets from RSP,
        // placing them above the fixed frame slots.
        let mut cur_offset = fixed_frame_storage_size;
        for reg in clobbered_callee_saves {
            let r_reg = reg.to_reg();
            let off = cur_offset;
            match r_reg.class() {
                RegClass::Int => {
                    insts.push(Inst::store(
                        types::I64,
                        r_reg.into(),
                        Amode::imm_reg(cur_offset, regs::rsp()),
                    ));
                    cur_offset += 8;
                }
                RegClass::Float => {
                    cur_offset = align_to(cur_offset, 16);
                    insts.push(Inst::store(
                        types::I8X16,
                        r_reg.into(),
                        Amode::imm_reg(cur_offset, regs::rsp()),
                    ));
                    cur_offset += 16;
                }
            };
            if flags.unwind_info() {
                insts.push(Inst::Unwind {
                    inst: UnwindInst::SaveReg {
                        clobber_offset: off - fixed_frame_storage_size,
                        reg: r_reg,
                    },
                });
            }
        }

        (clobbered_size as u64, insts)
    }

    fn gen_clobber_restore(
        call_conv: isa::CallConv,
        sig: &Signature,
        flags: &settings::Flags,
        clobbers: &[Writable<RealReg>],
        fixed_frame_storage_size: u32,
        _outgoing_args_size: u32,
    ) -> SmallVec<[Self::I; 16]> {
        let mut insts = SmallVec::new();

        let clobbered_callee_saves =
            Self::get_clobbered_callee_saves(call_conv, flags, sig, clobbers);
        let stack_size = fixed_frame_storage_size + compute_clobber_size(&clobbered_callee_saves);

        // Restore regs by loading from offsets of RSP. RSP will be
        // returned to nominal-RSP at this point, so we can use the
        // same offsets that we used when saving clobbers above.
        let mut cur_offset = fixed_frame_storage_size;
        for reg in &clobbered_callee_saves {
            let rreg = reg.to_reg();
            match rreg.class() {
                RegClass::Int => {
                    insts.push(Inst::mov64_m_r(
                        Amode::imm_reg(cur_offset, regs::rsp()),
                        Writable::from_reg(rreg.into()),
                    ));
                    cur_offset += 8;
                }
                RegClass::Float => {
                    cur_offset = align_to(cur_offset, 16);
                    insts.push(Inst::load(
                        types::I8X16,
                        Amode::imm_reg(cur_offset, regs::rsp()),
                        Writable::from_reg(rreg.into()),
                        ExtKind::None,
                    ));
                    cur_offset += 16;
                }
            }
        }
        // Adjust RSP back upward.
        if stack_size > 0 {
            insts.push(Inst::alu_rmi_r(
                OperandSize::Size64,
                AluRmiROpcode::Add,
                RegMemImm::imm(stack_size),
                Writable::from_reg(regs::rsp()),
            ));
        }

        insts
    }

    /// Generate a call instruction/sequence.
    fn gen_call(
        dest: &CallDest,
        uses: CallArgList,
        defs: CallRetList,
        clobbers: PRegSet,
        opcode: ir::Opcode,
        tmp: Writable<Reg>,
        _callee_conv: isa::CallConv,
        _caller_conv: isa::CallConv,
    ) -> SmallVec<[Self::I; 2]> {
        let mut insts = SmallVec::new();
        match dest {
            &CallDest::ExtName(ref name, RelocDistance::Near) => {
                insts.push(Inst::call_known(name.clone(), uses, defs, clobbers, opcode));
            }
            &CallDest::ExtName(ref name, RelocDistance::Far) => {
                insts.push(Inst::LoadExtName {
                    dst: tmp,
                    name: Box::new(name.clone()),
                    offset: 0,
                });
                insts.push(Inst::call_unknown(
                    RegMem::reg(tmp.to_reg()),
                    uses,
                    defs,
                    clobbers,
                    opcode,
                ));
            }
            &CallDest::Reg(reg) => {
                insts.push(Inst::call_unknown(
                    RegMem::reg(reg),
                    uses,
                    defs,
                    clobbers,
                    opcode,
                ));
            }
        }
        insts
    }

    fn gen_memcpy<F: FnMut(Type) -> Writable<Reg>>(
        call_conv: isa::CallConv,
        dst: Reg,
        src: Reg,
        size: usize,
        mut alloc_tmp: F,
    ) -> SmallVec<[Self::I; 8]> {
        let mut insts = SmallVec::new();
        let arg0 = get_intreg_for_arg(&call_conv, 0, 0).unwrap();
        let arg1 = get_intreg_for_arg(&call_conv, 1, 1).unwrap();
        let arg2 = get_intreg_for_arg(&call_conv, 2, 2).unwrap();
        let temp = alloc_tmp(Self::word_type());
        let temp2 = alloc_tmp(Self::word_type());
        insts.extend(
            Inst::gen_constant(ValueRegs::one(temp), size as u128, I64, |_| {
                panic!("tmp should not be needed")
            })
            .into_iter(),
        );
        // We use an indirect call and a full LoadExtName because we do not have
        // information about the libcall `RelocDistance` here, so we
        // conservatively use the more flexible calling sequence.
        insts.push(Inst::LoadExtName {
            dst: temp2,
            name: Box::new(ExternalName::LibCall(LibCall::Memcpy)),
            offset: 0,
        });
        insts.push(Inst::call_unknown(
            RegMem::reg(temp2.to_reg()),
            /* uses = */
            smallvec![
                CallArgPair {
                    vreg: dst,
                    preg: arg0
                },
                CallArgPair {
                    vreg: src,
                    preg: arg1
                },
                CallArgPair {
                    vreg: temp.to_reg(),
                    preg: arg2
                },
            ],
            /* defs = */ smallvec![],
            /* clobbers = */ Self::get_regs_clobbered_by_call(call_conv),
            Opcode::Call,
        ));
        insts
    }

    fn get_number_of_spillslots_for_value(rc: RegClass, vector_scale: u32) -> u32 {
        // We allocate in terms of 8-byte slots.
        match rc {
            RegClass::Int => 1,
            RegClass::Float => vector_scale / 8,
        }
    }

    fn get_virtual_sp_offset_from_state(s: &<Self::I as MachInstEmit>::State) -> i64 {
        s.virtual_sp_offset
    }

    fn get_nominal_sp_to_fp(s: &<Self::I as MachInstEmit>::State) -> i64 {
        s.nominal_sp_to_fp
    }

    fn get_regs_clobbered_by_call(call_conv_of_callee: isa::CallConv) -> PRegSet {
        if call_conv_of_callee.extends_windows_fastcall() {
            WINDOWS_CLOBBERS
        } else {
            SYSV_CLOBBERS
        }
    }

    fn get_ext_mode(
        _call_conv: isa::CallConv,
        _specified: ir::ArgumentExtension,
    ) -> ir::ArgumentExtension {
        ir::ArgumentExtension::None
    }

    fn get_clobbered_callee_saves(
        call_conv: CallConv,
        flags: &settings::Flags,
        _sig: &Signature,
        regs: &[Writable<RealReg>],
    ) -> Vec<Writable<RealReg>> {
        let mut regs: Vec<Writable<RealReg>> = match call_conv {
            CallConv::Fast | CallConv::Cold | CallConv::SystemV | CallConv::WasmtimeSystemV => regs
                .iter()
                .cloned()
                .filter(|r| is_callee_save_systemv(r.to_reg(), flags.enable_pinned_reg()))
                .collect(),
            CallConv::WindowsFastcall | CallConv::WasmtimeFastcall => regs
                .iter()
                .cloned()
                .filter(|r| is_callee_save_fastcall(r.to_reg(), flags.enable_pinned_reg()))
                .collect(),
            CallConv::Probestack => todo!("probestack?"),
            CallConv::AppleAarch64 | CallConv::WasmtimeAppleAarch64 => unreachable!(),
        };
        // Sort registers for deterministic code output. We can do an unstable sort because the
        // registers will be unique (there are no dups).
        regs.sort_unstable_by_key(|r| VReg::from(r.to_reg()).vreg());
        regs
    }

    fn is_frame_setup_needed(
        _is_leaf: bool,
        _stack_args_size: u32,
        _num_clobbered_callee_saves: usize,
        _frame_storage_size: u32,
    ) -> bool {
        true
    }
}

impl From<StackAMode> for SyntheticAmode {
    fn from(amode: StackAMode) -> Self {
        // We enforce a 128 MB stack-frame size limit above, so these
        // `expect()`s should never fail.
        match amode {
            StackAMode::FPOffset(off, _ty) => {
                let off = i32::try_from(off)
                    .expect("Offset in FPOffset is greater than 2GB; should hit impl limit first");
                let simm32 = off as u32;
                SyntheticAmode::Real(Amode::ImmReg {
                    simm32,
                    base: regs::rbp(),
                    flags: MemFlags::trusted(),
                })
            }
            StackAMode::NominalSPOffset(off, _ty) => {
                let off = i32::try_from(off).expect(
                    "Offset in NominalSPOffset is greater than 2GB; should hit impl limit first",
                );
                let simm32 = off as u32;
                SyntheticAmode::nominal_sp_offset(simm32)
            }
            StackAMode::SPOffset(off, _ty) => {
                let off = i32::try_from(off)
                    .expect("Offset in SPOffset is greater than 2GB; should hit impl limit first");
                let simm32 = off as u32;
                SyntheticAmode::Real(Amode::ImmReg {
                    simm32,
                    base: regs::rsp(),
                    flags: MemFlags::trusted(),
                })
            }
        }
    }
}

fn get_intreg_for_arg(call_conv: &CallConv, idx: usize, arg_idx: usize) -> Option<Reg> {
    let is_fastcall = call_conv.extends_windows_fastcall();

    // Fastcall counts by absolute argument number; SysV counts by argument of
    // this (integer) class.
    let i = if is_fastcall { arg_idx } else { idx };
    match (i, is_fastcall) {
        (0, false) => Some(regs::rdi()),
        (1, false) => Some(regs::rsi()),
        (2, false) => Some(regs::rdx()),
        (3, false) => Some(regs::rcx()),
        (4, false) => Some(regs::r8()),
        (5, false) => Some(regs::r9()),
        (0, true) => Some(regs::rcx()),
        (1, true) => Some(regs::rdx()),
        (2, true) => Some(regs::r8()),
        (3, true) => Some(regs::r9()),
        _ => None,
    }
}

fn get_fltreg_for_arg(call_conv: &CallConv, idx: usize, arg_idx: usize) -> Option<Reg> {
    let is_fastcall = call_conv.extends_windows_fastcall();

    // Fastcall counts by absolute argument number; SysV counts by argument of
    // this (floating-point) class.
    let i = if is_fastcall { arg_idx } else { idx };
    match (i, is_fastcall) {
        (0, false) => Some(regs::xmm0()),
        (1, false) => Some(regs::xmm1()),
        (2, false) => Some(regs::xmm2()),
        (3, false) => Some(regs::xmm3()),
        (4, false) => Some(regs::xmm4()),
        (5, false) => Some(regs::xmm5()),
        (6, false) => Some(regs::xmm6()),
        (7, false) => Some(regs::xmm7()),
        (0, true) => Some(regs::xmm0()),
        (1, true) => Some(regs::xmm1()),
        (2, true) => Some(regs::xmm2()),
        (3, true) => Some(regs::xmm3()),
        _ => None,
    }
}

fn get_intreg_for_retval(
    call_conv: &CallConv,
    intreg_idx: usize,
    retval_idx: usize,
) -> Option<Reg> {
    match call_conv {
        CallConv::Fast | CallConv::Cold | CallConv::SystemV => match intreg_idx {
            0 => Some(regs::rax()),
            1 => Some(regs::rdx()),
            _ => None,
        },
        CallConv::WasmtimeSystemV | CallConv::WasmtimeFastcall => {
            if intreg_idx == 0 && retval_idx == 0 {
                Some(regs::rax())
            } else {
                None
            }
        }
        CallConv::WindowsFastcall => match intreg_idx {
            0 => Some(regs::rax()),
            1 => Some(regs::rdx()), // The Rust ABI for i128s needs this.
            _ => None,
        },
        CallConv::Probestack => todo!(),
        CallConv::AppleAarch64 | CallConv::WasmtimeAppleAarch64 => unreachable!(),
    }
}

fn get_fltreg_for_retval(
    call_conv: &CallConv,
    fltreg_idx: usize,
    retval_idx: usize,
) -> Option<Reg> {
    match call_conv {
        CallConv::Fast | CallConv::Cold | CallConv::SystemV => match fltreg_idx {
            0 => Some(regs::xmm0()),
            1 => Some(regs::xmm1()),
            _ => None,
        },
        CallConv::WasmtimeFastcall | CallConv::WasmtimeSystemV => {
            if fltreg_idx == 0 && retval_idx == 0 {
                Some(regs::xmm0())
            } else {
                None
            }
        }
        CallConv::WindowsFastcall => match fltreg_idx {
            0 => Some(regs::xmm0()),
            _ => None,
        },
        CallConv::Probestack => todo!(),
        CallConv::AppleAarch64 | CallConv::WasmtimeAppleAarch64 => unreachable!(),
    }
}

fn is_callee_save_systemv(r: RealReg, enable_pinned_reg: bool) -> bool {
    use regs::*;
    match r.class() {
        RegClass::Int => match r.hw_enc() {
            ENC_RBX | ENC_RBP | ENC_R12 | ENC_R13 | ENC_R14 => true,
            // R15 is the pinned register; if we're using it that way,
            // it is effectively globally-allocated, and is not
            // callee-saved.
            ENC_R15 => !enable_pinned_reg,
            _ => false,
        },
        RegClass::Float => false,
    }
}

fn is_callee_save_fastcall(r: RealReg, enable_pinned_reg: bool) -> bool {
    use regs::*;
    match r.class() {
        RegClass::Int => match r.hw_enc() {
            ENC_RBX | ENC_RBP | ENC_RSI | ENC_RDI | ENC_R12 | ENC_R13 | ENC_R14 => true,
            // See above for SysV: we must treat the pinned reg specially.
            ENC_R15 => !enable_pinned_reg,
            _ => false,
        },
        RegClass::Float => match r.hw_enc() {
            6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 => true,
            _ => false,
        },
    }
}

fn compute_clobber_size(clobbers: &[Writable<RealReg>]) -> u32 {
    let mut clobbered_size = 0;
    for reg in clobbers {
        match reg.to_reg().class() {
            RegClass::Int => {
                clobbered_size += 8;
            }
            RegClass::Float => {
                clobbered_size = align_to(clobbered_size, 16);
                clobbered_size += 16;
            }
        }
    }
    align_to(clobbered_size, 16)
}

const WINDOWS_CLOBBERS: PRegSet = windows_clobbers();
const SYSV_CLOBBERS: PRegSet = sysv_clobbers();

const fn windows_clobbers() -> PRegSet {
    PRegSet::empty()
        .with(regs::gpr_preg(regs::ENC_RAX))
        .with(regs::gpr_preg(regs::ENC_RCX))
        .with(regs::gpr_preg(regs::ENC_RDX))
        .with(regs::gpr_preg(regs::ENC_R8))
        .with(regs::gpr_preg(regs::ENC_R9))
        .with(regs::gpr_preg(regs::ENC_R10))
        .with(regs::gpr_preg(regs::ENC_R11))
        .with(regs::fpr_preg(0))
        .with(regs::fpr_preg(1))
        .with(regs::fpr_preg(2))
        .with(regs::fpr_preg(3))
        .with(regs::fpr_preg(4))
        .with(regs::fpr_preg(5))
}

const fn sysv_clobbers() -> PRegSet {
    PRegSet::empty()
        .with(regs::gpr_preg(regs::ENC_RAX))
        .with(regs::gpr_preg(regs::ENC_RCX))
        .with(regs::gpr_preg(regs::ENC_RDX))
        .with(regs::gpr_preg(regs::ENC_RSI))
        .with(regs::gpr_preg(regs::ENC_RDI))
        .with(regs::gpr_preg(regs::ENC_R8))
        .with(regs::gpr_preg(regs::ENC_R9))
        .with(regs::gpr_preg(regs::ENC_R10))
        .with(regs::gpr_preg(regs::ENC_R11))
        .with(regs::fpr_preg(0))
        .with(regs::fpr_preg(1))
        .with(regs::fpr_preg(2))
        .with(regs::fpr_preg(3))
        .with(regs::fpr_preg(4))
        .with(regs::fpr_preg(5))
        .with(regs::fpr_preg(6))
        .with(regs::fpr_preg(7))
        .with(regs::fpr_preg(8))
        .with(regs::fpr_preg(9))
        .with(regs::fpr_preg(10))
        .with(regs::fpr_preg(11))
        .with(regs::fpr_preg(12))
        .with(regs::fpr_preg(13))
        .with(regs::fpr_preg(14))
        .with(regs::fpr_preg(15))
}
