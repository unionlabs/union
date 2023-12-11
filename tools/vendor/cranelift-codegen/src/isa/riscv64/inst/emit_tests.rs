#[allow(unused)]
use crate::ir::LibCall;
use crate::isa::riscv64::inst::*;
use crate::settings;
use alloc::vec::Vec;

#[test]
fn test_riscv64_binemit() {
    struct TestUnit {
        inst: Inst,
        assembly: &'static str,
        code: u32,
    }

    impl TestUnit {
        fn new(i: Inst, ass: &'static str, code: u32) -> Self {
            Self {
                inst: i,
                assembly: ass,
                code: code,
            }
        }
    }

    let mut insns = Vec::<TestUnit>::with_capacity(500);

    insns.push(TestUnit::new(
        Inst::Mov {
            rd: writable_fa0(),
            rm: fa1(),
            ty: F32,
        },
        "fmv.s fa0,fa1",
        0x20b58553,
    ));

    insns.push(TestUnit::new(
        Inst::Mov {
            rd: writable_fa0(),
            rm: fa1(),
            ty: F64,
        },
        "fmv.d fa0,fa1",
        0x22b58553,
    ));

    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::Brev8,
            rd: writable_a1(),
            rs: a0(),
            imm12: Imm12::zero(),
        },
        "brev8 a1,a0",
        0x68755593,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::Rev8,
            rd: writable_a1(),
            rs: a0(),
            imm12: Imm12::zero(),
        },
        "rev8 a1,a0",
        0x6b855593,
    ));

    //
    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::Bclri,
            rd: writable_a1(),
            rs: a0(),
            imm12: Imm12::from_bits(5),
        },
        "bclri a1,a0,5",
        0x48551593,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::Bexti,
            rd: writable_a1(),
            rs: a0(),
            imm12: Imm12::from_bits(5),
        },
        "bexti a1,a0,5",
        0x48555593,
    ));

    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::Binvi,
            rd: writable_a1(),
            rs: a0(),
            imm12: Imm12::from_bits(5),
        },
        "binvi a1,a0,5",
        0x68551593,
    ));

    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::Bseti,
            rd: writable_a1(),
            rs: a0(),
            imm12: Imm12::from_bits(5),
        },
        "bseti a1,a0,5",
        0x28551593,
    ));

    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::Rori,
            rd: writable_a1(),
            rs: a0(),
            imm12: Imm12::from_bits(5),
        },
        "rori a1,a0,5",
        0x60555593,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::Roriw,
            rd: writable_a1(),
            rs: a0(),
            imm12: Imm12::from_bits(5),
        },
        "roriw a1,a0,5",
        0x6055559b,
    ));

    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::SlliUw,
            rd: writable_a1(),
            rs: a0(),
            imm12: Imm12::from_bits(5),
        },
        "slli.uw a1,a0,5",
        0x855159b,
    ));

    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::Clz,
            rd: writable_a1(),
            rs: a0(),
            imm12: Imm12::zero(),
        },
        "clz a1,a0",
        0x60051593,
    ));

    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::Clzw,
            rd: writable_a1(),
            rs: a0(),
            imm12: Imm12::zero(),
        },
        "clzw a1,a0",
        0x6005159b,
    ));

    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::Cpop,
            rd: writable_a1(),
            rs: a0(),
            imm12: Imm12::zero(),
        },
        "cpop a1,a0",
        0x60251593,
    ));

    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::Cpopw,
            rd: writable_a1(),
            rs: a0(),
            imm12: Imm12::zero(),
        },
        "cpopw a1,a0",
        0x6025159b,
    ));

    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::Ctz,
            rd: writable_a1(),
            rs: a0(),
            imm12: Imm12::zero(),
        },
        "ctz a1,a0",
        0x60151593,
    ));

    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::Ctzw,
            rd: writable_a1(),
            rs: a0(),
            imm12: Imm12::zero(),
        },
        "ctzw a1,a0",
        0x6015159b,
    ));

    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::Sextb,
            rd: writable_a1(),
            rs: a0(),
            imm12: Imm12::zero(),
        },
        "sext.b a1,a0",
        0x60451593,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::Sexth,
            rd: writable_a1(),
            rs: a0(),
            imm12: Imm12::zero(),
        },
        "sext.h a1,a0",
        0x60551593,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::Zexth,
            rd: writable_a1(),
            rs: a0(),
            imm12: Imm12::zero(),
        },
        "zext.h a1,a0",
        0x80545bb,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::Orcb,
            rd: writable_a1(),
            rs: a0(),
            imm12: Imm12::zero(),
        },
        "orc.b a1,a0",
        0x28755593,
    ));

    //
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Adduw,
            rd: writable_a1(),
            rs1: a0(),
            rs2: zero_reg(),
        },
        "add.uw a1,a0,zero",
        0x80505bb,
    ));

    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Andn,
            rd: writable_a1(),
            rs1: a0(),
            rs2: zero_reg(),
        },
        "andn a1,a0,zero",
        0x400575b3,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Bclr,
            rd: writable_a1(),
            rs1: a0(),
            rs2: zero_reg(),
        },
        "bclr a1,a0,zero",
        0x480515b3,
    ));

    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Bext,
            rd: writable_a1(),
            rs1: a0(),
            rs2: zero_reg(),
        },
        "bext a1,a0,zero",
        0x480555b3,
    ));

    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Binv,
            rd: writable_a1(),
            rs1: a0(),
            rs2: zero_reg(),
        },
        "binv a1,a0,zero",
        0x680515b3,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Bset,
            rd: writable_a1(),
            rs1: a0(),
            rs2: zero_reg(),
        },
        "bset a1,a0,zero",
        0x280515b3,
    ));

    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Clmul,
            rd: writable_a1(),
            rs1: a0(),
            rs2: zero_reg(),
        },
        "clmul a1,a0,zero",
        0xa0515b3,
    ));

    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Clmulh,
            rd: writable_a1(),
            rs1: a0(),
            rs2: zero_reg(),
        },
        "clmulh a1,a0,zero",
        0xa0535b3,
    ));

    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Clmulr,
            rd: writable_a1(),
            rs1: a0(),
            rs2: zero_reg(),
        },
        "clmulr a1,a0,zero",
        0xa0525b3,
    ));

    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Max,
            rd: writable_a1(),
            rs1: a0(),
            rs2: zero_reg(),
        },
        "max a1,a0,zero",
        0xa0565b3,
    ));

    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Maxu,
            rd: writable_a1(),
            rs1: a0(),
            rs2: zero_reg(),
        },
        "maxu a1,a0,zero",
        0xa0575b3,
    ));

    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Min,
            rd: writable_a1(),
            rs1: a0(),
            rs2: zero_reg(),
        },
        "min a1,a0,zero",
        0xa0545b3,
    ));

    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Minu,
            rd: writable_a1(),
            rs1: a0(),
            rs2: zero_reg(),
        },
        "minu a1,a0,zero",
        0xa0555b3,
    ));

    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Orn,
            rd: writable_a1(),
            rs1: a0(),
            rs2: zero_reg(),
        },
        "orn a1,a0,zero",
        0x400565b3,
    ));

    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Rol,
            rd: writable_a1(),
            rs1: a0(),
            rs2: zero_reg(),
        },
        "rol a1,a0,zero",
        0x600515b3,
    ));

    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Rolw,
            rd: writable_a1(),
            rs1: a0(),
            rs2: zero_reg(),
        },
        "rolw a1,a0,zero",
        0x600515bb,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Ror,
            rd: writable_a1(),
            rs1: a0(),
            rs2: zero_reg(),
        },
        "ror a1,a0,zero",
        0x600555b3,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Rorw,
            rd: writable_a1(),
            rs1: a0(),
            rs2: zero_reg(),
        },
        "rorw a1,a0,zero",
        0x600555bb,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Sh1add,
            rd: writable_a1(),
            rs1: a0(),
            rs2: zero_reg(),
        },
        "sh1add a1,a0,zero",
        0x200525b3,
    ));

    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Sh1adduw,
            rd: writable_a1(),
            rs1: a0(),
            rs2: zero_reg(),
        },
        "sh1add.uw a1,a0,zero",
        0x200525bb,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Sh2add,
            rd: writable_a1(),
            rs1: a0(),
            rs2: zero_reg(),
        },
        "sh2add a1,a0,zero",
        0x200545b3,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Sh2adduw,
            rd: writable_a1(),
            rs1: a0(),
            rs2: zero_reg(),
        },
        "sh2add.uw a1,a0,zero",
        0x200545bb,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Sh3add,
            rd: writable_a1(),
            rs1: a0(),
            rs2: zero_reg(),
        },
        "sh3add a1,a0,zero",
        0x200565b3,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Sh3adduw,
            rd: writable_a1(),
            rs1: a0(),
            rs2: zero_reg(),
        },
        "sh3add.uw a1,a0,zero",
        0x200565bb,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Xnor,
            rd: writable_a1(),
            rs1: a0(),
            rs2: zero_reg(),
        },
        "xnor a1,a0,zero",
        0x400545b3,
    ));

    //
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Add,
            rd: writable_fp_reg(),
            rs1: fp_reg(),
            rs2: zero_reg(),
        },
        "add fp,fp,zero",
        0x40433,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::Addi,
            rd: writable_fp_reg(),
            rs: stack_reg(),
            imm12: Imm12::maybe_from_u64(100).unwrap(),
        },
        "addi fp,sp,100",
        0x6410413,
    ));
    insns.push(TestUnit::new(
        Inst::Lui {
            rd: writable_zero_reg(),
            imm: Imm20::from_bits(120),
        },
        "lui zero,120",
        0x78037,
    ));
    insns.push(TestUnit::new(
        Inst::Auipc {
            rd: writable_zero_reg(),
            imm: Imm20::from_bits(120),
        },
        "auipc zero,120",
        0x78017,
    ));

    insns.push(TestUnit::new(
        Inst::Jalr {
            rd: writable_a0(),
            base: a0(),
            offset: Imm12::from_bits(100),
        },
        "jalr a0,100(a0)",
        0x6450567,
    ));

    insns.push(TestUnit::new(
        Inst::Load {
            rd: writable_a0(),
            op: LoadOP::Lb,
            flags: MemFlags::new(),
            from: AMode::RegOffset(a1(), 100, I8),
        },
        "lb a0,100(a1)",
        0x6458503,
    ));
    insns.push(TestUnit::new(
        Inst::Load {
            rd: writable_a0(),
            op: LoadOP::Lh,
            flags: MemFlags::new(),
            from: AMode::RegOffset(a1(), 100, I16),
        },
        "lh a0,100(a1)",
        0x6459503,
    ));

    insns.push(TestUnit::new(
        Inst::Load {
            rd: writable_a0(),
            op: LoadOP::Lw,
            flags: MemFlags::new(),
            from: AMode::RegOffset(a1(), 100, I32),
        },
        "lw a0,100(a1)",
        0x645a503,
    ));

    insns.push(TestUnit::new(
        Inst::Load {
            rd: writable_a0(),
            op: LoadOP::Ld,
            flags: MemFlags::new(),
            from: AMode::RegOffset(a1(), 100, I64),
        },
        "ld a0,100(a1)",
        0x645b503,
    ));
    insns.push(TestUnit::new(
        Inst::Load {
            rd: Writable::from_reg(fa0()),
            op: LoadOP::Flw,
            flags: MemFlags::new(),
            from: AMode::RegOffset(a1(), 100, I64),
        },
        "flw fa0,100(a1)",
        0x645a507,
    ));

    insns.push(TestUnit::new(
        Inst::Load {
            rd: Writable::from_reg(fa0()),
            op: LoadOP::Fld,
            flags: MemFlags::new(),
            from: AMode::RegOffset(a1(), 100, I64),
        },
        "fld fa0,100(a1)",
        0x645b507,
    ));
    insns.push(TestUnit::new(
        Inst::Store {
            to: AMode::SPOffset(100, I8),
            op: StoreOP::Sb,
            flags: MemFlags::new(),
            src: a0(),
        },
        "sb a0,100(sp)",
        0x6a10223,
    ));
    insns.push(TestUnit::new(
        Inst::Store {
            to: AMode::SPOffset(100, I16),
            op: StoreOP::Sh,
            flags: MemFlags::new(),
            src: a0(),
        },
        "sh a0,100(sp)",
        0x6a11223,
    ));
    insns.push(TestUnit::new(
        Inst::Store {
            to: AMode::SPOffset(100, I32),
            op: StoreOP::Sw,
            flags: MemFlags::new(),
            src: a0(),
        },
        "sw a0,100(sp)",
        0x6a12223,
    ));
    insns.push(TestUnit::new(
        Inst::Store {
            to: AMode::SPOffset(100, I64),
            op: StoreOP::Sd,
            flags: MemFlags::new(),
            src: a0(),
        },
        "sd a0,100(sp)",
        0x6a13223,
    ));
    insns.push(TestUnit::new(
        Inst::Store {
            to: AMode::SPOffset(100, I64),
            op: StoreOP::Fsw,
            flags: MemFlags::new(),
            src: fa0(),
        },
        "fsw fa0,100(sp)",
        0x6a12227,
    ));
    insns.push(TestUnit::new(
        Inst::Store {
            to: AMode::SPOffset(100, I64),
            op: StoreOP::Fsd,
            flags: MemFlags::new(),
            src: fa0(),
        },
        "fsd fa0,100(sp)",
        0x6a13227,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::Addi,
            rd: writable_a0(),
            rs: a0(),
            imm12: Imm12::from_bits(100),
        },
        "addi a0,a0,100",
        0x6450513,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::Slti,
            rd: writable_a0(),
            rs: a0(),
            imm12: Imm12::from_bits(100),
        },
        "slti a0,a0,100",
        0x6452513,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::SltiU,
            rd: writable_a0(),
            rs: a0(),
            imm12: Imm12::from_bits(100),
        },
        "sltiu a0,a0,100",
        0x6453513,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::Xori,
            rd: writable_a0(),
            rs: a0(),
            imm12: Imm12::from_bits(100),
        },
        "xori a0,a0,100",
        0x6454513,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::Andi,
            rd: writable_a0(),
            rs: a0(),
            imm12: Imm12::from_bits(100),
        },
        "andi a0,a0,100",
        0x6457513,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::Slli,
            rd: writable_a0(),
            rs: a0(),
            imm12: Imm12::from_bits(5),
        },
        "slli a0,a0,5",
        0x551513,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::Srli,
            rd: writable_a0(),
            rs: a0(),
            imm12: Imm12::from_bits(5),
        },
        "srli a0,a0,5",
        0x555513,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::Srai,
            rd: writable_a0(),
            rs: a0(),
            imm12: Imm12::from_bits(5),
        },
        "srai a0,a0,5",
        0x40555513,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::Addiw,
            rd: writable_a0(),
            rs: a0(),
            imm12: Imm12::from_bits(120),
        },
        "addiw a0,a0,120",
        0x785051b,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::Slliw,
            rd: writable_a0(),
            rs: a0(),
            imm12: Imm12::from_bits(5),
        },
        "slliw a0,a0,5",
        0x55151b,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::SrliW,
            rd: writable_a0(),
            rs: a0(),
            imm12: Imm12::from_bits(5),
        },
        "srliw a0,a0,5",
        0x55551b,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::Sraiw,
            rd: writable_a0(),
            rs: a0(),
            imm12: Imm12::from_bits(5),
        },
        "sraiw a0,a0,5",
        0x4055551b,
    ));

    insns.push(TestUnit::new(
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::Sraiw,
            rd: writable_a0(),
            rs: a0(),
            imm12: Imm12::from_bits(5),
        },
        "sraiw a0,a0,5",
        0x4055551b,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Add,
            rd: writable_a0(),
            rs1: a0(),
            rs2: a1(),
        },
        "add a0,a0,a1",
        0xb50533,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Sub,
            rd: writable_a0(),
            rs1: a0(),
            rs2: a1(),
        },
        "sub a0,a0,a1",
        0x40b50533,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Sll,
            rd: writable_a0(),
            rs1: a0(),
            rs2: a1(),
        },
        "sll a0,a0,a1",
        0xb51533,
    ));

    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Slt,
            rd: writable_a0(),
            rs1: a0(),
            rs2: a1(),
        },
        "slt a0,a0,a1",
        0xb52533,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::SltU,
            rd: writable_a0(),
            rs1: a0(),
            rs2: a1(),
        },
        "sltu a0,a0,a1",
        0xb53533,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Xor,
            rd: writable_a0(),
            rs1: a0(),
            rs2: a1(),
        },
        "xor a0,a0,a1",
        0xb54533,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Srl,
            rd: writable_a0(),
            rs1: a0(),
            rs2: a1(),
        },
        "srl a0,a0,a1",
        0xb55533,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Sra,
            rd: writable_a0(),
            rs1: a0(),
            rs2: a1(),
        },
        "sra a0,a0,a1",
        0x40b55533,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Or,
            rd: writable_a0(),
            rs1: a0(),
            rs2: a1(),
        },
        "or a0,a0,a1",
        0xb56533,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::And,
            rd: writable_a0(),
            rs1: a0(),
            rs2: a1(),
        },
        "and a0,a0,a1",
        0xb57533,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Addw,
            rd: writable_a0(),
            rs1: a0(),
            rs2: a1(),
        },
        "addw a0,a0,a1",
        0xb5053b,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Subw,
            rd: writable_a0(),
            rs1: a0(),
            rs2: a1(),
        },
        "subw a0,a0,a1",
        0x40b5053b,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Sllw,
            rd: writable_a0(),
            rs1: a0(),
            rs2: a1(),
        },
        "sllw a0,a0,a1",
        0xb5153b,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Srlw,
            rd: writable_a0(),
            rs1: a0(),
            rs2: a1(),
        },
        "srlw a0,a0,a1",
        0xb5553b,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Sraw,
            rd: writable_a0(),
            rs1: a0(),
            rs2: a1(),
        },
        "sraw a0,a0,a1",
        0x40b5553b,
    ));

    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Mul,
            rd: writable_a0(),
            rs1: a0(),
            rs2: a1(),
        },
        "mul a0,a0,a1",
        0x2b50533,
    ));

    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Mulh,
            rd: writable_a0(),
            rs1: a0(),
            rs2: a1(),
        },
        "mulh a0,a0,a1",
        0x2b51533,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Mulhsu,
            rd: writable_a0(),
            rs1: a0(),
            rs2: a1(),
        },
        "mulhsu a0,a0,a1",
        0x2b52533,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Mulhu,
            rd: writable_a0(),
            rs1: a0(),
            rs2: a1(),
        },
        "mulhu a0,a0,a1",
        0x2b53533,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Div,
            rd: writable_a0(),
            rs1: a0(),
            rs2: a1(),
        },
        "div a0,a0,a1",
        0x2b54533,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::DivU,
            rd: writable_a0(),
            rs1: a0(),
            rs2: a1(),
        },
        "divu a0,a0,a1",
        0x2b55533,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Rem,
            rd: writable_a0(),
            rs1: a0(),
            rs2: a1(),
        },
        "rem a0,a0,a1",
        0x2b56533,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::RemU,
            rd: writable_a0(),
            rs1: a0(),
            rs2: a1(),
        },
        "remu a0,a0,a1",
        0x2b57533,
    ));

    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Mulw,
            rd: writable_a0(),
            rs1: a0(),
            rs2: a1(),
        },
        "mulw a0,a0,a1",
        0x2b5053b,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Divw,
            rd: writable_a0(),
            rs1: a0(),
            rs2: a1(),
        },
        "divw a0,a0,a1",
        0x2b5453b,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Remw,
            rd: writable_a0(),
            rs1: a0(),
            rs2: a1(),
        },
        "remw a0,a0,a1",
        0x2b5653b,
    ));
    insns.push(TestUnit::new(
        Inst::AluRRR {
            alu_op: AluOPRRR::Remuw,
            rd: writable_a0(),
            rs1: a0(),
            rs2: a1(),
        },
        "remuw a0,a0,a1",
        0x2b5753b,
    ));

    //
    insns.push(TestUnit::new(
        Inst::FpuRRR {
            frm: Some(FRM::RNE),
            alu_op: FpuOPRRR::FaddS,
            rd: writable_fa0(),
            rs1: fa0(),
            rs2: fa1(),
        },
        "fadd.s fa0,fa0,fa1,rne",
        0xb50553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRRR {
            frm: Some(FRM::RTZ),
            alu_op: FpuOPRRR::FsubS,
            rd: writable_fa0(),
            rs1: fa0(),
            rs2: fa1(),
        },
        "fsub.s fa0,fa0,fa1,rtz",
        0x8b51553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRRR {
            frm: Some(FRM::RUP),
            alu_op: FpuOPRRR::FmulS,
            rd: writable_fa0(),
            rs1: fa0(),
            rs2: fa1(),
        },
        "fmul.s fa0,fa0,fa1,rup",
        0x10b53553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRRR {
            frm: None,
            alu_op: FpuOPRRR::FdivS,
            rd: writable_fa0(),
            rs1: fa0(),
            rs2: fa1(),
        },
        "fdiv.s fa0,fa0,fa1",
        0x18b57553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRRR {
            frm: None,
            alu_op: FpuOPRRR::FsgnjS,
            rd: writable_fa0(),
            rs1: fa0(),
            rs2: fa1(),
        },
        "fsgnj.s fa0,fa0,fa1",
        0x20b50553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRRR {
            frm: None,
            alu_op: FpuOPRRR::FsgnjnS,
            rd: writable_fa0(),
            rs1: fa0(),
            rs2: fa1(),
        },
        "fsgnjn.s fa0,fa0,fa1",
        0x20b51553,
    ));

    insns.push(TestUnit::new(
        Inst::FpuRRR {
            frm: None,
            alu_op: FpuOPRRR::FsgnjxS,
            rd: writable_fa0(),
            rs1: fa0(),
            rs2: fa1(),
        },
        "fsgnjx.s fa0,fa0,fa1",
        0x20b52553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRRR {
            frm: None,
            alu_op: FpuOPRRR::FminS,
            rd: writable_fa0(),
            rs1: fa0(),
            rs2: fa1(),
        },
        "fmin.s fa0,fa0,fa1",
        0x28b50553,
    ));

    insns.push(TestUnit::new(
        Inst::FpuRRR {
            frm: None,
            alu_op: FpuOPRRR::FmaxS,
            rd: writable_fa0(),
            rs1: fa0(),
            rs2: fa1(),
        },
        "fmax.s fa0,fa0,fa1",
        0x28b51553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRRR {
            frm: None,
            alu_op: FpuOPRRR::FeqS,
            rd: writable_a0(),
            rs1: fa0(),
            rs2: fa1(),
        },
        "feq.s a0,fa0,fa1",
        0xa0b52553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRRR {
            frm: None,
            alu_op: FpuOPRRR::FltS,
            rd: writable_a0(),
            rs1: fa0(),
            rs2: fa1(),
        },
        "flt.s a0,fa0,fa1",
        0xa0b51553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRRR {
            frm: None,
            alu_op: FpuOPRRR::FleS,
            rd: writable_a0(),
            rs1: fa0(),
            rs2: fa1(),
        },
        "fle.s a0,fa0,fa1",
        0xa0b50553,
    ));

    //
    insns.push(TestUnit::new(
        Inst::FpuRRR {
            frm: None,
            alu_op: FpuOPRRR::FaddD,
            rd: writable_fa0(),
            rs1: fa0(),
            rs2: fa1(),
        },
        "fadd.d fa0,fa0,fa1",
        0x2b57553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRRR {
            frm: None,
            alu_op: FpuOPRRR::FsubD,
            rd: writable_fa0(),
            rs1: fa0(),
            rs2: fa1(),
        },
        "fsub.d fa0,fa0,fa1",
        0xab57553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRRR {
            frm: None,
            alu_op: FpuOPRRR::FmulD,
            rd: writable_fa0(),
            rs1: fa0(),
            rs2: fa1(),
        },
        "fmul.d fa0,fa0,fa1",
        0x12b57553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRRR {
            frm: None,
            alu_op: FpuOPRRR::FdivD,
            rd: writable_fa0(),
            rs1: fa0(),
            rs2: fa1(),
        },
        "fdiv.d fa0,fa0,fa1",
        0x1ab57553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRRR {
            frm: None,
            alu_op: FpuOPRRR::FsgnjD,
            rd: writable_fa0(),
            rs1: fa0(),
            rs2: fa1(),
        },
        "fsgnj.d fa0,fa0,fa1",
        0x22b50553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRRR {
            frm: None,
            alu_op: FpuOPRRR::FsgnjnD,
            rd: writable_fa0(),
            rs1: fa0(),
            rs2: fa1(),
        },
        "fsgnjn.d fa0,fa0,fa1",
        0x22b51553,
    ));

    insns.push(TestUnit::new(
        Inst::FpuRRR {
            frm: None,
            alu_op: FpuOPRRR::FsgnjxD,
            rd: writable_fa0(),
            rs1: fa0(),
            rs2: fa1(),
        },
        "fsgnjx.d fa0,fa0,fa1",
        0x22b52553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRRR {
            frm: None,
            alu_op: FpuOPRRR::FminD,
            rd: writable_fa0(),
            rs1: fa0(),
            rs2: fa1(),
        },
        "fmin.d fa0,fa0,fa1",
        0x2ab50553,
    ));

    insns.push(TestUnit::new(
        Inst::FpuRRR {
            frm: None,
            alu_op: FpuOPRRR::FmaxD,
            rd: writable_fa0(),
            rs1: fa0(),
            rs2: fa1(),
        },
        "fmax.d fa0,fa0,fa1",
        0x2ab51553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRRR {
            frm: None,
            alu_op: FpuOPRRR::FeqD,
            rd: writable_a0(),
            rs1: fa0(),
            rs2: fa1(),
        },
        "feq.d a0,fa0,fa1",
        0xa2b52553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRRR {
            frm: None,
            alu_op: FpuOPRRR::FltD,
            rd: writable_a0(),
            rs1: fa0(),
            rs2: fa1(),
        },
        "flt.d a0,fa0,fa1",
        0xa2b51553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRRR {
            frm: None,
            alu_op: FpuOPRRR::FleD,
            rd: writable_a0(),
            rs1: fa0(),
            rs2: fa1(),
        },
        "fle.d a0,fa0,fa1",
        0xa2b50553,
    ));

    //
    insns.push(TestUnit::new(
        Inst::FpuRR {
            frm: Some(FRM::RNE),
            alu_op: FpuOPRR::FsqrtS,
            rd: writable_fa0(),
            rs: fa1(),
        },
        "fsqrt.s fa0,fa1,rne",
        0x58058553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRR {
            frm: None,
            alu_op: FpuOPRR::FcvtWS,
            rd: writable_a0(),
            rs: fa1(),
        },
        "fcvt.w.s a0,fa1",
        0xc005f553,
    ));

    insns.push(TestUnit::new(
        Inst::FpuRR {
            frm: None,
            alu_op: FpuOPRR::FcvtWuS,
            rd: writable_a0(),
            rs: fa1(),
        },
        "fcvt.wu.s a0,fa1",
        0xc015f553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRR {
            frm: None,
            alu_op: FpuOPRR::FmvXW,
            rd: writable_a0(),
            rs: fa1(),
        },
        "fmv.x.w a0,fa1",
        0xe0058553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRR {
            frm: None,
            alu_op: FpuOPRR::FclassS,
            rd: writable_a0(),
            rs: fa1(),
        },
        "fclass.s a0,fa1",
        0xe0059553,
    ));

    insns.push(TestUnit::new(
        Inst::FpuRR {
            frm: None,
            alu_op: FpuOPRR::FcvtSw,
            rd: writable_fa0(),
            rs: a0(),
        },
        "fcvt.s.w fa0,a0",
        0xd0057553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRR {
            frm: None,
            alu_op: FpuOPRR::FcvtSwU,
            rd: writable_fa0(),
            rs: a0(),
        },
        "fcvt.s.wu fa0,a0",
        0xd0157553,
    ));

    insns.push(TestUnit::new(
        Inst::FpuRR {
            frm: None,
            alu_op: FpuOPRR::FmvWX,
            rd: writable_fa0(),
            rs: a0(),
        },
        "fmv.w.x fa0,a0",
        0xf0050553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRR {
            frm: None,
            alu_op: FpuOPRR::FcvtLS,
            rd: writable_a0(),
            rs: fa0(),
        },
        "fcvt.l.s a0,fa0",
        0xc0257553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRR {
            frm: None,
            alu_op: FpuOPRR::FcvtLuS,
            rd: writable_a0(),
            rs: fa0(),
        },
        "fcvt.lu.s a0,fa0",
        0xc0357553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRR {
            frm: None,

            alu_op: FpuOPRR::FcvtSL,
            rd: writable_fa0(),
            rs: a0(),
        },
        "fcvt.s.l fa0,a0",
        0xd0257553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRR {
            frm: None,
            alu_op: FpuOPRR::FcvtSLU,
            rd: writable_fa0(),
            rs: a0(),
        },
        "fcvt.s.lu fa0,a0",
        0xd0357553,
    ));

    //
    insns.push(TestUnit::new(
        Inst::FpuRR {
            frm: None,
            alu_op: FpuOPRR::FsqrtD,
            rd: writable_fa0(),
            rs: fa1(),
        },
        "fsqrt.d fa0,fa1",
        0x5a05f553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRR {
            frm: None,
            alu_op: FpuOPRR::FcvtWD,
            rd: writable_a0(),
            rs: fa1(),
        },
        "fcvt.w.d a0,fa1",
        0xc205f553,
    ));

    insns.push(TestUnit::new(
        Inst::FpuRR {
            frm: None,
            alu_op: FpuOPRR::FcvtWuD,
            rd: writable_a0(),
            rs: fa1(),
        },
        "fcvt.wu.d a0,fa1",
        0xc215f553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRR {
            frm: None,
            alu_op: FpuOPRR::FmvXD,
            rd: writable_a0(),
            rs: fa1(),
        },
        "fmv.x.d a0,fa1",
        0xe2058553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRR {
            frm: None,
            alu_op: FpuOPRR::FclassD,
            rd: writable_a0(),
            rs: fa1(),
        },
        "fclass.d a0,fa1",
        0xe2059553,
    ));

    insns.push(TestUnit::new(
        Inst::FpuRR {
            frm: None,
            alu_op: FpuOPRR::FcvtSD,
            rd: writable_fa0(),
            rs: fa0(),
        },
        "fcvt.s.d fa0,fa0",
        0x40157553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRR {
            frm: None,
            alu_op: FpuOPRR::FcvtDWU,
            rd: writable_fa0(),
            rs: a0(),
        },
        "fcvt.d.wu fa0,a0",
        0xd2150553,
    ));

    insns.push(TestUnit::new(
        Inst::FpuRR {
            frm: None,
            alu_op: FpuOPRR::FmvDX,
            rd: writable_fa0(),
            rs: a0(),
        },
        "fmv.d.x fa0,a0",
        0xf2050553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRR {
            frm: None,
            alu_op: FpuOPRR::FcvtLD,
            rd: writable_a0(),
            rs: fa0(),
        },
        "fcvt.l.d a0,fa0",
        0xc2257553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRR {
            frm: None,
            alu_op: FpuOPRR::FcvtLuD,
            rd: writable_a0(),
            rs: fa0(),
        },
        "fcvt.lu.d a0,fa0",
        0xc2357553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRR {
            frm: None,
            alu_op: FpuOPRR::FcvtDL,
            rd: writable_fa0(),
            rs: a0(),
        },
        "fcvt.d.l fa0,a0",
        0xd2257553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRR {
            frm: None,
            alu_op: FpuOPRR::FcvtDLu,
            rd: writable_fa0(),
            rs: a0(),
        },
        "fcvt.d.lu fa0,a0",
        0xd2357553,
    ));
    //////////////////////

    insns.push(TestUnit::new(
        Inst::FpuRRRR {
            frm: Some(FRM::RNE),
            alu_op: FpuOPRRRR::FmaddS,
            rd: writable_fa0(),
            rs1: fa0(),
            rs2: fa1(),
            rs3: fa7(),
        },
        "fmadd.s fa0,fa0,fa1,fa7,rne",
        0x88b50543,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRRRR {
            frm: None,
            alu_op: FpuOPRRRR::FmsubS,
            rd: writable_fa0(),
            rs1: fa0(),
            rs2: fa1(),
            rs3: fa7(),
        },
        "fmsub.s fa0,fa0,fa1,fa7",
        0x88b57547,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRRRR {
            frm: None,
            alu_op: FpuOPRRRR::FnmsubS,
            rd: writable_fa0(),
            rs1: fa0(),
            rs2: fa1(),
            rs3: fa7(),
        },
        "fnmsub.s fa0,fa0,fa1,fa7",
        0x88b5754b,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRRRR {
            frm: None,
            alu_op: FpuOPRRRR::FnmaddS,
            rd: writable_fa0(),
            rs1: fa0(),
            rs2: fa1(),
            rs3: fa7(),
        },
        "fnmadd.s fa0,fa0,fa1,fa7",
        0x88b5754f,
    ));

    insns.push(TestUnit::new(
        Inst::FpuRRRR {
            frm: None,
            alu_op: FpuOPRRRR::FmaddD,
            rd: writable_fa0(),
            rs1: fa0(),
            rs2: fa1(),
            rs3: fa7(),
        },
        "fmadd.d fa0,fa0,fa1,fa7",
        0x8ab57543,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRRRR {
            frm: None,

            alu_op: FpuOPRRRR::FmsubD,
            rd: writable_fa0(),
            rs1: fa0(),
            rs2: fa1(),
            rs3: fa7(),
        },
        "fmsub.d fa0,fa0,fa1,fa7",
        0x8ab57547,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRRRR {
            frm: None,
            alu_op: FpuOPRRRR::FnmsubD,
            rd: writable_fa0(),
            rs1: fa0(),
            rs2: fa1(),
            rs3: fa7(),
        },
        "fnmsub.d fa0,fa0,fa1,fa7",
        0x8ab5754b,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRRRR {
            frm: None,
            alu_op: FpuOPRRRR::FnmaddD,
            rd: writable_fa0(),
            rs1: fa0(),
            rs2: fa1(),
            rs3: fa7(),
        },
        "fnmadd.d fa0,fa0,fa1,fa7",
        0x8ab5754f,
    ));

    insns.push(TestUnit::new(
        Inst::Atomic {
            op: AtomicOP::LrW,
            rd: writable_a0(),
            addr: a1(),
            src: zero_reg(),
            amo: AMO::Relax,
        },
        "lr.w a0,(a1)",
        0x1005a52f,
    ));
    insns.push(TestUnit::new(
        Inst::Atomic {
            op: AtomicOP::ScW,
            rd: writable_a0(),
            addr: a1(),
            src: a2(),
            amo: AMO::Release,
        },
        "sc.w.rl a0,a2,(a1)",
        0x1ac5a52f,
    ));
    insns.push(TestUnit::new(
        Inst::Atomic {
            op: AtomicOP::AmoswapW,
            rd: writable_a0(),
            addr: a1(),
            src: a2(),
            amo: AMO::Aquire,
        },
        "amoswap.w.aq a0,a2,(a1)",
        0xcc5a52f,
    ));

    insns.push(TestUnit::new(
        Inst::Atomic {
            op: AtomicOP::AmoaddW,
            rd: writable_a0(),
            addr: a1(),
            src: a2(),
            amo: AMO::SeqCst,
        },
        "amoadd.w.aqrl a0,a2,(a1)",
        0x6c5a52f,
    ));
    insns.push(TestUnit::new(
        Inst::Atomic {
            op: AtomicOP::AmoxorW,
            rd: writable_a0(),
            addr: a1(),
            src: a2(),
            amo: AMO::Relax,
        },
        "amoxor.w a0,a2,(a1)",
        0x20c5a52f,
    ));
    insns.push(TestUnit::new(
        Inst::Atomic {
            op: AtomicOP::AmoandW,
            rd: writable_a0(),
            addr: a1(),
            src: a2(),
            amo: AMO::Relax,
        },
        "amoand.w a0,a2,(a1)",
        0x60c5a52f,
    ));

    insns.push(TestUnit::new(
        Inst::Atomic {
            op: AtomicOP::AmoorW,
            rd: writable_a0(),
            addr: a1(),
            src: a2(),
            amo: AMO::Relax,
        },
        "amoor.w a0,a2,(a1)",
        0x40c5a52f,
    ));
    insns.push(TestUnit::new(
        Inst::Atomic {
            op: AtomicOP::AmominW,
            rd: writable_a0(),
            addr: a1(),
            src: a2(),
            amo: AMO::Relax,
        },
        "amomin.w a0,a2,(a1)",
        0x80c5a52f,
    ));
    insns.push(TestUnit::new(
        Inst::Atomic {
            op: AtomicOP::AmomaxW,
            rd: writable_a0(),
            addr: a1(),
            src: a2(),
            amo: AMO::Relax,
        },
        "amomax.w a0,a2,(a1)",
        0xa0c5a52f,
    ));
    insns.push(TestUnit::new(
        Inst::Atomic {
            op: AtomicOP::AmominuW,
            rd: writable_a0(),
            addr: a1(),
            src: a2(),
            amo: AMO::Relax,
        },
        "amominu.w a0,a2,(a1)",
        0xc0c5a52f,
    ));
    insns.push(TestUnit::new(
        Inst::Atomic {
            op: AtomicOP::AmomaxuW,
            rd: writable_a0(),
            addr: a1(),
            src: a2(),
            amo: AMO::Relax,
        },
        "amomaxu.w a0,a2,(a1)",
        0xe0c5a52f,
    ));

    insns.push(TestUnit::new(
        Inst::Atomic {
            op: AtomicOP::LrD,
            rd: writable_a0(),
            addr: a1(),
            src: zero_reg(),
            amo: AMO::Relax,
        },
        "lr.d a0,(a1)",
        0x1005b52f,
    ));
    insns.push(TestUnit::new(
        Inst::Atomic {
            op: AtomicOP::ScD,
            rd: writable_a0(),
            addr: a1(),
            src: a2(),
            amo: AMO::Relax,
        },
        "sc.d a0,a2,(a1)",
        0x18c5b52f,
    ));

    insns.push(TestUnit::new(
        Inst::Atomic {
            op: AtomicOP::AmoswapD,
            rd: writable_a0(),
            addr: a1(),
            src: a2(),
            amo: AMO::Relax,
        },
        "amoswap.d a0,a2,(a1)",
        0x8c5b52f,
    ));

    insns.push(TestUnit::new(
        Inst::Atomic {
            op: AtomicOP::AmoaddD,
            rd: writable_a0(),
            addr: a1(),
            src: a2(),
            amo: AMO::Relax,
        },
        "amoadd.d a0,a2,(a1)",
        0xc5b52f,
    ));
    insns.push(TestUnit::new(
        Inst::Atomic {
            op: AtomicOP::AmoxorD,
            rd: writable_a0(),
            addr: a1(),
            src: a2(),
            amo: AMO::Relax,
        },
        "amoxor.d a0,a2,(a1)",
        0x20c5b52f,
    ));
    insns.push(TestUnit::new(
        Inst::Atomic {
            op: AtomicOP::AmoandD,
            rd: writable_a0(),
            addr: a1(),
            src: a2(),
            amo: AMO::Relax,
        },
        "amoand.d a0,a2,(a1)",
        0x60c5b52f,
    ));

    insns.push(TestUnit::new(
        Inst::Atomic {
            op: AtomicOP::AmoorD,
            rd: writable_a0(),
            addr: a1(),
            src: a2(),
            amo: AMO::Relax,
        },
        "amoor.d a0,a2,(a1)",
        0x40c5b52f,
    ));
    insns.push(TestUnit::new(
        Inst::Atomic {
            op: AtomicOP::AmominD,
            rd: writable_a0(),
            addr: a1(),
            src: a2(),
            amo: AMO::Relax,
        },
        "amomin.d a0,a2,(a1)",
        0x80c5b52f,
    ));
    insns.push(TestUnit::new(
        Inst::Atomic {
            op: AtomicOP::AmomaxD,
            rd: writable_a0(),
            addr: a1(),
            src: a2(),
            amo: AMO::Relax,
        },
        "amomax.d a0,a2,(a1)",
        0xa0c5b52f,
    ));
    insns.push(TestUnit::new(
        Inst::Atomic {
            op: AtomicOP::AmominuD,
            rd: writable_a0(),
            addr: a1(),
            src: a2(),
            amo: AMO::Relax,
        },
        "amominu.d a0,a2,(a1)",
        0xc0c5b52f,
    ));
    insns.push(TestUnit::new(
        Inst::Atomic {
            op: AtomicOP::AmomaxuD,
            rd: writable_a0(),
            addr: a1(),
            src: a2(),
            amo: AMO::Relax,
        },
        "amomaxu.d a0,a2,(a1)",
        0xe0c5b52f,
    ));

    /////////
    insns.push(TestUnit::new(
        Inst::Fence {
            pred: 1,
            succ: 1 << 1,
        },
        "fence w,r",
        0x120000f,
    ));
    insns.push(TestUnit::new(Inst::FenceI {}, "fence.i", 0x100f));
    insns.push(TestUnit::new(Inst::ECall {}, "ecall", 0x73));
    insns.push(TestUnit::new(Inst::EBreak {}, "ebreak", 0x100073));

    insns.push(TestUnit::new(
        Inst::FpuRRR {
            alu_op: FpuOPRRR::FsgnjS,
            frm: None,
            rd: writable_fa0(),
            rs1: fa1(),
            rs2: fa1(),
        },
        "fmv.s fa0,fa1",
        0x20b58553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRRR {
            alu_op: FpuOPRRR::FsgnjD,
            frm: None,
            rd: writable_fa0(),
            rs1: fa1(),
            rs2: fa1(),
        },
        "fmv.d fa0,fa1",
        0x22b58553,
    ));

    insns.push(TestUnit::new(
        Inst::FpuRRR {
            alu_op: FpuOPRRR::FsgnjnS,
            frm: None,
            rd: writable_fa0(),
            rs1: fa1(),
            rs2: fa1(),
        },
        "fneg.s fa0,fa1",
        0x20b59553,
    ));
    insns.push(TestUnit::new(
        Inst::FpuRRR {
            alu_op: FpuOPRRR::FsgnjnD,
            frm: None,
            rd: writable_fa0(),
            rs1: fa1(),
            rs2: fa1(),
        },
        "fneg.d fa0,fa1",
        0x22b59553,
    ));

    let (flags, isa_flags) = make_test_flags();
    let emit_info = EmitInfo::new(flags, isa_flags);

    for unit in insns.iter() {
        println!("Riscv64: {:?}, {}", unit.inst, unit.assembly);
        // Check the printed text is as expected.
        let actual_printing = unit
            .inst
            .print_with_state(&mut EmitState::default(), &mut AllocationConsumer::new(&[]));
        assert_eq!(unit.assembly, actual_printing);
        let mut buffer = MachBuffer::new();
        unit.inst
            .emit(&[], &mut buffer, &emit_info, &mut Default::default());
        let buffer = buffer.finish();
        if buffer.data() != unit.code.to_le_bytes() {
            {
                let gnu = DebugRTypeInst::from_bs(&unit.code.to_le_bytes());
                let my = DebugRTypeInst::from_bs(buffer.data());
                println!("gnu:{:?}", gnu);
                println!("my :{:?}", my);
                // println!("gnu:{:b}", gnu.funct7);
                // println!("my :{:b}", my.funct7);
            }

            {
                let gnu = DebugITypeInst::from_bs(&unit.code.to_le_bytes());
                let my = DebugITypeInst::from_bs(buffer.data());
                println!("gnu:{:?}", gnu);
                println!("my :{:?}", my);
                println!("gnu:{:b}", gnu.op_code);
                println!("my :{:b}", my.op_code);
            }
            assert_eq!(buffer.data(), unit.code.to_le_bytes());
        }
    }
}

