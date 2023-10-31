//! Implementation of the standard x64 ABI.

use crate::ir::types::*;
use crate::ir::{self, types, ExternalName, LibCall, MemFlags, Opcode, TrapCode, Type};
use crate::isa;
use crate::isa::{unwind::UnwindInst, x64::inst::*, CallConv};
use crate::machinst::abi_impl::*;
use crate::machinst::*;
use crate::settings;
use crate::{CodegenError, CodegenResult};
use alloc::boxed::Box;
use alloc::vec::Vec;
use args::*;
use regalloc::{RealReg, Reg, RegClass, Set, Writable};
use smallvec::{smallvec, SmallVec};
use std::convert::TryFrom;

/// This is the limit for the size of argument and return-value areas on the
/// stack. We place a reasonable limit here to avoid integer overflow issues
/// with 32-bit arithmetic: for now, 128 MB.
static STACK_ARG_RET_SIZE_LIMIT: u64 = 128 * 1024 * 1024;

/// Offset in stack-arg area to callee-TLS slot in Baldrdash-2020 calling convention.
static BALDRDASH_CALLEE_TLS_OFFSET: i64 = 0;
/// Offset in stack-arg area to caller-TLS slot in Baldrdash-2020 calling convention.
static BALDRDASH_CALLER_TLS_OFFSET: i64 = 8;

/// Try to fill a Baldrdash register, returning it if it was found.
fn try_fill_baldrdash_reg(call_conv: CallConv, param: &ir::AbiParam) -> Option<ABIArg> {
    if call_conv.extends_baldrdash() {
        match &param.purpose {
            &ir::ArgumentPurpose::VMContext => {
                // This is SpiderMonkey's `WasmTlsReg`.
                Some(ABIArg::reg(
                    regs::r14().to_real_reg(),
                    types::I64,
                    param.extension,
                    param.purpose,
                ))
            }
            &ir::ArgumentPurpose::SignatureId => {
                // This is SpiderMonkey's `WasmTableCallSigReg`.
                Some(ABIArg::reg(
                    regs::r10().to_real_reg(),
                    types::I64,
                    param.extension,
                    param.purpose,
                ))
            }
            &ir::ArgumentPurpose::CalleeTLS => {
                // This is SpiderMonkey's callee TLS slot in the extended frame of Wasm's ABI-2020.
                assert!(call_conv == isa::CallConv::Baldrdash2020);
                Some(ABIArg::stack(
                    BALDRDASH_CALLEE_TLS_OFFSET,
                    ir::types::I64,
                    ir::ArgumentExtension::None,
                    param.purpose,
                ))
            }
            &ir::ArgumentPurpose::CallerTLS => {
                // This is SpiderMonkey's caller TLS slot in the extended frame of Wasm's ABI-2020.
                assert!(call_conv == isa::CallConv::Baldrdash2020);
                Some(ABIArg::stack(
                    BALDRDASH_CALLER_TLS_OFFSET,
                    ir::types::I64,
                    ir::ArgumentExtension::None,
                    param.purpose,
                ))
            }
            _ => None,
        }
    } else {
        None
    }
}

/// Support for the x64 ABI from the callee side (within a function body).
pub(crate) type X64ABICallee = ABICalleeImpl<X64ABIMachineSpec>;

/// Support for the x64 ABI from the caller side (at a callsite).
pub(crate) type X64ABICaller = ABICallerImpl<X64ABIMachineSpec>;

/// Implementation of ABI primitives for x64.
pub(crate) struct X64ABIMachineSpec;

impl ABIMachineSpec for X64ABIMachineSpec {
    type I = Inst;

    fn word_bits() -> u32 {
        64
    }

    /// Return required stack alignment in bytes.
    fn stack_align(_call_conv: isa::CallConv) -> u32 {
        16
    }

