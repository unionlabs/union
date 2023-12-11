//! Implementation of a standard AArch64 ABI.

use crate::ir;
use crate::ir::types;
use crate::ir::types::*;
use crate::ir::MemFlags;
use crate::ir::Opcode;
use crate::ir::{dynamic_to_fixed, ExternalName, LibCall, Signature};
use crate::isa;
use crate::isa::aarch64::{inst::EmitState, inst::*, settings as aarch64_settings};
use crate::isa::unwind::UnwindInst;
use crate::machinst::*;
use crate::settings;
use crate::{CodegenError, CodegenResult};
use alloc::boxed::Box;
use alloc::vec::Vec;
use regalloc2::{PRegSet, VReg};
use smallvec::{smallvec, SmallVec};

// We use a generic implementation that factors out AArch64 and x64 ABI commonalities, because
// these ABIs are very similar.

/// Support for the AArch64 ABI from the callee side (within a function body).
pub(crate) type AArch64Callee = Callee<AArch64MachineDeps>;

/// Support for the AArch64 ABI from the caller side (at a callsite).
pub(crate) type AArch64Caller = Caller<AArch64MachineDeps>;

/// This is the limit for the size of argument and return-value areas on the
/// stack. We place a reasonable limit here to avoid integer overflow issues
/// with 32-bit arithmetic: for now, 128 MB.
static STACK_ARG_RET_SIZE_LIMIT: u64 = 128 * 1024 * 1024;

impl Into<AMode> for StackAMode {
    fn into(self) -> AMode {
        match self {
            StackAMode::FPOffset(off, ty) => AMode::FPOffset { off, ty },
            StackAMode::NominalSPOffset(off, ty) => AMode::NominalSPOffset { off, ty },
            StackAMode::SPOffset(off, ty) => AMode::SPOffset { off, ty },
        }
    }
}

// Returns the size of stack space needed to store the
// `int_reg` and `vec_reg`.
fn saved_reg_stack_size(
    int_reg: &[Writable<RealReg>],
    vec_reg: &[Writable<RealReg>],
) -> (usize, usize) {
    // Round up to multiple of 2, to keep 16-byte stack alignment.
    let int_save_bytes = (int_reg.len() + (int_reg.len() & 1)) * 8;
    // The Procedure Call Standard for the Arm 64-bit Architecture
    // (AAPCS64, including several related ABIs such as the one used by
    // Windows) mandates saving only the bottom 8 bytes of the vector
    // registers, so we round up the number of registers to ensure
    // proper stack alignment (similarly to the situation with
    // `int_reg`).
    let vec_reg_size = 8;
    let vec_save_padding = vec_reg.len() & 1;
    // FIXME: SVE: ABI is different to Neon, so do we treat all vec regs as Z-regs?
    let vec_save_bytes = (vec_reg.len() + vec_save_padding) * vec_reg_size;

    (int_save_bytes, vec_save_bytes)
}

/// AArch64-specific ABI behavior. This struct just serves as an implementation
/// point for the trait; it is never actually instantiated.
pub struct AArch64MachineDeps;

impl IsaFlags for aarch64_settings::Flags {
    fn is_forward_edge_cfi_enabled(&self) -> bool {
        self.use_bti()
    }
}

impl ABIMachineSpec for AArch64MachineDeps {
    type I = Inst;

    type F = aarch64_settings::Flags;

    fn word_bits() -> u32 {
        64
    }