fn make_test_flags() -> (settings::Flags, super::super::riscv_settings::Flags) {
    let b = settings::builder();
    let flags = settings::Flags::new(b.clone());
    let b2 = super::super::riscv_settings::builder();
    let isa_flags = super::super::riscv_settings::Flags::new(&flags, b2);
    (flags, isa_flags)
}

#[derive(Debug)]
pub(crate) struct DebugRTypeInst {
    op_code: u32,
    rd: u32,
    funct3: u32,
    rs1: u32,
    rs2: u32,
    funct7: u32,
}

impl DebugRTypeInst {
    pub(crate) fn from_bs(x: &[u8]) -> Self {
        let a = [x[0], x[1], x[2], x[3]];
        Self::from_u32(u32::from_le_bytes(a))
    }

    pub(crate) fn from_u32(x: u32) -> Self {
        let op_code = x & 0b111_1111;
        let x = x >> 7;
        let rd = x & 0b1_1111;
        let x = x >> 5;
        let funct3 = x & 0b111;
        let x = x >> 3;
        let rs1 = x & 0b1_1111;
        let x = x >> 5;
        let rs2 = x & 0b1_1111;
        let x = x >> 5;
        let funct7 = x & 0b111_1111;
        Self {
            op_code,
            rd,
            funct3,
            rs1,
            rs2,
            funct7,
        }
    }
}