    fn compute_arg_locs(
        call_conv: isa::CallConv,
        flags: &settings::Flags,
        params: &[ir::AbiParam],
        args_or_rets: ArgsOrRets,
        add_ret_area_ptr: bool,
    ) -> CodegenResult<(Vec<ABIArg>, i64, Option<usize>)> {
        let is_baldrdash = call_conv.extends_baldrdash();
        let is_fastcall = call_conv.extends_windows_fastcall();
        let has_baldrdash_tls = call_conv == isa::CallConv::Baldrdash2020;

        let mut next_gpr = 0;
        let mut next_vreg = 0;
        let mut next_stack: u64 = 0;
        let mut next_param_idx = 0; // Fastcall cares about overall param index
        let mut ret = vec![];

        if args_or_rets == ArgsOrRets::Args && is_fastcall {
            // Fastcall always reserves 32 bytes of shadow space corresponding to
            // the four initial in-arg parameters.
            //
            // (See:
            // https://docs.microsoft.com/en-us/cpp/build/x64-calling-convention?view=msvc-160)
            next_stack = 32;
        }

        if args_or_rets == ArgsOrRets::Args && has_baldrdash_tls {
            // Baldrdash ABI-2020 always has two stack-arg slots reserved, for the callee and
            // caller TLS-register values, respectively.
            next_stack = 16;
        }

        for i in 0..params.len() {
            // Process returns backward, according to the SpiderMonkey ABI (which we
            // adopt internally if `is_baldrdash` is set).
            let param = match (args_or_rets, is_baldrdash) {
                (ArgsOrRets::Args, _) => &params[i],
                (ArgsOrRets::Rets, false) => &params[i],
                (ArgsOrRets::Rets, true) => &params[params.len() - 1 - i],
            };

            // Validate "purpose".
            match &param.purpose {
                &ir::ArgumentPurpose::VMContext
                | &ir::ArgumentPurpose::Normal
                | &ir::ArgumentPurpose::StackLimit
                | &ir::ArgumentPurpose::SignatureId
                | &ir::ArgumentPurpose::CalleeTLS
                | &ir::ArgumentPurpose::CallerTLS
                | &ir::ArgumentPurpose::StructReturn
                | &ir::ArgumentPurpose::StructArgument(_) => {}
                _ => panic!(
                    "Unsupported argument purpose {:?} in signature: {:?}",
                    param.purpose, params
                ),
            }

            if let Some(param) = try_fill_baldrdash_reg(call_conv, param) {
                ret.push(param);
                continue;
            }

            if let ir::ArgumentPurpose::StructArgument(size) = param.purpose {
                let offset = next_stack as i64;
                let size = size as u64;
                assert!(size % 8 == 0, "StructArgument size is not properly aligned");
                next_stack += size;
                ret.push(ABIArg::StructArg {
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

            let mut slots = vec![];
            for (rc, reg_ty) in rcs.iter().zip(reg_tys.iter()) {
                let intreg = *rc == RegClass::I64;
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
                        reg: reg.to_real_reg(),
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

            ret.push(ABIArg::Slots {
                slots,
                purpose: param.purpose,
            });
        }

        if args_or_rets == ArgsOrRets::Rets && is_baldrdash {
            ret.reverse();
        }

        let extra_arg = if add_ret_area_ptr {
            debug_assert!(args_or_rets == ArgsOrRets::Args);
            if let Some(reg) = get_intreg_for_arg(&call_conv, next_gpr, next_param_idx) {
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

        next_stack = align_to(next_stack, 16);

        // To avoid overflow issues, limit the arg/return size to something reasonable.
        if next_stack > STACK_ARG_RET_SIZE_LIMIT {
            return Err(CodegenError::ImplLimitExceeded);
        }

        Ok((ret, next_stack as i64, extra_arg))
    }

    fn fp_to_arg_offset(call_conv: isa::CallConv, flags: &settings::Flags) -> i64 {
        if call_conv.extends_baldrdash() {
            let num_words = flags.baldrdash_prologue_words() as i64;
            debug_assert!(num_words > 0, "baldrdash must set baldrdash_prologue_words");
            num_words * 8
        } else {
            16 // frame pointer + return address.
        }
    }

    fn gen_load_stack(mem: StackAMode, into_reg: Writable<Reg>, ty: Type) -> Self::I {
        // For integer-typed values, we always load a full 64 bits (and we always spill a full 64
        // bits as well -- see `Inst::store()`).
        let ty = match ty {
            types::B1
            | types::B8
            | types::I8
            | types::B16
            | types::I16
            | types::B32
            | types::I32 => types::I64,
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

    fn gen_ret() -> Self::I {
        Inst::ret()
    }

    fn gen_epilogue_placeholder() -> Self::I {
        Inst::epilogue_placeholder()
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
        debug_assert!(
            !is_callee_save_systemv(regs::r10().to_real_reg())
                && !is_callee_save_baldrdash(regs::r10().to_real_reg())
        );

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

    fn gen_probestack(frame_size: u32) -> SmallInstVec<Self::I> {
        let mut insts = SmallVec::new();
        insts.push(Inst::imm(
            OperandSize::Size32,
            frame_size as u64,
            Writable::from_reg(regs::rax()),
        ));
        insts.push(Inst::CallKnown {
            dest: ExternalName::LibCall(LibCall::Probestack),
            uses: vec![regs::rax()],
            defs: vec![],
            opcode: Opcode::Call,
        });
        insts
    }

    fn gen_clobber_save(
        _call_conv: isa::CallConv,
        setup_frame: bool,
        flags: &settings::Flags,
        clobbered_callee_saves: &Vec<Writable<RealReg>>,
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
            match r_reg.get_class() {
                RegClass::I64 => {
                    insts.push(Inst::store(
                        types::I64,
                        r_reg.to_reg(),
                        Amode::imm_reg(cur_offset, regs::rsp()),
                    ));
                    cur_offset += 8;
                }
                RegClass::V128 => {
                    cur_offset = align_to(cur_offset, 16);
                    insts.push(Inst::store(
                        types::I8X16,
                        r_reg.to_reg(),
                        Amode::imm_reg(cur_offset, regs::rsp()),
                    ));
                    cur_offset += 16;
                }
                _ => unreachable!(),
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
        flags: &settings::Flags,
        clobbers: &Set<Writable<RealReg>>,
        fixed_frame_storage_size: u32,
        _outgoing_args_size: u32,
    ) -> SmallVec<[Self::I; 16]> {
        let mut insts = SmallVec::new();

        let clobbered_callee_saves = Self::get_clobbered_callee_saves(call_conv, clobbers);
        let stack_size = fixed_frame_storage_size + compute_clobber_size(&clobbered_callee_saves);

        // Restore regs by loading from offsets of RSP. RSP will be
        // returned to nominal-RSP at this point, so we can use the
        // same offsets that we used when saving clobbers above.
        let mut cur_offset = fixed_frame_storage_size;
        for reg in &clobbered_callee_saves {
            let rreg = reg.to_reg();
            match rreg.get_class() {
                RegClass::I64 => {
                    insts.push(Inst::mov64_m_r(
                        Amode::imm_reg(cur_offset, regs::rsp()),
                        Writable::from_reg(rreg.to_reg()),
                    ));
                    cur_offset += 8;
                }
                RegClass::V128 => {
                    cur_offset = align_to(cur_offset, 16);
                    insts.push(Inst::load(
                        types::I8X16,
                        Amode::imm_reg(cur_offset, regs::rsp()),
                        Writable::from_reg(rreg.to_reg()),
                        ExtKind::None,
                    ));
                    cur_offset += 16;
                }
                _ => unreachable!(),
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

        // If this is Baldrdash-2020, restore the callee (i.e., our) TLS
        // register. We may have allocated it for something else and clobbered
        // it, but the ABI expects us to leave the TLS register unchanged.
        if call_conv == isa::CallConv::Baldrdash2020 {
            let off = BALDRDASH_CALLEE_TLS_OFFSET + Self::fp_to_arg_offset(call_conv, flags);
            insts.push(Inst::mov64_m_r(
                Amode::imm_reg(off as u32, regs::rbp()),
                Writable::from_reg(regs::r14()),
            ));
        }

        insts
    }

    /// Generate a call instruction/sequence.
    fn gen_call(
        dest: &CallDest,
        uses: Vec<Reg>,
        defs: Vec<Writable<Reg>>,
        opcode: ir::Opcode,
        tmp: Writable<Reg>,
        _callee_conv: isa::CallConv,
        _caller_conv: isa::CallConv,
    ) -> SmallVec<[(InstIsSafepoint, Self::I); 2]> {
        let mut insts = SmallVec::new();
        match dest {
            &CallDest::ExtName(ref name, RelocDistance::Near) => {
                insts.push((
                    InstIsSafepoint::Yes,
                    Inst::call_known(name.clone(), uses, defs, opcode),
                ));
            }
            &CallDest::ExtName(ref name, RelocDistance::Far) => {
                insts.push((
                    InstIsSafepoint::No,
                    Inst::LoadExtName {
                        dst: tmp,
                        name: Box::new(name.clone()),
                        offset: 0,
                    },
                ));
                insts.push((
                    InstIsSafepoint::Yes,
                    Inst::call_unknown(RegMem::reg(tmp.to_reg()), uses, defs, opcode),
                ));
            }
            &CallDest::Reg(reg) => {
                insts.push((
                    InstIsSafepoint::Yes,
                    Inst::call_unknown(RegMem::reg(reg), uses, defs, opcode),
                ));
            }
        }
        insts
    }

    fn gen_memcpy(
        call_conv: isa::CallConv,
        dst: Reg,
        src: Reg,
        size: usize,
    ) -> SmallVec<[Self::I; 8]> {
        // Baldrdash should not use struct args.
        assert!(!call_conv.extends_baldrdash());
        let mut insts = SmallVec::new();
        let arg0 = get_intreg_for_arg(&call_conv, 0, 0).unwrap();
        let arg1 = get_intreg_for_arg(&call_conv, 1, 1).unwrap();
        let arg2 = get_intreg_for_arg(&call_conv, 2, 2).unwrap();
        // We need a register to load the address of `memcpy()` below and we
        // don't have a lowering context to allocate a temp here; so just use a
        // register we know we are free to mutate as part of this sequence
        // (because it is clobbered by the call as per the ABI anyway).
        let memcpy_addr = get_intreg_for_arg(&call_conv, 3, 3).unwrap();
        insts.push(Inst::gen_move(Writable::from_reg(arg0), dst, I64));
        insts.push(Inst::gen_move(Writable::from_reg(arg1), src, I64));
        insts.extend(
            Inst::gen_constant(
                ValueRegs::one(Writable::from_reg(arg2)),
                size as u128,
                I64,
                |_| panic!("tmp should not be needed"),
            )
            .into_iter(),
        );
        // We use an indirect call and a full LoadExtName because we do not have
        // information about the libcall `RelocDistance` here, so we
        // conservatively use the more flexible calling sequence.
        insts.push(Inst::LoadExtName {
            dst: Writable::from_reg(memcpy_addr),
            name: Box::new(ExternalName::LibCall(LibCall::Memcpy)),
            offset: 0,
        });
        insts.push(Inst::call_unknown(
            RegMem::reg(memcpy_addr),
            /* uses = */ vec![arg0, arg1, arg2],
            /* defs = */ Self::get_regs_clobbered_by_call(call_conv),
            Opcode::Call,
        ));
        insts
    }

    fn get_number_of_spillslots_for_value(rc: RegClass) -> u32 {
        // We allocate in terms of 8-byte slots.
        match rc {
            RegClass::I64 => 1,
            RegClass::V128 => 2,
            _ => panic!("Unexpected register class!"),
        }
    }

    fn get_virtual_sp_offset_from_state(s: &<Self::I as MachInstEmit>::State) -> i64 {
        s.virtual_sp_offset
    }

    fn get_nominal_sp_to_fp(s: &<Self::I as MachInstEmit>::State) -> i64 {
        s.nominal_sp_to_fp
    }

    fn get_regs_clobbered_by_call(call_conv_of_callee: isa::CallConv) -> Vec<Writable<Reg>> {
        let mut caller_saved = vec![
            // intersection of Systemv and FastCall calling conventions:
            // - GPR: all except RDI, RSI, RBX, RBP, R12 to R15.
            //        SysV adds RDI, RSI (FastCall makes these callee-saved).
            Writable::from_reg(regs::rax()),
            Writable::from_reg(regs::rcx()),
            Writable::from_reg(regs::rdx()),
            Writable::from_reg(regs::r8()),
            Writable::from_reg(regs::r9()),
            Writable::from_reg(regs::r10()),
            Writable::from_reg(regs::r11()),
            // - XMM: XMM0-5. SysV adds the rest (XMM6-XMM15).
            Writable::from_reg(regs::xmm0()),
            Writable::from_reg(regs::xmm1()),
            Writable::from_reg(regs::xmm2()),
            Writable::from_reg(regs::xmm3()),
            Writable::from_reg(regs::xmm4()),
            Writable::from_reg(regs::xmm5()),
        ];

        if !call_conv_of_callee.extends_windows_fastcall() {
            caller_saved.push(Writable::from_reg(regs::rsi()));
            caller_saved.push(Writable::from_reg(regs::rdi()));
            caller_saved.push(Writable::from_reg(regs::xmm6()));
            caller_saved.push(Writable::from_reg(regs::xmm7()));
            caller_saved.push(Writable::from_reg(regs::xmm8()));
            caller_saved.push(Writable::from_reg(regs::xmm9()));
            caller_saved.push(Writable::from_reg(regs::xmm10()));
            caller_saved.push(Writable::from_reg(regs::xmm11()));
            caller_saved.push(Writable::from_reg(regs::xmm12()));
            caller_saved.push(Writable::from_reg(regs::xmm13()));
            caller_saved.push(Writable::from_reg(regs::xmm14()));
            caller_saved.push(Writable::from_reg(regs::xmm15()));
        }

        if call_conv_of_callee.extends_baldrdash() {
            caller_saved.push(Writable::from_reg(regs::r12()));
            caller_saved.push(Writable::from_reg(regs::r13()));
            // Not r14; implicitly preserved in the entry.
            caller_saved.push(Writable::from_reg(regs::r15()));
            caller_saved.push(Writable::from_reg(regs::rbx()));
        }

        caller_saved
    }

    fn get_ext_mode(
        call_conv: isa::CallConv,
        specified: ir::ArgumentExtension,
    ) -> ir::ArgumentExtension {
        if call_conv.extends_baldrdash() {
            // Baldrdash (SpiderMonkey) always extends args and return values to the full register.
            specified
        } else {
            // No other supported ABI on x64 does so.
            ir::ArgumentExtension::None
        }
    }

    fn get_clobbered_callee_saves(
        call_conv: CallConv,
        regs: &Set<Writable<RealReg>>,
    ) -> Vec<Writable<RealReg>> {
        let mut regs: Vec<Writable<RealReg>> = match call_conv {
            CallConv::BaldrdashSystemV | CallConv::Baldrdash2020 => regs
                .iter()
                .cloned()
                .filter(|r| is_callee_save_baldrdash(r.to_reg()))
                .collect(),
            CallConv::BaldrdashWindows => {
                todo!("baldrdash windows");
            }
            CallConv::Fast | CallConv::Cold | CallConv::SystemV | CallConv::WasmtimeSystemV => regs
                .iter()
                .cloned()
                .filter(|r| is_callee_save_systemv(r.to_reg()))
                .collect(),
            CallConv::WindowsFastcall | CallConv::WasmtimeFastcall => regs
                .iter()
                .cloned()
                .filter(|r| is_callee_save_fastcall(r.to_reg()))
                .collect(),
            CallConv::Probestack => todo!("probestack?"),
            CallConv::AppleAarch64 | CallConv::WasmtimeAppleAarch64 => unreachable!(),
        };
        // Sort registers for deterministic code output. We can do an unstable sort because the
        // registers will be unique (there are no dups).
        regs.sort_unstable_by_key(|r| r.to_reg().get_index());
        regs
    }

    fn is_frame_setup_needed(
        _is_leaf: bool,
        _stack_args_size: u32,
        _num_clobbered_callee_saves: usize,
        _fixed_frame_storage_size: u32,
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
        CallConv::BaldrdashSystemV
        | CallConv::Baldrdash2020
        | CallConv::WasmtimeSystemV
        | CallConv::WasmtimeFastcall => {
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
        CallConv::BaldrdashWindows | CallConv::Probestack => todo!(),
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
        CallConv::BaldrdashSystemV
        | CallConv::Baldrdash2020
        | CallConv::WasmtimeFastcall
        | CallConv::WasmtimeSystemV => {
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
        CallConv::BaldrdashWindows | CallConv::Probestack => todo!(),
        CallConv::AppleAarch64 | CallConv::WasmtimeAppleAarch64 => unreachable!(),
    }
}

fn is_callee_save_systemv(r: RealReg) -> bool {
    use regs::*;
    match r.get_class() {
        RegClass::I64 => match r.get_hw_encoding() as u8 {
            ENC_RBX | ENC_RBP | ENC_R12 | ENC_R13 | ENC_R14 | ENC_R15 => true,
            _ => false,
        },
        RegClass::V128 => false,
        _ => unimplemented!(),
    }
}

fn is_callee_save_baldrdash(r: RealReg) -> bool {
    use regs::*;
    match r.get_class() {
        RegClass::I64 => {
            if r.get_hw_encoding() as u8 == ENC_R14 {
                // r14 is the WasmTlsReg and is preserved implicitly.
                false
            } else {
                // Defer to native for the other ones.
                is_callee_save_systemv(r)
            }
        }
        RegClass::V128 => false,
        _ => unimplemented!(),
    }
}

fn is_callee_save_fastcall(r: RealReg) -> bool {
    use regs::*;
    match r.get_class() {
        RegClass::I64 => match r.get_hw_encoding() as u8 {
            ENC_RBX | ENC_RBP | ENC_RSI | ENC_RDI | ENC_R12 | ENC_R13 | ENC_R14 | ENC_R15 => true,
            _ => false,
        },
        RegClass::V128 => match r.get_hw_encoding() as u8 {
            6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 => true,
            _ => false,
        },
        _ => panic!("Unknown register class: {:?}", r.get_class()),
    }
}

fn compute_clobber_size(clobbers: &Vec<Writable<RealReg>>) -> u32 {
    let mut clobbered_size = 0;
    for reg in clobbers {
        match reg.to_reg().get_class() {
            RegClass::I64 => {
                clobbered_size += 8;
            }
            RegClass::V128 => {
                clobbered_size = align_to(clobbered_size, 16);
                clobbered_size += 16;
            }
            _ => unreachable!(),
        }
    }
    align_to(clobbered_size, 16)
}