    /// Return required stack alignment in bytes.
    fn stack_align(_call_conv: isa::CallConv) -> u32 {
        16
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
        let is_apple_cc = call_conv.extends_apple_aarch64();

        // See AArch64 ABI (https://github.com/ARM-software/abi-aa/blob/2021Q1/aapcs64/aapcs64.rst#64parameter-passing), sections 6.4.
        //
        // MacOS aarch64 is slightly different, see also
        // https://developer.apple.com/documentation/xcode/writing_arm64_code_for_apple_platforms.
        // We are diverging from the MacOS aarch64 implementation in the
        // following ways:
        // - sign- and zero- extensions of data types less than 32 bits are not
        // implemented yet.
        // - we align the arguments stack space to a 16-bytes boundary, while
        // the MacOS allows aligning only on 8 bytes. In practice it means we're
        // slightly overallocating when calling, which is fine, and doesn't
        // break our other invariants that the stack is always allocated in
        // 16-bytes chunks.

        let mut next_xreg = 0;
        let mut next_vreg = 0;
        let mut next_stack: u64 = 0;

        let (max_per_class_reg_vals, mut remaining_reg_vals) = match args_or_rets {
            ArgsOrRets::Args => (8, 16), // x0-x7 and v0-v7

            // Note on return values: on the regular ABI, we may return values
            // in 8 registers for V128 and I64 registers independently of the
            // number of register values returned in the other class. That is,
            // we can return values in up to 8 integer and
            // 8 vector registers at once.
            //
            // In Wasmtime, we can only use one register for return
            // value for all the register classes. That is, we can't
            // return values in both one integer and one vector
            // register; only one return value may be in a register.
            ArgsOrRets::Rets => {
                if call_conv.extends_wasmtime() {
                    (1, 1) // x0 or v0, but not both
                } else {
                    (8, 16) // x0-x7 and v0-v7
                }
            }
        };

        for param in params {
            assert!(
                legal_type_for_machine(param.value_type),
                "Invalid type for AArch64: {:?}",
                param.value_type
            );

            let (rcs, reg_types) = Inst::rc_for_type(param.value_type)?;

            if let ir::ArgumentPurpose::StructArgument(size) = param.purpose {
                assert_eq!(args_or_rets, ArgsOrRets::Args);
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

            if let ir::ArgumentPurpose::StructReturn = param.purpose {
                // FIXME add assert_eq!(args_or_rets, ArgsOrRets::Args); once
                // ensure_struct_return_ptr_is_returned is gone.
                assert!(
                    param.value_type == types::I64,
                    "StructReturn must be a pointer sized integer"
                );
                args.push(ABIArg::Slots {
                    slots: smallvec![ABIArgSlot::Reg {
                        reg: xreg(8).to_real_reg().unwrap(),
                        ty: types::I64,
                        extension: param.extension,
                    },],
                    purpose: ir::ArgumentPurpose::StructReturn,
                });
                continue;
            }

            // Handle multi register params
            //
            // See AArch64 ABI (https://github.com/ARM-software/abi-aa/blob/2021Q1/aapcs64/aapcs64.rst#642parameter-passing-rules), (Section 6.4.2 Stage C).
            //
            // For arguments with alignment of 16 we round up the the register number
            // to the next even value. So we can never allocate for example an i128
            // to X1 and X2, we have to skip one register and do X2, X3
            // (Stage C.8)
            // Note: The Apple ABI deviates a bit here. They don't respect Stage C.8
            // and will happily allocate a i128 to X1 and X2
            //
            // For integer types with alignment of 16 we also have the additional
            // restriction of passing the lower half in Xn and the upper half in Xn+1
            // (Stage C.9)
            //
            // For examples of how LLVM handles this: https://godbolt.org/z/bhd3vvEfh
            //
            // On the Apple ABI it is unspecified if we can spill half the value into the stack
            // i.e load the lower half into x7 and the upper half into the stack
            // LLVM does not seem to do this, so we are going to replicate that behaviour
            let is_multi_reg = rcs.len() >= 2;
            if is_multi_reg {
                assert!(
                    rcs.len() == 2,
                    "Unable to handle multi reg params with more than 2 regs"
                );
                assert!(
                    rcs == &[RegClass::Int, RegClass::Int],
                    "Unable to handle non i64 regs"
                );

                let reg_class_space = max_per_class_reg_vals - next_xreg;
                let reg_space = remaining_reg_vals;

                if reg_space >= 2 && reg_class_space >= 2 {
                    // The aarch64 ABI does not allow us to start a split argument
                    // at an odd numbered register. So we need to skip one register
                    //
                    // TODO: The Fast ABI should probably not skip the register
                    if !is_apple_cc && next_xreg % 2 != 0 {
                        next_xreg += 1;
                    }

                    let lower_reg = xreg(next_xreg);
                    let upper_reg = xreg(next_xreg + 1);

                    args.push(ABIArg::Slots {
                        slots: smallvec![
                            ABIArgSlot::Reg {
                                reg: lower_reg.to_real_reg().unwrap(),
                                ty: reg_types[0],
                                extension: param.extension,
                            },
                            ABIArgSlot::Reg {
                                reg: upper_reg.to_real_reg().unwrap(),
                                ty: reg_types[1],
                                extension: param.extension,
                            },
                        ],
                        purpose: param.purpose,
                    });

                    next_xreg += 2;
                    remaining_reg_vals -= 2;
                    continue;
                }
            } else {
                // Single Register parameters
                let rc = rcs[0];
                let next_reg = match rc {
                    RegClass::Int => &mut next_xreg,
                    RegClass::Float => &mut next_vreg,
                };

                if *next_reg < max_per_class_reg_vals && remaining_reg_vals > 0 {
                    let reg = match rc {
                        RegClass::Int => xreg(*next_reg),
                        RegClass::Float => vreg(*next_reg),
                    };
                    // Overlay Z-regs on V-regs for parameter passing.
                    let ty = if param.value_type.is_dynamic_vector() {
                        dynamic_to_fixed(param.value_type)
                    } else {
                        param.value_type
                    };
                    args.push(ABIArg::reg(
                        reg.to_real_reg().unwrap(),
                        ty,
                        param.extension,
                        param.purpose,
                    ));
                    *next_reg += 1;
                    remaining_reg_vals -= 1;
                    continue;
                }
            }

            // Spill to the stack

            // Compute the stack slot's size.
            let size = (ty_bits(param.value_type) / 8) as u64;

            let size = if is_apple_cc
                || (call_conv.extends_wasmtime() && args_or_rets == ArgsOrRets::Rets)
            {
                // MacOS aarch64 and Wasmtime allow stack slots with
                // sizes less than 8 bytes. They still need to be
                // properly aligned on their natural data alignment,
                // though.
                size
            } else {
                // Every arg takes a minimum slot of 8 bytes. (16-byte stack
                // alignment happens separately after all args.)
                std::cmp::max(size, 8)
            };

            // Align the stack slot.
            debug_assert!(size.is_power_of_two());
            next_stack = align_to(next_stack, size);

            let slots = reg_types
                .iter()
                .copied()
                // Build the stack locations from each slot
                .scan(next_stack, |next_stack, ty| {
                    let slot_offset = *next_stack as i64;
                    *next_stack += (ty_bits(ty) / 8) as u64;

                    Some((ty, slot_offset))
                })
                .map(|(ty, offset)| ABIArgSlot::Stack {
                    offset,
                    ty,
                    extension: param.extension,
                })
                .collect();

            args.push(ABIArg::Slots {
                slots,
                purpose: param.purpose,
            });

            next_stack += size;
        }

        let extra_arg = if add_ret_area_ptr {
            debug_assert!(args_or_rets == ArgsOrRets::Args);
            if next_xreg < max_per_class_reg_vals && remaining_reg_vals > 0 {
                args.push(ABIArg::reg(
                    xreg(next_xreg).to_real_reg().unwrap(),
                    I64,
                    ir::ArgumentExtension::None,
                    ir::ArgumentPurpose::Normal,
                ));
            } else {
                args.push(ABIArg::stack(
                    next_stack as i64,
                    I64,
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

        // To avoid overflow issues, limit the arg/return size to something
        // reasonable -- here, 128 MB.
        if next_stack > STACK_ARG_RET_SIZE_LIMIT {
            return Err(CodegenError::ImplLimitExceeded);
        }

        Ok((next_stack as i64, extra_arg))
    }

    fn fp_to_arg_offset(_call_conv: isa::CallConv, _flags: &settings::Flags) -> i64 {
        16 // frame pointer + return address.
    }

    fn gen_load_stack(mem: StackAMode, into_reg: Writable<Reg>, ty: Type) -> Inst {
        Inst::gen_load(into_reg, mem.into(), ty, MemFlags::trusted())
    }

    fn gen_store_stack(mem: StackAMode, from_reg: Reg, ty: Type) -> Inst {
        Inst::gen_store(mem.into(), from_reg, ty, MemFlags::trusted())
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

    fn gen_args(_isa_flags: &aarch64_settings::Flags, args: Vec<ArgPair>) -> Inst {
        Inst::Args { args }
    }

    fn gen_ret(setup_frame: bool, isa_flags: &aarch64_settings::Flags, rets: Vec<RetPair>) -> Inst {
        if isa_flags.sign_return_address() && (setup_frame || isa_flags.sign_return_address_all()) {
            let key = if isa_flags.sign_return_address_with_bkey() {
                APIKey::B
            } else {
                APIKey::A
            };

            Inst::AuthenticatedRet {
                key,
                is_hint: !isa_flags.has_pauth(),
                rets,
            }
        } else {
            Inst::Ret { rets }
        }
    }

    fn gen_add_imm(into_reg: Writable<Reg>, from_reg: Reg, imm: u32) -> SmallInstVec<Inst> {
        let imm = imm as u64;
        let mut insts = SmallVec::new();
        if let Some(imm12) = Imm12::maybe_from_u64(imm) {
            insts.push(Inst::AluRRImm12 {
                alu_op: ALUOp::Add,
                size: OperandSize::Size64,
                rd: into_reg,
                rn: from_reg,
                imm12,
            });
        } else {
            let scratch2 = writable_tmp2_reg();
            assert_ne!(scratch2.to_reg(), from_reg);
            // `gen_add_imm` is only ever called after register allocation has take place, and as a
            // result it's ok to reuse the scratch2 register here. If that changes, we'll need to
            // plumb through a way to allocate temporary virtual registers
            insts.extend(Inst::load_constant(scratch2, imm.into(), &mut |_| scratch2));
            insts.push(Inst::AluRRRExtend {
                alu_op: ALUOp::Add,
                size: OperandSize::Size64,
                rd: into_reg,
                rn: from_reg,
                rm: scratch2.to_reg(),
                extendop: ExtendOp::UXTX,
            });
        }
        insts
    }

    fn gen_stack_lower_bound_trap(limit_reg: Reg) -> SmallInstVec<Inst> {
        let mut insts = SmallVec::new();
        insts.push(Inst::AluRRRExtend {
            alu_op: ALUOp::SubS,
            size: OperandSize::Size64,
            rd: writable_zero_reg(),
            rn: stack_reg(),
            rm: limit_reg,
            extendop: ExtendOp::UXTX,
        });
        insts.push(Inst::TrapIf {
            trap_code: ir::TrapCode::StackOverflow,
            // Here `Lo` == "less than" when interpreting the two
            // operands as unsigned integers.
            kind: CondBrKind::Cond(Cond::Lo),
        });
        insts
    }

    fn gen_get_stack_addr(mem: StackAMode, into_reg: Writable<Reg>, _ty: Type) -> Inst {
        // FIXME: Do something different for dynamic types?
        let mem = mem.into();
        Inst::LoadAddr { rd: into_reg, mem }
    }

    fn get_stacklimit_reg() -> Reg {
        spilltmp_reg()
    }

    fn gen_load_base_offset(into_reg: Writable<Reg>, base: Reg, offset: i32, ty: Type) -> Inst {
        let mem = AMode::RegOffset {
            rn: base,
            off: offset as i64,
            ty,
        };
        Inst::gen_load(into_reg, mem, ty, MemFlags::trusted())
    }

    fn gen_store_base_offset(base: Reg, offset: i32, from_reg: Reg, ty: Type) -> Inst {
        let mem = AMode::RegOffset {
            rn: base,
            off: offset as i64,
            ty,
        };
        Inst::gen_store(mem, from_reg, ty, MemFlags::trusted())
    }

    fn gen_sp_reg_adjust(amount: i32) -> SmallInstVec<Inst> {
        if amount == 0 {
            return SmallVec::new();
        }

        let (amount, is_sub) = if amount > 0 {
            (amount as u64, false)
        } else {
            (-amount as u64, true)
        };

        let alu_op = if is_sub { ALUOp::Sub } else { ALUOp::Add };

        let mut ret = SmallVec::new();
        if let Some(imm12) = Imm12::maybe_from_u64(amount) {
            let adj_inst = Inst::AluRRImm12 {
                alu_op,
                size: OperandSize::Size64,
                rd: writable_stack_reg(),
                rn: stack_reg(),
                imm12,
            };
            ret.push(adj_inst);
        } else {
            let tmp = writable_spilltmp_reg();
            // `gen_sp_reg_adjust` is called after regalloc2, so it's acceptable to reuse `tmp` for
            // intermediates in `load_constant`.
            let const_inst = Inst::load_constant(tmp, amount, &mut |_| tmp);
            let adj_inst = Inst::AluRRRExtend {
                alu_op,
                size: OperandSize::Size64,
                rd: writable_stack_reg(),
                rn: stack_reg(),
                rm: tmp.to_reg(),
                extendop: ExtendOp::UXTX,
            };
            ret.extend(const_inst);
            ret.push(adj_inst);
        }
        ret
    }

    fn gen_nominal_sp_adj(offset: i32) -> Inst {
        Inst::VirtualSPOffsetAdj {
            offset: offset as i64,
        }
    }

    fn gen_prologue_start(
        setup_frame: bool,
        call_conv: isa::CallConv,
        flags: &settings::Flags,
        isa_flags: &aarch64_settings::Flags,
    ) -> SmallInstVec<Inst> {
        let mut insts = SmallVec::new();

        if isa_flags.sign_return_address() && (setup_frame || isa_flags.sign_return_address_all()) {
            let key = if isa_flags.sign_return_address_with_bkey() {
                APIKey::B
            } else {
                APIKey::A
            };

            insts.push(Inst::Pacisp { key });

            if flags.unwind_info() {
                insts.push(Inst::Unwind {
                    inst: UnwindInst::Aarch64SetPointerAuth {
                        return_addresses: true,
                    },
                });
            }
        } else {
            if isa_flags.use_bti() {
                insts.push(Inst::Bti {
                    targets: BranchTargetType::C,
                });
            }

            if flags.unwind_info() && call_conv.extends_apple_aarch64() {
                // The macOS unwinder seems to require this.
                insts.push(Inst::Unwind {
                    inst: UnwindInst::Aarch64SetPointerAuth {
                        return_addresses: false,
                    },
                });
            }
        }

        insts
    }

    fn gen_prologue_frame_setup(flags: &settings::Flags) -> SmallInstVec<Inst> {
        let mut insts = SmallVec::new();

        // stp fp (x29), lr (x30), [sp, #-16]!
        insts.push(Inst::StoreP64 {
            rt: fp_reg(),
            rt2: link_reg(),
            mem: PairAMode::SPPreIndexed(SImm7Scaled::maybe_from_i64(-16, types::I64).unwrap()),
            flags: MemFlags::trusted(),
        });

        if flags.unwind_info() {
            insts.push(Inst::Unwind {
                inst: UnwindInst::PushFrameRegs {
                    offset_upward_to_caller_sp: 16, // FP, LR
                },
            });
        }

        // mov fp (x29), sp. This uses the ADDI rd, rs, 0 form of `MOV` because
        // the usual encoding (`ORR`) does not work with SP.
        insts.push(Inst::AluRRImm12 {
            alu_op: ALUOp::Add,
            size: OperandSize::Size64,
            rd: writable_fp_reg(),
            rn: stack_reg(),
            imm12: Imm12 {
                bits: 0,
                shift12: false,
            },
        });
        insts
    }

    fn gen_epilogue_frame_restore(_: &settings::Flags) -> SmallInstVec<Inst> {
        let mut insts = SmallVec::new();

        // N.B.: sp is already adjusted to the appropriate place by the
        // clobber-restore code (which also frees the fixed frame). Hence, there
        // is no need for the usual `mov sp, fp` here.

        // `ldp fp, lr, [sp], #16`
        insts.push(Inst::LoadP64 {
            rt: writable_fp_reg(),
            rt2: writable_link_reg(),
            mem: PairAMode::SPPostIndexed(SImm7Scaled::maybe_from_i64(16, types::I64).unwrap()),
            flags: MemFlags::trusted(),
        });
        insts
    }

    fn gen_probestack(_insts: &mut SmallInstVec<Self::I>, _: u32) {
        // TODO: implement if we ever require stack probes on an AArch64 host
        // (unlikely unless Lucet is ported)
        unimplemented!("Stack probing is unimplemented on AArch64");
    }

    fn gen_inline_probestack(insts: &mut SmallInstVec<Self::I>, frame_size: u32, guard_size: u32) {
        // The stack probe loop currently takes 6 instructions and each inline
        // probe takes 2 (ish, these numbers sort of depend on the constants).
        // Set this to 3 to keep the max size of the probe to 6 instructions.
        const PROBE_MAX_UNROLL: u32 = 3;

        let probe_count = align_to(frame_size, guard_size) / guard_size;
        if probe_count <= PROBE_MAX_UNROLL {
            // When manually unrolling stick an instruction that stores 0 at a
            // constant offset relative to the stack pointer. This will
            // turn into something like `movn tmp, #n ; stur xzr [sp, tmp]`.
            //
            // Note that this may actually store beyond the stack size for the
            // last item but that's ok since it's unused stack space and if
            // that faults accidentally we're so close to faulting it shouldn't
            // make too much difference to fault there.
            insts.reserve(probe_count as usize);
            for i in 0..probe_count {
                let offset = (guard_size * (i + 1)) as i64;
                insts.push(Self::gen_store_stack(
                    StackAMode::SPOffset(-offset, I8),
                    zero_reg(),
                    I32,
                ));
            }
        } else {
            // The non-unrolled version uses two temporary registers. The
            // `start` contains the current offset from sp and counts downwards
            // during the loop by increments of `guard_size`. The `end` is
            // the size of the frame and where we stop.
            //
            // Note that this emission is all post-regalloc so it should be ok
            // to use the temporary registers here as input/output as the loop
            // itself is not allowed to use the registers.
            let start = writable_spilltmp_reg();
            let end = writable_tmp2_reg();
            // `gen_inline_probestack` is called after regalloc2, so it's acceptable to reuse
            // `start` and `end` as temporaries in load_constant.
            insts.extend(Inst::load_constant(start, 0, &mut |_| start));
            insts.extend(Inst::load_constant(end, frame_size.into(), &mut |_| end));
            insts.push(Inst::StackProbeLoop {
                start,
                end: end.to_reg(),
                step: Imm12::maybe_from_u64(guard_size.into()).unwrap(),
            });
        }
    }

    // Returns stack bytes used as well as instructions. Does not adjust
    // nominal SP offset; abi generic code will do that.
    fn gen_clobber_save(
        _call_conv: isa::CallConv,
        setup_frame: bool,
        flags: &settings::Flags,
        clobbered_callee_saves: &[Writable<RealReg>],
        fixed_frame_storage_size: u32,
        _outgoing_args_size: u32,
    ) -> (u64, SmallVec<[Inst; 16]>) {
        let mut clobbered_int = vec![];
        let mut clobbered_vec = vec![];

        for &reg in clobbered_callee_saves.iter() {
            match reg.to_reg().class() {
                RegClass::Int => clobbered_int.push(reg),
                RegClass::Float => clobbered_vec.push(reg),
            }
        }

        let (int_save_bytes, vec_save_bytes) = saved_reg_stack_size(&clobbered_int, &clobbered_vec);
        let total_save_bytes = int_save_bytes + vec_save_bytes;
        let clobber_size = total_save_bytes as i32;
        let mut insts = SmallVec::new();

        if flags.unwind_info() && setup_frame {
            // The *unwind* frame (but not the actual frame) starts at the
            // clobbers, just below the saved FP/LR pair.
            insts.push(Inst::Unwind {
                inst: UnwindInst::DefineNewFrame {
                    offset_downward_to_clobbers: clobber_size as u32,
                    offset_upward_to_caller_sp: 16, // FP, LR
                },
            });
        }

        // We use pre-indexed addressing modes here, rather than the possibly
        // more efficient "subtract sp once then used fixed offsets" scheme,
        // because (i) we cannot necessarily guarantee that the offset of a
        // clobber-save slot will be within a SImm7Scaled (+504-byte) offset
        // range of the whole frame including other slots, it is more complex to
        // conditionally generate a two-stage SP adjustment (clobbers then fixed
        // frame) otherwise, and generally we just want to maintain simplicity
        // here for maintainability.  Because clobbers are at the top of the
        // frame, just below FP, all that is necessary is to use the pre-indexed
        // "push" `[sp, #-16]!` addressing mode.
        //
        // `frame_offset` tracks offset above start-of-clobbers for unwind-info
        // purposes.
        let mut clobber_offset = clobber_size as u32;
        let clobber_offset_change = 16;
        let iter = clobbered_int.chunks_exact(2);

        if let [rd] = iter.remainder() {
            let rd: Reg = rd.to_reg().into();

            debug_assert_eq!(rd.class(), RegClass::Int);
            // str rd, [sp, #-16]!
            insts.push(Inst::Store64 {
                rd,
                mem: AMode::SPPreIndexed {
                    simm9: SImm9::maybe_from_i64(-clobber_offset_change).unwrap(),
                },
                flags: MemFlags::trusted(),
            });

            if flags.unwind_info() {
                clobber_offset -= clobber_offset_change as u32;
                insts.push(Inst::Unwind {
                    inst: UnwindInst::SaveReg {
                        clobber_offset,
                        reg: rd.to_real_reg().unwrap(),
                    },
                });
            }
        }

        let mut iter = iter.rev();

        while let Some([rt, rt2]) = iter.next() {
            // .to_reg().into(): Writable<RealReg> --> RealReg --> Reg
            let rt: Reg = rt.to_reg().into();
            let rt2: Reg = rt2.to_reg().into();

            debug_assert!(rt.class() == RegClass::Int);
            debug_assert!(rt2.class() == RegClass::Int);

            // stp rt, rt2, [sp, #-16]!
            insts.push(Inst::StoreP64 {
                rt,
                rt2,
                mem: PairAMode::SPPreIndexed(
                    SImm7Scaled::maybe_from_i64(-clobber_offset_change, types::I64).unwrap(),
                ),
                flags: MemFlags::trusted(),
            });

            if flags.unwind_info() {
                clobber_offset -= clobber_offset_change as u32;
                insts.push(Inst::Unwind {
                    inst: UnwindInst::SaveReg {
                        clobber_offset,
                        reg: rt.to_real_reg().unwrap(),
                    },
                });
                insts.push(Inst::Unwind {
                    inst: UnwindInst::SaveReg {
                        clobber_offset: clobber_offset + (clobber_offset_change / 2) as u32,
                        reg: rt2.to_real_reg().unwrap(),
                    },
                });
            }
        }

        let store_vec_reg = |rd| Inst::FpuStore64 {
            rd,
            mem: AMode::SPPreIndexed {
                simm9: SImm9::maybe_from_i64(-clobber_offset_change).unwrap(),
            },
            flags: MemFlags::trusted(),
        };
        let iter = clobbered_vec.chunks_exact(2);

        if let [rd] = iter.remainder() {
            let rd: Reg = rd.to_reg().into();

            debug_assert_eq!(rd.class(), RegClass::Float);
            insts.push(store_vec_reg(rd));

            if flags.unwind_info() {
                clobber_offset -= clobber_offset_change as u32;
                insts.push(Inst::Unwind {
                    inst: UnwindInst::SaveReg {
                        clobber_offset,
                        reg: rd.to_real_reg().unwrap(),
                    },
                });
            }
        }

        let store_vec_reg_pair = |rt, rt2| {
            let clobber_offset_change = 16;

            (
                Inst::FpuStoreP64 {
                    rt,
                    rt2,
                    mem: PairAMode::SPPreIndexed(
                        SImm7Scaled::maybe_from_i64(-clobber_offset_change, F64).unwrap(),
                    ),
                    flags: MemFlags::trusted(),
                },
                clobber_offset_change as u32,
            )
        };
        let mut iter = iter.rev();

        while let Some([rt, rt2]) = iter.next() {
            let rt: Reg = rt.to_reg().into();
            let rt2: Reg = rt2.to_reg().into();

            debug_assert_eq!(rt.class(), RegClass::Float);
            debug_assert_eq!(rt2.class(), RegClass::Float);

            let (inst, clobber_offset_change) = store_vec_reg_pair(rt, rt2);

            insts.push(inst);

            if flags.unwind_info() {
                clobber_offset -= clobber_offset_change;
                insts.push(Inst::Unwind {
                    inst: UnwindInst::SaveReg {
                        clobber_offset,
                        reg: rt.to_real_reg().unwrap(),
                    },
                });
                insts.push(Inst::Unwind {
                    inst: UnwindInst::SaveReg {
                        clobber_offset: clobber_offset + clobber_offset_change / 2,
                        reg: rt2.to_real_reg().unwrap(),
                    },
                });
            }
        }

        // Allocate the fixed frame below the clobbers if necessary.
        if fixed_frame_storage_size > 0 {
            insts.extend(Self::gen_sp_reg_adjust(-(fixed_frame_storage_size as i32)));
        }

        (total_save_bytes as u64, insts)
    }

    fn gen_clobber_restore(
        _call_conv: isa::CallConv,
        sig: &Signature,
        flags: &settings::Flags,
        clobbers: &[Writable<RealReg>],
        fixed_frame_storage_size: u32,
        _outgoing_args_size: u32,
    ) -> SmallVec<[Inst; 16]> {
        let mut insts = SmallVec::new();
        let (clobbered_int, clobbered_vec) = get_regs_restored_in_epilogue(flags, sig, clobbers);

        // Free the fixed frame if necessary.
        if fixed_frame_storage_size > 0 {
            insts.extend(Self::gen_sp_reg_adjust(fixed_frame_storage_size as i32));
        }

        let load_vec_reg = |rd| Inst::FpuLoad64 {
            rd,
            mem: AMode::SPPostIndexed {
                simm9: SImm9::maybe_from_i64(16).unwrap(),
            },
            flags: MemFlags::trusted(),
        };
        let load_vec_reg_pair = |rt, rt2| Inst::FpuLoadP64 {
            rt,
            rt2,
            mem: PairAMode::SPPostIndexed(SImm7Scaled::maybe_from_i64(16, F64).unwrap()),
            flags: MemFlags::trusted(),
        };

        let mut iter = clobbered_vec.chunks_exact(2);

        while let Some([rt, rt2]) = iter.next() {
            let rt: Writable<Reg> = rt.map(|r| r.into());
            let rt2: Writable<Reg> = rt2.map(|r| r.into());

            debug_assert_eq!(rt.to_reg().class(), RegClass::Float);
            debug_assert_eq!(rt2.to_reg().class(), RegClass::Float);
            insts.push(load_vec_reg_pair(rt, rt2));
        }

        debug_assert!(iter.remainder().len() <= 1);

        if let [rd] = iter.remainder() {
            let rd: Writable<Reg> = rd.map(|r| r.into());

            debug_assert_eq!(rd.to_reg().class(), RegClass::Float);
            insts.push(load_vec_reg(rd));
        }

        let mut iter = clobbered_int.chunks_exact(2);

        while let Some([rt, rt2]) = iter.next() {
            let rt: Writable<Reg> = rt.map(|r| r.into());
            let rt2: Writable<Reg> = rt2.map(|r| r.into());

            debug_assert_eq!(rt.to_reg().class(), RegClass::Int);
            debug_assert_eq!(rt2.to_reg().class(), RegClass::Int);
            // ldp rt, rt2, [sp], #16
            insts.push(Inst::LoadP64 {
                rt,
                rt2,
                mem: PairAMode::SPPostIndexed(SImm7Scaled::maybe_from_i64(16, I64).unwrap()),
                flags: MemFlags::trusted(),
            });
        }

        debug_assert!(iter.remainder().len() <= 1);

        if let [rd] = iter.remainder() {
            let rd: Writable<Reg> = rd.map(|r| r.into());

            debug_assert_eq!(rd.to_reg().class(), RegClass::Int);
            // ldr rd, [sp], #16
            insts.push(Inst::ULoad64 {
                rd,
                mem: AMode::SPPostIndexed {
                    simm9: SImm9::maybe_from_i64(16).unwrap(),
                },
                flags: MemFlags::trusted(),
            });
        }

        insts
    }

    fn gen_call(
        dest: &CallDest,
        uses: CallArgList,
        defs: CallRetList,
        clobbers: PRegSet,
        opcode: ir::Opcode,
        tmp: Writable<Reg>,
        callee_conv: isa::CallConv,
        caller_conv: isa::CallConv,
    ) -> SmallVec<[Inst; 2]> {
        let mut insts = SmallVec::new();
        match &dest {
            &CallDest::ExtName(ref name, RelocDistance::Near) => insts.push(Inst::Call {
                info: Box::new(CallInfo {
                    dest: name.clone(),
                    uses,
                    defs,
                    clobbers,
                    opcode,
                    caller_callconv: caller_conv,
                    callee_callconv: callee_conv,
                }),
            }),
            &CallDest::ExtName(ref name, RelocDistance::Far) => {
                insts.push(Inst::LoadExtName {
                    rd: tmp,
                    name: Box::new(name.clone()),
                    offset: 0,
                });
                insts.push(Inst::CallInd {
                    info: Box::new(CallIndInfo {
                        rn: tmp.to_reg(),
                        uses,
                        defs,
                        clobbers,
                        opcode,
                        caller_callconv: caller_conv,
                        callee_callconv: callee_conv,
                    }),
                });
            }
            &CallDest::Reg(reg) => insts.push(Inst::CallInd {
                info: Box::new(CallIndInfo {
                    rn: *reg,
                    uses,
                    defs,
                    clobbers,
                    opcode,
                    caller_callconv: caller_conv,
                    callee_callconv: callee_conv,
                }),
            }),
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
        let arg0 = writable_xreg(0);
        let arg1 = writable_xreg(1);
        let arg2 = writable_xreg(2);
        let tmp = alloc_tmp(Self::word_type());
        insts.extend(Inst::load_constant(tmp, size as u64, &mut alloc_tmp));
        insts.push(Inst::Call {
            info: Box::new(CallInfo {
                dest: ExternalName::LibCall(LibCall::Memcpy),
                uses: smallvec![
                    CallArgPair {
                        vreg: dst,
                        preg: arg0.to_reg()
                    },
                    CallArgPair {
                        vreg: src,
                        preg: arg1.to_reg()
                    },
                    CallArgPair {
                        vreg: tmp.to_reg(),
                        preg: arg2.to_reg()
                    }
                ],
                defs: smallvec![],
                clobbers: Self::get_regs_clobbered_by_call(call_conv),
                opcode: Opcode::Call,
                caller_callconv: call_conv,
                callee_callconv: call_conv,
            }),
        });
        insts
    }

    fn get_number_of_spillslots_for_value(rc: RegClass, vector_size: u32) -> u32 {
        assert_eq!(vector_size % 8, 0);
        // We allocate in terms of 8-byte slots.
        match rc {
            RegClass::Int => 1,
            RegClass::Float => vector_size / 8,
        }
    }

    /// Get the current virtual-SP offset from an instruction-emission state.
    fn get_virtual_sp_offset_from_state(s: &EmitState) -> i64 {
        s.virtual_sp_offset
    }

    /// Get the nominal-SP-to-FP offset from an instruction-emission state.
    fn get_nominal_sp_to_fp(s: &EmitState) -> i64 {
        s.nominal_sp_to_fp
    }

    fn get_regs_clobbered_by_call(_call_conv_of_callee: isa::CallConv) -> PRegSet {
        DEFAULT_AAPCS_CLOBBERS
    }

    fn get_ext_mode(
        _call_conv: isa::CallConv,
        _specified: ir::ArgumentExtension,
    ) -> ir::ArgumentExtension {
        ir::ArgumentExtension::None
    }

    fn get_clobbered_callee_saves(
        _call_conv: isa::CallConv,
        flags: &settings::Flags,
        sig: &Signature,
        regs: &[Writable<RealReg>],
    ) -> Vec<Writable<RealReg>> {
        let mut regs: Vec<Writable<RealReg>> = regs
            .iter()
            .cloned()
            .filter(|r| is_reg_saved_in_prologue(flags.enable_pinned_reg(), sig, r.to_reg()))
            .collect();

        // Sort registers for deterministic code output. We can do an unstable
        // sort because the registers will be unique (there are no dups).
        regs.sort_unstable_by_key(|r| VReg::from(r.to_reg()).vreg());
        regs
    }

    fn is_frame_setup_needed(
        is_leaf: bool,
        stack_args_size: u32,
        num_clobbered_callee_saves: usize,
        fixed_frame_storage_size: u32,
    ) -> bool {
        !is_leaf
            // The function arguments that are passed on the stack are addressed
            // relative to the Frame Pointer.
            || stack_args_size > 0
            || num_clobbered_callee_saves > 0
            || fixed_frame_storage_size > 0
    }
}

/// Is this type supposed to be seen on this machine? E.g. references of the
/// wrong width are invalid.
fn legal_type_for_machine(ty: Type) -> bool {
    match ty {
        R32 => false,
        _ => true,
    }
}

/// Is the given register saved in the prologue if clobbered, i.e., is it a
/// callee-save?
fn is_reg_saved_in_prologue(enable_pinned_reg: bool, sig: &Signature, r: RealReg) -> bool {
    // FIXME: We need to inspect whether a function is returning Z or P regs too.
    let save_z_regs = sig
        .params
        .iter()
        .filter(|p| p.value_type.is_dynamic_vector())
        .count()
        != 0;

    match r.class() {
        RegClass::Int => {
            // x19 - x28 inclusive are callee-saves.
            // However, x21 is the pinned reg if `enable_pinned_reg`
            // is set, and is implicitly globally-allocated, hence not
            // callee-saved in prologues.
            if enable_pinned_reg && r.hw_enc() == PINNED_REG {
                false
            } else {
                r.hw_enc() >= 19 && r.hw_enc() <= 28
            }
        }
        RegClass::Float => {
            // If a subroutine takes at least one argument in scalable vector registers
            // or scalable predicate registers, or if it is a function that returns
            // results in such registers, it must ensure that the entire contents of
            // z8-z23 are preserved across the call. In other cases it need only
            // preserve the low 64 bits of z8-z15.
            if save_z_regs {
                r.hw_enc() >= 8 && r.hw_enc() <= 23
            } else {
                // v8 - v15 inclusive are callee-saves.
                r.hw_enc() >= 8 && r.hw_enc() <= 15
            }
        }
    }
}

/// Return the set of all integer and vector registers that must be saved in the
/// prologue and restored in the epilogue, given the set of all registers
/// written by the function's body.
fn get_regs_restored_in_epilogue(
    flags: &settings::Flags,
    sig: &Signature,
    regs: &[Writable<RealReg>],
) -> (Vec<Writable<RealReg>>, Vec<Writable<RealReg>>) {
    let mut int_saves = vec![];
    let mut vec_saves = vec![];
    for &reg in regs {
        if is_reg_saved_in_prologue(flags.enable_pinned_reg(), sig, reg.to_reg()) {
            match reg.to_reg().class() {
                RegClass::Int => int_saves.push(reg),
                RegClass::Float => vec_saves.push(reg),
            }
        }
    }
    // Sort registers for deterministic code output. We can do an unstable sort because the
    // registers will be unique (there are no dups).
    int_saves.sort_unstable_by_key(|r| VReg::from(r.to_reg()).vreg());
    vec_saves.sort_unstable_by_key(|r| VReg::from(r.to_reg()).vreg());
    (int_saves, vec_saves)
}

const fn default_aapcs_clobbers() -> PRegSet {
    PRegSet::empty()
        // x0 - x17 inclusive are caller-saves.
        .with(xreg_preg(0))
        .with(xreg_preg(1))
        .with(xreg_preg(2))
        .with(xreg_preg(3))
        .with(xreg_preg(4))
        .with(xreg_preg(5))
        .with(xreg_preg(6))
        .with(xreg_preg(7))
        .with(xreg_preg(8))
        .with(xreg_preg(9))
        .with(xreg_preg(10))
        .with(xreg_preg(11))
        .with(xreg_preg(12))
        .with(xreg_preg(13))
        .with(xreg_preg(14))
        .with(xreg_preg(15))
        .with(xreg_preg(16))
        .with(xreg_preg(17))
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
        .with(vreg_preg(0))
        .with(vreg_preg(1))
        .with(vreg_preg(2))
        .with(vreg_preg(3))
        .with(vreg_preg(4))
        .with(vreg_preg(5))
        .with(vreg_preg(6))
        .with(vreg_preg(7))
        .with(vreg_preg(8))
        .with(vreg_preg(9))
        .with(vreg_preg(10))
        .with(vreg_preg(11))
        .with(vreg_preg(12))
        .with(vreg_preg(13))
        .with(vreg_preg(14))
        .with(vreg_preg(15))
        .with(vreg_preg(16))
        .with(vreg_preg(17))
        .with(vreg_preg(18))
        .with(vreg_preg(19))
        .with(vreg_preg(20))
        .with(vreg_preg(21))
        .with(vreg_preg(22))
        .with(vreg_preg(23))
        .with(vreg_preg(24))
        .with(vreg_preg(25))
        .with(vreg_preg(26))
        .with(vreg_preg(27))
        .with(vreg_preg(28))
        .with(vreg_preg(29))
        .with(vreg_preg(30))
        .with(vreg_preg(31))
}

const DEFAULT_AAPCS_CLOBBERS: PRegSet = default_aapcs_clobbers();
