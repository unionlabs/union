//! Riscv64 ISA: binary code emission.

use crate::binemit::StackMap;
use crate::ir::RelSourceLoc;
use crate::ir::TrapCode;
use crate::isa::riscv64::inst::*;
use crate::isa::riscv64::inst::{zero_reg, AluOPRRR};
use crate::machinst::{AllocationConsumer, Reg, Writable};
use regalloc2::Allocation;

pub struct EmitInfo {
    shared_flag: settings::Flags,
    isa_flags: super::super::riscv_settings::Flags,
}

impl EmitInfo {
    pub(crate) fn new(
        shared_flag: settings::Flags,
        isa_flags: super::super::riscv_settings::Flags,
    ) -> Self {
        Self {
            shared_flag,
            isa_flags,
        }
    }
}

/// load constant by put the constant in the code stream.
/// calculate the pc and using load instruction.
#[derive(Clone, Copy)]
pub(crate) enum LoadConstant {
    U32(u32),
    U64(u64),
}

impl LoadConstant {
    fn to_le_bytes(self) -> Vec<u8> {
        match self {
            LoadConstant::U32(x) => Vec::from_iter(x.to_le_bytes().into_iter()),
            LoadConstant::U64(x) => Vec::from_iter(x.to_le_bytes().into_iter()),
        }
    }
    fn load_op(self) -> LoadOP {
        match self {
            LoadConstant::U32(_) => LoadOP::Lwu,
            LoadConstant::U64(_) => LoadOP::Ld,
        }
    }
    fn load_ty(self) -> Type {
        match self {
            LoadConstant::U32(_) => R32,
            LoadConstant::U64(_) => R64,
        }
    }

    pub(crate) fn load_constant(self, rd: Writable<Reg>) -> SmallInstVec<Inst> {
        let mut insts = SmallInstVec::new();
        // get current pc.
        insts.push(Inst::Auipc {
            rd,
            imm: Imm20 { bits: 0 },
        });
        // load
        insts.push(Inst::Load {
            rd,
            op: self.load_op(),
            flags: MemFlags::new(),
            from: AMode::RegOffset(rd.to_reg(), 12, self.load_ty()),
        });
        let data = self.to_le_bytes();
        // jump over.
        insts.push(Inst::Jal {
            dest: BranchTarget::ResolvedOffset(Inst::INSTRUCTION_SIZE + data.len() as i32),
        });
        insts.push(Inst::RawData { data });
        insts
    }

    // load and perform an extra add.
    pub(crate) fn load_constant_and_add(self, rd: Writable<Reg>, rs: Reg) -> SmallInstVec<Inst> {
        let mut insts = self.load_constant(rd);
        insts.push(Inst::AluRRR {
            alu_op: AluOPRRR::Add,
            rd: rd,
            rs1: rd.to_reg(),
            rs2: rs,
        });
        insts
    }
}

pub(crate) fn reg_to_gpr_num(m: Reg) -> u32 {
    u32::try_from(m.to_real_reg().unwrap().hw_enc() & 31).unwrap()
}

/// State carried between emissions of a sequence of instructions.
#[derive(Default, Clone, Debug)]
pub struct EmitState {
    pub(crate) virtual_sp_offset: i64,
    pub(crate) nominal_sp_to_fp: i64,
    /// Safepoint stack map for upcoming instruction, as provided to `pre_safepoint()`.
    stack_map: Option<StackMap>,
    /// Current source-code location corresponding to instruction to be emitted.
    cur_srcloc: RelSourceLoc,
}

impl EmitState {
    fn take_stack_map(&mut self) -> Option<StackMap> {
        self.stack_map.take()
    }

    fn clear_post_insn(&mut self) {
        self.stack_map = None;
    }

    fn cur_srcloc(&self) -> RelSourceLoc {
        self.cur_srcloc
    }
}

impl MachInstEmitState<Inst> for EmitState {
    fn new(abi: &Callee<crate::isa::riscv64::abi::Riscv64MachineDeps>) -> Self {
        EmitState {
            virtual_sp_offset: 0,
            nominal_sp_to_fp: abi.frame_size() as i64,
            stack_map: None,
            cur_srcloc: RelSourceLoc::default(),
        }
    }

    fn pre_safepoint(&mut self, stack_map: StackMap) {
        self.stack_map = Some(stack_map);
    }

    fn pre_sourceloc(&mut self, srcloc: RelSourceLoc) {
        self.cur_srcloc = srcloc;
    }
}

impl Inst {
    /// construct a "imm - rs".
    pub(crate) fn construct_imm_sub_rs(rd: Writable<Reg>, imm: u64, rs: Reg) -> SmallInstVec<Inst> {
        let mut insts = Inst::load_constant_u64(rd, imm);
        insts.push(Inst::AluRRR {
            alu_op: AluOPRRR::Sub,
            rd,
            rs1: rd.to_reg(),
            rs2: rs,
        });
        insts
    }

    /// Load int mask.
    /// If ty is int then 0xff in rd.
    pub(crate) fn load_int_mask(rd: Writable<Reg>, ty: Type) -> SmallInstVec<Inst> {
        let mut insts = SmallInstVec::new();
        assert!(ty.is_int() && ty.bits() <= 64);
        match ty {
            I64 => {
                insts.push(Inst::load_imm12(rd, Imm12::from_bits(-1)));
            }
            I32 | I16 => {
                insts.push(Inst::load_imm12(rd, Imm12::from_bits(-1)));
                insts.push(Inst::Extend {
                    rd: rd,
                    rn: rd.to_reg(),
                    signed: false,
                    from_bits: ty.bits() as u8,
                    to_bits: 64,
                });
            }
            I8 => {
                insts.push(Inst::load_imm12(rd, Imm12::from_bits(255)));
            }
            _ => unreachable!("ty:{:?}", ty),
        }
        insts
    }
    ///  inverse all bit
    pub(crate) fn construct_bit_not(rd: Writable<Reg>, rs: Reg) -> Inst {
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::Xori,
            rd,
            rs,
            imm12: Imm12::from_bits(-1),
        }
    }

    // emit a float is not a nan.
    pub(crate) fn emit_not_nan(rd: Writable<Reg>, rs: Reg, ty: Type) -> Inst {
        Inst::FpuRRR {
            alu_op: if ty == F32 {
                FpuOPRRR::FeqS
            } else {
                FpuOPRRR::FeqD
            },
            frm: None,
            rd: rd,
            rs1: rs,
            rs2: rs,
        }
    }

    pub(crate) fn emit_fabs(rd: Writable<Reg>, rs: Reg, ty: Type) -> Inst {
        Inst::FpuRRR {
            alu_op: if ty == F32 {
                FpuOPRRR::FsgnjxS
            } else {
                FpuOPRRR::FsgnjxD
            },
            frm: None,
            rd: rd,
            rs1: rs,
            rs2: rs,
        }
    }
    /// If a float is zero.
    pub(crate) fn emit_if_float_not_zero(
        tmp: Writable<Reg>,
        rs: Reg,
        ty: Type,
        taken: BranchTarget,
        not_taken: BranchTarget,
    ) -> SmallInstVec<Inst> {
        let mut insts = SmallInstVec::new();
        let class_op = if ty == F32 {
            FpuOPRR::FclassS
        } else {
            FpuOPRR::FclassD
        };
        insts.push(Inst::FpuRR {
            alu_op: class_op,
            frm: None,
            rd: tmp,
            rs: rs,
        });
        insts.push(Inst::AluRRImm12 {
            alu_op: AluOPRRI::Andi,
            rd: tmp,
            rs: tmp.to_reg(),
            imm12: Imm12::from_bits(FClassResult::is_zero_bits() as i16),
        });
        insts.push(Inst::CondBr {
            taken,
            not_taken,
            kind: IntegerCompare {
                kind: IntCC::Equal,
                rs1: tmp.to_reg(),
                rs2: zero_reg(),
            },
        });
        insts
    }
    pub(crate) fn emit_fneg(rd: Writable<Reg>, rs: Reg, ty: Type) -> Inst {
        Inst::FpuRRR {
            alu_op: if ty == F32 {
                FpuOPRRR::FsgnjnS
            } else {
                FpuOPRRR::FsgnjnD
            },
            frm: None,
            rd: rd,
            rs1: rs,
            rs2: rs,
        }
    }

    pub(crate) fn lower_br_fcmp(
        cc: FloatCC,
        x: Reg,
        y: Reg,
        taken: BranchTarget,
        not_taken: BranchTarget,
        ty: Type,
        tmp: Writable<Reg>,
    ) -> SmallInstVec<Inst> {
        assert!(tmp.to_reg().class() == RegClass::Int);
        let mut insts = SmallInstVec::new();
        let mut cc_args = FloatCCArgs::from_floatcc(cc);
        let eq_op = if ty == F32 {
            FpuOPRRR::FeqS
        } else {
            FpuOPRRR::FeqD
        };
        let lt_op = if ty == F32 {
            FpuOPRRR::FltS
        } else {
            FpuOPRRR::FltD
        };
        let le_op = if ty == F32 {
            FpuOPRRR::FleS
        } else {
            FpuOPRRR::FleD
        };

        // >=
        if cc_args.has_and_clear(FloatCCArgs::GT | FloatCCArgs::EQ) {
            insts.push(Inst::FpuRRR {
                frm: None,
                alu_op: le_op,
                rd: tmp,
                rs1: y, //  x and y order reversed.
                rs2: x,
            });
            insts.push(Inst::CondBr {
                taken: taken,
                not_taken: BranchTarget::zero(),
                kind: IntegerCompare {
                    kind: IntCC::NotEqual,
                    rs1: tmp.to_reg(),
                    rs2: zero_reg(),
                },
            });
        }

        // <=
        if cc_args.has_and_clear(FloatCCArgs::LT | FloatCCArgs::EQ) {
            insts.push(Inst::FpuRRR {
                frm: None,
                alu_op: le_op,
                rd: tmp,
                rs1: x,
                rs2: y,
            });
            insts.push(Inst::CondBr {
                taken: taken,
                not_taken: BranchTarget::zero(),
                kind: IntegerCompare {
                    kind: IntCC::NotEqual,
                    rs1: tmp.to_reg(),
                    rs2: zero_reg(),
                },
            });
        }

        // if eq
        if cc_args.has_and_clear(FloatCCArgs::EQ) {
            insts.push(Inst::FpuRRR {
                frm: None,
                alu_op: eq_op,
                rd: tmp,
                rs1: x,
                rs2: y,
            });
            insts.push(Inst::CondBr {
                taken: taken,
                not_taken: BranchTarget::zero(),
                kind: IntegerCompare {
                    kind: IntCC::NotEqual,
                    rs1: tmp.to_reg(),
                    rs2: zero_reg(),
                },
            });
        }
        // if ne
        if cc_args.has_and_clear(FloatCCArgs::NE) {
            insts.push(Inst::FpuRRR {
                frm: None,
                alu_op: eq_op,
                rd: tmp,
                rs1: x,
                rs2: y,
            });
            insts.push(Inst::CondBr {
                taken: taken,
                not_taken: BranchTarget::zero(),
                kind: IntegerCompare {
                    kind: IntCC::Equal,
                    rs1: tmp.to_reg(),
                    rs2: zero_reg(),
                },
            });
        }

        // if <
        if cc_args.has_and_clear(FloatCCArgs::LT) {
            insts.push(Inst::FpuRRR {
                frm: None,
                alu_op: lt_op,
                rd: tmp,
                rs1: x,
                rs2: y,
            });
            insts.push(Inst::CondBr {
                taken: taken,
                not_taken: BranchTarget::zero(),
                kind: IntegerCompare {
                    kind: IntCC::NotEqual,
                    rs1: tmp.to_reg(),
                    rs2: zero_reg(),
                },
            });
        }
        // if gt
        if cc_args.has_and_clear(FloatCCArgs::GT) {
            insts.push(Inst::FpuRRR {
                frm: None,
                alu_op: lt_op,
                rd: tmp,
                rs1: y, // x and y order reversed.
                rs2: x,
            });
            insts.push(Inst::CondBr {
                taken,
                not_taken: BranchTarget::zero(),
                kind: IntegerCompare {
                    kind: IntCC::NotEqual,
                    rs1: tmp.to_reg(),
                    rs2: zero_reg(),
                },
            });
        }
        // if unordered
        if cc_args.has_and_clear(FloatCCArgs::UN) {
            insts.extend(Inst::lower_float_unordered(tmp, ty, x, y, taken, not_taken));
        } else {
            //make sure we goto the not_taken.
            //finally goto not_taken
            insts.push(Inst::Jal { dest: not_taken });
        }
        // make sure we handle all cases.
        assert!(cc_args.0 == 0);
        insts
    }
    pub(crate) fn lower_br_icmp(
        cc: IntCC,
        a: ValueRegs<Reg>,
        b: ValueRegs<Reg>,
        taken: BranchTarget,
        not_taken: BranchTarget,
        ty: Type,
    ) -> SmallInstVec<Inst> {
        let mut insts = SmallInstVec::new();
        if ty.bits() <= 64 {
            let rs1 = a.only_reg().unwrap();
            let rs2 = b.only_reg().unwrap();
            let inst = Inst::CondBr {
                taken,
                not_taken,
                kind: IntegerCompare { kind: cc, rs1, rs2 },
            };
            insts.push(inst);
            return insts;
        }
        // compare i128
        let low = |cc: IntCC| -> IntegerCompare {
            IntegerCompare {
                rs1: a.regs()[0],
                rs2: b.regs()[0],
                kind: cc,
            }
        };
        let high = |cc: IntCC| -> IntegerCompare {
            IntegerCompare {
                rs1: a.regs()[1],
                rs2: b.regs()[1],
                kind: cc,
            }
        };
        match cc {
            IntCC::Equal => {
                // if high part not equal,
                // then we can go to not_taken otherwise fallthrough.
                insts.push(Inst::CondBr {
                    taken: not_taken,
                    not_taken: BranchTarget::zero(),
                    kind: high(IntCC::NotEqual),
                });
                // the rest part.
                insts.push(Inst::CondBr {
                    taken,
                    not_taken,
                    kind: low(IntCC::Equal),
                });
            }

            IntCC::NotEqual => {
                // if the high part not equal ,
                // we know the whole must be not equal,
                // we can goto the taken part , otherwise fallthrought.
                insts.push(Inst::CondBr {
                    taken,
                    not_taken: BranchTarget::zero(), //  no branch
                    kind: high(IntCC::NotEqual),
                });

                insts.push(Inst::CondBr {
                    taken,
                    not_taken,
                    kind: low(IntCC::NotEqual),
                });
            }
            IntCC::SignedGreaterThanOrEqual
            | IntCC::SignedLessThanOrEqual
            | IntCC::UnsignedGreaterThanOrEqual
            | IntCC::UnsignedLessThanOrEqual
            | IntCC::SignedGreaterThan
            | IntCC::SignedLessThan
            | IntCC::UnsignedLessThan
            | IntCC::UnsignedGreaterThan => {
                //
                insts.push(Inst::CondBr {
                    taken,
                    not_taken: BranchTarget::zero(),
                    kind: high(cc.without_equal()),
                });
                //
                insts.push(Inst::CondBr {
                    taken: not_taken,
                    not_taken: BranchTarget::zero(),
                    kind: high(IntCC::NotEqual),
                });
                insts.push(Inst::CondBr {
                    taken,
                    not_taken,
                    kind: low(cc.unsigned()),
                });
            }
        }
        insts
    }

    /// check if float is unordered.
    pub(crate) fn lower_float_unordered(
        tmp: Writable<Reg>,
        ty: Type,
        x: Reg,
        y: Reg,
        taken: BranchTarget,
        not_taken: BranchTarget,
    ) -> SmallInstVec<Inst> {
        let mut insts = SmallInstVec::new();
        let class_op = if ty == F32 {
            FpuOPRR::FclassS
        } else {
            FpuOPRR::FclassD
        };
        // if x is nan
        insts.push(Inst::FpuRR {
            frm: None,
            alu_op: class_op,
            rd: tmp,
            rs: x,
        });
        insts.push(Inst::AluRRImm12 {
            alu_op: AluOPRRI::Andi,
            rd: tmp,
            rs: tmp.to_reg(),
            imm12: Imm12::from_bits(FClassResult::is_nan_bits() as i16),
        });
        insts.push(Inst::CondBr {
            taken,
            not_taken: BranchTarget::zero(),
            kind: IntegerCompare {
                kind: IntCC::NotEqual,
                rs1: tmp.to_reg(),
                rs2: zero_reg(),
            },
        });
        // if y is nan.
        insts.push(Inst::FpuRR {
            frm: None,
            alu_op: class_op,
            rd: tmp,
            rs: y,
        });
        insts.push(Inst::AluRRImm12 {
            alu_op: AluOPRRI::Andi,
            rd: tmp,
            rs: tmp.to_reg(),
            imm12: Imm12::from_bits(FClassResult::is_nan_bits() as i16),
        });
        insts.push(Inst::CondBr {
            taken,
            not_taken,
            kind: IntegerCompare {
                kind: IntCC::NotEqual,
                rs1: tmp.to_reg(),
                rs2: zero_reg(),
            },
        });
        insts
    }
}