#[derive(Debug)]
pub(crate) struct DebugITypeInst {
    op_code: u32,
    rd: u32,
    funct3: u32,
    rs: u32,
    imm12: u32,
    shamt5: u32,
    shamt6: u32,
    funct7: u32,
    funct6: u32,
}

impl DebugITypeInst {
    pub(crate) fn from_bs(x: &[u8]) -> Self {
        let a = [x[0], x[1], x[2], x[3]];
        Self::from_u32(u32::from_le_bytes(a))
    }
    pub(crate) fn from_u32(x: u32) -> Self {
        let op_code = x & 0b111_1111;
        let x = x >> 7;
        let rd = x & 0b1_1111;
        let x = x >> 5;
        let funct3 = x & 0b111;
        let x = x >> 3;
        let rs = x & 0b1_1111;
        let x = x >> 5;
        let imm12 = x & 0b1111_1111_1111;
        let shamt5 = imm12 & 0b1_1111;
        let shamt6 = imm12 & 0b11_1111;
        let funct7 = imm12 >> 5;
        let funct6 = funct7 >> 1;
        Self {
            op_code,
            rd,
            funct3,
            rs,
            imm12,
            shamt5,
            shamt6,
            funct7,
            funct6,
        }
    }
    fn print_b(self) {
        println!("opcode:{:b}", self.op_code);
        println!("rd:{}", self.rd);
        println!("funct3:{:b}", self.funct3);
        println!("rs:{}", self.rs);
        println!("shamt5:{:b}", self.shamt5);
        println!("shamt6:{:b}", self.shamt6);
        println!("funct6:{:b}", self.funct6);
        println!("funct7:{:b}", self.funct7);
    }
}