impl MachInstEmit for Inst {
    type State = EmitState;
    type Info = EmitInfo;

    fn emit(
        &self,
        allocs: &[Allocation],
        sink: &mut MachBuffer<Inst>,
        emit_info: &Self::Info,
        state: &mut EmitState,
    ) {
        let mut allocs = AllocationConsumer::new(allocs);
        // N.B.: we *must* not exceed the "worst-case size" used to compute
        // where to insert islands, except when islands are explicitly triggered
        // (with an `EmitIsland`). We check this in debug builds. This is `mut`
        // to allow disabling the check for `JTSequence`, which is always
        // emitted following an `EmitIsland`.
        let mut start_off = sink.cur_offset();
        match self {
            &Inst::Nop0 => {
                // do nothing
            }
            // Addi x0, x0, 0
            &Inst::Nop4 => {
                let x = Inst::AluRRImm12 {
                    alu_op: AluOPRRI::Addi,
                    rd: Writable::from_reg(zero_reg()),
                    rs: zero_reg(),
                    imm12: Imm12::zero(),
                };
                x.emit(&[], sink, emit_info, state)
            }
            &Inst::RawData { ref data } => {
                // emit_island if need, right now data is not very long.
                let length = data.len() as CodeOffset;
                if sink.island_needed(length) {
                    sink.emit_island(length);
                }
                sink.put_data(&data[..]);
                // safe to disable code length check.
                start_off = sink.cur_offset();
            }
            &Inst::Lui { rd, ref imm } => {
                let rd = allocs.next_writable(rd);
                let x: u32 = 0b0110111 | reg_to_gpr_num(rd.to_reg()) << 7 | (imm.as_u32() << 12);
                sink.put4(x);
            }
            &Inst::FpuRR {
                frm,
                alu_op,
                rd,
                rs,
            } => {
                let rs = allocs.next(rs);
                let rd = allocs.next_writable(rd);
                let x = alu_op.op_code()
                    | reg_to_gpr_num(rd.to_reg()) << 7
                    | alu_op.funct3(frm) << 12
                    | reg_to_gpr_num(rs) << 15
                    | alu_op.rs2_funct5() << 20
                    | alu_op.funct7() << 25;
                let srcloc = state.cur_srcloc();
                if !srcloc.is_default() && alu_op.is_convert_to_int() {
                    sink.add_trap(TrapCode::BadConversionToInteger);
                }
                sink.put4(x);
            }
            &Inst::FpuRRRR {
                alu_op,
                rd,
                rs1,
                rs2,
                rs3,
                frm,
            } => {
                let rs1 = allocs.next(rs1);
                let rs2 = allocs.next(rs2);
                let rs3 = allocs.next(rs3);
                let rd = allocs.next_writable(rd);
                let x = alu_op.op_code()
                    | reg_to_gpr_num(rd.to_reg()) << 7
                    | alu_op.funct3(frm) << 12
                    | reg_to_gpr_num(rs1) << 15
                    | reg_to_gpr_num(rs2) << 20
                    | alu_op.funct2() << 25
                    | reg_to_gpr_num(rs3) << 27;

                sink.put4(x);
            }
            &Inst::FpuRRR {
                alu_op,
                frm,
                rd,
                rs1,
                rs2,
            } => {
                let rs1 = allocs.next(rs1);
                let rs2 = allocs.next(rs2);
                let rd = allocs.next_writable(rd);

                let x: u32 = alu_op.op_code()
                    | reg_to_gpr_num(rd.to_reg()) << 7
                    | (alu_op.funct3(frm)) << 12
                    | reg_to_gpr_num(rs1) << 15
                    | reg_to_gpr_num(rs2) << 20
                    | alu_op.funct7() << 25;
                sink.put4(x);
            }
            &Inst::Unwind { ref inst } => {
                sink.add_unwind(inst.clone());
            }
            &Inst::DummyUse { reg } => {
                allocs.next(reg);
            }
            &Inst::AluRRR {
                alu_op,
                rd,
                rs1,
                rs2,
            } => {
                let rs1 = allocs.next(rs1);
                let rs2 = allocs.next(rs2);
                let rd = allocs.next_writable(rd);
                let (rs1, rs2) = if alu_op.reverse_rs() {
                    (rs2, rs1)
                } else {
                    (rs1, rs2)
                };

                let x: u32 = alu_op.op_code()
                    | reg_to_gpr_num(rd.to_reg()) << 7
                    | (alu_op.funct3()) << 12
                    | reg_to_gpr_num(rs1) << 15
                    | reg_to_gpr_num(rs2) << 20
                    | alu_op.funct7() << 25;
                sink.put4(x);
            }
            &Inst::AluRRImm12 {
                alu_op,
                rd,
                rs,
                imm12,
            } => {
                let rs = allocs.next(rs);
                let rd = allocs.next_writable(rd);
                let x = alu_op.op_code()
                    | reg_to_gpr_num(rd.to_reg()) << 7
                    | alu_op.funct3() << 12
                    | reg_to_gpr_num(rs) << 15
                    | alu_op.imm12(imm12) << 20;
                sink.put4(x);
            }
            &Inst::Load {
                rd,
                op,
                from,
                flags,
            } => {
                let x;
                let base = from.get_base_register();
                let base = allocs.next(base);
                let rd = allocs.next_writable(rd);
                let offset = from.get_offset_with_state(state);
                if let Some(imm12) = Imm12::maybe_from_u64(offset as u64) {
                    let srcloc = state.cur_srcloc();
                    if !srcloc.is_default() && !flags.notrap() {
                        // Register the offset at which the actual load instruction starts.
                        sink.add_trap(TrapCode::HeapOutOfBounds);
                    }
                    x = op.op_code()
                        | reg_to_gpr_num(rd.to_reg()) << 7
                        | op.funct3() << 12
                        | reg_to_gpr_num(base) << 15
                        | (imm12.as_u32()) << 20;
                    sink.put4(x);
                } else {
                    let tmp = writable_spilltmp_reg();
                    let mut insts =
                        LoadConstant::U64(offset as u64).load_constant_and_add(tmp, base);
                    let srcloc = state.cur_srcloc();
                    if !srcloc.is_default() && !flags.notrap() {
                        // Register the offset at which the actual load instruction starts.
                        sink.add_trap(TrapCode::HeapOutOfBounds);
                    }
                    insts.push(Inst::Load {
                        op,
                        from: AMode::RegOffset(tmp.to_reg(), 0, I64),
                        rd,
                        flags,
                    });
                    insts
                        .into_iter()
                        .for_each(|inst| inst.emit(&[], sink, emit_info, state));
                }
            }
            &Inst::Store { op, src, flags, to } => {
                let base = allocs.next(to.get_base_register());
                let src = allocs.next(src);
                let offset = to.get_offset_with_state(state);
                let x;
                if let Some(imm12) = Imm12::maybe_from_u64(offset as u64) {
                    let srcloc = state.cur_srcloc();
                    if !srcloc.is_default() && !flags.notrap() {
                        // Register the offset at which the actual load instruction starts.
                        sink.add_trap(TrapCode::HeapOutOfBounds);
                    }
                    x = op.op_code()
                        | (imm12.as_u32() & 0x1f) << 7
                        | op.funct3() << 12
                        | reg_to_gpr_num(base) << 15
                        | reg_to_gpr_num(src) << 20
                        | (imm12.as_u32() >> 5) << 25;
                    sink.put4(x);
                } else {
                    let tmp = writable_spilltmp_reg();
                    let mut insts =
                        LoadConstant::U64(offset as u64).load_constant_and_add(tmp, base);
                    let srcloc = state.cur_srcloc();
                    if !srcloc.is_default() && !flags.notrap() {
                        // Register the offset at which the actual load instruction starts.
                        sink.add_trap(TrapCode::HeapOutOfBounds);
                    }
                    insts.push(Inst::Store {
                        op,
                        to: AMode::RegOffset(tmp.to_reg(), 0, I64),
                        flags,
                        src,
                    });
                    insts
                        .into_iter()
                        .for_each(|inst| inst.emit(&[], sink, emit_info, state));
                }
            }

            &Inst::ReferenceCheck { rd, op, x } => {
                let x = allocs.next(x);
                let rd = allocs.next_writable(rd);
                let mut insts = SmallInstVec::new();
                match op {
                    ReferenceCheckOP::IsNull => {
                        insts.push(Inst::CondBr {
                            taken: BranchTarget::ResolvedOffset(Inst::INSTRUCTION_SIZE * 3),
                            not_taken: BranchTarget::zero(),
                            kind: IntegerCompare {
                                kind: IntCC::Equal,
                                rs1: zero_reg(),
                                rs2: x,
                            },
                        });
                        // here is false
                        insts.push(Inst::load_imm12(rd, Imm12::FALSE));
                        insts.push(Inst::Jal {
                            dest: BranchTarget::ResolvedOffset(Inst::INSTRUCTION_SIZE * 2),
                        });
                        // here is true
                        insts.push(Inst::load_imm12(rd, Imm12::TRUE));
                    }

                    ReferenceCheckOP::IsInvalid => {
                        // todo:: right now just check if it is null
                        // null is a valid reference??????
                        insts.push(Inst::CondBr {
                            taken: BranchTarget::ResolvedOffset(Inst::INSTRUCTION_SIZE * 3),
                            not_taken: BranchTarget::zero(),
                            kind: IntegerCompare {
                                kind: IntCC::Equal,
                                rs1: zero_reg(),
                                rs2: x,
                            },
                        });
                        // here is false
                        insts.push(Inst::load_imm12(rd, Imm12::FALSE));
                        insts.push(Inst::Jal {
                            dest: BranchTarget::ResolvedOffset(Inst::INSTRUCTION_SIZE * 2),
                        });
                        // here is true
                        insts.push(Inst::load_imm12(rd, Imm12::TRUE));
                    }
                }

                insts
                    .into_iter()
                    .for_each(|i| i.emit(&[], sink, emit_info, state));
            }
            &Inst::Args { .. } => {
                // Nothing: this is a pseudoinstruction that serves
                // only to constrain registers at a certain point.
            }
            &Inst::Ret { .. } => {
                //jalr x0, x1, 0
                let x: u32 = (0b1100111) | (1 << 15);
                sink.put4(x);
            }

            &Inst::Extend {
                rd,
                rn,
                signed,
                from_bits,
                to_bits: _to_bits,
            } => {
                let rn = allocs.next(rn);
                let rd = allocs.next_writable(rd);
                let mut insts = SmallInstVec::new();
                let shift_bits = (64 - from_bits) as i16;
                let is_u8 = || from_bits == 8 && signed == false;
                if is_u8() {
                    // special for u8.
                    insts.push(Inst::AluRRImm12 {
                        alu_op: AluOPRRI::Andi,
                        rd,
                        rs: rn,
                        imm12: Imm12::from_bits(255),
                    });
                } else {
                    insts.push(Inst::AluRRImm12 {
                        alu_op: AluOPRRI::Slli,
                        rd,
                        rs: rn,
                        imm12: Imm12::from_bits(shift_bits),
                    });
                    insts.push(Inst::AluRRImm12 {
                        alu_op: if signed {
                            AluOPRRI::Srai
                        } else {
                            AluOPRRI::Srli
                        },
                        rd,
                        rs: rd.to_reg(),
                        imm12: Imm12::from_bits(shift_bits),
                    });
                }
                insts
                    .into_iter()
                    .for_each(|i| i.emit(&[], sink, emit_info, state));
            }
            &Inst::AjustSp { amount } => {
                if let Some(imm) = Imm12::maybe_from_u64(amount as u64) {
                    Inst::AluRRImm12 {
                        alu_op: AluOPRRI::Addi,
                        rd: writable_stack_reg(),
                        rs: stack_reg(),
                        imm12: imm,
                    }
                    .emit(&[], sink, emit_info, state);
                } else {
                    let tmp = writable_spilltmp_reg();
                    let mut insts = Inst::load_constant_u64(tmp, amount as u64);
                    insts.push(Inst::AluRRR {
                        alu_op: AluOPRRR::Add,
                        rd: writable_stack_reg(),
                        rs1: tmp.to_reg(),
                        rs2: stack_reg(),
                    });
                    insts
                        .into_iter()
                        .for_each(|i| i.emit(&[], sink, emit_info, state));
                }
            }
            &Inst::Call { ref info } => {
                // call
                match info.dest {
                    ExternalName::User { .. } => {
                        if info.opcode.is_call() {
                            sink.add_call_site(info.opcode);
                        }
                        sink.add_reloc(Reloc::RiscvCall, &info.dest, 0);
                        if let Some(s) = state.take_stack_map() {
                            sink.add_stack_map(StackMapExtent::UpcomingBytes(8), s);
                        }
                        Inst::construct_auipc_and_jalr(
                            Some(writable_link_reg()),
                            writable_link_reg(),
                            0,
                        )
                        .into_iter()
                        .for_each(|i| i.emit(&[], sink, emit_info, state));
                    }
                    ExternalName::LibCall(..)
                    | ExternalName::TestCase { .. }
                    | ExternalName::KnownSymbol(..) => {
                        // use indirect call. it is more simple.
                        // load ext name.
                        Inst::LoadExtName {
                            rd: writable_spilltmp_reg2(),
                            name: Box::new(info.dest.clone()),
                            offset: 0,
                        }
                        .emit(&[], sink, emit_info, state);

                        if let Some(s) = state.take_stack_map() {
                            sink.add_stack_map(StackMapExtent::UpcomingBytes(4), s);
                        }
                        if info.opcode.is_call() {
                            sink.add_call_site(info.opcode);
                        }
                        // call
                        Inst::Jalr {
                            rd: writable_link_reg(),
                            base: spilltmp_reg2(),
                            offset: Imm12::zero(),
                        }
                        .emit(&[], sink, emit_info, state);
                    }
                }
            }
            &Inst::CallInd { ref info } => {
                let rn = allocs.next(info.rn);
                if let Some(s) = state.take_stack_map() {
                    sink.add_stack_map(StackMapExtent::UpcomingBytes(4), s);
                }

                if info.opcode.is_call() {
                    sink.add_call_site(info.opcode);
                }
                Inst::Jalr {
                    rd: writable_link_reg(),
                    base: rn,
                    offset: Imm12::zero(),
                }
                .emit(&[], sink, emit_info, state);
            }

            &Inst::Jal { dest } => {
                let code: u32 = 0b1101111;
                match dest {
                    BranchTarget::Label(lable) => {
                        sink.use_label_at_offset(start_off, lable, LabelUse::Jal20);
                        sink.add_uncond_branch(start_off, start_off + 4, lable);
                        sink.put4(code);
                    }
                    BranchTarget::ResolvedOffset(offset) => {
                        let offset = offset as i64;
                        if offset != 0 {
                            if LabelUse::Jal20.offset_in_range(offset) {
                                let mut code = code.to_le_bytes();
                                LabelUse::Jal20.patch_raw_offset(&mut code, offset);
                                sink.put_data(&code[..]);
                            } else {
                                Inst::construct_auipc_and_jalr(
                                    None,
                                    writable_spilltmp_reg(),
                                    offset,
                                )
                                .into_iter()
                                .for_each(|i| i.emit(&[], sink, emit_info, state));
                            }
                        } else {
                            // CondBr often generate Jal {dest : 0}, means otherwise no jump.
                        }
                    }
                }
            }
            &Inst::CondBr {
                taken,
                not_taken,
                mut kind,
            } => {
                kind.rs1 = allocs.next(kind.rs1);
                kind.rs2 = allocs.next(kind.rs2);
                match taken {
                    BranchTarget::Label(label) => {
                        let code = kind.emit();
                        let code_inverse = kind.inverse().emit().to_le_bytes();
                        sink.use_label_at_offset(start_off, label, LabelUse::B12);
                        sink.add_cond_branch(start_off, start_off + 4, label, &code_inverse);
                        sink.put4(code);
                    }
                    BranchTarget::ResolvedOffset(offset) => {
                        assert!(offset != 0);
                        if LabelUse::B12.offset_in_range(offset as i64) {
                            let code = kind.emit();
                            let mut code = code.to_le_bytes();
                            LabelUse::B12.patch_raw_offset(&mut code, offset as i64);
                            sink.put_data(&code[..])
                        } else {
                            let mut code = kind.emit().to_le_bytes();
                            // jump over the condbr , 4 bytes.
                            LabelUse::B12.patch_raw_offset(&mut code[..], 4);
                            sink.put_data(&code[..]);
                            Inst::construct_auipc_and_jalr(
                                None,
                                writable_spilltmp_reg(),
                                offset as i64,
                            )
                            .into_iter()
                            .for_each(|i| i.emit(&[], sink, emit_info, state));
                        }
                    }
                }
                Inst::Jal { dest: not_taken }.emit(&[], sink, emit_info, state);
            }

            &Inst::Mov { rd, rm, ty } => {
                if rd.to_reg() != rm {
                    let rm = allocs.next(rm);
                    let rd = allocs.next_writable(rd);
                    if ty.is_float() {
                        Inst::FpuRRR {
                            alu_op: if ty == F32 {
                                FpuOPRRR::FsgnjS
                            } else {
                                FpuOPRRR::FsgnjD
                            },
                            frm: None,
                            rd: rd,
                            rs1: rm,
                            rs2: rm,
                        }
                        .emit(&[], sink, emit_info, state);
                    } else {
                        let x = Inst::AluRRImm12 {
                            alu_op: AluOPRRI::Ori,
                            rd: rd,
                            rs: rm,
                            imm12: Imm12::zero(),
                        };
                        x.emit(&[], sink, emit_info, state);
                    }
                }
            }
            &Inst::BrTableCheck {
                index,
                targets_len,
                default_,
            } => {
                let index = allocs.next(index);
                // load
                Inst::load_constant_u32(writable_spilltmp_reg(), targets_len as u64)
                    .iter()
                    .for_each(|i| i.emit(&[], sink, emit_info, state));
                Inst::CondBr {
                    taken: BranchTarget::offset(Inst::INSTRUCTION_SIZE * 3),
                    not_taken: BranchTarget::zero(),
                    kind: IntegerCompare {
                        kind: IntCC::UnsignedLessThan,
                        rs1: index,
                        rs2: spilltmp_reg(),
                    },
                }
                .emit(&[], sink, emit_info, state);
                sink.use_label_at_offset(
                    sink.cur_offset(),
                    default_.as_label().unwrap(),
                    LabelUse::PCRel32,
                );
                Inst::construct_auipc_and_jalr(None, writable_spilltmp_reg(), 0)
                    .iter()
                    .for_each(|i| i.emit(&[], sink, emit_info, state));
            }
            &Inst::BrTable {
                index,
                tmp1,
                ref targets,
            } => {
                let index = allocs.next(index);
                let tmp1 = allocs.next_writable(tmp1);
                let mut insts = SmallInstVec::new();
                // get current pc.
                insts.push(Inst::Auipc {
                    rd: tmp1,
                    imm: Imm20::from_bits(0),
                });
                // t *= 8; very jump that I emit is 8 byte size.
                insts.push(Inst::AluRRImm12 {
                    alu_op: AluOPRRI::Slli,
                    rd: writable_spilltmp_reg(),
                    rs: index,
                    imm12: Imm12::from_bits(3),
                });
                // tmp1 += t
                insts.push(Inst::AluRRR {
                    alu_op: AluOPRRR::Add,
                    rd: tmp1,
                    rs1: tmp1.to_reg(),
                    rs2: spilltmp_reg(),
                });
                insts.push(Inst::Jalr {
                    rd: writable_zero_reg(),
                    base: tmp1.to_reg(),
                    offset: Imm12::from_bits(16),
                });

                // here is all the jumps.
                let mut need_label_use = vec![];
                for t in targets {
                    need_label_use.push((insts.len(), t.clone()));
                    insts.extend(Inst::construct_auipc_and_jalr(
                        None,
                        writable_spilltmp_reg(),
                        0,
                    ));
                }
                // emit island if need.
                let distance = (insts.len() * 4) as u32;
                if sink.island_needed(distance) {
                    sink.emit_island(distance);
                }
                let mut need_label_use = &need_label_use[..];
                insts.into_iter().enumerate().for_each(|(index, inst)| {
                    if !need_label_use.is_empty() && need_label_use[0].0 == index {
                        sink.use_label_at_offset(
                            sink.cur_offset(),
                            need_label_use[0].1.as_label().unwrap(),
                            LabelUse::PCRel32,
                        );
                        need_label_use = &need_label_use[1..];
                    }
                    inst.emit(&[], sink, emit_info, state);
                });
                // emit the island before, so we can safely
                // disable the worst-case-size check in this case.
                start_off = sink.cur_offset();
            }

            &Inst::VirtualSPOffsetAdj { amount } => {
                log::trace!(
                    "virtual sp offset adjusted by {} -> {}",
                    amount,
                    state.virtual_sp_offset + amount
                );
                state.virtual_sp_offset += amount;
            }
            &Inst::Atomic {
                op,
                rd,
                addr,
                src,
                amo,
            } => {
                let addr = allocs.next(addr);
                let src = allocs.next(src);
                let rd = allocs.next_writable(rd);
                let srcloc = state.cur_srcloc();
                if !srcloc.is_default() {
                    sink.add_trap(TrapCode::HeapOutOfBounds);
                }
                let x = op.op_code()
                    | reg_to_gpr_num(rd.to_reg()) << 7
                    | op.funct3() << 12
                    | reg_to_gpr_num(addr) << 15
                    | reg_to_gpr_num(src) << 20
                    | op.funct7(amo) << 25;

                sink.put4(x);
            }
            &Inst::Fence { pred, succ } => {
                let x = 0b0001111
                    | 0b00000 << 7
                    | 0b000 << 12
                    | 0b00000 << 15
                    | (succ as u32) << 20
                    | (pred as u32) << 24;

                sink.put4(x);
            }
            &Inst::FenceI => sink.put4(0x0000100f),
            &Inst::Auipc { rd, imm } => {
                let rd = allocs.next_writable(rd);
                let x = enc_auipc(rd, imm);
                sink.put4(x);
            }

            &Inst::LoadAddr { rd, mem } => {
                let base = mem.get_base_register();
                let base = allocs.next(base);
                let rd = allocs.next_writable(rd);
                let offset = mem.get_offset_with_state(state);
                if let Some(offset) = Imm12::maybe_from_u64(offset as u64) {
                    Inst::AluRRImm12 {
                        alu_op: AluOPRRI::Addi,
                        rd: rd,
                        rs: base,
                        imm12: offset,
                    }
                    .emit(&[], sink, emit_info, state);
                } else {
                    let insts = LoadConstant::U64(offset as u64).load_constant_and_add(rd, base);
                    insts
                        .into_iter()
                        .for_each(|i| i.emit(&[], sink, emit_info, state));
                }
            }

            &Inst::Fcmp {
                rd,
                cc,
                ty,
                rs1,
                rs2,
            } => {
                let rs1 = allocs.next(rs1);
                let rs2 = allocs.next(rs2);
                let rd = allocs.next_writable(rd);
                let label_true = sink.get_label();
                let label_jump_over = sink.get_label();
                Inst::lower_br_fcmp(
                    cc,
                    rs1,
                    rs2,
                    BranchTarget::Label(label_true),
                    BranchTarget::zero(),
                    ty,
                    rd,
                )
                .iter()
                .for_each(|i| i.emit(&[], sink, emit_info, state));
                // here is not taken.
                Inst::load_imm12(rd, Imm12::FALSE).emit(&[], sink, emit_info, state);
                // jump over.
                Inst::Jal {
                    dest: BranchTarget::Label(label_jump_over),
                }
                .emit(&[], sink, emit_info, state);
                // here is true
                sink.bind_label(label_true);
                Inst::load_imm12(rd, Imm12::TRUE).emit(&[], sink, emit_info, state);
                sink.bind_label(label_jump_over);
            }

            &Inst::Select {
                ref dst,
                condition,
                ref x,
                ref y,
                ty: _ty,
            } => {
                let condition = allocs.next(condition);
                let x = alloc_value_regs(x, &mut allocs);
                let y = alloc_value_regs(y, &mut allocs);
                let dst: Vec<_> = dst
                    .clone()
                    .into_iter()
                    .map(|r| allocs.next_writable(r))
                    .collect();

                let mut insts = SmallInstVec::new();
                let label_false = sink.get_label();
                insts.push(Inst::CondBr {
                    taken: BranchTarget::Label(label_false),
                    not_taken: BranchTarget::zero(),
                    kind: IntegerCompare {
                        kind: IntCC::Equal,
                        rs1: condition,
                        rs2: zero_reg(),
                    },
                });
                // here is the true
                // select the first value
                insts.extend(gen_moves(&dst[..], x.regs()));
                let label_jump_over = sink.get_label();
                insts.push(Inst::Jal {
                    dest: BranchTarget::Label(label_jump_over),
                });
                // here is false
                insts
                    .drain(..)
                    .for_each(|i: Inst| i.emit(&[], sink, emit_info, state));
                sink.bind_label(label_false);
                // select second value1
                insts.extend(gen_moves(&dst[..], y.regs()));
                insts
                    .into_iter()
                    .for_each(|i| i.emit(&[], sink, emit_info, state));
                sink.bind_label(label_jump_over);
            }
            &Inst::Jalr { rd, base, offset } => {
                let rd = allocs.next_writable(rd);
                let x = enc_jalr(rd, base, offset);
                sink.put4(x);
            }
            &Inst::ECall => {
                sink.put4(0x00000073);
            }
            &Inst::EBreak => {
                sink.put4(0x00100073);
            }
            &Inst::Icmp {
                cc,
                rd,
                ref a,
                ref b,
                ty,
            } => {
                let a = alloc_value_regs(a, &mut allocs);
                let b = alloc_value_regs(b, &mut allocs);
                let rd = allocs.next_writable(rd);
                let label_true = sink.get_label();
                let label_false = sink.get_label();
                Inst::lower_br_icmp(
                    cc,
                    a,
                    b,
                    BranchTarget::Label(label_true),
                    BranchTarget::Label(label_false),
                    ty,
                )
                .into_iter()
                .for_each(|i| i.emit(&[], sink, emit_info, state));

                sink.bind_label(label_true);
                Inst::load_imm12(rd, Imm12::TRUE).emit(&[], sink, emit_info, state);
                Inst::Jal {
                    dest: BranchTarget::offset(Inst::INSTRUCTION_SIZE * 2),
                }
                .emit(&[], sink, emit_info, state);
                sink.bind_label(label_false);
                Inst::load_imm12(rd, Imm12::FALSE).emit(&[], sink, emit_info, state);
            }
            &Inst::AtomicCas {
                offset,
                t0,
                dst,
                e,
                addr,
                v,
                ty,
            } => {
                let offset = allocs.next(offset);
                let e = allocs.next(e);
                let addr = allocs.next(addr);
                let v = allocs.next(v);
                let t0 = allocs.next_writable(t0);
                let dst = allocs.next_writable(dst);

                //     # addr holds address of memory location
                //     # e holds expected value
                //     # v holds desired value
                //     # dst holds return value
                // cas:
                //     lr.w dst, (addr)       # Load original value.
                //     bne dst, e, fail       # Doesn’t match, so fail.
                //     sc.w t0, v, (addr)     # Try to update.
                //     bnez t0 , cas          # if store not ok,retry.
                // fail:
                let fail_label = sink.get_label();
                let cas_lebel = sink.get_label();
                sink.bind_label(cas_lebel);
                Inst::Atomic {
                    op: AtomicOP::load_op(ty),
                    rd: dst,
                    addr,
                    src: zero_reg(),
                    amo: AMO::SeqCst,
                }
                .emit(&[], sink, emit_info, state);
                let origin_value = if ty.bits() < 32 {
                    AtomicOP::extract(t0, offset, dst.to_reg(), ty)
                        .iter()
                        .for_each(|i| i.emit(&[], sink, emit_info, state));
                    t0.to_reg()
                } else if ty.bits() == 32 {
                    Inst::Extend {
                        rd: t0,
                        rn: dst.to_reg(),
                        signed: false,
                        from_bits: 32,
                        to_bits: 64,
                    }
                    .emit(&[], sink, emit_info, state);
                    t0.to_reg()
                } else {
                    dst.to_reg()
                };
                Inst::CondBr {
                    taken: BranchTarget::Label(fail_label),
                    not_taken: BranchTarget::zero(),
                    kind: IntegerCompare {
                        kind: IntCC::NotEqual,
                        rs1: e,
                        rs2: origin_value,
                    },
                }
                .emit(&[], sink, emit_info, state);
                let store_value = if ty.bits() < 32 {
                    // reload value to t0.
                    Inst::Atomic {
                        op: AtomicOP::load_op(ty),
                        rd: t0,
                        addr,
                        src: zero_reg(),
                        amo: AMO::SeqCst,
                    }
                    .emit(&[], sink, emit_info, state);
                    // set reset part.
                    AtomicOP::merge(t0, writable_spilltmp_reg(), offset, v, ty)
                        .iter()
                        .for_each(|i| i.emit(&[], sink, emit_info, state));
                    t0.to_reg()
                } else {
                    v
                };
                Inst::Atomic {
                    op: AtomicOP::store_op(ty),
                    rd: t0,
                    addr,
                    src: store_value,
                    amo: AMO::SeqCst,
                }
                .emit(&[], sink, emit_info, state);
                // check is our value stored.
                Inst::CondBr {
                    taken: BranchTarget::Label(cas_lebel),
                    not_taken: BranchTarget::zero(),
                    kind: IntegerCompare {
                        kind: IntCC::NotEqual,
                        rs1: t0.to_reg(),
                        rs2: zero_reg(),
                    },
                }
                .emit(&[], sink, emit_info, state);
                sink.bind_label(fail_label);
            }
            &Inst::AtomicRmwLoop {
                offset,
                op,
                dst,
                ty,
                p,
                x,
                t0,
            } => {
                let offset = allocs.next(offset);
                let p = allocs.next(p);
                let x = allocs.next(x);
                let t0 = allocs.next_writable(t0);
                let dst = allocs.next_writable(dst);
                let retry = sink.get_label();
                sink.bind_label(retry);
                // load old value.
                Inst::Atomic {
                    op: AtomicOP::load_op(ty),
                    rd: dst,
                    addr: p,
                    src: zero_reg(),
                    amo: AMO::SeqCst,
                }
                .emit(&[], sink, emit_info, state);
                //

                let store_value: Reg = match op {
                    crate::ir::AtomicRmwOp::Add
                    | crate::ir::AtomicRmwOp::Sub
                    | crate::ir::AtomicRmwOp::And
                    | crate::ir::AtomicRmwOp::Or
                    | crate::ir::AtomicRmwOp::Xor => {
                        AtomicOP::extract(t0, offset, dst.to_reg(), ty)
                            .iter()
                            .for_each(|i| i.emit(&[], sink, emit_info, state));
                        Inst::AluRRR {
                            alu_op: match op {
                                crate::ir::AtomicRmwOp::Add => AluOPRRR::Add,
                                crate::ir::AtomicRmwOp::Sub => AluOPRRR::Sub,
                                crate::ir::AtomicRmwOp::And => AluOPRRR::And,
                                crate::ir::AtomicRmwOp::Or => AluOPRRR::Or,
                                crate::ir::AtomicRmwOp::Xor => AluOPRRR::Xor,
                                _ => unreachable!(),
                            },
                            rd: t0,
                            rs1: t0.to_reg(),
                            rs2: x,
                        }
                        .emit(&[], sink, emit_info, state);
                        Inst::Atomic {
                            op: AtomicOP::load_op(ty),
                            rd: writable_spilltmp_reg2(),
                            addr: p,
                            src: zero_reg(),
                            amo: AMO::SeqCst,
                        }
                        .emit(&[], sink, emit_info, state);
                        AtomicOP::merge(
                            writable_spilltmp_reg2(),
                            writable_spilltmp_reg(),
                            offset,
                            t0.to_reg(),
                            ty,
                        )
                        .iter()
                        .for_each(|i| i.emit(&[], sink, emit_info, state));
                        spilltmp_reg2()
                    }
                    crate::ir::AtomicRmwOp::Nand => {
                        let x2 = if ty.bits() < 32 {
                            AtomicOP::extract(t0, offset, dst.to_reg(), ty)
                                .iter()
                                .for_each(|i| i.emit(&[], sink, emit_info, state));
                            t0.to_reg()
                        } else {
                            dst.to_reg()
                        };
                        Inst::AluRRR {
                            alu_op: AluOPRRR::And,
                            rd: t0,
                            rs1: x,
                            rs2: x2,
                        }
                        .emit(&[], sink, emit_info, state);
                        Inst::construct_bit_not(t0, t0.to_reg()).emit(&[], sink, emit_info, state);
                        if ty.bits() < 32 {
                            Inst::Atomic {
                                op: AtomicOP::load_op(ty),
                                rd: writable_spilltmp_reg2(),
                                addr: p,
                                src: zero_reg(),
                                amo: AMO::SeqCst,
                            }
                            .emit(&[], sink, emit_info, state);
                            AtomicOP::merge(
                                writable_spilltmp_reg2(),
                                writable_spilltmp_reg(),
                                offset,
                                t0.to_reg(),
                                ty,
                            )
                            .iter()
                            .for_each(|i| i.emit(&[], sink, emit_info, state));
                            spilltmp_reg2()
                        } else {
                            t0.to_reg()
                        }
                    }

                    crate::ir::AtomicRmwOp::Umin
                    | crate::ir::AtomicRmwOp::Umax
                    | crate::ir::AtomicRmwOp::Smin
                    | crate::ir::AtomicRmwOp::Smax => {
                        let label_select_done = sink.get_label();
                        if op == crate::ir::AtomicRmwOp::Umin || op == crate::ir::AtomicRmwOp::Umax
                        {
                            AtomicOP::extract(t0, offset, dst.to_reg(), ty)
                        } else {
                            AtomicOP::extract_sext(t0, offset, dst.to_reg(), ty)
                        }
                        .iter()
                        .for_each(|i| i.emit(&[], sink, emit_info, state));
                        Inst::lower_br_icmp(
                            match op {
                                crate::ir::AtomicRmwOp::Umin => IntCC::UnsignedLessThan,
                                crate::ir::AtomicRmwOp::Umax => IntCC::UnsignedGreaterThan,
                                crate::ir::AtomicRmwOp::Smin => IntCC::SignedLessThan,
                                crate::ir::AtomicRmwOp::Smax => IntCC::SignedGreaterThan,
                                _ => unreachable!(),
                            },
                            ValueRegs::one(t0.to_reg()),
                            ValueRegs::one(x),
                            BranchTarget::Label(label_select_done),
                            BranchTarget::zero(),
                            ty,
                        )
                        .iter()
                        .for_each(|i| i.emit(&[], sink, emit_info, state));
                        // here we select x.
                        Inst::gen_move(t0, x, I64).emit(&[], sink, emit_info, state);
                        sink.bind_label(label_select_done);
                        Inst::Atomic {
                            op: AtomicOP::load_op(ty),
                            rd: writable_spilltmp_reg2(),
                            addr: p,
                            src: zero_reg(),
                            amo: AMO::SeqCst,
                        }
                        .emit(&[], sink, emit_info, state);
                        AtomicOP::merge(
                            writable_spilltmp_reg2(),
                            writable_spilltmp_reg(),
                            offset,
                            t0.to_reg(),
                            ty,
                        )
                        .iter()
                        .for_each(|i| i.emit(&[], sink, emit_info, state));
                        spilltmp_reg2()
                    }
                    crate::ir::AtomicRmwOp::Xchg => {
                        Inst::Atomic {
                            op: AtomicOP::load_op(ty),
                            rd: writable_spilltmp_reg2(),
                            addr: p,
                            src: zero_reg(),
                            amo: AMO::SeqCst,
                        }
                        .emit(&[], sink, emit_info, state);
                        AtomicOP::merge(
                            writable_spilltmp_reg2(),
                            writable_spilltmp_reg(),
                            offset,
                            x,
                            ty,
                        )
                        .iter()
                        .for_each(|i| i.emit(&[], sink, emit_info, state));
                        spilltmp_reg2()
                    }
                };

                Inst::Atomic {
                    op: AtomicOP::store_op(ty),
                    rd: t0,
                    addr: p,
                    src: store_value,
                    amo: AMO::SeqCst,
                }
                .emit(&[], sink, emit_info, state);

                // if store is not ok,retry.
                Inst::CondBr {
                    taken: BranchTarget::Label(retry),
                    not_taken: BranchTarget::zero(),
                    kind: IntegerCompare {
                        kind: IntCC::NotEqual,
                        rs1: t0.to_reg(),
                        rs2: zero_reg(),
                    },
                }
                .emit(&[], sink, emit_info, state);
            }

            &Inst::IntSelect {
                op,
                ref dst,
                ref x,
                ref y,
                ty,
            } => {
                let x = alloc_value_regs(x, &mut allocs);
                let y = alloc_value_regs(y, &mut allocs);
                let dst: Vec<_> = dst.iter().map(|r| allocs.next_writable(*r)).collect();
                let label_true = sink.get_label();
                let label_false = sink.get_label();
                let label_done = sink.get_label();
                Inst::lower_br_icmp(
                    op.to_int_cc(),
                    x,
                    y,
                    BranchTarget::Label(label_true),
                    BranchTarget::Label(label_false),
                    ty,
                )
                .into_iter()
                .for_each(|i| i.emit(&[], sink, emit_info, state));

                let gen_move = |dst: &Vec<Writable<Reg>>,
                                val: &ValueRegs<Reg>,
                                sink: &mut MachBuffer<Inst>,
                                state: &mut EmitState| {
                    let ty = if ty.bits() == 128 { I64 } else { ty };
                    let mut insts = SmallInstVec::new();
                    insts.push(Inst::Mov {
                        rd: dst[0],
                        rm: val.regs()[0],
                        ty,
                    });
                    if ty.bits() == 128 {
                        insts.push(Inst::Mov {
                            rd: dst[1],
                            rm: val.regs()[1],
                            ty,
                        });
                    }
                    insts
                        .into_iter()
                        .for_each(|i| i.emit(&[], sink, emit_info, state));
                };
                //here is true , use x.
                sink.bind_label(label_true);
                gen_move(&dst, &x, sink, state);
                Inst::gen_jump(label_done).emit(&[], sink, emit_info, state);
                // here is false use y
                sink.bind_label(label_false);
                gen_move(&dst, &y, sink, state);
                sink.bind_label(label_done);
            }
            &Inst::Csr {
                csr_op,
                rd,
                rs,
                imm,
                csr,
            } => {
                let rs = rs.map(|r| allocs.next(r));
                let rd = allocs.next_writable(rd);
                let x = csr_op.op_code()
                    | reg_to_gpr_num(rd.to_reg()) << 7
                    | csr_op.funct3() << 12
                    | csr_op.rs1(rs, imm) << 15
                    | csr.as_u32() << 20;

                sink.put4(x);
            }

            &Inst::SelectReg {
                condition,
                rd,
                rs1,
                rs2,
            } => {
                let mut condition = condition.clone();
                condition.rs1 = allocs.next(condition.rs1);
                condition.rs2 = allocs.next(condition.rs2);
                let rs1 = allocs.next(rs1);
                let rs2 = allocs.next(rs2);
                let rd = allocs.next_writable(rd);
                let label_true = sink.get_label();
                let label_jump_over = sink.get_label();
                sink.use_label_at_offset(sink.cur_offset(), label_true, LabelUse::B12);
                let x = condition.emit();
                sink.put4(x);
                // here is false , use rs2
                Inst::gen_move(rd, rs2, I64).emit(&[], sink, emit_info, state);
                // and jump over
                Inst::Jal {
                    dest: BranchTarget::Label(label_jump_over),
                }
                .emit(&[], sink, emit_info, state);
                // here condition is true , use rs1
                sink.bind_label(label_true);
                Inst::gen_move(rd, rs1, I64).emit(&[], sink, emit_info, state);
                sink.bind_label(label_jump_over);
            }
            &Inst::FcvtToInt {
                is_sat,
                rd,
                rs,
                is_signed,
                in_type,
                out_type,
                tmp,
            } => {
                let rs = allocs.next(rs);
                let tmp = allocs.next_writable(tmp);
                let rd = allocs.next_writable(rd);
                let label_nan = sink.get_label();
                let label_jump_over = sink.get_label();
                // get if nan.
                Inst::emit_not_nan(rd, rs, in_type).emit(&[], sink, emit_info, state);
                // jump to nan.
                Inst::CondBr {
                    taken: BranchTarget::Label(label_nan),
                    not_taken: BranchTarget::zero(),
                    kind: IntegerCompare {
                        kind: IntCC::Equal,
                        rs2: zero_reg(),
                        rs1: rd.to_reg(),
                    },
                }
                .emit(&[], sink, emit_info, state);

                if !is_sat {
                    let f32_bounds = f32_cvt_to_int_bounds(is_signed, out_type.bits() as u8);
                    let f64_bounds = f64_cvt_to_int_bounds(is_signed, out_type.bits() as u8);
                    if in_type == F32 {
                        Inst::load_fp_constant32(
                            tmp,
                            f32_bits(f32_bounds.0),
                            writable_spilltmp_reg(),
                        )
                    } else {
                        Inst::load_fp_constant64(
                            tmp,
                            f64_bits(f64_bounds.0),
                            writable_spilltmp_reg(),
                        )
                    }
                    .iter()
                    .for_each(|i| i.emit(&[], sink, emit_info, state));
                    Inst::TrapFf {
                        cc: FloatCC::LessThanOrEqual,
                        x: rs,
                        y: tmp.to_reg(),
                        ty: in_type,
                        tmp: rd,
                        trap_code: TrapCode::IntegerOverflow,
                    }
                    .emit(&[], sink, emit_info, state);
                    if in_type == F32 {
                        Inst::load_fp_constant32(
                            tmp,
                            f32_bits(f32_bounds.1),
                            writable_spilltmp_reg(),
                        )
                    } else {
                        Inst::load_fp_constant64(
                            tmp,
                            f64_bits(f64_bounds.1),
                            writable_spilltmp_reg(),
                        )
                    }
                    .iter()
                    .for_each(|i| i.emit(&[], sink, emit_info, state));
                    Inst::TrapFf {
                        cc: FloatCC::GreaterThanOrEqual,
                        x: rs,
                        y: tmp.to_reg(),
                        ty: in_type,
                        tmp: rd,
                        trap_code: TrapCode::IntegerOverflow,
                    }
                    .emit(&[], sink, emit_info, state);
                }
                // convert to int normally.
                Inst::FpuRR {
                    frm: Some(FRM::RTZ),
                    alu_op: FpuOPRR::float_convert_2_int_op(in_type, is_signed, out_type),
                    rd,
                    rs,
                }
                .emit(&[], sink, emit_info, state);
                // I already have the result,jump over.
                Inst::Jal {
                    dest: BranchTarget::Label(label_jump_over),
                }
                .emit(&[], sink, emit_info, state);
                // here is nan , move 0 into rd register
                sink.bind_label(label_nan);
                if is_sat {
                    Inst::load_imm12(rd, Imm12::from_bits(0)).emit(&[], sink, emit_info, state);
                } else {
                    // here is ud2.
                    Inst::Udf {
                        trap_code: TrapCode::BadConversionToInteger,
                    }
                    .emit(&[], sink, emit_info, state);
                }
                // bind jump_over
                sink.bind_label(label_jump_over);
            }

            &Inst::LoadExtName {
                rd,
                ref name,
                offset,
            } => {
                let rd = allocs.next_writable(rd);
                // get the current pc.
                Inst::Auipc {
                    rd: rd,
                    imm: Imm20::from_bits(0),
                }
                .emit(&[], sink, emit_info, state);
                // load the value.
                Inst::Load {
                    rd: rd,
                    op: LoadOP::Ld,
                    flags: MemFlags::trusted(),
                    from: AMode::RegOffset(
                        rd.to_reg(),
                        12, // auipc load and jal.
                        I64,
                    ),
                }
                .emit(&[], sink, emit_info, state);
                // jump over.
                Inst::Jal {
                    // jal and abs8 size for 12.
                    dest: BranchTarget::offset(12),
                }
                .emit(&[], sink, emit_info, state);

                sink.add_reloc(Reloc::Abs8, name.as_ref(), offset);
                sink.put8(0);
            }
            &Inst::TrapIfC {
                rs1,
                rs2,
                cc,
                trap_code,
            } => {
                let rs1 = allocs.next(rs1);
                let rs2 = allocs.next(rs2);
                let label_trap = sink.get_label();
                let label_jump_over = sink.get_label();
                Inst::CondBr {
                    taken: BranchTarget::Label(label_trap),
                    not_taken: BranchTarget::Label(label_jump_over),
                    kind: IntegerCompare { kind: cc, rs1, rs2 },
                }
                .emit(&[], sink, emit_info, state);
                // trap
                sink.bind_label(label_trap);
                Inst::Udf {
                    trap_code: trap_code,
                }
                .emit(&[], sink, emit_info, state);
                sink.bind_label(label_jump_over);
            }
            &Inst::TrapIf { test, trap_code } => {
                let test = allocs.next(test);
                let label_trap = sink.get_label();
                let label_jump_over = sink.get_label();
                Inst::CondBr {
                    taken: BranchTarget::Label(label_trap),
                    not_taken: BranchTarget::Label(label_jump_over),
                    kind: IntegerCompare {
                        kind: IntCC::NotEqual,
                        rs1: test,
                        rs2: zero_reg(),
                    },
                }
                .emit(&[], sink, emit_info, state);
                // trap
                sink.bind_label(label_trap);
                Inst::Udf {
                    trap_code: trap_code,
                }
                .emit(&[], sink, emit_info, state);
                sink.bind_label(label_jump_over);
            }
            &Inst::TrapFf {
                cc,
                x,
                y,
                ty,
                trap_code,
                tmp,
            } => {
                let x = allocs.next(x);
                let y = allocs.next(y);
                let tmp = allocs.next_writable(tmp);
                let label_trap = sink.get_label();
                let label_jump_over = sink.get_label();
                Inst::lower_br_fcmp(
                    cc,
                    x,
                    y,
                    BranchTarget::Label(label_trap),
                    BranchTarget::Label(label_jump_over),
                    ty,
                    tmp,
                )
                .iter()
                .for_each(|i| i.emit(&[], sink, emit_info, state));
                // trap
                sink.bind_label(label_trap);
                Inst::Udf {
                    trap_code: trap_code,
                }
                .emit(&[], sink, emit_info, state);
                sink.bind_label(label_jump_over);
            }

            &Inst::Udf { trap_code } => {
                sink.add_trap(trap_code);
                if let Some(s) = state.take_stack_map() {
                    sink.add_stack_map(StackMapExtent::UpcomingBytes(4), s);
                }
                // https://github.com/riscv/riscv-isa-manual/issues/850
                // all zero will cause invalid opcode.
                sink.put4(0);
            }
            &Inst::SelectIf {
                if_spectre_guard: _if_spectre_guard, // _if_spectre_guard not use because it is used to not be removed by optimization pass and some other staff.
                ref rd,
                test,
                ref x,
                ref y,
            } => {
                let label_select_x = sink.get_label();
                let label_select_y = sink.get_label();
                let label_jump_over = sink.get_label();
                let test = allocs.next(test);
                let x = alloc_value_regs(x, &mut allocs);
                let y = alloc_value_regs(y, &mut allocs);
                let rd: Vec<_> = rd.iter().map(|r| allocs.next_writable(*r)).collect();
                Inst::CondBr {
                    taken: BranchTarget::Label(label_select_x),
                    not_taken: BranchTarget::Label(label_select_y),
                    kind: IntegerCompare {
                        kind: IntCC::NotEqual,
                        rs1: test,
                        rs2: zero_reg(),
                    },
                }
                .emit(&[], sink, emit_info, state);

                // here select x.
                sink.bind_label(label_select_x);
                gen_moves(&rd[..], x.regs())
                    .into_iter()
                    .for_each(|i| i.emit(&[], sink, emit_info, state));
                // jump over
                Inst::Jal {
                    dest: BranchTarget::Label(label_jump_over),
                }
                .emit(&[], sink, emit_info, state);
                // here select y.
                sink.bind_label(label_select_y);
                gen_moves(&rd[..], y.regs())
                    .into_iter()
                    .for_each(|i| i.emit(&[], sink, emit_info, state));
                sink.bind_label(label_jump_over);
            }
            &Inst::AtomicLoad { rd, ty, p } => {
                let p = allocs.next(p);
                let rd = allocs.next_writable(rd);
                // emit the fence.
                Inst::Fence {
                    pred: Inst::FENCE_REQ_R | Inst::FENCE_REQ_W,
                    succ: Inst::FENCE_REQ_R | Inst::FENCE_REQ_W,
                }
                .emit(&[], sink, emit_info, state);
                // load.
                Inst::Load {
                    rd: rd,
                    op: LoadOP::from_type(ty),
                    flags: MemFlags::new(),
                    from: AMode::RegOffset(p, 0, ty),
                }
                .emit(&[], sink, emit_info, state);
                Inst::Fence {
                    pred: Inst::FENCE_REQ_R,
                    succ: Inst::FENCE_REQ_R | Inst::FENCE_REQ_W,
                }
                .emit(&[], sink, emit_info, state);
            }
            &Inst::AtomicStore { src, ty, p } => {
                let src = allocs.next(src);
                let p = allocs.next(p);
                Inst::Fence {
                    pred: Inst::FENCE_REQ_R | Inst::FENCE_REQ_W,
                    succ: Inst::FENCE_REQ_W,
                }
                .emit(&[], sink, emit_info, state);
                Inst::Store {
                    to: AMode::RegOffset(p, 0, ty),
                    op: StoreOP::from_type(ty),
                    flags: MemFlags::new(),
                    src,
                }
                .emit(&[], sink, emit_info, state);
            }
            &Inst::FloatRound {
                op,
                rd,
                int_tmp,
                f_tmp,
                rs,
                ty,
            } => {
                // this code is port from glibc ceil floor ... implementation.
                let rs = allocs.next(rs);
                let int_tmp = allocs.next_writable(int_tmp);
                let f_tmp = allocs.next_writable(f_tmp);
                let rd = allocs.next_writable(rd);
                let label_nan = sink.get_label();
                let label_x = sink.get_label();
                let label_jump_over = sink.get_label();
                // check if is nan.
                Inst::emit_not_nan(int_tmp, rs, ty).emit(&[], sink, emit_info, state);
                Inst::CondBr {
                    taken: BranchTarget::Label(label_nan),
                    not_taken: BranchTarget::zero(),
                    kind: IntegerCompare {
                        kind: IntCC::Equal,
                        rs1: int_tmp.to_reg(),
                        rs2: zero_reg(),
                    },
                }
                .emit(&[], sink, emit_info, state);
                fn max_value_need_round(ty: Type) -> u64 {
                    match ty {
                        F32 => {
                            let x: u64 = 1 << f32::MANTISSA_DIGITS;
                            let x = x as f32;
                            let x = u32::from_le_bytes(x.to_le_bytes());
                            x as u64
                        }
                        F64 => {
                            let x: u64 = 1 << f64::MANTISSA_DIGITS;
                            let x = x as f64;
                            u64::from_le_bytes(x.to_le_bytes())
                        }
                        _ => unreachable!(),
                    }
                }
                // load max value need to round.
                if ty == F32 {
                    Inst::load_fp_constant32(
                        f_tmp,
                        max_value_need_round(ty) as u32,
                        writable_spilltmp_reg(),
                    )
                } else {
                    Inst::load_fp_constant64(
                        f_tmp,
                        max_value_need_round(ty),
                        writable_spilltmp_reg(),
                    )
                }
                .into_iter()
                .for_each(|i| i.emit(&[], sink, emit_info, state));

                // get abs value.
                Inst::emit_fabs(rd, rs, ty).emit(&[], sink, emit_info, state);
                Inst::lower_br_fcmp(
                    FloatCC::GreaterThan,
                    // abs value > max_value_need_round
                    rd.to_reg(),
                    f_tmp.to_reg(),
                    BranchTarget::Label(label_x),
                    BranchTarget::zero(),
                    ty,
                    int_tmp,
                )
                .into_iter()
                .for_each(|i| i.emit(&[], sink, emit_info, state));
                //convert to int.
                Inst::FpuRR {
                    alu_op: FpuOPRR::float_convert_2_int_op(ty, true, I64),
                    frm: Some(op.to_frm()),
                    rd: int_tmp,
                    rs: rs,
                }
                .emit(&[], sink, emit_info, state);
                //convert back.
                Inst::FpuRR {
                    alu_op: FpuOPRR::int_convert_2_float_op(I64, true, ty),
                    frm: Some(op.to_frm()),
                    rd,
                    rs: int_tmp.to_reg(),
                }
                .emit(&[], sink, emit_info, state);
                // copy sign.
                Inst::FpuRRR {
                    alu_op: if ty == F32 {
                        FpuOPRRR::FsgnjS
                    } else {
                        FpuOPRRR::FsgnjD
                    },
                    frm: None,
                    rd,
                    rs1: rd.to_reg(),
                    rs2: rs,
                }
                .emit(&[], sink, emit_info, state);
                // jump over.
                Inst::Jal {
                    dest: BranchTarget::Label(label_jump_over),
                }
                .emit(&[], sink, emit_info, state);
                // here is nan.
                sink.bind_label(label_nan);
                Inst::FpuRRR {
                    alu_op: if ty == F32 {
                        FpuOPRRR::FaddS
                    } else {
                        FpuOPRRR::FaddD
                    },
                    frm: None,
                    rd: rd,
                    rs1: rs,
                    rs2: rs,
                }
                .emit(&[], sink, emit_info, state);
                Inst::Jal {
                    dest: BranchTarget::Label(label_jump_over),
                }
                .emit(&[], sink, emit_info, state);
                // here select origin x.
                sink.bind_label(label_x);
                Inst::gen_move(rd, rs, ty).emit(&[], sink, emit_info, state);
                sink.bind_label(label_jump_over);
            }
            &Inst::FloatSelectPseudo {
                op,
                rd,
                tmp,
                rs1,
                rs2,
                ty,
            } => {
                let rs1 = allocs.next(rs1);
                let rs2 = allocs.next(rs2);
                let tmp = allocs.next_writable(tmp);
                let rd = allocs.next_writable(rd);
                let label_rs2 = sink.get_label();
                let label_jump_over = sink.get_label();
                let lt_op = if ty == F32 {
                    FpuOPRRR::FltS
                } else {
                    FpuOPRRR::FltD
                };
                Inst::FpuRRR {
                    alu_op: lt_op,
                    frm: None,
                    rd: tmp,
                    rs1: if op == FloatSelectOP::Max { rs1 } else { rs2 },
                    rs2: if op == FloatSelectOP::Max { rs2 } else { rs1 },
                }
                .emit(&[], sink, emit_info, state);
                Inst::CondBr {
                    taken: BranchTarget::Label(label_rs2),
                    not_taken: BranchTarget::zero(),
                    kind: IntegerCompare {
                        kind: IntCC::NotEqual,
                        rs1: tmp.to_reg(),
                        rs2: zero_reg(),
                    },
                }
                .emit(&[], sink, emit_info, state);
                // here select rs1 as result.
                Inst::gen_move(rd, rs1, ty).emit(&[], sink, emit_info, state);
                Inst::Jal {
                    dest: BranchTarget::Label(label_jump_over),
                }
                .emit(&[], sink, emit_info, state);
                sink.bind_label(label_rs2);
                Inst::gen_move(rd, rs2, ty).emit(&[], sink, emit_info, state);
                sink.bind_label(label_jump_over);
            }

            &Inst::FloatSelect {
                op,
                rd,
                tmp,
                rs1,
                rs2,
                ty,
            } => {
                let rs1 = allocs.next(rs1);
                let rs2 = allocs.next(rs2);
                let tmp = allocs.next_writable(tmp);
                let rd = allocs.next_writable(rd);
                let label_nan = sink.get_label();
                let label_jump_over = sink.get_label();
                // check if rs1 is nan.
                Inst::emit_not_nan(tmp, rs1, ty).emit(&[], sink, emit_info, state);
                Inst::CondBr {
                    taken: BranchTarget::Label(label_nan),
                    not_taken: BranchTarget::zero(),
                    kind: IntegerCompare {
                        kind: IntCC::Equal,
                        rs1: tmp.to_reg(),
                        rs2: zero_reg(),
                    },
                }
                .emit(&[], sink, emit_info, state);
                // check if rs2 is nan.
                Inst::emit_not_nan(tmp, rs2, ty).emit(&[], sink, emit_info, state);
                Inst::CondBr {
                    taken: BranchTarget::Label(label_nan),
                    not_taken: BranchTarget::zero(),
                    kind: IntegerCompare {
                        kind: IntCC::Equal,
                        rs1: tmp.to_reg(),
                        rs2: zero_reg(),
                    },
                }
                .emit(&[], sink, emit_info, state);
                // here rs1 and rs2 is not nan.
                Inst::FpuRRR {
                    alu_op: op.to_fpuoprrr(ty),
                    frm: None,
                    rd: rd,
                    rs1: rs1,
                    rs2: rs2,
                }
                .emit(&[], sink, emit_info, state);
                // special handle for +0 or -0.
                {
                    // check is rs1 and rs2 all equal to zero.
                    let label_done = sink.get_label();
                    {
                        // if rs1 == 0
                        let mut insts = Inst::emit_if_float_not_zero(
                            tmp,
                            rs1,
                            ty,
                            BranchTarget::Label(label_done),
                            BranchTarget::zero(),
                        );
                        insts.extend(Inst::emit_if_float_not_zero(
                            tmp,
                            rs2,
                            ty,
                            BranchTarget::Label(label_done),
                            BranchTarget::zero(),
                        ));
                        insts
                            .iter()
                            .for_each(|i| i.emit(&[], sink, emit_info, state));
                    }
                    Inst::FpuRR {
                        alu_op: FpuOPRR::move_f_to_x_op(ty),
                        frm: None,
                        rd: tmp,
                        rs: rs1,
                    }
                    .emit(&[], sink, emit_info, state);
                    Inst::FpuRR {
                        alu_op: FpuOPRR::move_f_to_x_op(ty),
                        frm: None,
                        rd: writable_spilltmp_reg(),
                        rs: rs2,
                    }
                    .emit(&[], sink, emit_info, state);
                    Inst::AluRRR {
                        alu_op: if op == FloatSelectOP::Max {
                            AluOPRRR::And
                        } else {
                            AluOPRRR::Or
                        },
                        rd: tmp,
                        rs1: tmp.to_reg(),
                        rs2: spilltmp_reg(),
                    }
                    .emit(&[], sink, emit_info, state);
                    // move back to rd.
                    Inst::FpuRR {
                        alu_op: FpuOPRR::move_x_to_f_op(ty),
                        frm: None,
                        rd,
                        rs: tmp.to_reg(),
                    }
                    .emit(&[], sink, emit_info, state);
                    //
                    sink.bind_label(label_done);
                }
                // we have the reuslt,jump over.
                Inst::Jal {
                    dest: BranchTarget::Label(label_jump_over),
                }
                .emit(&[], sink, emit_info, state);
                // here is nan.
                sink.bind_label(label_nan);
                op.snan_bits(tmp, ty)
                    .into_iter()
                    .for_each(|i| i.emit(&[], sink, emit_info, state));
                // move to rd.
                Inst::FpuRR {
                    alu_op: FpuOPRR::move_x_to_f_op(ty),
                    frm: None,
                    rd,
                    rs: tmp.to_reg(),
                }
                .emit(&[], sink, emit_info, state);
                sink.bind_label(label_jump_over);
            }
            &Inst::Popcnt {
                sum,
                tmp,
                step,
                rs,
                ty,
            } => {
                let rs = allocs.next(rs);
                let tmp = allocs.next_writable(tmp);
                let step = allocs.next_writable(step);
                let sum = allocs.next_writable(sum);
                // load 0 to sum , init.
                Inst::gen_move(sum, zero_reg(), I64).emit(&[], sink, emit_info, state);
                // load
                Inst::load_imm12(step, Imm12::from_bits(ty.bits() as i16)).emit(
                    &[],
                    sink,
                    emit_info,
                    state,
                );
                //
                Inst::load_imm12(tmp, Imm12::from_bits(1)).emit(&[], sink, emit_info, state);
                Inst::AluRRImm12 {
                    alu_op: AluOPRRI::Slli,
                    rd: tmp,
                    rs: tmp.to_reg(),
                    imm12: Imm12::from_bits((ty.bits() - 1) as i16),
                }
                .emit(&[], sink, emit_info, state);
                let label_done = sink.get_label();
                let label_loop = sink.get_label();
                sink.bind_label(label_loop);
                Inst::CondBr {
                    taken: BranchTarget::Label(label_done),
                    not_taken: BranchTarget::zero(),
                    kind: IntegerCompare {
                        kind: IntCC::SignedLessThanOrEqual,
                        rs1: step.to_reg(),
                        rs2: zero_reg(),
                    },
                }
                .emit(&[], sink, emit_info, state);
                // test and add sum.
                {
                    Inst::AluRRR {
                        alu_op: AluOPRRR::And,
                        rd: writable_spilltmp_reg2(),
                        rs1: tmp.to_reg(),
                        rs2: rs,
                    }
                    .emit(&[], sink, emit_info, state);
                    let label_over = sink.get_label();
                    Inst::CondBr {
                        taken: BranchTarget::Label(label_over),
                        not_taken: BranchTarget::zero(),
                        kind: IntegerCompare {
                            kind: IntCC::Equal,
                            rs1: zero_reg(),
                            rs2: spilltmp_reg2(),
                        },
                    }
                    .emit(&[], sink, emit_info, state);
                    Inst::AluRRImm12 {
                        alu_op: AluOPRRI::Addi,
                        rd: sum,
                        rs: sum.to_reg(),
                        imm12: Imm12::from_bits(1),
                    }
                    .emit(&[], sink, emit_info, state);
                    sink.bind_label(label_over);
                }
                // set step and tmp.
                {
                    Inst::AluRRImm12 {
                        alu_op: AluOPRRI::Addi,
                        rd: step,
                        rs: step.to_reg(),
                        imm12: Imm12::from_bits(-1),
                    }
                    .emit(&[], sink, emit_info, state);
                    Inst::AluRRImm12 {
                        alu_op: AluOPRRI::Srli,
                        rd: tmp,
                        rs: tmp.to_reg(),
                        imm12: Imm12::from_bits(1),
                    }
                    .emit(&[], sink, emit_info, state);
                    Inst::Jal {
                        dest: BranchTarget::Label(label_loop),
                    }
                    .emit(&[], sink, emit_info, state);
                }
                sink.bind_label(label_done);
            }
            &Inst::Rev8 { rs, rd, tmp, step } => {
                let rs = allocs.next(rs);
                let tmp = allocs.next_writable(tmp);
                let step = allocs.next_writable(step);
                let rd = allocs.next_writable(rd);
                // init.
                Inst::gen_move(rd, zero_reg(), I64).emit(&[], sink, emit_info, state);
                Inst::gen_move(tmp, rs, I64).emit(&[], sink, emit_info, state);
                // load 56 to step.
                Inst::load_imm12(step, Imm12::from_bits(56)).emit(&[], sink, emit_info, state);
                let label_done = sink.get_label();
                let label_loop = sink.get_label();
                sink.bind_label(label_loop);
                Inst::CondBr {
                    taken: BranchTarget::Label(label_done),
                    not_taken: BranchTarget::zero(),
                    kind: IntegerCompare {
                        kind: IntCC::SignedLessThan,
                        rs1: step.to_reg(),
                        rs2: zero_reg(),
                    },
                }
                .emit(&[], sink, emit_info, state);
                Inst::AluRRImm12 {
                    alu_op: AluOPRRI::Andi,
                    rd: writable_spilltmp_reg(),
                    rs: tmp.to_reg(),
                    imm12: Imm12::from_bits(255),
                }
                .emit(&[], sink, emit_info, state);
                Inst::AluRRR {
                    alu_op: AluOPRRR::Sll,
                    rd: writable_spilltmp_reg(),
                    rs1: spilltmp_reg(),
                    rs2: step.to_reg(),
                }
                .emit(&[], sink, emit_info, state);

                Inst::AluRRR {
                    alu_op: AluOPRRR::Or,
                    rd: rd,
                    rs1: rd.to_reg(),
                    rs2: spilltmp_reg(),
                }
                .emit(&[], sink, emit_info, state);
                {
                    // reset step
                    Inst::AluRRImm12 {
                        alu_op: AluOPRRI::Addi,
                        rd: step,
                        rs: step.to_reg(),
                        imm12: Imm12::from_bits(-8),
                    }
                    .emit(&[], sink, emit_info, state);
                    //reset tmp.
                    Inst::AluRRImm12 {
                        alu_op: AluOPRRI::Srli,
                        rd: tmp,
                        rs: tmp.to_reg(),
                        imm12: Imm12::from_bits(8),
                    }
                    .emit(&[], sink, emit_info, state);
                    // loop.
                    Inst::Jal {
                        dest: BranchTarget::Label(label_loop),
                    }
                }
                .emit(&[], sink, emit_info, state);
                sink.bind_label(label_done);
            }
            &Inst::Cltz {
                sum,
                tmp,
                step,
                rs,
                leading,
                ty,
            } => {
                let rs = allocs.next(rs);
                let tmp = allocs.next_writable(tmp);
                let step = allocs.next_writable(step);
                let sum = allocs.next_writable(sum);
                // load 0 to sum , init.
                Inst::gen_move(sum, zero_reg(), I64).emit(&[], sink, emit_info, state);
                // load
                Inst::load_imm12(step, Imm12::from_bits(ty.bits() as i16)).emit(
                    &[],
                    sink,
                    emit_info,
                    state,
                );
                //
                Inst::load_imm12(tmp, Imm12::from_bits(1)).emit(&[], sink, emit_info, state);
                if leading {
                    Inst::AluRRImm12 {
                        alu_op: AluOPRRI::Slli,
                        rd: tmp,
                        rs: tmp.to_reg(),
                        imm12: Imm12::from_bits((ty.bits() - 1) as i16),
                    }
                    .emit(&[], sink, emit_info, state);
                }
                let label_done = sink.get_label();
                let label_loop = sink.get_label();
                sink.bind_label(label_loop);
                Inst::CondBr {
                    taken: BranchTarget::Label(label_done),
                    not_taken: BranchTarget::zero(),
                    kind: IntegerCompare {
                        kind: IntCC::SignedLessThanOrEqual,
                        rs1: step.to_reg(),
                        rs2: zero_reg(),
                    },
                }
                .emit(&[], sink, emit_info, state);
                // test and add sum.
                {
                    Inst::AluRRR {
                        alu_op: AluOPRRR::And,
                        rd: writable_spilltmp_reg2(),
                        rs1: tmp.to_reg(),
                        rs2: rs,
                    }
                    .emit(&[], sink, emit_info, state);
                    Inst::CondBr {
                        taken: BranchTarget::Label(label_done),
                        not_taken: BranchTarget::zero(),
                        kind: IntegerCompare {
                            kind: IntCC::NotEqual,
                            rs1: zero_reg(),
                            rs2: spilltmp_reg2(),
                        },
                    }
                    .emit(&[], sink, emit_info, state);
                    Inst::AluRRImm12 {
                        alu_op: AluOPRRI::Addi,
                        rd: sum,
                        rs: sum.to_reg(),
                        imm12: Imm12::from_bits(1),
                    }
                    .emit(&[], sink, emit_info, state);
                }
                // set step and tmp.
                {
                    Inst::AluRRImm12 {
                        alu_op: AluOPRRI::Addi,
                        rd: step,
                        rs: step.to_reg(),
                        imm12: Imm12::from_bits(-1),
                    }
                    .emit(&[], sink, emit_info, state);
                    Inst::AluRRImm12 {
                        alu_op: if leading {
                            AluOPRRI::Srli
                        } else {
                            AluOPRRI::Slli
                        },
                        rd: tmp,
                        rs: tmp.to_reg(),
                        imm12: Imm12::from_bits(1),
                    }
                    .emit(&[], sink, emit_info, state);
                    Inst::Jal {
                        dest: BranchTarget::Label(label_loop),
                    }
                    .emit(&[], sink, emit_info, state);
                }
                sink.bind_label(label_done);
            }
            &Inst::Brev8 {
                rs,
                ty,
                step,
                tmp,
                tmp2,
                rd,
            } => {
                let rs = allocs.next(rs);
                let step = allocs.next_writable(step);
                let tmp = allocs.next_writable(tmp);
                let tmp2 = allocs.next_writable(tmp2);
                let rd = allocs.next_writable(rd);
                Inst::gen_move(rd, zero_reg(), I64).emit(&[], sink, emit_info, state);
                Inst::load_imm12(step, Imm12::from_bits(ty.bits() as i16)).emit(
                    &[],
                    sink,
                    emit_info,
                    state,
                );
                //
                Inst::load_imm12(tmp, Imm12::from_bits(1)).emit(&[], sink, emit_info, state);
                Inst::AluRRImm12 {
                    alu_op: AluOPRRI::Slli,
                    rd: tmp,
                    rs: tmp.to_reg(),
                    imm12: Imm12::from_bits((ty.bits() - 1) as i16),
                }
                .emit(&[], sink, emit_info, state);
                Inst::load_imm12(tmp2, Imm12::from_bits(1)).emit(&[], sink, emit_info, state);
                Inst::AluRRImm12 {
                    alu_op: AluOPRRI::Slli,
                    rd: tmp2,
                    rs: tmp2.to_reg(),
                    imm12: Imm12::from_bits((ty.bits() - 8) as i16),
                }
                .emit(&[], sink, emit_info, state);

                let label_done = sink.get_label();
                let label_loop = sink.get_label();
                sink.bind_label(label_loop);
                Inst::CondBr {
                    taken: BranchTarget::Label(label_done),
                    not_taken: BranchTarget::zero(),
                    kind: IntegerCompare {
                        kind: IntCC::SignedLessThanOrEqual,
                        rs1: step.to_reg(),
                        rs2: zero_reg(),
                    },
                }
                .emit(&[], sink, emit_info, state);
                // test and set bit.
                {
                    Inst::AluRRR {
                        alu_op: AluOPRRR::And,
                        rd: writable_spilltmp_reg2(),
                        rs1: tmp.to_reg(),
                        rs2: rs,
                    }
                    .emit(&[], sink, emit_info, state);
                    let label_over = sink.get_label();
                    Inst::CondBr {
                        taken: BranchTarget::Label(label_over),
                        not_taken: BranchTarget::zero(),
                        kind: IntegerCompare {
                            kind: IntCC::Equal,
                            rs1: zero_reg(),
                            rs2: spilltmp_reg2(),
                        },
                    }
                    .emit(&[], sink, emit_info, state);
                    Inst::AluRRR {
                        alu_op: AluOPRRR::Or,
                        rd: rd,
                        rs1: rd.to_reg(),
                        rs2: tmp2.to_reg(),
                    }
                    .emit(&[], sink, emit_info, state);
                    sink.bind_label(label_over);
                }
                // set step and tmp.
                {
                    Inst::AluRRImm12 {
                        alu_op: AluOPRRI::Addi,
                        rd: step,
                        rs: step.to_reg(),
                        imm12: Imm12::from_bits(-1),
                    }
                    .emit(&[], sink, emit_info, state);
                    Inst::AluRRImm12 {
                        alu_op: AluOPRRI::Srli,
                        rd: tmp,
                        rs: tmp.to_reg(),
                        imm12: Imm12::from_bits(1),
                    }
                    .emit(&[], sink, emit_info, state);
                    {
                        // reset tmp2
                        // if (step %=8 == 0) then tmp2 = tmp2 >> 15
                        // if (step %=8 != 0) then tmp2 = tmp2 << 1
                        let label_over = sink.get_label();
                        let label_sll_1 = sink.get_label();
                        Inst::load_imm12(writable_spilltmp_reg2(), Imm12::from_bits(8)).emit(
                            &[],
                            sink,
                            emit_info,
                            state,
                        );
                        Inst::AluRRR {
                            alu_op: AluOPRRR::Rem,
                            rd: writable_spilltmp_reg2(),
                            rs1: step.to_reg(),
                            rs2: spilltmp_reg2(),
                        }
                        .emit(&[], sink, emit_info, state);
                        Inst::CondBr {
                            taken: BranchTarget::Label(label_sll_1),
                            not_taken: BranchTarget::zero(),
                            kind: IntegerCompare {
                                kind: IntCC::NotEqual,
                                rs1: spilltmp_reg2(),
                                rs2: zero_reg(),
                            },
                        }
                        .emit(&[], sink, emit_info, state);
                        Inst::AluRRImm12 {
                            alu_op: AluOPRRI::Srli,
                            rd: tmp2,
                            rs: tmp2.to_reg(),
                            imm12: Imm12::from_bits(15),
                        }
                        .emit(&[], sink, emit_info, state);
                        Inst::Jal {
                            dest: BranchTarget::Label(label_over),
                        }
                        .emit(&[], sink, emit_info, state);
                        sink.bind_label(label_sll_1);
                        Inst::AluRRImm12 {
                            alu_op: AluOPRRI::Slli,
                            rd: tmp2,
                            rs: tmp2.to_reg(),
                            imm12: Imm12::from_bits(1),
                        }
                        .emit(&[], sink, emit_info, state);
                        sink.bind_label(label_over);
                    }
                    Inst::Jal {
                        dest: BranchTarget::Label(label_loop),
                    }
                    .emit(&[], sink, emit_info, state);
                }
                sink.bind_label(label_done);
            }
            &Inst::StackProbeLoop {
                guard_size,
                probe_count,
                tmp: guard_size_tmp,
            } => {
                let step = writable_spilltmp_reg();
                Inst::load_constant_u64(step, (guard_size as u64) * (probe_count as u64))
                    .iter()
                    .for_each(|i| i.emit(&[], sink, emit_info, state));
                Inst::load_constant_u64(guard_size_tmp, guard_size as u64)
                    .iter()
                    .for_each(|i| i.emit(&[], sink, emit_info, state));

                let loop_start = sink.get_label();
                let label_done = sink.get_label();
                sink.bind_label(loop_start);
                Inst::CondBr {
                    taken: BranchTarget::Label(label_done),
                    not_taken: BranchTarget::zero(),
                    kind: IntegerCompare {
                        kind: IntCC::UnsignedLessThanOrEqual,
                        rs1: step.to_reg(),
                        rs2: guard_size_tmp.to_reg(),
                    },
                }
                .emit(&[], sink, emit_info, state);
                // compute address.
                Inst::AluRRR {
                    alu_op: AluOPRRR::Sub,
                    rd: writable_spilltmp_reg2(),
                    rs1: stack_reg(),
                    rs2: step.to_reg(),
                }
                .emit(&[], sink, emit_info, state);
                Inst::Store {
                    to: AMode::RegOffset(spilltmp_reg2(), 0, I8),
                    op: StoreOP::Sb,
                    flags: MemFlags::new(),
                    src: zero_reg(),
                }
                .emit(&[], sink, emit_info, state);
                // reset step.
                Inst::AluRRR {
                    alu_op: AluOPRRR::Sub,
                    rd: step,
                    rs1: step.to_reg(),
                    rs2: guard_size_tmp.to_reg(),
                }
                .emit(&[], sink, emit_info, state);
                Inst::Jal {
                    dest: BranchTarget::Label(loop_start),
                }
                .emit(&[], sink, emit_info, state);
                sink.bind_label(label_done);
            }
        };
        let end_off = sink.cur_offset();
        assert!(
            (end_off - start_off) <= Inst::worst_case_size(),
            "Inst:{:?} length:{} worst_case_size:{}",
            self,
            end_off - start_off,
            Inst::worst_case_size()
        );
    }

    fn pretty_print_inst(&self, allocs: &[Allocation], state: &mut Self::State) -> String {
        let mut allocs = AllocationConsumer::new(allocs);
        self.print_with_state(state, &mut allocs)
    }
}

// helper function.
fn alloc_value_regs(orgin: &ValueRegs<Reg>, alloc: &mut AllocationConsumer) -> ValueRegs<Reg> {
    match orgin.regs().len() {
        1 => ValueRegs::one(alloc.next(orgin.regs()[0])),
        2 => ValueRegs::two(alloc.next(orgin.regs()[0]), alloc.next(orgin.regs()[1])),
        _ => unreachable!(),
    }
}