#[test]
fn xxx() {
    let x = 1240847763;
    let x = DebugITypeInst::from_u32(x);
    x.print_b();
}

#[test]
fn riscv64_worst_case_instruction_size() {
    let (flags, isa_flags) = make_test_flags();
    let emit_info = EmitInfo::new(flags, isa_flags);

    //there are all candidates potential generate a lot of bytes.
    let mut candidates: Vec<MInst> = vec![];

    candidates.push(Inst::Fcmp {
        rd: writable_a0(),
        cc: FloatCC::UnorderedOrLessThanOrEqual,
        ty: F64,
        rs1: fa1(),
        rs2: fa0(),
    });

    candidates.push(Inst::IntSelect {
        dst: vec![writable_a0(), writable_a0()],
        ty: I128,
        op: IntSelectOP::Smax,
        x: ValueRegs::two(x_reg(1), x_reg(2)),
        y: ValueRegs::two(x_reg(3), x_reg(4)),
    });

    candidates.push(Inst::FcvtToInt {
        rd: writable_a0(),
        rs: fa0(),
        is_signed: true,
        in_type: F64,
        out_type: I64,
        is_sat: true,
        tmp: writable_a1(),
    });

    candidates.push(Inst::FcvtToInt {
        rd: writable_a0(),
        rs: fa0(),
        is_signed: true,
        in_type: F64,
        out_type: I64,
        is_sat: false,
        tmp: writable_a1(),
    });

    candidates.push(Inst::FloatRound {
        op: FloatRoundOP::Trunc,
        int_tmp: writable_a0(),
        f_tmp: writable_a0(),
        rd: writable_fa0(),
        rs: fa0(),
        ty: F64,
    });

    candidates.push(Inst::FloatSelect {
        op: FloatSelectOP::Max,
        rd: writable_fa0(),
        tmp: writable_a0(),
        rs1: fa0(),
        rs2: fa0(),
        ty: F64,
    });

    let mut max: (u32, MInst) = (0, Inst::Nop0);
    for i in candidates {
        let mut buffer = MachBuffer::new();
        i.emit(&[], &mut buffer, &emit_info, &mut Default::default());
        let buffer = buffer.finish();
        let length = buffer.data().len() as u32;
        if length > max.0 {
            let length = buffer.data().len() as u32;
            max = (length, i.clone());
        }
        println!("insn:{:?}  length: {}", i, length);
    }
    println!("calculate max size is {} , inst is {:?}", max.0, max.1);
    assert!(max.0 <= Inst::worst_case_size());
}
