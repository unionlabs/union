use crate::ir::{MemFlags, TrapCode};
use crate::isa::s390x::inst::*;
use crate::isa::s390x::settings as s390x_settings;
use crate::settings;
use alloc::vec::Vec;

#[test]
fn test_s390x_binemit() {
    let mut insns = Vec::<(Inst, &str, &str)>::new();

    insns.push((Inst::Nop0, "", "nop-zero-len"));
    insns.push((Inst::Nop2, "0707", "nop"));

    insns.push((
        Inst::AluRRR {
            alu_op: ALUOp::Add32,
            rd: writable_gpr(1),
            rn: gpr(2),
            rm: gpr(3),
        },
        "B9F83012",
        "ark %r1, %r2, %r3",
    ));
    insns.push((
        Inst::AluRRR {
            alu_op: ALUOp::Add64,
            rd: writable_gpr(4),
            rn: gpr(5),
            rm: gpr(6),
        },
        "B9E86045",
        "agrk %r4, %r5, %r6",
    ));
    insns.push((
        Inst::AluRRR {
            alu_op: ALUOp::AddLogical32,
            rd: writable_gpr(1),
            rn: gpr(2),
            rm: gpr(3),
        },
        "B9FA3012",
        "alrk %r1, %r2, %r3",
    ));
    insns.push((
        Inst::AluRRR {
            alu_op: ALUOp::AddLogical64,
            rd: writable_gpr(4),
            rn: gpr(5),
            rm: gpr(6),
        },
        "B9EA6045",
        "algrk %r4, %r5, %r6",
    ));
    insns.push((
        Inst::AluRRR {
            alu_op: ALUOp::Sub32,
            rd: writable_gpr(1),
            rn: gpr(2),
            rm: gpr(3),
        },
        "B9F93012",
        "srk %r1, %r2, %r3",
    ));
    insns.push((
        Inst::AluRRR {
            alu_op: ALUOp::Sub64,
            rd: writable_gpr(4),
            rn: gpr(5),
            rm: gpr(6),
        },
        "B9E96045",
        "sgrk %r4, %r5, %r6",
    ));
    insns.push((
        Inst::AluRRR {
            alu_op: ALUOp::SubLogical32,
            rd: writable_gpr(1),
            rn: gpr(2),
            rm: gpr(3),
        },
        "B9FB3012",
        "slrk %r1, %r2, %r3",
    ));
    insns.push((
        Inst::AluRRR {
            alu_op: ALUOp::SubLogical64,
            rd: writable_gpr(4),
            rn: gpr(5),
            rm: gpr(6),
        },
        "B9EB6045",
        "slgrk %r4, %r5, %r6",
    ));
    insns.push((
        Inst::AluRRR {
            alu_op: ALUOp::Mul32,
            rd: writable_gpr(1),
            rn: gpr(2),
            rm: gpr(3),
        },
        "B9FD3012",
        "msrkc %r1, %r2, %r3",
    ));
    insns.push((
        Inst::AluRRR {
            alu_op: ALUOp::Mul64,
            rd: writable_gpr(4),
            rn: gpr(5),
            rm: gpr(6),
        },
        "B9ED6045",
        "msgrkc %r4, %r5, %r6",
    ));
    insns.push((
        Inst::AluRRR {
            alu_op: ALUOp::And32,
            rd: writable_gpr(1),
            rn: gpr(2),
            rm: gpr(3),
        },
        "B9F43012",
        "nrk %r1, %r2, %r3",
    ));
    insns.push((
        Inst::AluRRR {
            alu_op: ALUOp::And64,
            rd: writable_gpr(4),
            rn: gpr(5),
            rm: gpr(6),
        },
        "B9E46045",
        "ngrk %r4, %r5, %r6",
    ));
    insns.push((
        Inst::AluRRR {
            alu_op: ALUOp::Orr32,
            rd: writable_gpr(1),
            rn: gpr(2),
            rm: gpr(3),
        },
        "B9F63012",
        "ork %r1, %r2, %r3",
    ));
    insns.push((
        Inst::AluRRR {
            alu_op: ALUOp::Orr64,
            rd: writable_gpr(4),
            rn: gpr(5),
            rm: gpr(6),
        },
        "B9E66045",
        "ogrk %r4, %r5, %r6",
    ));
    insns.push((
        Inst::AluRRR {
            alu_op: ALUOp::Xor32,
            rd: writable_gpr(1),
            rn: gpr(2),
            rm: gpr(3),
        },
        "B9F73012",
        "xrk %r1, %r2, %r3",
    ));
    insns.push((
        Inst::AluRRR {
            alu_op: ALUOp::Xor64,
            rd: writable_gpr(4),
            rn: gpr(5),
            rm: gpr(6),
        },
        "B9E76045",
        "xgrk %r4, %r5, %r6",
    ));
    insns.push((
        Inst::AluRRR {
            alu_op: ALUOp::AndNot32,
            rd: writable_gpr(1),
            rn: gpr(2),
            rm: gpr(3),
        },
        "B9743012",
        "nnrk %r1, %r2, %r3",
    ));
    insns.push((
        Inst::AluRRR {
            alu_op: ALUOp::AndNot64,
            rd: writable_gpr(4),
            rn: gpr(5),
            rm: gpr(6),
        },
        "B9646045",
        "nngrk %r4, %r5, %r6",
    ));
    insns.push((
        Inst::AluRRR {
            alu_op: ALUOp::OrrNot32,
            rd: writable_gpr(1),
            rn: gpr(2),
            rm: gpr(3),
        },
        "B9763012",
        "nork %r1, %r2, %r3",
    ));
    insns.push((
        Inst::AluRRR {
            alu_op: ALUOp::OrrNot64,
            rd: writable_gpr(4),
            rn: gpr(5),
            rm: gpr(6),
        },
        "B9666045",
        "nogrk %r4, %r5, %r6",
    ));
    insns.push((
        Inst::AluRRR {
            alu_op: ALUOp::XorNot32,
            rd: writable_gpr(1),
            rn: gpr(2),
            rm: gpr(3),
        },
        "B9773012",
        "nxrk %r1, %r2, %r3",
    ));
    insns.push((
        Inst::AluRRR {
            alu_op: ALUOp::XorNot64,
            rd: writable_gpr(4),
            rn: gpr(5),
            rm: gpr(6),
        },
        "B9676045",
        "nxgrk %r4, %r5, %r6",
    ));

    insns.push((
        Inst::AluRRSImm16 {
            alu_op: ALUOp::Add32,
            rd: writable_gpr(4),
            rn: gpr(5),
            imm: -32768,
        },
        "EC45800000D8",
        "ahik %r4, %r5, -32768",
    ));
    insns.push((
        Inst::AluRRSImm16 {
            alu_op: ALUOp::Add32,
            rd: writable_gpr(4),
            rn: gpr(5),
            imm: 32767,
        },
        "EC457FFF00D8",
        "ahik %r4, %r5, 32767",
    ));
    insns.push((
        Inst::AluRRSImm16 {
            alu_op: ALUOp::Add64,
            rd: writable_gpr(4),
            rn: gpr(5),
            imm: -32768,
        },
        "EC45800000D9",
        "aghik %r4, %r5, -32768",
    ));
    insns.push((
        Inst::AluRRSImm16 {
            alu_op: ALUOp::Add64,
            rd: writable_gpr(4),
            rn: gpr(5),
            imm: 32767,
        },
        "EC457FFF00D9",
        "aghik %r4, %r5, 32767",
    ));

    insns.push((
        Inst::AluRR {
            alu_op: ALUOp::Add32,
            rd: writable_gpr(1),
            rm: gpr(2),
        },
        "1A12",
        "ar %r1, %r2",
    ));
    insns.push((
        Inst::AluRR {
            alu_op: ALUOp::Add64,
            rd: writable_gpr(4),
            rm: gpr(5),
        },
        "B9080045",
        "agr %r4, %r5",
    ));
    insns.push((
        Inst::AluRR {
            alu_op: ALUOp::Add64Ext32,
            rd: writable_gpr(4),
            rm: gpr(5),
        },
        "B9180045",
        "agfr %r4, %r5",
    ));
    insns.push((
        Inst::AluRR {
            alu_op: ALUOp::AddLogical32,
            rd: writable_gpr(1),
            rm: gpr(2),
        },
        "1E12",
        "alr %r1, %r2",
    ));
    insns.push((
        Inst::AluRR {
            alu_op: ALUOp::AddLogical64,
            rd: writable_gpr(4),
            rm: gpr(5),
        },
        "B90A0045",
        "algr %r4, %r5",
    ));
    insns.push((
        Inst::AluRR {
            alu_op: ALUOp::AddLogical64Ext32,
            rd: writable_gpr(4),
            rm: gpr(5),
        },
        "B91A0045",
        "algfr %r4, %r5",
    ));
    insns.push((
        Inst::AluRR {
            alu_op: ALUOp::Sub32,
            rd: writable_gpr(1),
            rm: gpr(2),
        },
        "1B12",
        "sr %r1, %r2",
    ));
    insns.push((
        Inst::AluRR {
            alu_op: ALUOp::Sub64,
            rd: writable_gpr(4),
            rm: gpr(5),
        },
        "B9090045",
        "sgr %r4, %r5",
    ));
    insns.push((
        Inst::AluRR {
            alu_op: ALUOp::Sub64Ext32,
            rd: writable_gpr(4),
            rm: gpr(5),
        },
        "B9190045",
        "sgfr %r4, %r5",
    ));
    insns.push((
        Inst::AluRR {
            alu_op: ALUOp::SubLogical32,
            rd: writable_gpr(1),
            rm: gpr(2),
        },
        "1F12",
        "slr %r1, %r2",
    ));
    insns.push((
        Inst::AluRR {
            alu_op: ALUOp::SubLogical64,
            rd: writable_gpr(4),
            rm: gpr(5),
        },
        "B90B0045",
        "slgr %r4, %r5",
    ));
    insns.push((
        Inst::AluRR {
            alu_op: ALUOp::SubLogical64Ext32,
            rd: writable_gpr(4),
            rm: gpr(5),
        },
        "B91B0045",
        "slgfr %r4, %r5",
    ));
    insns.push((
        Inst::AluRR {
            alu_op: ALUOp::Mul32,
            rd: writable_gpr(1),
            rm: gpr(2),
        },
        "B2520012",
        "msr %r1, %r2",
    ));
    insns.push((
        Inst::AluRR {
            alu_op: ALUOp::Mul64,
            rd: writable_gpr(4),
            rm: gpr(5),
        },
        "B90C0045",
        "msgr %r4, %r5",
    ));
    insns.push((
        Inst::AluRR {
            alu_op: ALUOp::Mul64Ext32,
            rd: writable_gpr(4),
            rm: gpr(5),
        },
        "B91C0045",
        "msgfr %r4, %r5",
    ));
    insns.push((
        Inst::AluRR {
            alu_op: ALUOp::And32,
            rd: writable_gpr(1),
            rm: gpr(2),
        },
        "1412",
        "nr %r1, %r2",
    ));
    insns.push((
        Inst::AluRR {
            alu_op: ALUOp::And64,
            rd: writable_gpr(4),
            rm: gpr(5),
        },
        "B9800045",
        "ngr %r4, %r5",
    ));
    insns.push((
        Inst::AluRR {
            alu_op: ALUOp::Orr32,
            rd: writable_gpr(1),
            rm: gpr(2),
        },
        "1612",
        "or %r1, %r2",
    ));
    insns.push((
        Inst::AluRR {
            alu_op: ALUOp::Orr64,
            rd: writable_gpr(4),
            rm: gpr(5),
        },
        "B9810045",
        "ogr %r4, %r5",
    ));
    insns.push((
        Inst::AluRR {
            alu_op: ALUOp::Xor32,
            rd: writable_gpr(1),
            rm: gpr(2),
        },
        "1712",
        "xr %r1, %r2",
    ));
    insns.push((
        Inst::AluRR {
            alu_op: ALUOp::Xor64,
            rd: writable_gpr(4),
            rm: gpr(5),
        },
        "B9820045",
        "xgr %r4, %r5",
    ));

    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::Add32,
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "5A102000",
        "a %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::Add32Ext16,
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "4A102000",
        "ah %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::Add32,
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102000005A",
        "ay %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::Add32Ext16,
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102000004A",
        "ahy %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::Add64,
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020000008",
        "ag %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::Add64Ext16,
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020000038",
        "agh %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::Add64Ext32,
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020000018",
        "agf %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::AddLogical32,
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "5E102000",
        "al %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::AddLogical32,
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102000005E",
        "aly %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::AddLogical64,
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102000000A",
        "alg %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::AddLogical64Ext32,
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102000001A",
        "algf %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::Sub32,
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "5B102000",
        "s %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::Sub32Ext16,
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "4B102000",
        "sh %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::Sub32,
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102000005B",
        "sy %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::Sub32Ext16,
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102000007B",
        "shy %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::Sub64,
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020000009",
        "sg %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::Sub64Ext16,
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020000039",
        "sgh %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::Sub64Ext32,
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020000019",
        "sgf %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::SubLogical32,
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "5F102000",
        "sl %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::SubLogical32,
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102000005F",
        "sly %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::SubLogical64,
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102000000B",
        "slg %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::SubLogical64Ext32,
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102000001B",
        "slgf %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::Mul32,
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "71102000",
        "ms %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::Mul32Ext16,
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "4C102000",
        "mh %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::Mul32,
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020000051",
        "msy %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::Mul32Ext16,
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102000007C",
        "mhy %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::Mul64,
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102000000C",
        "msg %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::Mul64Ext16,
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102000003C",
        "mgh %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::Mul64Ext32,
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102000001C",
        "msgf %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::And32,
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "54102000",
        "n %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::And32,
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020000054",
        "ny %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::And64,
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020000080",
        "ng %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::Orr32,
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "56102000",
        "o %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::Orr32,
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020000056",
        "oy %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::Orr64,
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020000081",
        "og %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::Xor32,
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "57102000",
        "x %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::Xor32,
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020000057",
        "xy %r1, 0(%r2)",
    ));
    insns.push((
        Inst::AluRX {
            alu_op: ALUOp::Xor64,
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020000082",
        "xg %r1, 0(%r2)",
    ));

    insns.push((
        Inst::AluRSImm16 {
            alu_op: ALUOp::Add32,
            rd: writable_gpr(7),
            imm: -32768,
        },
        "A77A8000",
        "ahi %r7, -32768",
    ));
    insns.push((
        Inst::AluRSImm16 {
            alu_op: ALUOp::Add32,
            rd: writable_gpr(7),
            imm: 32767,
        },
        "A77A7FFF",
        "ahi %r7, 32767",
    ));
    insns.push((
        Inst::AluRSImm16 {
            alu_op: ALUOp::Add64,
            rd: writable_gpr(7),
            imm: -32768,
        },
        "A77B8000",
        "aghi %r7, -32768",
    ));
    insns.push((
        Inst::AluRSImm16 {
            alu_op: ALUOp::Add64,
            rd: writable_gpr(7),
            imm: 32767,
        },
        "A77B7FFF",
        "aghi %r7, 32767",
    ));
    insns.push((
        Inst::AluRSImm16 {
            alu_op: ALUOp::Mul32,
            rd: writable_gpr(7),
            imm: -32768,
        },
        "A77C8000",
        "mhi %r7, -32768",
    ));
    insns.push((
        Inst::AluRSImm16 {
            alu_op: ALUOp::Mul32,
            rd: writable_gpr(7),
            imm: 32767,
        },
        "A77C7FFF",
        "mhi %r7, 32767",
    ));
    insns.push((
        Inst::AluRSImm16 {
            alu_op: ALUOp::Mul64,
            rd: writable_gpr(7),
            imm: -32768,
        },
        "A77D8000",
        "mghi %r7, -32768",
    ));
    insns.push((
        Inst::AluRSImm16 {
            alu_op: ALUOp::Mul64,
            rd: writable_gpr(7),
            imm: 32767,
        },
        "A77D7FFF",
        "mghi %r7, 32767",
    ));

    insns.push((
        Inst::AluRSImm32 {
            alu_op: ALUOp::Add32,
            rd: writable_gpr(7),
            imm: -2147483648,
        },
        "C27980000000",
        "afi %r7, -2147483648",
    ));
    insns.push((
        Inst::AluRSImm32 {
            alu_op: ALUOp::Add32,
            rd: writable_gpr(7),
            imm: 2147483647,
        },
        "C2797FFFFFFF",
        "afi %r7, 2147483647",
    ));
    insns.push((
        Inst::AluRSImm32 {
            alu_op: ALUOp::Mul32,
            rd: writable_gpr(7),
            imm: -2147483648,
        },
        "C27180000000",
        "msfi %r7, -2147483648",
    ));
    insns.push((
        Inst::AluRSImm32 {
            alu_op: ALUOp::Mul32,
            rd: writable_gpr(7),
            imm: 2147483647,
        },
        "C2717FFFFFFF",
        "msfi %r7, 2147483647",
    ));
    insns.push((
        Inst::AluRSImm32 {
            alu_op: ALUOp::Add64,
            rd: writable_gpr(7),
            imm: -2147483648,
        },
        "C27880000000",
        "agfi %r7, -2147483648",
    ));
    insns.push((
        Inst::AluRSImm32 {
            alu_op: ALUOp::Add64,
            rd: writable_gpr(7),
            imm: 2147483647,
        },
        "C2787FFFFFFF",
        "agfi %r7, 2147483647",
    ));
    insns.push((
        Inst::AluRSImm32 {
            alu_op: ALUOp::Mul64,
            rd: writable_gpr(7),
            imm: -2147483648,
        },
        "C27080000000",
        "msgfi %r7, -2147483648",
    ));
    insns.push((
        Inst::AluRSImm32 {
            alu_op: ALUOp::Mul64,
            rd: writable_gpr(7),
            imm: 2147483647,
        },
        "C2707FFFFFFF",
        "msgfi %r7, 2147483647",
    ));

    insns.push((
        Inst::AluRUImm32 {
            alu_op: ALUOp::AddLogical32,
            rd: writable_gpr(7),
            imm: 0,
        },
        "C27B00000000",
        "alfi %r7, 0",
    ));
    insns.push((
        Inst::AluRUImm32 {
            alu_op: ALUOp::AddLogical32,
            rd: writable_gpr(7),
            imm: 4294967295,
        },
        "C27BFFFFFFFF",
        "alfi %r7, 4294967295",
    ));
    insns.push((
        Inst::AluRUImm32 {
            alu_op: ALUOp::SubLogical32,
            rd: writable_gpr(7),
            imm: 0,
        },
        "C27500000000",
        "slfi %r7, 0",
    ));
    insns.push((
        Inst::AluRUImm32 {
            alu_op: ALUOp::SubLogical32,
            rd: writable_gpr(7),
            imm: 4294967295,
        },
        "C275FFFFFFFF",
        "slfi %r7, 4294967295",
    ));
    insns.push((
        Inst::AluRUImm32 {
            alu_op: ALUOp::AddLogical64,
            rd: writable_gpr(7),
            imm: 0,
        },
        "C27A00000000",
        "algfi %r7, 0",
    ));
    insns.push((
        Inst::AluRUImm32 {
            alu_op: ALUOp::AddLogical64,
            rd: writable_gpr(7),
            imm: 4294967295,
        },
        "C27AFFFFFFFF",
        "algfi %r7, 4294967295",
    ));
    insns.push((
        Inst::AluRUImm32 {
            alu_op: ALUOp::SubLogical64,
            rd: writable_gpr(7),
            imm: 0,
        },
        "C27400000000",
        "slgfi %r7, 0",
    ));
    insns.push((
        Inst::AluRUImm32 {
            alu_op: ALUOp::SubLogical64,
            rd: writable_gpr(7),
            imm: 4294967295,
        },
        "C274FFFFFFFF",
        "slgfi %r7, 4294967295",
    ));

    insns.push((
        Inst::AluRUImm16Shifted {
            alu_op: ALUOp::And32,
            rd: writable_gpr(8),
            imm: UImm16Shifted::maybe_from_u64(0x0000_ffff).unwrap(),
        },
        "A587FFFF",
        "nill %r8, 65535",
    ));
    insns.push((
        Inst::AluRUImm16Shifted {
            alu_op: ALUOp::And32,
            rd: writable_gpr(8),
            imm: UImm16Shifted::maybe_from_u64(0xffff_0000).unwrap(),
        },
        "A586FFFF",
        "nilh %r8, 65535",
    ));
    insns.push((
        Inst::AluRUImm16Shifted {
            alu_op: ALUOp::And64,
            rd: writable_gpr(8),
            imm: UImm16Shifted::maybe_from_u64(0x0000_0000_0000_ffff).unwrap(),
        },
        "A587FFFF",
        "nill %r8, 65535",
    ));
    insns.push((
        Inst::AluRUImm16Shifted {
            alu_op: ALUOp::And64,
            rd: writable_gpr(8),
            imm: UImm16Shifted::maybe_from_u64(0x0000_0000_ffff_0000).unwrap(),
        },
        "A586FFFF",
        "nilh %r8, 65535",
    ));
    insns.push((
        Inst::AluRUImm16Shifted {
            alu_op: ALUOp::And64,
            rd: writable_gpr(8),
            imm: UImm16Shifted::maybe_from_u64(0x0000_ffff_0000_0000).unwrap(),
        },
        "A585FFFF",
        "nihl %r8, 65535",
    ));
    insns.push((
        Inst::AluRUImm16Shifted {
            alu_op: ALUOp::And64,
            rd: writable_gpr(8),
            imm: UImm16Shifted::maybe_from_u64(0xffff_0000_0000_0000).unwrap(),
        },
        "A584FFFF",
        "nihh %r8, 65535",
    ));
    insns.push((
        Inst::AluRUImm16Shifted {
            alu_op: ALUOp::Orr32,
            rd: writable_gpr(8),
            imm: UImm16Shifted::maybe_from_u64(0x0000_ffff).unwrap(),
        },
        "A58BFFFF",
        "oill %r8, 65535",
    ));
    insns.push((
        Inst::AluRUImm16Shifted {
            alu_op: ALUOp::Orr32,
            rd: writable_gpr(8),
            imm: UImm16Shifted::maybe_from_u64(0xffff_0000).unwrap(),
        },
        "A58AFFFF",
        "oilh %r8, 65535",
    ));
    insns.push((
        Inst::AluRUImm16Shifted {
            alu_op: ALUOp::Orr64,
            rd: writable_gpr(8),
            imm: UImm16Shifted::maybe_from_u64(0x0000_0000_0000_ffff).unwrap(),
        },
        "A58BFFFF",
        "oill %r8, 65535",
    ));
    insns.push((
        Inst::AluRUImm16Shifted {
            alu_op: ALUOp::Orr64,
            rd: writable_gpr(8),
            imm: UImm16Shifted::maybe_from_u64(0x0000_0000_ffff_0000).unwrap(),
        },
        "A58AFFFF",
        "oilh %r8, 65535",
    ));
    insns.push((
        Inst::AluRUImm16Shifted {
            alu_op: ALUOp::Orr64,
            rd: writable_gpr(8),
            imm: UImm16Shifted::maybe_from_u64(0x0000_ffff_0000_0000).unwrap(),
        },
        "A589FFFF",
        "oihl %r8, 65535",
    ));
    insns.push((
        Inst::AluRUImm16Shifted {
            alu_op: ALUOp::Orr64,
            rd: writable_gpr(8),
            imm: UImm16Shifted::maybe_from_u64(0xffff_0000_0000_0000).unwrap(),
        },
        "A588FFFF",
        "oihh %r8, 65535",
    ));

    insns.push((
        Inst::AluRUImm32Shifted {
            alu_op: ALUOp::And32,
            rd: writable_gpr(8),
            imm: UImm32Shifted::maybe_from_u64(0xffff_ffff).unwrap(),
        },
        "C08BFFFFFFFF",
        "nilf %r8, 4294967295",
    ));
    insns.push((
        Inst::AluRUImm32Shifted {
            alu_op: ALUOp::And64,
            rd: writable_gpr(8),
            imm: UImm32Shifted::maybe_from_u64(0x0000_0000_ffff_ffff).unwrap(),
        },
        "C08BFFFFFFFF",
        "nilf %r8, 4294967295",
    ));
    insns.push((
        Inst::AluRUImm32Shifted {
            alu_op: ALUOp::And64,
            rd: writable_gpr(8),
            imm: UImm32Shifted::maybe_from_u64(0xffff_ffff_0000_0000).unwrap(),
        },
        "C08AFFFFFFFF",
        "nihf %r8, 4294967295",
    ));
    insns.push((
        Inst::AluRUImm32Shifted {
            alu_op: ALUOp::Orr32,
            rd: writable_gpr(8),
            imm: UImm32Shifted::maybe_from_u64(0xffff_ffff).unwrap(),
        },
        "C08DFFFFFFFF",
        "oilf %r8, 4294967295",
    ));
    insns.push((
        Inst::AluRUImm32Shifted {
            alu_op: ALUOp::Orr64,
            rd: writable_gpr(8),
            imm: UImm32Shifted::maybe_from_u64(0x0000_0000_ffff_ffff).unwrap(),
        },
        "C08DFFFFFFFF",
        "oilf %r8, 4294967295",
    ));
    insns.push((
        Inst::AluRUImm32Shifted {
            alu_op: ALUOp::Orr64,
            rd: writable_gpr(8),
            imm: UImm32Shifted::maybe_from_u64(0xffff_ffff_0000_0000).unwrap(),
        },
        "C08CFFFFFFFF",
        "oihf %r8, 4294967295",
    ));
    insns.push((
        Inst::AluRUImm32Shifted {
            alu_op: ALUOp::Xor32,
            rd: writable_gpr(8),
            imm: UImm32Shifted::maybe_from_u64(0xffff_ffff).unwrap(),
        },
        "C087FFFFFFFF",
        "xilf %r8, 4294967295",
    ));
    insns.push((
        Inst::AluRUImm32Shifted {
            alu_op: ALUOp::Xor64,
            rd: writable_gpr(8),
            imm: UImm32Shifted::maybe_from_u64(0x0000_0000_ffff_ffff).unwrap(),
        },
        "C087FFFFFFFF",
        "xilf %r8, 4294967295",
    ));
    insns.push((
        Inst::AluRUImm32Shifted {
            alu_op: ALUOp::Xor64,
            rd: writable_gpr(8),
            imm: UImm32Shifted::maybe_from_u64(0xffff_ffff_0000_0000).unwrap(),
        },
        "C086FFFFFFFF",
        "xihf %r8, 4294967295",
    ));

    insns.push((
        Inst::UnaryRR {
            op: UnaryOp::Abs32,
            rd: writable_gpr(1),
            rn: gpr(10),
        },
        "101A",
        "lpr %r1, %r10",
    ));
    insns.push((
        Inst::UnaryRR {
            op: UnaryOp::Abs64,
            rd: writable_gpr(1),
            rn: gpr(10),
        },
        "B900001A",
        "lpgr %r1, %r10",
    ));
    insns.push((
        Inst::UnaryRR {
            op: UnaryOp::Abs64Ext32,
            rd: writable_gpr(1),
            rn: gpr(10),
        },
        "B910001A",
        "lpgfr %r1, %r10",
    ));
    insns.push((
        Inst::UnaryRR {
            op: UnaryOp::Neg32,
            rd: writable_gpr(1),
            rn: gpr(10),
        },
        "131A",
        "lcr %r1, %r10",
    ));
    insns.push((
        Inst::UnaryRR {
            op: UnaryOp::Neg64,
            rd: writable_gpr(1),
            rn: gpr(10),
        },
        "B903001A",
        "lcgr %r1, %r10",
    ));
    insns.push((
        Inst::UnaryRR {
            op: UnaryOp::Neg64Ext32,
            rd: writable_gpr(1),
            rn: gpr(10),
        },
        "B913001A",
        "lcgfr %r1, %r10",
    ));
    insns.push((
        Inst::UnaryRR {
            op: UnaryOp::PopcntByte,
            rd: writable_gpr(1),
            rn: gpr(10),
        },
        "B9E1001A",
        "popcnt %r1, %r10",
    ));
    insns.push((
        Inst::UnaryRR {
            op: UnaryOp::PopcntReg,
            rd: writable_gpr(1),
            rn: gpr(10),
        },
        "B9E1801A",
        "popcnt %r1, %r10, 8",
    ));
    insns.push((
        Inst::UnaryRR {
            op: UnaryOp::BSwap32,
            rd: writable_gpr(1),
            rn: gpr(10),
        },
        "B91F001A",
        "lrvr %r1, %r10",
    ));
    insns.push((
        Inst::UnaryRR {
            op: UnaryOp::BSwap64,
            rd: writable_gpr(1),
            rn: gpr(10),
        },
        "B90F001A",
        "lrvgr %r1, %r10",
    ));

    insns.push((
        Inst::CmpRR {
            op: CmpOp::CmpS32,
            rn: gpr(5),
            rm: gpr(6),
        },
        "1956",
        "cr %r5, %r6",
    ));
    insns.push((
        Inst::CmpRR {
            op: CmpOp::CmpS64,
            rn: gpr(5),
            rm: gpr(6),
        },
        "B9200056",
        "cgr %r5, %r6",
    ));
    insns.push((
        Inst::CmpRR {
            op: CmpOp::CmpS64Ext32,
            rn: gpr(5),
            rm: gpr(6),
        },
        "B9300056",
        "cgfr %r5, %r6",
    ));
    insns.push((
        Inst::CmpRR {
            op: CmpOp::CmpL32,
            rn: gpr(5),
            rm: gpr(6),
        },
        "1556",
        "clr %r5, %r6",
    ));
    insns.push((
        Inst::CmpRR {
            op: CmpOp::CmpL64,
            rn: gpr(5),
            rm: gpr(6),
        },
        "B9210056",
        "clgr %r5, %r6",
    ));
    insns.push((
        Inst::CmpRR {
            op: CmpOp::CmpL64Ext32,
            rn: gpr(5),
            rm: gpr(6),
        },
        "B9310056",
        "clgfr %r5, %r6",
    ));

    insns.push((
        Inst::CmpRX {
            op: CmpOp::CmpS32,
            rn: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "59102000",
        "c %r1, 0(%r2)",
    ));
    insns.push((
        Inst::CmpRX {
            op: CmpOp::CmpS32,
            rn: gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020000059",
        "cy %r1, 0(%r2)",
    ));
    insns.push((
        Inst::CmpRX {
            op: CmpOp::CmpS32,
            rn: gpr(1),
            mem: MemArg::Label {
                target: MachLabel::from_block(1),
            },
        },
        "C61D00000003",
        "crl %r1, label1",
    ));
    insns.push((
        Inst::CmpRX {
            op: CmpOp::CmpS32Ext16,
            rn: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "49102000",
        "ch %r1, 0(%r2)",
    ));
    insns.push((
        Inst::CmpRX {
            op: CmpOp::CmpS32Ext16,
            rn: gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020000079",
        "chy %r1, 0(%r2)",
    ));
    insns.push((
        Inst::CmpRX {
            op: CmpOp::CmpS32Ext16,
            rn: gpr(1),
            mem: MemArg::Label {
                target: MachLabel::from_block(1),
            },
        },
        "C61500000003",
        "chrl %r1, label1",
    ));
    insns.push((
        Inst::CmpRX {
            op: CmpOp::CmpS64,
            rn: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020000020",
        "cg %r1, 0(%r2)",
    ));
    insns.push((
        Inst::CmpRX {
            op: CmpOp::CmpS64,
            rn: gpr(1),
            mem: MemArg::Label {
                target: MachLabel::from_block(1),
            },
        },
        "C61800000003",
        "cgrl %r1, label1",
    ));
    insns.push((
        Inst::CmpRX {
            op: CmpOp::CmpS64Ext16,
            rn: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020000034",
        "cgh %r1, 0(%r2)",
    ));
    insns.push((
        Inst::CmpRX {
            op: CmpOp::CmpS64Ext16,
            rn: gpr(1),
            mem: MemArg::Label {
                target: MachLabel::from_block(1),
            },
        },
        "C61400000003",
        "cghrl %r1, label1",
    ));
    insns.push((
        Inst::CmpRX {
            op: CmpOp::CmpS64Ext32,
            rn: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020000030",
        "cgf %r1, 0(%r2)",
    ));
    insns.push((
        Inst::CmpRX {
            op: CmpOp::CmpS64Ext32,
            rn: gpr(1),
            mem: MemArg::Label {
                target: MachLabel::from_block(1),
            },
        },
        "C61C00000003",
        "cgfrl %r1, label1",
    ));
    insns.push((
        Inst::CmpRX {
            op: CmpOp::CmpL32,
            rn: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "55102000",
        "cl %r1, 0(%r2)",
    ));
    insns.push((
        Inst::CmpRX {
            op: CmpOp::CmpL32,
            rn: gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020000055",
        "cly %r1, 0(%r2)",
    ));
    insns.push((
        Inst::CmpRX {
            op: CmpOp::CmpL32,
            rn: gpr(1),
            mem: MemArg::Label {
                target: MachLabel::from_block(1),
            },
        },
        "C61F00000003",
        "clrl %r1, label1",
    ));
    insns.push((
        Inst::CmpRX {
            op: CmpOp::CmpL32Ext16,
            rn: gpr(1),
            mem: MemArg::Label {
                target: MachLabel::from_block(1),
            },
        },
        "C61700000003",
        "clhrl %r1, label1",
    ));
    insns.push((
        Inst::CmpRX {
            op: CmpOp::CmpL64,
            rn: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020000021",
        "clg %r1, 0(%r2)",
    ));
    insns.push((
        Inst::CmpRX {
            op: CmpOp::CmpL64,
            rn: gpr(1),
            mem: MemArg::Label {
                target: MachLabel::from_block(1),
            },
        },
        "C61A00000003",
        "clgrl %r1, label1",
    ));
    insns.push((
        Inst::CmpRX {
            op: CmpOp::CmpL64Ext16,
            rn: gpr(1),
            mem: MemArg::Label {
                target: MachLabel::from_block(1),
            },
        },
        "C61600000003",
        "clghrl %r1, label1",
    ));
    insns.push((
        Inst::CmpRX {
            op: CmpOp::CmpL64Ext32,
            rn: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020000031",
        "clgf %r1, 0(%r2)",
    ));
    insns.push((
        Inst::CmpRX {
            op: CmpOp::CmpL64Ext32,
            rn: gpr(1),
            mem: MemArg::Label {
                target: MachLabel::from_block(1),
            },
        },
        "C61E00000003",
        "clgfrl %r1, label1",
    ));

    insns.push((
        Inst::CmpRSImm16 {
            op: CmpOp::CmpS32,
            rn: gpr(7),
            imm: -32768,
        },
        "A77E8000",
        "chi %r7, -32768",
    ));
    insns.push((
        Inst::CmpRSImm16 {
            op: CmpOp::CmpS32,
            rn: gpr(7),
            imm: 32767,
        },
        "A77E7FFF",
        "chi %r7, 32767",
    ));
    insns.push((
        Inst::CmpRSImm16 {
            op: CmpOp::CmpS64,
            rn: gpr(7),
            imm: -32768,
        },
        "A77F8000",
        "cghi %r7, -32768",
    ));
    insns.push((
        Inst::CmpRSImm16 {
            op: CmpOp::CmpS64,
            rn: gpr(7),
            imm: 32767,
        },
        "A77F7FFF",
        "cghi %r7, 32767",
    ));
    insns.push((
        Inst::CmpRSImm32 {
            op: CmpOp::CmpS32,
            rn: gpr(7),
            imm: -2147483648,
        },
        "C27D80000000",
        "cfi %r7, -2147483648",
    ));
    insns.push((
        Inst::CmpRSImm32 {
            op: CmpOp::CmpS32,
            rn: gpr(7),
            imm: 2147483647,
        },
        "C27D7FFFFFFF",
        "cfi %r7, 2147483647",
    ));
    insns.push((
        Inst::CmpRSImm32 {
            op: CmpOp::CmpS64,
            rn: gpr(7),
            imm: -2147483648,
        },
        "C27C80000000",
        "cgfi %r7, -2147483648",
    ));
    insns.push((
        Inst::CmpRSImm32 {
            op: CmpOp::CmpS64,
            rn: gpr(7),
            imm: 2147483647,
        },
        "C27C7FFFFFFF",
        "cgfi %r7, 2147483647",
    ));
    insns.push((
        Inst::CmpRUImm32 {
            op: CmpOp::CmpL32,
            rn: gpr(7),
            imm: 0,
        },
        "C27F00000000",
        "clfi %r7, 0",
    ));
    insns.push((
        Inst::CmpRUImm32 {
            op: CmpOp::CmpL32,
            rn: gpr(7),
            imm: 4294967295,
        },
        "C27FFFFFFFFF",
        "clfi %r7, 4294967295",
    ));
    insns.push((
        Inst::CmpRUImm32 {
            op: CmpOp::CmpL64,
            rn: gpr(7),
            imm: 0,
        },
        "C27E00000000",
        "clgfi %r7, 0",
    ));
    insns.push((
        Inst::CmpRUImm32 {
            op: CmpOp::CmpL64,
            rn: gpr(7),
            imm: 4294967295,
        },
        "C27EFFFFFFFF",
        "clgfi %r7, 4294967295",
    ));

    insns.push((
        Inst::CmpTrapRR {
            op: CmpOp::CmpS32,
            rn: gpr(5),
            rm: gpr(6),
            cond: Cond::from_mask(8),
            trap_code: TrapCode::StackOverflow,
        },
        "B9728056",
        "crte %r5, %r6",
    ));
    insns.push((
        Inst::CmpTrapRR {
            op: CmpOp::CmpS64,
            rn: gpr(5),
            rm: gpr(6),
            cond: Cond::from_mask(8),
            trap_code: TrapCode::StackOverflow,
        },
        "B9608056",
        "cgrte %r5, %r6",
    ));
    insns.push((
        Inst::CmpTrapRR {
            op: CmpOp::CmpL32,
            rn: gpr(5),
            rm: gpr(6),
            cond: Cond::from_mask(8),
            trap_code: TrapCode::StackOverflow,
        },
        "B9738056",
        "clrte %r5, %r6",
    ));
    insns.push((
        Inst::CmpTrapRR {
            op: CmpOp::CmpL64,
            rn: gpr(5),
            rm: gpr(6),
            cond: Cond::from_mask(8),
            trap_code: TrapCode::StackOverflow,
        },
        "B9618056",
        "clgrte %r5, %r6",
    ));
    insns.push((
        Inst::CmpTrapRSImm16 {
            op: CmpOp::CmpS32,
            rn: gpr(7),
            imm: -32768,
            cond: Cond::from_mask(8),
            trap_code: TrapCode::StackOverflow,
        },
        "EC7080008072",
        "cite %r7, -32768",
    ));
    insns.push((
        Inst::CmpTrapRSImm16 {
            op: CmpOp::CmpS32,
            rn: gpr(7),
            imm: 32767,
            cond: Cond::from_mask(8),
            trap_code: TrapCode::StackOverflow,
        },
        "EC707FFF8072",
        "cite %r7, 32767",
    ));
    insns.push((
        Inst::CmpTrapRSImm16 {
            op: CmpOp::CmpS64,
            rn: gpr(7),
            imm: -32768,
            cond: Cond::from_mask(8),
            trap_code: TrapCode::StackOverflow,
        },
        "EC7080008070",
        "cgite %r7, -32768",
    ));
    insns.push((
        Inst::CmpTrapRSImm16 {
            op: CmpOp::CmpS64,
            rn: gpr(7),
            imm: 32767,
            cond: Cond::from_mask(8),
            trap_code: TrapCode::StackOverflow,
        },
        "EC707FFF8070",
        "cgite %r7, 32767",
    ));
    insns.push((
        Inst::CmpTrapRUImm16 {
            op: CmpOp::CmpL32,
            rn: gpr(7),
            imm: 0,
            cond: Cond::from_mask(8),
            trap_code: TrapCode::StackOverflow,
        },
        "EC7000008073",
        "clfite %r7, 0",
    ));
    insns.push((
        Inst::CmpTrapRUImm16 {
            op: CmpOp::CmpL32,
            rn: gpr(7),
            imm: 65535,
            cond: Cond::from_mask(8),
            trap_code: TrapCode::StackOverflow,
        },
        "EC70FFFF8073",
        "clfite %r7, 65535",
    ));
    insns.push((
        Inst::CmpTrapRUImm16 {
            op: CmpOp::CmpL64,
            rn: gpr(7),
            imm: 0,
            cond: Cond::from_mask(8),
            trap_code: TrapCode::StackOverflow,
        },
        "EC7000008071",
        "clgite %r7, 0",
    ));
    insns.push((
        Inst::CmpTrapRUImm16 {
            op: CmpOp::CmpL64,
            rn: gpr(7),
            imm: 65535,
            cond: Cond::from_mask(8),
            trap_code: TrapCode::StackOverflow,
        },
        "EC70FFFF8071",
        "clgite %r7, 65535",
    ));

    insns.push((
        Inst::SMulWide {
            rn: gpr(5),
            rm: gpr(6),
        },
        "B9EC6005",
        "mgrk %r0, %r5, %r6",
    ));
    insns.push((Inst::UMulWide { rn: gpr(5) }, "B9860005", "mlgr %r0, %r5"));
    insns.push((Inst::SDivMod32 { rn: gpr(5) }, "B91D0005", "dsgfr %r0, %r5"));
    insns.push((Inst::SDivMod64 { rn: gpr(5) }, "B90D0005", "dsgr %r0, %r5"));
    insns.push((Inst::UDivMod32 { rn: gpr(5) }, "B9970005", "dlr %r0, %r5"));
    insns.push((Inst::UDivMod64 { rn: gpr(5) }, "B9870005", "dlgr %r0, %r5"));

    insns.push((Inst::Flogr { rn: gpr(5) }, "B9830005", "flogr %r0, %r5"));

    insns.push((
        Inst::ShiftRR {
            shift_op: ShiftOp::RotL32,
            rd: writable_gpr(4),
            rn: gpr(5),
            shift_imm: 0,
            shift_reg: zero_reg(),
        },
        "EB450000001D",
        "rll %r4, %r5, 0",
    ));
    insns.push((
        Inst::ShiftRR {
            shift_op: ShiftOp::RotL32,
            rd: writable_gpr(4),
            rn: gpr(5),
            shift_imm: 63,
            shift_reg: zero_reg(),
        },
        "EB45003F001D",
        "rll %r4, %r5, 63",
    ));
    insns.push((
        Inst::ShiftRR {
            shift_op: ShiftOp::RotL32,
            rd: writable_gpr(4),
            rn: gpr(5),
            shift_imm: 0,
            shift_reg: gpr(6),
        },
        "EB456000001D",
        "rll %r4, %r5, 0(%r6)",
    ));
    insns.push((
        Inst::ShiftRR {
            shift_op: ShiftOp::RotL32,
            rd: writable_gpr(4),
            rn: gpr(5),
            shift_imm: 63,
            shift_reg: gpr(6),
        },
        "EB45603F001D",
        "rll %r4, %r5, 63(%r6)",
    ));
    insns.push((
        Inst::ShiftRR {
            shift_op: ShiftOp::RotL64,
            rd: writable_gpr(4),
            rn: gpr(5),
            shift_imm: 0,
            shift_reg: zero_reg(),
        },
        "EB450000001C",
        "rllg %r4, %r5, 0",
    ));
    insns.push((
        Inst::ShiftRR {
            shift_op: ShiftOp::RotL64,
            rd: writable_gpr(4),
            rn: gpr(5),
            shift_imm: 63,
            shift_reg: zero_reg(),
        },
        "EB45003F001C",
        "rllg %r4, %r5, 63",
    ));
    insns.push((
        Inst::ShiftRR {
            shift_op: ShiftOp::RotL64,
            rd: writable_gpr(4),
            rn: gpr(5),
            shift_imm: 0,
            shift_reg: gpr(6),
        },
        "EB456000001C",
        "rllg %r4, %r5, 0(%r6)",
    ));
    insns.push((
        Inst::ShiftRR {
            shift_op: ShiftOp::RotL64,
            rd: writable_gpr(4),
            rn: gpr(5),
            shift_imm: 63,
            shift_reg: gpr(6),
        },
        "EB45603F001C",
        "rllg %r4, %r5, 63(%r6)",
    ));
    insns.push((
        Inst::ShiftRR {
            shift_op: ShiftOp::LShL32,
            rd: writable_gpr(4),
            rn: gpr(5),
            shift_imm: 0,
            shift_reg: zero_reg(),
        },
        "EB45000000DF",
        "sllk %r4, %r5, 0",
    ));
    insns.push((
        Inst::ShiftRR {
            shift_op: ShiftOp::LShL32,
            rd: writable_gpr(4),
            rn: gpr(5),
            shift_imm: 63,
            shift_reg: zero_reg(),
        },
        "EB45003F00DF",
        "sllk %r4, %r5, 63",
    ));
    insns.push((
        Inst::ShiftRR {
            shift_op: ShiftOp::LShL32,
            rd: writable_gpr(4),
            rn: gpr(5),
            shift_imm: 0,
            shift_reg: gpr(6),
        },
        "EB45600000DF",
        "sllk %r4, %r5, 0(%r6)",
    ));
    insns.push((
        Inst::ShiftRR {
            shift_op: ShiftOp::LShL32,
            rd: writable_gpr(4),
            rn: gpr(5),
            shift_imm: 63,
            shift_reg: gpr(6),
        },
        "EB45603F00DF",
        "sllk %r4, %r5, 63(%r6)",
    ));
    insns.push((
        Inst::ShiftRR {
            shift_op: ShiftOp::LShL64,
            rd: writable_gpr(4),
            rn: gpr(5),
            shift_imm: 0,
            shift_reg: zero_reg(),
        },
        "EB450000000D",
        "sllg %r4, %r5, 0",
    ));
    insns.push((
        Inst::ShiftRR {
            shift_op: ShiftOp::LShL64,
            rd: writable_gpr(4),
            rn: gpr(5),
            shift_imm: 63,
            shift_reg: zero_reg(),
        },
        "EB45003F000D",
        "sllg %r4, %r5, 63",
    ));
    insns.push((
        Inst::ShiftRR {
            shift_op: ShiftOp::LShL64,
            rd: writable_gpr(4),
            rn: gpr(5),
            shift_imm: 0,
            shift_reg: gpr(6),
        },
        "EB456000000D",
        "sllg %r4, %r5, 0(%r6)",
    ));
    insns.push((
        Inst::ShiftRR {
            shift_op: ShiftOp::LShL64,
            rd: writable_gpr(4),
            rn: gpr(5),
            shift_imm: 63,
            shift_reg: gpr(6),
        },
        "EB45603F000D",
        "sllg %r4, %r5, 63(%r6)",
    ));
    insns.push((
        Inst::ShiftRR {
            shift_op: ShiftOp::LShR32,
            rd: writable_gpr(4),
            rn: gpr(5),
            shift_imm: 0,
            shift_reg: zero_reg(),
        },
        "EB45000000DE",
        "srlk %r4, %r5, 0",
    ));
    insns.push((
        Inst::ShiftRR {
            shift_op: ShiftOp::LShR32,
            rd: writable_gpr(4),
            rn: gpr(5),
            shift_imm: 63,
            shift_reg: zero_reg(),
        },
        "EB45003F00DE",
        "srlk %r4, %r5, 63",
    ));
    insns.push((
        Inst::ShiftRR {
            shift_op: ShiftOp::LShR32,
            rd: writable_gpr(4),
            rn: gpr(5),
            shift_imm: 0,
            shift_reg: gpr(6),
        },
        "EB45600000DE",
        "srlk %r4, %r5, 0(%r6)",
    ));
    insns.push((
        Inst::ShiftRR {
            shift_op: ShiftOp::LShR32,
            rd: writable_gpr(4),
            rn: gpr(5),
            shift_imm: 63,
            shift_reg: gpr(6),
        },
        "EB45603F00DE",
        "srlk %r4, %r5, 63(%r6)",
    ));
    insns.push((
        Inst::ShiftRR {
            shift_op: ShiftOp::LShR64,
            rd: writable_gpr(4),
            rn: gpr(5),
            shift_imm: 0,
            shift_reg: zero_reg(),
        },
        "EB450000000C",
        "srlg %r4, %r5, 0",
    ));
    insns.push((
        Inst::ShiftRR {
            shift_op: ShiftOp::LShR64,
            rd: writable_gpr(4),
            rn: gpr(5),
            shift_imm: 63,
            shift_reg: zero_reg(),
        },
        "EB45003F000C",
        "srlg %r4, %r5, 63",
    ));
    insns.push((
        Inst::ShiftRR {
            shift_op: ShiftOp::LShR64,
            rd: writable_gpr(4),
            rn: gpr(5),
            shift_imm: 0,
            shift_reg: gpr(6),
        },
        "EB456000000C",
        "srlg %r4, %r5, 0(%r6)",
    ));
    insns.push((
        Inst::ShiftRR {
            shift_op: ShiftOp::LShR64,
            rd: writable_gpr(4),
            rn: gpr(5),
            shift_imm: 63,
            shift_reg: gpr(6),
        },
        "EB45603F000C",
        "srlg %r4, %r5, 63(%r6)",
    ));
    insns.push((
        Inst::ShiftRR {
            shift_op: ShiftOp::AShR32,
            rd: writable_gpr(4),
            rn: gpr(5),
            shift_imm: 0,
            shift_reg: zero_reg(),
        },
        "EB45000000DC",
        "srak %r4, %r5, 0",
    ));
    insns.push((
        Inst::ShiftRR {
            shift_op: ShiftOp::AShR32,
            rd: writable_gpr(4),
            rn: gpr(5),
            shift_imm: 63,
            shift_reg: zero_reg(),
        },
        "EB45003F00DC",
        "srak %r4, %r5, 63",
    ));
    insns.push((
        Inst::ShiftRR {
            shift_op: ShiftOp::AShR32,
            rd: writable_gpr(4),
            rn: gpr(5),
            shift_imm: 0,
            shift_reg: gpr(6),
        },
        "EB45600000DC",
        "srak %r4, %r5, 0(%r6)",
    ));
    insns.push((
        Inst::ShiftRR {
            shift_op: ShiftOp::AShR32,
            rd: writable_gpr(4),
            rn: gpr(5),
            shift_imm: 63,
            shift_reg: gpr(6),
        },
        "EB45603F00DC",
        "srak %r4, %r5, 63(%r6)",
    ));
    insns.push((
        Inst::ShiftRR {
            shift_op: ShiftOp::AShR64,
            rd: writable_gpr(4),
            rn: gpr(5),
            shift_imm: 0,
            shift_reg: zero_reg(),
        },
        "EB450000000A",
        "srag %r4, %r5, 0",
    ));
    insns.push((
        Inst::ShiftRR {
            shift_op: ShiftOp::AShR64,
            rd: writable_gpr(4),
            rn: gpr(5),
            shift_imm: 63,
            shift_reg: zero_reg(),
        },
        "EB45003F000A",
        "srag %r4, %r5, 63",
    ));
    insns.push((
        Inst::ShiftRR {
            shift_op: ShiftOp::AShR64,
            rd: writable_gpr(4),
            rn: gpr(5),
            shift_imm: 0,
            shift_reg: gpr(6),
        },
        "EB456000000A",
        "srag %r4, %r5, 0(%r6)",
    ));
    insns.push((
        Inst::ShiftRR {
            shift_op: ShiftOp::AShR64,
            rd: writable_gpr(4),
            rn: gpr(5),
            shift_imm: 63,
            shift_reg: gpr(6),
        },
        "EB45603F000A",
        "srag %r4, %r5, 63(%r6)",
    ));

    insns.push((
        Inst::RxSBG {
            op: RxSBGOp::Insert,
            rd: writable_gpr(4),
            rn: gpr(5),
            start_bit: 8,
            end_bit: 32,
            rotate_amt: -16,
        },
        "EC4508203059",
        "risbgn %r4, %r5, 8, 32, 48",
    ));
    insns.push((
        Inst::RxSBG {
            op: RxSBGOp::And,
            rd: writable_gpr(4),
            rn: gpr(5),
            start_bit: 8,
            end_bit: 32,
            rotate_amt: 63,
        },
        "EC4508203F54",
        "rnsbg %r4, %r5, 8, 32, 63",
    ));
    insns.push((
        Inst::RxSBG {
            op: RxSBGOp::Or,
            rd: writable_gpr(4),
            rn: gpr(5),
            start_bit: 8,
            end_bit: 32,
            rotate_amt: 63,
        },
        "EC4508203F56",
        "rosbg %r4, %r5, 8, 32, 63",
    ));
    insns.push((
        Inst::RxSBG {
            op: RxSBGOp::Xor,
            rd: writable_gpr(4),
            rn: gpr(5),
            start_bit: 8,
            end_bit: 32,
            rotate_amt: 63,
        },
        "EC4508203F57",
        "rxsbg %r4, %r5, 8, 32, 63",
    ));
    insns.push((
        Inst::RxSBGTest {
            op: RxSBGOp::And,
            rd: gpr(4),
            rn: gpr(5),
            start_bit: 8,
            end_bit: 32,
            rotate_amt: 63,
        },
        "EC4588203F54",
        "rnsbg %r4, %r5, 136, 32, 63",
    ));
    insns.push((
        Inst::RxSBGTest {
            op: RxSBGOp::Or,
            rd: gpr(4),
            rn: gpr(5),
            start_bit: 8,
            end_bit: 32,
            rotate_amt: 63,
        },
        "EC4588203F56",
        "rosbg %r4, %r5, 136, 32, 63",
    ));
    insns.push((
        Inst::RxSBGTest {
            op: RxSBGOp::Xor,
            rd: gpr(4),
            rn: gpr(5),
            start_bit: 8,
            end_bit: 32,
            rotate_amt: 63,
        },
        "EC4588203F57",
        "rxsbg %r4, %r5, 136, 32, 63",
    ));

    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::Add32,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: zero_reg(),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB45000080F8",
        "laa %r4, %r5, -524288",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::Add32,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: zero_reg(),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB450FFF7FF8",
        "laa %r4, %r5, 524287",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::Add32,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: gpr(6),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB45600080F8",
        "laa %r4, %r5, -524288(%r6)",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::Add32,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: gpr(6),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB456FFF7FF8",
        "laa %r4, %r5, 524287(%r6)",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::Add64,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: zero_reg(),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB45000080E8",
        "laag %r4, %r5, -524288",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::Add64,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: zero_reg(),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB450FFF7FE8",
        "laag %r4, %r5, 524287",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::Add64,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: gpr(6),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB45600080E8",
        "laag %r4, %r5, -524288(%r6)",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::Add64,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: gpr(6),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB456FFF7FE8",
        "laag %r4, %r5, 524287(%r6)",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::AddLogical32,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: zero_reg(),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB45000080FA",
        "laal %r4, %r5, -524288",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::AddLogical32,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: zero_reg(),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB450FFF7FFA",
        "laal %r4, %r5, 524287",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::AddLogical32,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: gpr(6),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB45600080FA",
        "laal %r4, %r5, -524288(%r6)",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::AddLogical32,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: gpr(6),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB456FFF7FFA",
        "laal %r4, %r5, 524287(%r6)",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::AddLogical64,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: zero_reg(),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB45000080EA",
        "laalg %r4, %r5, -524288",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::AddLogical64,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: zero_reg(),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB450FFF7FEA",
        "laalg %r4, %r5, 524287",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::AddLogical64,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: gpr(6),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB45600080EA",
        "laalg %r4, %r5, -524288(%r6)",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::AddLogical64,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: gpr(6),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB456FFF7FEA",
        "laalg %r4, %r5, 524287(%r6)",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::And32,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: zero_reg(),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB45000080F4",
        "lan %r4, %r5, -524288",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::And32,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: zero_reg(),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB450FFF7FF4",
        "lan %r4, %r5, 524287",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::And32,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: gpr(6),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB45600080F4",
        "lan %r4, %r5, -524288(%r6)",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::And32,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: gpr(6),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB456FFF7FF4",
        "lan %r4, %r5, 524287(%r6)",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::And64,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: zero_reg(),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB45000080E4",
        "lang %r4, %r5, -524288",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::And64,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: zero_reg(),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB450FFF7FE4",
        "lang %r4, %r5, 524287",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::And64,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: gpr(6),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB45600080E4",
        "lang %r4, %r5, -524288(%r6)",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::And64,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: gpr(6),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB456FFF7FE4",
        "lang %r4, %r5, 524287(%r6)",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::Orr32,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: zero_reg(),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB45000080F6",
        "lao %r4, %r5, -524288",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::Orr32,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: zero_reg(),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB450FFF7FF6",
        "lao %r4, %r5, 524287",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::Orr32,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: gpr(6),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB45600080F6",
        "lao %r4, %r5, -524288(%r6)",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::Orr32,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: gpr(6),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB456FFF7FF6",
        "lao %r4, %r5, 524287(%r6)",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::Orr64,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: zero_reg(),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB45000080E6",
        "laog %r4, %r5, -524288",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::Orr64,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: zero_reg(),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB450FFF7FE6",
        "laog %r4, %r5, 524287",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::Orr64,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: gpr(6),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB45600080E6",
        "laog %r4, %r5, -524288(%r6)",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::Orr64,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: gpr(6),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB456FFF7FE6",
        "laog %r4, %r5, 524287(%r6)",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::Xor32,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: zero_reg(),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB45000080F7",
        "lax %r4, %r5, -524288",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::Xor32,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: zero_reg(),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB450FFF7FF7",
        "lax %r4, %r5, 524287",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::Xor32,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: gpr(6),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB45600080F7",
        "lax %r4, %r5, -524288(%r6)",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::Xor32,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: gpr(6),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB456FFF7FF7",
        "lax %r4, %r5, 524287(%r6)",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::Xor64,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: zero_reg(),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB45000080E7",
        "laxg %r4, %r5, -524288",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::Xor64,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: zero_reg(),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB450FFF7FE7",
        "laxg %r4, %r5, 524287",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::Xor64,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: gpr(6),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB45600080E7",
        "laxg %r4, %r5, -524288(%r6)",
    ));
    insns.push((
        Inst::AtomicRmw {
            alu_op: ALUOp::Xor64,
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: gpr(6),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB456FFF7FE7",
        "laxg %r4, %r5, 524287(%r6)",
    ));
    insns.push((
        Inst::AtomicCas32 {
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD12 {
                base: zero_reg(),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(0).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "BA450000",
        "cs %r4, %r5, 0",
    ));
    insns.push((
        Inst::AtomicCas32 {
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD12 {
                base: zero_reg(),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "BA450FFF",
        "cs %r4, %r5, 4095",
    ));
    insns.push((
        Inst::AtomicCas32 {
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: zero_reg(),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB4500008014",
        "csy %r4, %r5, -524288",
    ));
    insns.push((
        Inst::AtomicCas32 {
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: zero_reg(),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB450FFF7F14",
        "csy %r4, %r5, 524287",
    ));
    insns.push((
        Inst::AtomicCas32 {
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD12 {
                base: gpr(6),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(0).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "BA456000",
        "cs %r4, %r5, 0(%r6)",
    ));
    insns.push((
        Inst::AtomicCas32 {
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD12 {
                base: gpr(6),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "BA456FFF",
        "cs %r4, %r5, 4095(%r6)",
    ));
    insns.push((
        Inst::AtomicCas32 {
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: gpr(6),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB4560008014",
        "csy %r4, %r5, -524288(%r6)",
    ));
    insns.push((
        Inst::AtomicCas32 {
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: gpr(6),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB456FFF7F14",
        "csy %r4, %r5, 524287(%r6)",
    ));
    insns.push((
        Inst::AtomicCas64 {
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: zero_reg(),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB4500008030",
        "csg %r4, %r5, -524288",
    ));
    insns.push((
        Inst::AtomicCas64 {
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: zero_reg(),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB450FFF7F30",
        "csg %r4, %r5, 524287",
    ));
    insns.push((
        Inst::AtomicCas64 {
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: gpr(6),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB4560008030",
        "csg %r4, %r5, -524288(%r6)",
    ));
    insns.push((
        Inst::AtomicCas64 {
            rd: writable_gpr(4),
            rn: gpr(5),
            mem: MemArg::BXD20 {
                base: gpr(6),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB456FFF7F30",
        "csg %r4, %r5, 524287(%r6)",
    ));
    insns.push((Inst::Fence, "07E0", "bcr 14, 0"));

    insns.push((
        Inst::Load32 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "58102000",
        "l %r1, 0(%r2)",
    ));
    insns.push((
        Inst::Load32 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "58102FFF",
        "l %r1, 4095(%r2)",
    ));
    insns.push((
        Inst::Load32 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020008058",
        "ly %r1, -524288(%r2)",
    ));
    insns.push((
        Inst::Load32 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF7F58",
        "ly %r1, 524287(%r2)",
    ));
    insns.push((
        Inst::Load32 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "58123000",
        "l %r1, 0(%r2,%r3)",
    ));
    insns.push((
        Inst::Load32 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "58123FFF",
        "l %r1, 4095(%r2,%r3)",
    ));
    insns.push((
        Inst::Load32 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31230008058",
        "ly %r1, -524288(%r2,%r3)",
    ));
    insns.push((
        Inst::Load32 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF7F58",
        "ly %r1, 524287(%r2,%r3)",
    ));
    insns.push((
        Inst::Load32ZExt8 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020000094",
        "llc %r1, 0(%r2)",
    ));
    insns.push((
        Inst::Load32ZExt8 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF0094",
        "llc %r1, 4095(%r2)",
    ));
    insns.push((
        Inst::Load32ZExt8 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020008094",
        "llc %r1, -524288(%r2)",
    ));
    insns.push((
        Inst::Load32ZExt8 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF7F94",
        "llc %r1, 524287(%r2)",
    ));
    insns.push((
        Inst::Load32ZExt8 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31230000094",
        "llc %r1, 0(%r2,%r3)",
    ));
    insns.push((
        Inst::Load32ZExt8 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF0094",
        "llc %r1, 4095(%r2,%r3)",
    ));
    insns.push((
        Inst::Load32ZExt8 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31230008094",
        "llc %r1, -524288(%r2,%r3)",
    ));
    insns.push((
        Inst::Load32ZExt8 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF7F94",
        "llc %r1, 524287(%r2,%r3)",
    ));
    insns.push((
        Inst::Load32SExt8 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020000076",
        "lb %r1, 0(%r2)",
    ));
    insns.push((
        Inst::Load32SExt8 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF0076",
        "lb %r1, 4095(%r2)",
    ));
    insns.push((
        Inst::Load32SExt8 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020008076",
        "lb %r1, -524288(%r2)",
    ));
    insns.push((
        Inst::Load32SExt8 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF7F76",
        "lb %r1, 524287(%r2)",
    ));
    insns.push((
        Inst::Load32SExt8 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31230000076",
        "lb %r1, 0(%r2,%r3)",
    ));
    insns.push((
        Inst::Load32SExt8 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF0076",
        "lb %r1, 4095(%r2,%r3)",
    ));
    insns.push((
        Inst::Load32SExt8 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31230008076",
        "lb %r1, -524288(%r2,%r3)",
    ));
    insns.push((
        Inst::Load32SExt8 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF7F76",
        "lb %r1, 524287(%r2,%r3)",
    ));
    insns.push((
        Inst::Load32ZExt16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020000095",
        "llh %r1, 0(%r2)",
    ));
    insns.push((
        Inst::Load32ZExt16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF0095",
        "llh %r1, 4095(%r2)",
    ));
    insns.push((
        Inst::Load32ZExt16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020008095",
        "llh %r1, -524288(%r2)",
    ));
    insns.push((
        Inst::Load32ZExt16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF7F95",
        "llh %r1, 524287(%r2)",
    ));
    insns.push((
        Inst::Load32ZExt16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31230000095",
        "llh %r1, 0(%r2,%r3)",
    ));
    insns.push((
        Inst::Load32ZExt16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF0095",
        "llh %r1, 4095(%r2,%r3)",
    ));
    insns.push((
        Inst::Load32ZExt16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31230008095",
        "llh %r1, -524288(%r2,%r3)",
    ));
    insns.push((
        Inst::Load32ZExt16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF7F95",
        "llh %r1, 524287(%r2,%r3)",
    ));
    insns.push((
        Inst::Load32SExt16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "48102000",
        "lh %r1, 0(%r2)",
    ));
    insns.push((
        Inst::Load32SExt16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "48102FFF",
        "lh %r1, 4095(%r2)",
    ));
    insns.push((
        Inst::Load32SExt16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020008078",
        "lhy %r1, -524288(%r2)",
    ));
    insns.push((
        Inst::Load32SExt16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF7F78",
        "lhy %r1, 524287(%r2)",
    ));
    insns.push((
        Inst::Load32SExt16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "48123000",
        "lh %r1, 0(%r2,%r3)",
    ));
    insns.push((
        Inst::Load32SExt16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "48123FFF",
        "lh %r1, 4095(%r2,%r3)",
    ));
    insns.push((
        Inst::Load32SExt16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31230008078",
        "lhy %r1, -524288(%r2,%r3)",
    ));
    insns.push((
        Inst::Load32SExt16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF7F78",
        "lhy %r1, 524287(%r2,%r3)",
    ));
    insns.push((
        Inst::Load64 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020000004",
        "lg %r1, 0(%r2)",
    ));
    insns.push((
        Inst::Load64 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF0004",
        "lg %r1, 4095(%r2)",
    ));
    insns.push((
        Inst::Load64 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020008004",
        "lg %r1, -524288(%r2)",
    ));
    insns.push((
        Inst::Load64 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF7F04",
        "lg %r1, 524287(%r2)",
    ));
    insns.push((
        Inst::Load64 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31230000004",
        "lg %r1, 0(%r2,%r3)",
    ));
    insns.push((
        Inst::Load64 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF0004",
        "lg %r1, 4095(%r2,%r3)",
    ));
    insns.push((
        Inst::Load64 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31230008004",
        "lg %r1, -524288(%r2,%r3)",
    ));
    insns.push((
        Inst::Load64 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF7F04",
        "lg %r1, 524287(%r2,%r3)",
    ));
    insns.push((
        Inst::Load64ZExt8 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020000090",
        "llgc %r1, 0(%r2)",
    ));
    insns.push((
        Inst::Load64ZExt8 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF0090",
        "llgc %r1, 4095(%r2)",
    ));
    insns.push((
        Inst::Load64ZExt8 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020008090",
        "llgc %r1, -524288(%r2)",
    ));
    insns.push((
        Inst::Load64ZExt8 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF7F90",
        "llgc %r1, 524287(%r2)",
    ));
    insns.push((
        Inst::Load64ZExt8 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31230000090",
        "llgc %r1, 0(%r2,%r3)",
    ));
    insns.push((
        Inst::Load64ZExt8 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF0090",
        "llgc %r1, 4095(%r2,%r3)",
    ));
    insns.push((
        Inst::Load64ZExt8 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31230008090",
        "llgc %r1, -524288(%r2,%r3)",
    ));
    insns.push((
        Inst::Load64ZExt8 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF7F90",
        "llgc %r1, 524287(%r2,%r3)",
    ));
    insns.push((
        Inst::Load64SExt8 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020000077",
        "lgb %r1, 0(%r2)",
    ));
    insns.push((
        Inst::Load64SExt8 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF0077",
        "lgb %r1, 4095(%r2)",
    ));
    insns.push((
        Inst::Load64SExt8 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020008077",
        "lgb %r1, -524288(%r2)",
    ));
    insns.push((
        Inst::Load64SExt8 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF7F77",
        "lgb %r1, 524287(%r2)",
    ));
    insns.push((
        Inst::Load64SExt8 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31230000077",
        "lgb %r1, 0(%r2,%r3)",
    ));
    insns.push((
        Inst::Load64SExt8 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF0077",
        "lgb %r1, 4095(%r2,%r3)",
    ));
    insns.push((
        Inst::Load64SExt8 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31230008077",
        "lgb %r1, -524288(%r2,%r3)",
    ));
    insns.push((
        Inst::Load64SExt8 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF7F77",
        "lgb %r1, 524287(%r2,%r3)",
    ));
    insns.push((
        Inst::Load64ZExt16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020000091",
        "llgh %r1, 0(%r2)",
    ));
    insns.push((
        Inst::Load64ZExt16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF0091",
        "llgh %r1, 4095(%r2)",
    ));
    insns.push((
        Inst::Load64ZExt16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020008091",
        "llgh %r1, -524288(%r2)",
    ));
    insns.push((
        Inst::Load64ZExt16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF7F91",
        "llgh %r1, 524287(%r2)",
    ));
    insns.push((
        Inst::Load64ZExt16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31230000091",
        "llgh %r1, 0(%r2,%r3)",
    ));
    insns.push((
        Inst::Load64ZExt16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF0091",
        "llgh %r1, 4095(%r2,%r3)",
    ));
    insns.push((
        Inst::Load64ZExt16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31230008091",
        "llgh %r1, -524288(%r2,%r3)",
    ));
    insns.push((
        Inst::Load64ZExt16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF7F91",
        "llgh %r1, 524287(%r2,%r3)",
    ));
    insns.push((
        Inst::Load64SExt16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020000015",
        "lgh %r1, 0(%r2)",
    ));
    insns.push((
        Inst::Load64SExt16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF0015",
        "lgh %r1, 4095(%r2)",
    ));
    insns.push((
        Inst::Load64SExt16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020008015",
        "lgh %r1, -524288(%r2)",
    ));
    insns.push((
        Inst::Load64SExt16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF7F15",
        "lgh %r1, 524287(%r2)",
    ));
    insns.push((
        Inst::Load64SExt16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31230000015",
        "lgh %r1, 0(%r2,%r3)",
    ));
    insns.push((
        Inst::Load64SExt16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF0015",
        "lgh %r1, 4095(%r2,%r3)",
    ));
    insns.push((
        Inst::Load64SExt16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31230008015",
        "lgh %r1, -524288(%r2,%r3)",
    ));
    insns.push((
        Inst::Load64SExt16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF7F15",
        "lgh %r1, 524287(%r2,%r3)",
    ));
    insns.push((
        Inst::Load64ZExt32 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020000016",
        "llgf %r1, 0(%r2)",
    ));
    insns.push((
        Inst::Load64ZExt32 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF0016",
        "llgf %r1, 4095(%r2)",
    ));
    insns.push((
        Inst::Load64ZExt32 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020008016",
        "llgf %r1, -524288(%r2)",
    ));
    insns.push((
        Inst::Load64ZExt32 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF7F16",
        "llgf %r1, 524287(%r2)",
    ));
    insns.push((
        Inst::Load64ZExt32 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31230000016",
        "llgf %r1, 0(%r2,%r3)",
    ));
    insns.push((
        Inst::Load64ZExt32 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF0016",
        "llgf %r1, 4095(%r2,%r3)",
    ));
    insns.push((
        Inst::Load64ZExt32 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31230008016",
        "llgf %r1, -524288(%r2,%r3)",
    ));
    insns.push((
        Inst::Load64ZExt32 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF7F16",
        "llgf %r1, 524287(%r2,%r3)",
    ));
    insns.push((
        Inst::Load64SExt32 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020000014",
        "lgf %r1, 0(%r2)",
    ));
    insns.push((
        Inst::Load64SExt32 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF0014",
        "lgf %r1, 4095(%r2)",
    ));
    insns.push((
        Inst::Load64SExt32 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020008014",
        "lgf %r1, -524288(%r2)",
    ));
    insns.push((
        Inst::Load64SExt32 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF7F14",
        "lgf %r1, 524287(%r2)",
    ));
    insns.push((
        Inst::Load64SExt32 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31230000014",
        "lgf %r1, 0(%r2,%r3)",
    ));
    insns.push((
        Inst::Load64SExt32 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF0014",
        "lgf %r1, 4095(%r2,%r3)",
    ));
    insns.push((
        Inst::Load64SExt32 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31230008014",
        "lgf %r1, -524288(%r2,%r3)",
    ));
    insns.push((
        Inst::Load64SExt32 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF7F14",
        "lgf %r1, 524287(%r2,%r3)",
    ));

    insns.push((
        Inst::Load32 {
            rd: writable_gpr(1),
            mem: MemArg::Label {
                target: MachLabel::from_block(1),
            },
        },
        "C41D00000003",
        "lrl %r1, label1",
    ));
    insns.push((
        Inst::Load32SExt16 {
            rd: writable_gpr(1),
            mem: MemArg::Label {
                target: MachLabel::from_block(1),
            },
        },
        "C41500000003",
        "lhrl %r1, label1",
    ));
    insns.push((
        Inst::Load32ZExt16 {
            rd: writable_gpr(1),
            mem: MemArg::Label {
                target: MachLabel::from_block(1),
            },
        },
        "C41200000003",
        "llhrl %r1, label1",
    ));
    insns.push((
        Inst::Load64 {
            rd: writable_gpr(1),
            mem: MemArg::Label {
                target: MachLabel::from_block(1),
            },
        },
        "C41800000003",
        "lgrl %r1, label1",
    ));
    insns.push((
        Inst::Load64SExt16 {
            rd: writable_gpr(1),
            mem: MemArg::Label {
                target: MachLabel::from_block(1),
            },
        },
        "C41400000003",
        "lghrl %r1, label1",
    ));
    insns.push((
        Inst::Load64ZExt16 {
            rd: writable_gpr(1),
            mem: MemArg::Label {
                target: MachLabel::from_block(1),
            },
        },
        "C41600000003",
        "llghrl %r1, label1",
    ));
    insns.push((
        Inst::Load64SExt32 {
            rd: writable_gpr(1),
            mem: MemArg::Label {
                target: MachLabel::from_block(1),
            },
        },
        "C41C00000003",
        "lgfrl %r1, label1",
    ));
    insns.push((
        Inst::Load64ZExt32 {
            rd: writable_gpr(1),
            mem: MemArg::Label {
                target: MachLabel::from_block(1),
            },
        },
        "C41E00000003",
        "llgfrl %r1, label1",
    ));
    insns.push((
        Inst::LoadRev16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102000001F",
        "lrvh %r1, 0(%r2)",
    ));
    insns.push((
        Inst::LoadRev16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF001F",
        "lrvh %r1, 4095(%r2)",
    ));
    insns.push((
        Inst::LoadRev16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102000801F",
        "lrvh %r1, -524288(%r2)",
    ));
    insns.push((
        Inst::LoadRev16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF7F1F",
        "lrvh %r1, 524287(%r2)",
    ));
    insns.push((
        Inst::LoadRev16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123000001F",
        "lrvh %r1, 0(%r2,%r3)",
    ));
    insns.push((
        Inst::LoadRev16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF001F",
        "lrvh %r1, 4095(%r2,%r3)",
    ));
    insns.push((
        Inst::LoadRev16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123000801F",
        "lrvh %r1, -524288(%r2,%r3)",
    ));
    insns.push((
        Inst::LoadRev16 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF7F1F",
        "lrvh %r1, 524287(%r2,%r3)",
    ));
    insns.push((
        Inst::LoadRev32 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102000001E",
        "lrv %r1, 0(%r2)",
    ));
    insns.push((
        Inst::LoadRev32 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF001E",
        "lrv %r1, 4095(%r2)",
    ));
    insns.push((
        Inst::LoadRev32 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102000801E",
        "lrv %r1, -524288(%r2)",
    ));
    insns.push((
        Inst::LoadRev32 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF7F1E",
        "lrv %r1, 524287(%r2)",
    ));
    insns.push((
        Inst::LoadRev32 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123000001E",
        "lrv %r1, 0(%r2,%r3)",
    ));
    insns.push((
        Inst::LoadRev32 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF001E",
        "lrv %r1, 4095(%r2,%r3)",
    ));
    insns.push((
        Inst::LoadRev32 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123000801E",
        "lrv %r1, -524288(%r2,%r3)",
    ));
    insns.push((
        Inst::LoadRev32 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF7F1E",
        "lrv %r1, 524287(%r2,%r3)",
    ));
    insns.push((
        Inst::LoadRev64 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102000000F",
        "lrvg %r1, 0(%r2)",
    ));
    insns.push((
        Inst::LoadRev64 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF000F",
        "lrvg %r1, 4095(%r2)",
    ));
    insns.push((
        Inst::LoadRev64 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102000800F",
        "lrvg %r1, -524288(%r2)",
    ));
    insns.push((
        Inst::LoadRev64 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF7F0F",
        "lrvg %r1, 524287(%r2)",
    ));
    insns.push((
        Inst::LoadRev64 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123000000F",
        "lrvg %r1, 0(%r2,%r3)",
    ));
    insns.push((
        Inst::LoadRev64 {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF000F",
        "lrvg %r1, 4095(%r2,%r3)",
    ));
    insns.push((
        Inst::LoadRev64 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123000800F",
        "lrvg %r1, -524288(%r2,%r3)",
    ));
    insns.push((
        Inst::LoadRev64 {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF7F0F",
        "lrvg %r1, 524287(%r2,%r3)",
    ));

    insns.push((
        Inst::Store8 {
            rd: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "42102000",
        "stc %r1, 0(%r2)",
    ));
    insns.push((
        Inst::Store8 {
            rd: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "42102FFF",
        "stc %r1, 4095(%r2)",
    ));
    insns.push((
        Inst::Store8 {
            rd: gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020008072",
        "stcy %r1, -524288(%r2)",
    ));
    insns.push((
        Inst::Store8 {
            rd: gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF7F72",
        "stcy %r1, 524287(%r2)",
    ));
    insns.push((
        Inst::Store8 {
            rd: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "42123000",
        "stc %r1, 0(%r2,%r3)",
    ));
    insns.push((
        Inst::Store8 {
            rd: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "42123FFF",
        "stc %r1, 4095(%r2,%r3)",
    ));
    insns.push((
        Inst::Store8 {
            rd: gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31230008072",
        "stcy %r1, -524288(%r2,%r3)",
    ));
    insns.push((
        Inst::Store8 {
            rd: gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF7F72",
        "stcy %r1, 524287(%r2,%r3)",
    ));
    insns.push((
        Inst::Store16 {
            rd: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "40102000",
        "sth %r1, 0(%r2)",
    ));
    insns.push((
        Inst::Store16 {
            rd: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "40102FFF",
        "sth %r1, 4095(%r2)",
    ));
    insns.push((
        Inst::Store16 {
            rd: gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020008070",
        "sthy %r1, -524288(%r2)",
    ));
    insns.push((
        Inst::Store16 {
            rd: gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF7F70",
        "sthy %r1, 524287(%r2)",
    ));
    insns.push((
        Inst::Store16 {
            rd: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "40123000",
        "sth %r1, 0(%r2,%r3)",
    ));
    insns.push((
        Inst::Store16 {
            rd: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "40123FFF",
        "sth %r1, 4095(%r2,%r3)",
    ));
    insns.push((
        Inst::Store16 {
            rd: gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31230008070",
        "sthy %r1, -524288(%r2,%r3)",
    ));
    insns.push((
        Inst::Store16 {
            rd: gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF7F70",
        "sthy %r1, 524287(%r2,%r3)",
    ));
    insns.push((
        Inst::Store32 {
            rd: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "50102000",
        "st %r1, 0(%r2)",
    ));
    insns.push((
        Inst::Store32 {
            rd: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "50102FFF",
        "st %r1, 4095(%r2)",
    ));
    insns.push((
        Inst::Store32 {
            rd: gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020008050",
        "sty %r1, -524288(%r2)",
    ));
    insns.push((
        Inst::Store32 {
            rd: gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF7F50",
        "sty %r1, 524287(%r2)",
    ));
    insns.push((
        Inst::Store32 {
            rd: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "50123000",
        "st %r1, 0(%r2,%r3)",
    ));
    insns.push((
        Inst::Store32 {
            rd: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "50123FFF",
        "st %r1, 4095(%r2,%r3)",
    ));
    insns.push((
        Inst::Store32 {
            rd: gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31230008050",
        "sty %r1, -524288(%r2,%r3)",
    ));
    insns.push((
        Inst::Store32 {
            rd: gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF7F50",
        "sty %r1, 524287(%r2,%r3)",
    ));
    insns.push((
        Inst::Store64 {
            rd: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020000024",
        "stg %r1, 0(%r2)",
    ));
    insns.push((
        Inst::Store64 {
            rd: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF0024",
        "stg %r1, 4095(%r2)",
    ));
    insns.push((
        Inst::Store64 {
            rd: gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020008024",
        "stg %r1, -524288(%r2)",
    ));
    insns.push((
        Inst::Store64 {
            rd: gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF7F24",
        "stg %r1, 524287(%r2)",
    ));
    insns.push((
        Inst::Store64 {
            rd: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E31230000024",
        "stg %r1, 0(%r2,%r3)",
    ));
    insns.push((
        Inst::Store64 {
            rd: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF0024",
        "stg %r1, 4095(%r2,%r3)",
    ));
    insns.push((
        Inst::Store64 {
            rd: gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31230008024",
        "stg %r1, -524288(%r2,%r3)",
    ));
    insns.push((
        Inst::Store64 {
            rd: gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF7F24",
        "stg %r1, 524287(%r2,%r3)",
    ));

    insns.push((
        Inst::StoreImm8 {
            imm: 255,
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "92FF2000",
        "mvi 0(%r2), 255",
    ));
    insns.push((
        Inst::StoreImm8 {
            imm: 0,
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "92002FFF",
        "mvi 4095(%r2), 0",
    ));
    insns.push((
        Inst::StoreImm8 {
            imm: 255,
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EBFF20008052",
        "mviy -524288(%r2), 255",
    ));
    insns.push((
        Inst::StoreImm8 {
            imm: 0,
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB002FFF7F52",
        "mviy 524287(%r2), 0",
    ));
    insns.push((
        Inst::StoreImm16 {
            imm: -32768,
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E54420008000",
        "mvhhi 0(%r2), -32768",
    ));
    insns.push((
        Inst::StoreImm16 {
            imm: 32767,
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E5442FFF7FFF",
        "mvhhi 4095(%r2), 32767",
    ));
    insns.push((
        Inst::StoreImm32SExt16 {
            imm: -32768,
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E54C20008000",
        "mvhi 0(%r2), -32768",
    ));
    insns.push((
        Inst::StoreImm32SExt16 {
            imm: 32767,
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E54C2FFF7FFF",
        "mvhi 4095(%r2), 32767",
    ));
    insns.push((
        Inst::StoreImm64SExt16 {
            imm: -32768,
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E54820008000",
        "mvghi 0(%r2), -32768",
    ));
    insns.push((
        Inst::StoreImm64SExt16 {
            imm: 32767,
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E5482FFF7FFF",
        "mvghi 4095(%r2), 32767",
    ));

    insns.push((
        Inst::StoreRev16 {
            rd: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102000003F",
        "strvh %r1, 0(%r2)",
    ));
    insns.push((
        Inst::StoreRev16 {
            rd: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF003F",
        "strvh %r1, 4095(%r2)",
    ));
    insns.push((
        Inst::StoreRev16 {
            rd: gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102000803F",
        "strvh %r1, -524288(%r2)",
    ));
    insns.push((
        Inst::StoreRev16 {
            rd: gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF7F3F",
        "strvh %r1, 524287(%r2)",
    ));
    insns.push((
        Inst::StoreRev16 {
            rd: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123000003F",
        "strvh %r1, 0(%r2,%r3)",
    ));
    insns.push((
        Inst::StoreRev16 {
            rd: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF003F",
        "strvh %r1, 4095(%r2,%r3)",
    ));
    insns.push((
        Inst::StoreRev16 {
            rd: gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123000803F",
        "strvh %r1, -524288(%r2,%r3)",
    ));
    insns.push((
        Inst::StoreRev16 {
            rd: gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF7F3F",
        "strvh %r1, 524287(%r2,%r3)",
    ));
    insns.push((
        Inst::StoreRev32 {
            rd: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102000003E",
        "strv %r1, 0(%r2)",
    ));
    insns.push((
        Inst::StoreRev32 {
            rd: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF003E",
        "strv %r1, 4095(%r2)",
    ));
    insns.push((
        Inst::StoreRev32 {
            rd: gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102000803E",
        "strv %r1, -524288(%r2)",
    ));
    insns.push((
        Inst::StoreRev32 {
            rd: gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF7F3E",
        "strv %r1, 524287(%r2)",
    ));
    insns.push((
        Inst::StoreRev32 {
            rd: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123000003E",
        "strv %r1, 0(%r2,%r3)",
    ));
    insns.push((
        Inst::StoreRev32 {
            rd: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF003E",
        "strv %r1, 4095(%r2,%r3)",
    ));
    insns.push((
        Inst::StoreRev32 {
            rd: gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123000803E",
        "strv %r1, -524288(%r2,%r3)",
    ));
    insns.push((
        Inst::StoreRev32 {
            rd: gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF7F3E",
        "strv %r1, 524287(%r2,%r3)",
    ));
    insns.push((
        Inst::StoreRev64 {
            rd: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102000002F",
        "strvg %r1, 0(%r2)",
    ));
    insns.push((
        Inst::StoreRev64 {
            rd: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF002F",
        "strvg %r1, 4095(%r2)",
    ));
    insns.push((
        Inst::StoreRev64 {
            rd: gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102000802F",
        "strvg %r1, -524288(%r2)",
    ));
    insns.push((
        Inst::StoreRev64 {
            rd: gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF7F2F",
        "strvg %r1, 524287(%r2)",
    ));
    insns.push((
        Inst::StoreRev64 {
            rd: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123000002F",
        "strvg %r1, 0(%r2,%r3)",
    ));
    insns.push((
        Inst::StoreRev64 {
            rd: gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF002F",
        "strvg %r1, 4095(%r2,%r3)",
    ));
    insns.push((
        Inst::StoreRev64 {
            rd: gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123000802F",
        "strvg %r1, -524288(%r2,%r3)",
    ));
    insns.push((
        Inst::StoreRev64 {
            rd: gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF7F2F",
        "strvg %r1, 524287(%r2,%r3)",
    ));

    insns.push((
        Inst::Store16 {
            rd: gpr(1),
            mem: MemArg::Label {
                target: MachLabel::from_block(1),
            },
        },
        "C41700000003",
        "sthrl %r1, label1",
    ));
    insns.push((
        Inst::Store32 {
            rd: gpr(1),
            mem: MemArg::Label {
                target: MachLabel::from_block(1),
            },
        },
        "C41F00000003",
        "strl %r1, label1",
    ));
    insns.push((
        Inst::Store64 {
            rd: gpr(1),
            mem: MemArg::Label {
                target: MachLabel::from_block(1),
            },
        },
        "C41B00000003",
        "stgrl %r1, label1",
    ));

    insns.push((
        Inst::LoadMultiple64 {
            rt: writable_gpr(8),
            rt2: writable_gpr(12),
            mem: MemArg::BXD20 {
                base: gpr(15),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB8CF0008004",
        "lmg %r8, %r12, -524288(%r15)",
    ));
    insns.push((
        Inst::LoadMultiple64 {
            rt: writable_gpr(8),
            rt2: writable_gpr(12),
            mem: MemArg::BXD20 {
                base: gpr(15),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB8CFFFF7F04",
        "lmg %r8, %r12, 524287(%r15)",
    ));

    insns.push((
        Inst::StoreMultiple64 {
            rt: gpr(8),
            rt2: gpr(12),
            mem: MemArg::BXD20 {
                base: gpr(15),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB8CF0008024",
        "stmg %r8, %r12, -524288(%r15)",
    ));
    insns.push((
        Inst::StoreMultiple64 {
            rt: gpr(8),
            rt2: gpr(12),
            mem: MemArg::BXD20 {
                base: gpr(15),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "EB8CFFFF7F24",
        "stmg %r8, %r12, 524287(%r15)",
    ));

    insns.push((
        Inst::LoadAddr {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: zero_reg(),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "41100000",
        "la %r1, 0",
    ));
    insns.push((
        Inst::LoadAddr {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: zero_reg(),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "41100FFF",
        "la %r1, 4095",
    ));
    insns.push((
        Inst::LoadAddr {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: zero_reg(),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31000008071",
        "lay %r1, -524288",
    ));
    insns.push((
        Inst::LoadAddr {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: zero_reg(),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3100FFF7F71",
        "lay %r1, 524287",
    ));
    insns.push((
        Inst::LoadAddr {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "41102000",
        "la %r1, 0(%r2)",
    ));
    insns.push((
        Inst::LoadAddr {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "41102FFF",
        "la %r1, 4095(%r2)",
    ));
    insns.push((
        Inst::LoadAddr {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020008071",
        "lay %r1, -524288(%r2)",
    ));
    insns.push((
        Inst::LoadAddr {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF7F71",
        "lay %r1, 524287(%r2)",
    ));
    insns.push((
        Inst::LoadAddr {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "41123000",
        "la %r1, 0(%r2,%r3)",
    ));
    insns.push((
        Inst::LoadAddr {
            rd: writable_gpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "41123FFF",
        "la %r1, 4095(%r2,%r3)",
    ));
    insns.push((
        Inst::LoadAddr {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31230008071",
        "lay %r1, -524288(%r2,%r3)",
    ));
    insns.push((
        Inst::LoadAddr {
            rd: writable_gpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF7F71",
        "lay %r1, 524287(%r2,%r3)",
    ));
    insns.push((
        Inst::LoadAddr {
            rd: writable_gpr(1),
            mem: MemArg::Label {
                target: MachLabel::from_block(1),
            },
        },
        "C01000000003",
        "larl %r1, label1",
    ));
    insns.push((
        Inst::LoadAddr {
            rd: writable_gpr(1),
            mem: MemArg::Symbol {
                name: Box::new(ExternalName::testcase("test0")),
                offset: 64,
                flags: MemFlags::trusted(),
            },
        },
        "C01000000000",
        "larl %r1, %test0 + 64",
    ));

    insns.push((
        Inst::LoadAddr {
            rd: writable_gpr(1),
            mem: MemArg::RegOffset {
                reg: gpr(2),
                off: 0,
                flags: MemFlags::trusted(),
            },
        },
        "41102000",
        "la %r1, 0(%r2)",
    ));
    insns.push((
        Inst::LoadAddr {
            rd: writable_gpr(1),
            mem: MemArg::RegOffset {
                reg: gpr(2),
                off: 4095,
                flags: MemFlags::trusted(),
            },
        },
        "41102FFF",
        "la %r1, 4095(%r2)",
    ));
    insns.push((
        Inst::LoadAddr {
            rd: writable_gpr(1),
            mem: MemArg::RegOffset {
                reg: gpr(2),
                off: -524288,
                flags: MemFlags::trusted(),
            },
        },
        "E31020008071",
        "lay %r1, -524288(%r2)",
    ));
    insns.push((
        Inst::LoadAddr {
            rd: writable_gpr(1),
            mem: MemArg::RegOffset {
                reg: gpr(2),
                off: 524287,
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF7F71",
        "lay %r1, 524287(%r2)",
    ));
    insns.push((
        Inst::LoadAddr {
            rd: writable_gpr(1),
            mem: MemArg::RegOffset {
                reg: gpr(2),
                off: -2147483648,
                flags: MemFlags::trusted(),
            },
        },
        "C0118000000041112000",
        "lgfi %r1, -2147483648 ; la %r1, 0(%r1,%r2)",
    ));
    insns.push((
        Inst::LoadAddr {
            rd: writable_gpr(1),
            mem: MemArg::RegOffset {
                reg: gpr(2),
                off: 2147483647,
                flags: MemFlags::trusted(),
            },
        },
        "C0117FFFFFFF41112000",
        "lgfi %r1, 2147483647 ; la %r1, 0(%r1,%r2)",
    ));
    insns.push((
        Inst::LoadAddr {
            rd: writable_gpr(1),
            mem: MemArg::RegOffset {
                reg: gpr(2),
                off: -9223372036854775808,
                flags: MemFlags::trusted(),
            },
        },
        "A51C800041112000",
        "llihh %r1, 32768 ; la %r1, 0(%r1,%r2)",
    ));
    insns.push((
        Inst::LoadAddr {
            rd: writable_gpr(1),
            mem: MemArg::RegOffset {
                reg: gpr(2),
                off: 9223372036854775807,
                flags: MemFlags::trusted(),
            },
        },
        "C01E7FFFFFFFC019FFFFFFFF41112000",
        "llihf %r1, 2147483647 ; iilf %r1, 4294967295 ; la %r1, 0(%r1,%r2)",
    ));

    insns.push((
        Inst::Mov64 {
            rd: writable_gpr(8),
            rm: gpr(9),
        },
        "B9040089",
        "lgr %r8, %r9",
    ));
    insns.push((
        Inst::Mov32 {
            rd: writable_gpr(8),
            rm: gpr(9),
        },
        "1889",
        "lr %r8, %r9",
    ));

    insns.push((
        Inst::Mov32SImm16 {
            rd: writable_gpr(8),
            imm: -32768,
        },
        "A7888000",
        "lhi %r8, -32768",
    ));
    insns.push((
        Inst::Mov32SImm16 {
            rd: writable_gpr(8),
            imm: 32767,
        },
        "A7887FFF",
        "lhi %r8, 32767",
    ));
    insns.push((
        Inst::Mov32Imm {
            rd: writable_gpr(8),
            imm: 2147483648,
        },
        "C08980000000",
        "iilf %r8, 2147483648",
    ));
    insns.push((
        Inst::Mov32Imm {
            rd: writable_gpr(8),
            imm: 2147483647,
        },
        "C0897FFFFFFF",
        "iilf %r8, 2147483647",
    ));
    insns.push((
        Inst::Mov64SImm16 {
            rd: writable_gpr(8),
            imm: -32768,
        },
        "A7898000",
        "lghi %r8, -32768",
    ));
    insns.push((
        Inst::Mov64SImm16 {
            rd: writable_gpr(8),
            imm: 32767,
        },
        "A7897FFF",
        "lghi %r8, 32767",
    ));
    insns.push((
        Inst::Mov64SImm32 {
            rd: writable_gpr(8),
            imm: -2147483648,
        },
        "C08180000000",
        "lgfi %r8, -2147483648",
    ));
    insns.push((
        Inst::Mov64SImm32 {
            rd: writable_gpr(8),
            imm: 2147483647,
        },
        "C0817FFFFFFF",
        "lgfi %r8, 2147483647",
    ));
    insns.push((
        Inst::Mov64UImm16Shifted {
            rd: writable_gpr(8),
            imm: UImm16Shifted::maybe_from_u64(0x0000_0000_0000_ffff).unwrap(),
        },
        "A58FFFFF",
        "llill %r8, 65535",
    ));
    insns.push((
        Inst::Mov64UImm16Shifted {
            rd: writable_gpr(8),
            imm: UImm16Shifted::maybe_from_u64(0x0000_0000_ffff_0000).unwrap(),
        },
        "A58EFFFF",
        "llilh %r8, 65535",
    ));
    insns.push((
        Inst::Mov64UImm16Shifted {
            rd: writable_gpr(8),
            imm: UImm16Shifted::maybe_from_u64(0x0000_ffff_0000_0000).unwrap(),
        },
        "A58DFFFF",
        "llihl %r8, 65535",
    ));
    insns.push((
        Inst::Mov64UImm16Shifted {
            rd: writable_gpr(8),
            imm: UImm16Shifted::maybe_from_u64(0xffff_0000_0000_0000).unwrap(),
        },
        "A58CFFFF",
        "llihh %r8, 65535",
    ));
    insns.push((
        Inst::Mov64UImm32Shifted {
            rd: writable_gpr(8),
            imm: UImm32Shifted::maybe_from_u64(0x0000_0000_ffff_ffff).unwrap(),
        },
        "C08FFFFFFFFF",
        "llilf %r8, 4294967295",
    ));
    insns.push((
        Inst::Mov64UImm32Shifted {
            rd: writable_gpr(8),
            imm: UImm32Shifted::maybe_from_u64(0xffff_ffff_0000_0000).unwrap(),
        },
        "C08EFFFFFFFF",
        "llihf %r8, 4294967295",
    ));

    insns.push((
        Inst::Insert64UImm16Shifted {
            rd: writable_gpr(8),
            imm: UImm16Shifted::maybe_from_u64(0x0000_0000_0000_ffff).unwrap(),
        },
        "A583FFFF",
        "iill %r8, 65535",
    ));
    insns.push((
        Inst::Insert64UImm16Shifted {
            rd: writable_gpr(8),
            imm: UImm16Shifted::maybe_from_u64(0x0000_0000_ffff_0000).unwrap(),
        },
        "A582FFFF",
        "iilh %r8, 65535",
    ));
    insns.push((
        Inst::Insert64UImm16Shifted {
            rd: writable_gpr(8),
            imm: UImm16Shifted::maybe_from_u64(0x0000_ffff_0000_0000).unwrap(),
        },
        "A581FFFF",
        "iihl %r8, 65535",
    ));
    insns.push((
        Inst::Insert64UImm16Shifted {
            rd: writable_gpr(8),
            imm: UImm16Shifted::maybe_from_u64(0xffff_0000_0000_0000).unwrap(),
        },
        "A580FFFF",
        "iihh %r8, 65535",
    ));
    insns.push((
        Inst::Insert64UImm32Shifted {
            rd: writable_gpr(8),
            imm: UImm32Shifted::maybe_from_u64(0x0000_0000_ffff_ffff).unwrap(),
        },
        "C089FFFFFFFF",
        "iilf %r8, 4294967295",
    ));
    insns.push((
        Inst::Insert64UImm32Shifted {
            rd: writable_gpr(8),
            imm: UImm32Shifted::maybe_from_u64(0xffff_ffff_0000_0000).unwrap(),
        },
        "C088FFFFFFFF",
        "iihf %r8, 4294967295",
    ));

    insns.push((
        Inst::CMov32 {
            rd: writable_gpr(8),
            cond: Cond::from_mask(1),
            rm: gpr(9),
        },
        "B9F21089",
        "locro %r8, %r9",
    ));
    insns.push((
        Inst::CMov64 {
            rd: writable_gpr(8),
            cond: Cond::from_mask(1),
            rm: gpr(9),
        },
        "B9E21089",
        "locgro %r8, %r9",
    ));

    insns.push((
        Inst::CMov32SImm16 {
            rd: writable_gpr(8),
            cond: Cond::from_mask(1),
            imm: -32768,
        },
        "EC8180000042",
        "lochio %r8, -32768",
    ));
    insns.push((
        Inst::CMov32SImm16 {
            rd: writable_gpr(8),
            cond: Cond::from_mask(1),
            imm: 32767,
        },
        "EC817FFF0042",
        "lochio %r8, 32767",
    ));
    insns.push((
        Inst::CMov64SImm16 {
            rd: writable_gpr(8),
            cond: Cond::from_mask(1),
            imm: -32768,
        },
        "EC8180000046",
        "locghio %r8, -32768",
    ));
    insns.push((
        Inst::CMov64SImm16 {
            rd: writable_gpr(8),
            cond: Cond::from_mask(1),
            imm: 32767,
        },
        "EC817FFF0046",
        "locghio %r8, 32767",
    ));

    insns.push((
        Inst::Extend {
            rd: writable_gpr(1),
            rn: gpr(2),
            signed: false,
            from_bits: 8,
            to_bits: 32,
        },
        "B9940012",
        "llcr %r1, %r2",
    ));
    insns.push((
        Inst::Extend {
            rd: writable_gpr(1),
            rn: gpr(2),
            signed: true,
            from_bits: 8,
            to_bits: 32,
        },
        "B9260012",
        "lbr %r1, %r2",
    ));
    insns.push((
        Inst::Extend {
            rd: writable_gpr(1),
            rn: gpr(2),
            signed: false,
            from_bits: 16,
            to_bits: 32,
        },
        "B9950012",
        "llhr %r1, %r2",
    ));
    insns.push((
        Inst::Extend {
            rd: writable_gpr(1),
            rn: gpr(2),
            signed: true,
            from_bits: 16,
            to_bits: 32,
        },
        "B9270012",
        "lhr %r1, %r2",
    ));
    insns.push((
        Inst::Extend {
            rd: writable_gpr(1),
            rn: gpr(2),
            signed: false,
            from_bits: 8,
            to_bits: 64,
        },
        "B9840012",
        "llgcr %r1, %r2",
    ));
    insns.push((
        Inst::Extend {
            rd: writable_gpr(1),
            rn: gpr(2),
            signed: true,
            from_bits: 8,
            to_bits: 64,
        },
        "B9060012",
        "lgbr %r1, %r2",
    ));
    insns.push((
        Inst::Extend {
            rd: writable_gpr(1),
            rn: gpr(2),
            signed: false,
            from_bits: 16,
            to_bits: 64,
        },
        "B9850012",
        "llghr %r1, %r2",
    ));
    insns.push((
        Inst::Extend {
            rd: writable_gpr(1),
            rn: gpr(2),
            signed: true,
            from_bits: 16,
            to_bits: 64,
        },
        "B9070012",
        "lghr %r1, %r2",
    ));
    insns.push((
        Inst::Extend {
            rd: writable_gpr(1),
            rn: gpr(2),
            signed: false,
            from_bits: 32,
            to_bits: 64,
        },
        "B9160012",
        "llgfr %r1, %r2",
    ));
    insns.push((
        Inst::Extend {
            rd: writable_gpr(1),
            rn: gpr(2),
            signed: true,
            from_bits: 32,
            to_bits: 64,
        },
        "B9140012",
        "lgfr %r1, %r2",
    ));

    insns.push((
        Inst::Jump {
            dest: MachLabel::from_block(0),
        },
        "C0F400000000",
        "jg label0",
    ));

    insns.push((
        Inst::OneWayCondBr {
            target: MachLabel::from_block(0),
            cond: Cond::from_mask(1),
        },
        "C01400000000",
        "jgo label0",
    ));
    insns.push((
        Inst::OneWayCondBr {
            target: MachLabel::from_block(0),
            cond: Cond::from_mask(2),
        },
        "C02400000000",
        "jgh label0",
    ));
    insns.push((
        Inst::OneWayCondBr {
            target: MachLabel::from_block(0),
            cond: Cond::from_mask(3),
        },
        "C03400000000",
        "jgnle label0",
    ));
    insns.push((
        Inst::OneWayCondBr {
            target: MachLabel::from_block(0),
            cond: Cond::from_mask(4),
        },
        "C04400000000",
        "jgl label0",
    ));
    insns.push((
        Inst::OneWayCondBr {
            target: MachLabel::from_block(0),
            cond: Cond::from_mask(5),
        },
        "C05400000000",
        "jgnhe label0",
    ));
    insns.push((
        Inst::OneWayCondBr {
            target: MachLabel::from_block(0),
            cond: Cond::from_mask(6),
        },
        "C06400000000",
        "jglh label0",
    ));
    insns.push((
        Inst::OneWayCondBr {
            target: MachLabel::from_block(0),
            cond: Cond::from_mask(7),
        },
        "C07400000000",
        "jgne label0",
    ));
    insns.push((
        Inst::OneWayCondBr {
            target: MachLabel::from_block(0),
            cond: Cond::from_mask(8),
        },
        "C08400000000",
        "jge label0",
    ));
    insns.push((
        Inst::OneWayCondBr {
            target: MachLabel::from_block(0),
            cond: Cond::from_mask(9),
        },
        "C09400000000",
        "jgnlh label0",
    ));
    insns.push((
        Inst::OneWayCondBr {
            target: MachLabel::from_block(0),
            cond: Cond::from_mask(10),
        },
        "C0A400000000",
        "jghe label0",
    ));
    insns.push((
        Inst::OneWayCondBr {
            target: MachLabel::from_block(0),
            cond: Cond::from_mask(11),
        },
        "C0B400000000",
        "jgnl label0",
    ));
    insns.push((
        Inst::OneWayCondBr {
            target: MachLabel::from_block(0),
            cond: Cond::from_mask(12),
        },
        "C0C400000000",
        "jgle label0",
    ));
    insns.push((
        Inst::OneWayCondBr {
            target: MachLabel::from_block(0),
            cond: Cond::from_mask(13),
        },
        "C0D400000000",
        "jgnh label0",
    ));
    insns.push((
        Inst::OneWayCondBr {
            target: MachLabel::from_block(0),
            cond: Cond::from_mask(14),
        },
        "C0E400000000",
        "jgno label0",
    ));

    insns.push((
        Inst::CondBr {
            taken: MachLabel::from_block(0),
            not_taken: MachLabel::from_block(0),
            cond: Cond::from_mask(1),
        },
        "C01400000000C0F4FFFFFFFD",
        "jgo label0 ; jg label0",
    ));
    insns.push((
        Inst::CondBr {
            taken: MachLabel::from_block(0),
            not_taken: MachLabel::from_block(0),
            cond: Cond::from_mask(2),
        },
        "C02400000000C0F4FFFFFFFD",
        "jgh label0 ; jg label0",
    ));
    insns.push((
        Inst::CondBr {
            taken: MachLabel::from_block(0),
            not_taken: MachLabel::from_block(0),
            cond: Cond::from_mask(3),
        },
        "C03400000000C0F4FFFFFFFD",
        "jgnle label0 ; jg label0",
    ));
    insns.push((
        Inst::CondBr {
            taken: MachLabel::from_block(0),
            not_taken: MachLabel::from_block(0),
            cond: Cond::from_mask(4),
        },
        "C04400000000C0F4FFFFFFFD",
        "jgl label0 ; jg label0",
    ));
    insns.push((
        Inst::CondBr {
            taken: MachLabel::from_block(0),
            not_taken: MachLabel::from_block(0),
            cond: Cond::from_mask(5),
        },
        "C05400000000C0F4FFFFFFFD",
        "jgnhe label0 ; jg label0",
    ));
    insns.push((
        Inst::CondBr {
            taken: MachLabel::from_block(0),
            not_taken: MachLabel::from_block(0),
            cond: Cond::from_mask(6),
        },
        "C06400000000C0F4FFFFFFFD",
        "jglh label0 ; jg label0",
    ));
    insns.push((
        Inst::CondBr {
            taken: MachLabel::from_block(0),
            not_taken: MachLabel::from_block(0),
            cond: Cond::from_mask(7),
        },
        "C07400000000C0F4FFFFFFFD",
        "jgne label0 ; jg label0",
    ));
    insns.push((
        Inst::CondBr {
            taken: MachLabel::from_block(0),
            not_taken: MachLabel::from_block(0),
            cond: Cond::from_mask(8),
        },
        "C08400000000C0F4FFFFFFFD",
        "jge label0 ; jg label0",
    ));
    insns.push((
        Inst::CondBr {
            taken: MachLabel::from_block(0),
            not_taken: MachLabel::from_block(0),
            cond: Cond::from_mask(9),
        },
        "C09400000000C0F4FFFFFFFD",
        "jgnlh label0 ; jg label0",
    ));
    insns.push((
        Inst::CondBr {
            taken: MachLabel::from_block(0),
            not_taken: MachLabel::from_block(0),
            cond: Cond::from_mask(10),
        },
        "C0A400000000C0F4FFFFFFFD",
        "jghe label0 ; jg label0",
    ));
    insns.push((
        Inst::CondBr {
            taken: MachLabel::from_block(0),
            not_taken: MachLabel::from_block(0),
            cond: Cond::from_mask(11),
        },
        "C0B400000000C0F4FFFFFFFD",
        "jgnl label0 ; jg label0",
    ));
    insns.push((
        Inst::CondBr {
            taken: MachLabel::from_block(0),
            not_taken: MachLabel::from_block(0),
            cond: Cond::from_mask(12),
        },
        "C0C400000000C0F4FFFFFFFD",
        "jgle label0 ; jg label0",
    ));
    insns.push((
        Inst::CondBr {
            taken: MachLabel::from_block(0),
            not_taken: MachLabel::from_block(0),
            cond: Cond::from_mask(13),
        },
        "C0D400000000C0F4FFFFFFFD",
        "jgnh label0 ; jg label0",
    ));
    insns.push((
        Inst::CondBr {
            taken: MachLabel::from_block(0),
            not_taken: MachLabel::from_block(0),
            cond: Cond::from_mask(14),
        },
        "C0E400000000C0F4FFFFFFFD",
        "jgno label0 ; jg label0",
    ));

    insns.push((
        Inst::IndirectBr {
            rn: gpr(3),
            targets: vec![],
        },
        "07F3",
        "br %r3",
    ));

    insns.push((
        Inst::Call {
            link: writable_gpr(14),
            info: Box::new(CallInfo {
                dest: ExternalName::testcase("test0"),
                uses: Vec::new(),
                defs: Vec::new(),
                opcode: Opcode::Call,
            }),
        },
        "C0E500000000",
        "brasl %r14, %test0",
    ));

    insns.push((
        Inst::CallInd {
            link: writable_gpr(14),
            info: Box::new(CallIndInfo {
                rn: gpr(1),
                uses: Vec::new(),
                defs: Vec::new(),
                opcode: Opcode::CallIndirect,
            }),
        },
        "0DE1",
        "basr %r14, %r1",
    ));

    insns.push((Inst::Ret { link: gpr(14) }, "07FE", "br %r14"));

    insns.push((Inst::Debugtrap, "0001", "debugtrap"));

    insns.push((
        Inst::Trap {
            trap_code: TrapCode::StackOverflow,
        },
        "0000",
        "trap",
    ));
    insns.push((
        Inst::TrapIf {
            cond: Cond::from_mask(1),
            trap_code: TrapCode::StackOverflow,
        },
        "A7E400030000",
        "jno 6 ; trap",
    ));

    insns.push((
        Inst::Loop {
            body: vec![
                Inst::CmpRR {
                    op: CmpOp::CmpS32,
                    rn: gpr(2),
                    rm: gpr(3),
                },
                Inst::CondBreak {
                    cond: Cond::from_mask(13),
                },
                Inst::AtomicCas32 {
                    rd: writable_gpr(4),
                    rn: gpr(5),
                    mem: MemArg::BXD12 {
                        base: gpr(6),
                        index: zero_reg(),
                        disp: UImm12::maybe_from_u64(0).unwrap(),
                        flags: MemFlags::trusted(),
                    },
                },
            ],
            cond: Cond::from_mask(6),
        },
        "1923C0D400000008BA456000C064FFFFFFFA",
        "0: cr %r2, %r3 ; jgnh 1f ; cs %r4, %r5, 0(%r6) ; jglh 0b ; 1:",
    ));

    insns.push((
        Inst::FpuMove32 {
            rd: writable_fpr(8),
            rn: fpr(4),
        },
        "3884",
        "ler %f8, %f4",
    ));
    insns.push((
        Inst::FpuMove64 {
            rd: writable_fpr(8),
            rn: fpr(4),
        },
        "2884",
        "ldr %f8, %f4",
    ));
    insns.push((
        Inst::FpuCMov32 {
            rd: writable_fpr(8),
            rm: fpr(4),
            cond: Cond::from_mask(1),
        },
        "A7E400033884",
        "jno 6 ; ler %f8, %f4",
    ));
    insns.push((
        Inst::FpuCMov64 {
            rd: writable_fpr(8),
            rm: fpr(4),
            cond: Cond::from_mask(1),
        },
        "A7E400032884",
        "jno 6 ; ldr %f8, %f4",
    ));

    insns.push((
        Inst::MovToFpr {
            rd: writable_fpr(8),
            rn: gpr(4),
        },
        "B3C10084",
        "ldgr %f8, %r4",
    ));
    insns.push((
        Inst::MovFromFpr {
            rd: writable_gpr(8),
            rn: fpr(4),
        },
        "B3CD0084",
        "lgdr %r8, %f4",
    ));

    insns.push((
        Inst::FpuRR {
            fpu_op: FPUOp1::Abs32,
            rd: writable_fpr(8),
            rn: fpr(12),
        },
        "B300008C",
        "lpebr %f8, %f12",
    ));
    insns.push((
        Inst::FpuRR {
            fpu_op: FPUOp1::Abs64,
            rd: writable_fpr(8),
            rn: fpr(12),
        },
        "B310008C",
        "lpdbr %f8, %f12",
    ));
    insns.push((
        Inst::FpuRR {
            fpu_op: FPUOp1::Neg32,
            rd: writable_fpr(8),
            rn: fpr(12),
        },
        "B303008C",
        "lcebr %f8, %f12",
    ));
    insns.push((
        Inst::FpuRR {
            fpu_op: FPUOp1::Neg64,
            rd: writable_fpr(8),
            rn: fpr(12),
        },
        "B313008C",
        "lcdbr %f8, %f12",
    ));
    insns.push((
        Inst::FpuRR {
            fpu_op: FPUOp1::NegAbs32,
            rd: writable_fpr(8),
            rn: fpr(12),
        },
        "B301008C",
        "lnebr %f8, %f12",
    ));
    insns.push((
        Inst::FpuRR {
            fpu_op: FPUOp1::NegAbs64,
            rd: writable_fpr(8),
            rn: fpr(12),
        },
        "B311008C",
        "lndbr %f8, %f12",
    ));
    insns.push((
        Inst::FpuRR {
            fpu_op: FPUOp1::Sqrt32,
            rd: writable_fpr(8),
            rn: fpr(12),
        },
        "B314008C",
        "sqebr %f8, %f12",
    ));
    insns.push((
        Inst::FpuRR {
            fpu_op: FPUOp1::Sqrt64,
            rd: writable_fpr(8),
            rn: fpr(12),
        },
        "B315008C",
        "sqdbr %f8, %f12",
    ));
    insns.push((
        Inst::FpuRR {
            fpu_op: FPUOp1::Cvt32To64,
            rd: writable_fpr(8),
            rn: fpr(12),
        },
        "B304008C",
        "ldebr %f8, %f12",
    ));
    insns.push((
        Inst::FpuRR {
            fpu_op: FPUOp1::Cvt64To32,
            rd: writable_fpr(8),
            rn: fpr(12),
        },
        "B344008C",
        "ledbr %f8, %f12",
    ));

    insns.push((
        Inst::FpuRRR {
            fpu_op: FPUOp2::Add32,
            rd: writable_fpr(8),
            rm: fpr(12),
        },
        "B30A008C",
        "aebr %f8, %f12",
    ));
    insns.push((
        Inst::FpuRRR {
            fpu_op: FPUOp2::Add64,
            rd: writable_fpr(8),
            rm: fpr(12),
        },
        "B31A008C",
        "adbr %f8, %f12",
    ));
    insns.push((
        Inst::FpuRRR {
            fpu_op: FPUOp2::Sub32,
            rd: writable_fpr(8),
            rm: fpr(12),
        },
        "B30B008C",
        "sebr %f8, %f12",
    ));
    insns.push((
        Inst::FpuRRR {
            fpu_op: FPUOp2::Sub64,
            rd: writable_fpr(8),
            rm: fpr(12),
        },
        "B31B008C",
        "sdbr %f8, %f12",
    ));
    insns.push((
        Inst::FpuRRR {
            fpu_op: FPUOp2::Mul32,
            rd: writable_fpr(8),
            rm: fpr(12),
        },
        "B317008C",
        "meebr %f8, %f12",
    ));
    insns.push((
        Inst::FpuRRR {
            fpu_op: FPUOp2::Mul64,
            rd: writable_fpr(8),
            rm: fpr(12),
        },
        "B31C008C",
        "mdbr %f8, %f12",
    ));
    insns.push((
        Inst::FpuRRR {
            fpu_op: FPUOp2::Div32,
            rd: writable_fpr(8),
            rm: fpr(12),
        },
        "B30D008C",
        "debr %f8, %f12",
    ));
    insns.push((
        Inst::FpuRRR {
            fpu_op: FPUOp2::Div64,
            rd: writable_fpr(8),
            rm: fpr(12),
        },
        "B31D008C",
        "ddbr %f8, %f12",
    ));

    insns.push((
        Inst::FpuRRRR {
            fpu_op: FPUOp3::MAdd32,
            rd: writable_fpr(8),
            rn: fpr(12),
            rm: fpr(13),
        },
        "B30E80CD",
        "maebr %f8, %f12, %f13",
    ));
    insns.push((
        Inst::FpuRRRR {
            fpu_op: FPUOp3::MAdd64,
            rd: writable_fpr(8),
            rn: fpr(12),
            rm: fpr(13),
        },
        "B31E80CD",
        "madbr %f8, %f12, %f13",
    ));
    insns.push((
        Inst::FpuRRRR {
            fpu_op: FPUOp3::MSub32,
            rd: writable_fpr(8),
            rn: fpr(12),
            rm: fpr(13),
        },
        "B30F80CD",
        "msebr %f8, %f12, %f13",
    ));
    insns.push((
        Inst::FpuRRRR {
            fpu_op: FPUOp3::MSub64,
            rd: writable_fpr(8),
            rn: fpr(12),
            rm: fpr(13),
        },
        "B31F80CD",
        "msdbr %f8, %f12, %f13",
    ));

    insns.push((
        Inst::FpuToInt {
            op: FpuToIntOp::F32ToU32,
            rd: writable_gpr(1),
            rn: fpr(4),
        },
        "B39C5014",
        "clfebr %r1, 5, %f4, 0",
    ));

    insns.push((
        Inst::FpuToInt {
            op: FpuToIntOp::F32ToU64,
            rd: writable_gpr(1),
            rn: fpr(4),
        },
        "B3AC5014",
        "clgebr %r1, 5, %f4, 0",
    ));

    insns.push((
        Inst::FpuToInt {
            op: FpuToIntOp::F32ToI32,
            rd: writable_gpr(1),
            rn: fpr(4),
        },
        "B3985014",
        "cfebra %r1, 5, %f4, 0",
    ));

    insns.push((
        Inst::FpuToInt {
            op: FpuToIntOp::F32ToI64,
            rd: writable_gpr(1),
            rn: fpr(4),
        },
        "B3A85014",
        "cgebra %r1, 5, %f4, 0",
    ));

    insns.push((
        Inst::FpuToInt {
            op: FpuToIntOp::F64ToU32,
            rd: writable_gpr(1),
            rn: fpr(4),
        },
        "B39D5014",
        "clfdbr %r1, 5, %f4, 0",
    ));

    insns.push((
        Inst::FpuToInt {
            op: FpuToIntOp::F64ToU64,
            rd: writable_gpr(1),
            rn: fpr(4),
        },
        "B3AD5014",
        "clgdbr %r1, 5, %f4, 0",
    ));

    insns.push((
        Inst::FpuToInt {
            op: FpuToIntOp::F64ToI32,
            rd: writable_gpr(1),
            rn: fpr(4),
        },
        "B3995014",
        "cfdbra %r1, 5, %f4, 0",
    ));

    insns.push((
        Inst::FpuToInt {
            op: FpuToIntOp::F64ToI64,
            rd: writable_gpr(1),
            rn: fpr(4),
        },
        "B3A95014",
        "cgdbra %r1, 5, %f4, 0",
    ));

    insns.push((
        Inst::IntToFpu {
            op: IntToFpuOp::U32ToF32,
            rd: writable_fpr(1),
            rn: gpr(4),
        },
        "B3900014",
        "celfbr %f1, 0, %r4, 0",
    ));

    insns.push((
        Inst::IntToFpu {
            op: IntToFpuOp::I32ToF32,
            rd: writable_fpr(1),
            rn: gpr(4),
        },
        "B3940014",
        "cefbra %f1, 0, %r4, 0",
    ));

    insns.push((
        Inst::IntToFpu {
            op: IntToFpuOp::U32ToF64,
            rd: writable_fpr(1),
            rn: gpr(4),
        },
        "B3910014",
        "cdlfbr %f1, 0, %r4, 0",
    ));

    insns.push((
        Inst::IntToFpu {
            op: IntToFpuOp::I32ToF64,
            rd: writable_fpr(1),
            rn: gpr(4),
        },
        "B3950014",
        "cdfbra %f1, 0, %r4, 0",
    ));

    insns.push((
        Inst::IntToFpu {
            op: IntToFpuOp::U64ToF32,
            rd: writable_fpr(1),
            rn: gpr(4),
        },
        "B3A00014",
        "celgbr %f1, 0, %r4, 0",
    ));

    insns.push((
        Inst::IntToFpu {
            op: IntToFpuOp::I64ToF32,
            rd: writable_fpr(1),
            rn: gpr(4),
        },
        "B3A40014",
        "cegbra %f1, 0, %r4, 0",
    ));

    insns.push((
        Inst::IntToFpu {
            op: IntToFpuOp::U64ToF64,
            rd: writable_fpr(1),
            rn: gpr(4),
        },
        "B3A10014",
        "cdlgbr %f1, 0, %r4, 0",
    ));

    insns.push((
        Inst::IntToFpu {
            op: IntToFpuOp::I64ToF64,
            rd: writable_fpr(1),
            rn: gpr(4),
        },
        "B3A50014",
        "cdgbra %f1, 0, %r4, 0",
    ));

    insns.push((
        Inst::FpuCopysign {
            rd: writable_fpr(4),
            rn: fpr(8),
            rm: fpr(12),
        },
        "B372C048",
        "cpsdr %f4, %f12, %f8",
    ));

    insns.push((
        Inst::FpuCmp32 {
            rn: fpr(8),
            rm: fpr(12),
        },
        "B309008C",
        "cebr %f8, %f12",
    ));
    insns.push((
        Inst::FpuCmp64 {
            rn: fpr(8),
            rm: fpr(12),
        },
        "B319008C",
        "cdbr %f8, %f12",
    ));

    insns.push((
        Inst::FpuLoad32 {
            rd: writable_fpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "78102000",
        "le %f1, 0(%r2)",
    ));
    insns.push((
        Inst::FpuLoad32 {
            rd: writable_fpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "78102FFF",
        "le %f1, 4095(%r2)",
    ));
    insns.push((
        Inst::FpuLoad32 {
            rd: writable_fpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "ED1020008064",
        "ley %f1, -524288(%r2)",
    ));
    insns.push((
        Inst::FpuLoad32 {
            rd: writable_fpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "ED102FFF7F64",
        "ley %f1, 524287(%r2)",
    ));
    insns.push((
        Inst::FpuLoad32 {
            rd: writable_fpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "78123000",
        "le %f1, 0(%r2,%r3)",
    ));
    insns.push((
        Inst::FpuLoad32 {
            rd: writable_fpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "78123FFF",
        "le %f1, 4095(%r2,%r3)",
    ));
    insns.push((
        Inst::FpuLoad32 {
            rd: writable_fpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "ED1230008064",
        "ley %f1, -524288(%r2,%r3)",
    ));
    insns.push((
        Inst::FpuLoad32 {
            rd: writable_fpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "ED123FFF7F64",
        "ley %f1, 524287(%r2,%r3)",
    ));
    insns.push((
        Inst::FpuLoad64 {
            rd: writable_fpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "68102000",
        "ld %f1, 0(%r2)",
    ));
    insns.push((
        Inst::FpuLoad64 {
            rd: writable_fpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "68102FFF",
        "ld %f1, 4095(%r2)",
    ));
    insns.push((
        Inst::FpuLoad64 {
            rd: writable_fpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "ED1020008065",
        "ldy %f1, -524288(%r2)",
    ));
    insns.push((
        Inst::FpuLoad64 {
            rd: writable_fpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "ED102FFF7F65",
        "ldy %f1, 524287(%r2)",
    ));
    insns.push((
        Inst::FpuLoad64 {
            rd: writable_fpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "68123000",
        "ld %f1, 0(%r2,%r3)",
    ));
    insns.push((
        Inst::FpuLoad64 {
            rd: writable_fpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "68123FFF",
        "ld %f1, 4095(%r2,%r3)",
    ));
    insns.push((
        Inst::FpuLoad64 {
            rd: writable_fpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "ED1230008065",
        "ldy %f1, -524288(%r2,%r3)",
    ));
    insns.push((
        Inst::FpuLoad64 {
            rd: writable_fpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "ED123FFF7F65",
        "ldy %f1, 524287(%r2,%r3)",
    ));
    insns.push((
        Inst::FpuStore32 {
            rd: fpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "70102000",
        "ste %f1, 0(%r2)",
    ));
    insns.push((
        Inst::FpuStore32 {
            rd: fpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "70102FFF",
        "ste %f1, 4095(%r2)",
    ));
    insns.push((
        Inst::FpuStore32 {
            rd: fpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "ED1020008066",
        "stey %f1, -524288(%r2)",
    ));
    insns.push((
        Inst::FpuStore32 {
            rd: fpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "ED102FFF7F66",
        "stey %f1, 524287(%r2)",
    ));
    insns.push((
        Inst::FpuStore32 {
            rd: fpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "70123000",
        "ste %f1, 0(%r2,%r3)",
    ));
    insns.push((
        Inst::FpuStore32 {
            rd: fpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "70123FFF",
        "ste %f1, 4095(%r2,%r3)",
    ));
    insns.push((
        Inst::FpuStore32 {
            rd: fpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "ED1230008066",
        "stey %f1, -524288(%r2,%r3)",
    ));
    insns.push((
        Inst::FpuStore32 {
            rd: fpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "ED123FFF7F66",
        "stey %f1, 524287(%r2,%r3)",
    ));
    insns.push((
        Inst::FpuStore64 {
            rd: fpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "60102000",
        "std %f1, 0(%r2)",
    ));
    insns.push((
        Inst::FpuStore64 {
            rd: fpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "60102FFF",
        "std %f1, 4095(%r2)",
    ));
    insns.push((
        Inst::FpuStore64 {
            rd: fpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "ED1020008067",
        "stdy %f1, -524288(%r2)",
    ));
    insns.push((
        Inst::FpuStore64 {
            rd: fpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "ED102FFF7F67",
        "stdy %f1, 524287(%r2)",
    ));
    insns.push((
        Inst::FpuStore64 {
            rd: fpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "60123000",
        "std %f1, 0(%r2,%r3)",
    ));
    insns.push((
        Inst::FpuStore64 {
            rd: fpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "60123FFF",
        "std %f1, 4095(%r2,%r3)",
    ));
    insns.push((
        Inst::FpuStore64 {
            rd: fpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "ED1230008067",
        "stdy %f1, -524288(%r2,%r3)",
    ));
    insns.push((
        Inst::FpuStore64 {
            rd: fpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "ED123FFF7F67",
        "stdy %f1, 524287(%r2,%r3)",
    ));

    insns.push((
        Inst::FpuLoadRev32 {
            rd: writable_fpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E61020000003",
        "vlebrf %f1, 0(%r2), 0",
    ));
    insns.push((
        Inst::FpuLoadRev32 {
            rd: writable_fpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E6102FFF0003",
        "vlebrf %f1, 4095(%r2), 0",
    ));
    insns.push((
        Inst::FpuLoadRev32 {
            rd: writable_fpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020008071E61010000003",
        "lay %r1, -524288(%r2) ; vlebrf %f1, 0(%r1), 0",
    ));
    insns.push((
        Inst::FpuLoadRev32 {
            rd: writable_fpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF7F71E61010000003",
        "lay %r1, 524287(%r2) ; vlebrf %f1, 0(%r1), 0",
    ));
    insns.push((
        Inst::FpuLoadRev32 {
            rd: writable_fpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E61230000003",
        "vlebrf %f1, 0(%r2,%r3), 0",
    ));
    insns.push((
        Inst::FpuLoadRev32 {
            rd: writable_fpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E6123FFF0003",
        "vlebrf %f1, 4095(%r2,%r3), 0",
    ));
    insns.push((
        Inst::FpuLoadRev32 {
            rd: writable_fpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31230008071E61010000003",
        "lay %r1, -524288(%r2,%r3) ; vlebrf %f1, 0(%r1), 0",
    ));
    insns.push((
        Inst::FpuLoadRev32 {
            rd: writable_fpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF7F71E61010000003",
        "lay %r1, 524287(%r2,%r3) ; vlebrf %f1, 0(%r1), 0",
    ));
    insns.push((
        Inst::FpuLoadRev64 {
            rd: writable_fpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E61020000002",
        "vlebrg %f1, 0(%r2), 0",
    ));
    insns.push((
        Inst::FpuLoadRev64 {
            rd: writable_fpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E6102FFF0002",
        "vlebrg %f1, 4095(%r2), 0",
    ));
    insns.push((
        Inst::FpuLoadRev64 {
            rd: writable_fpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020008071E61010000002",
        "lay %r1, -524288(%r2) ; vlebrg %f1, 0(%r1), 0",
    ));
    insns.push((
        Inst::FpuLoadRev64 {
            rd: writable_fpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF7F71E61010000002",
        "lay %r1, 524287(%r2) ; vlebrg %f1, 0(%r1), 0",
    ));
    insns.push((
        Inst::FpuLoadRev64 {
            rd: writable_fpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E61230000002",
        "vlebrg %f1, 0(%r2,%r3), 0",
    ));
    insns.push((
        Inst::FpuLoadRev64 {
            rd: writable_fpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E6123FFF0002",
        "vlebrg %f1, 4095(%r2,%r3), 0",
    ));
    insns.push((
        Inst::FpuLoadRev64 {
            rd: writable_fpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31230008071E61010000002",
        "lay %r1, -524288(%r2,%r3) ; vlebrg %f1, 0(%r1), 0",
    ));
    insns.push((
        Inst::FpuLoadRev64 {
            rd: writable_fpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF7F71E61010000002",
        "lay %r1, 524287(%r2,%r3) ; vlebrg %f1, 0(%r1), 0",
    ));
    insns.push((
        Inst::FpuStoreRev32 {
            rd: fpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E6102000000B",
        "vstebrf %f1, 0(%r2), 0",
    ));
    insns.push((
        Inst::FpuStoreRev32 {
            rd: fpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E6102FFF000B",
        "vstebrf %f1, 4095(%r2), 0",
    ));
    insns.push((
        Inst::FpuStoreRev32 {
            rd: fpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020008071E6101000000B",
        "lay %r1, -524288(%r2) ; vstebrf %f1, 0(%r1), 0",
    ));
    insns.push((
        Inst::FpuStoreRev32 {
            rd: fpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF7F71E6101000000B",
        "lay %r1, 524287(%r2) ; vstebrf %f1, 0(%r1), 0",
    ));
    insns.push((
        Inst::FpuStoreRev32 {
            rd: fpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E6123000000B",
        "vstebrf %f1, 0(%r2,%r3), 0",
    ));
    insns.push((
        Inst::FpuStoreRev32 {
            rd: fpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E6123FFF000B",
        "vstebrf %f1, 4095(%r2,%r3), 0",
    ));
    insns.push((
        Inst::FpuStoreRev32 {
            rd: fpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31230008071E6101000000B",
        "lay %r1, -524288(%r2,%r3) ; vstebrf %f1, 0(%r1), 0",
    ));
    insns.push((
        Inst::FpuStoreRev32 {
            rd: fpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF7F71E6101000000B",
        "lay %r1, 524287(%r2,%r3) ; vstebrf %f1, 0(%r1), 0",
    ));
    insns.push((
        Inst::FpuStoreRev64 {
            rd: fpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E6102000000A",
        "vstebrg %f1, 0(%r2), 0",
    ));
    insns.push((
        Inst::FpuStoreRev64 {
            rd: fpr(1),
            mem: MemArg::BXD12 {
                base: gpr(2),
                index: zero_reg(),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E6102FFF000A",
        "vstebrg %f1, 4095(%r2), 0",
    ));
    insns.push((
        Inst::FpuStoreRev64 {
            rd: fpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31020008071E6101000000A",
        "lay %r1, -524288(%r2) ; vstebrg %f1, 0(%r1), 0",
    ));
    insns.push((
        Inst::FpuStoreRev64 {
            rd: fpr(1),
            mem: MemArg::BXD20 {
                base: gpr(2),
                index: zero_reg(),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3102FFF7F71E6101000000A",
        "lay %r1, 524287(%r2) ; vstebrg %f1, 0(%r1), 0",
    ));
    insns.push((
        Inst::FpuStoreRev64 {
            rd: fpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::zero(),
                flags: MemFlags::trusted(),
            },
        },
        "E6123000000A",
        "vstebrg %f1, 0(%r2,%r3), 0",
    ));
    insns.push((
        Inst::FpuStoreRev64 {
            rd: fpr(1),
            mem: MemArg::BXD12 {
                base: gpr(3),
                index: gpr(2),
                disp: UImm12::maybe_from_u64(4095).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E6123FFF000A",
        "vstebrg %f1, 4095(%r2,%r3), 0",
    ));
    insns.push((
        Inst::FpuStoreRev64 {
            rd: fpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(-524288).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E31230008071E6101000000A",
        "lay %r1, -524288(%r2,%r3) ; vstebrg %f1, 0(%r1), 0",
    ));
    insns.push((
        Inst::FpuStoreRev64 {
            rd: fpr(1),
            mem: MemArg::BXD20 {
                base: gpr(3),
                index: gpr(2),
                disp: SImm20::maybe_from_i64(524287).unwrap(),
                flags: MemFlags::trusted(),
            },
        },
        "E3123FFF7F71E6101000000A",
        "lay %r1, 524287(%r2,%r3) ; vstebrg %f1, 0(%r1), 0",
    ));

    insns.push((
        Inst::LoadFpuConst32 {
            rd: writable_fpr(8),
            const_data: 1.0_f32.to_bits(),
        },
        "A71500043F80000078801000",
        "bras %r1, 8 ; data.f32 1 ; le %f8, 0(%r1)",
    ));
    insns.push((
        Inst::LoadFpuConst64 {
            rd: writable_fpr(8),
            const_data: 1.0_f64.to_bits(),
        },
        "A71500063FF000000000000068801000",
        "bras %r1, 12 ; data.f64 1 ; ld %f8, 0(%r1)",
    ));

    insns.push((
        Inst::FpuRound {
            rd: writable_fpr(8),
            rn: fpr(12),
            op: FpuRoundMode::Minus32,
        },
        "B357708C",
        "fiebr %f8, %f12, 7",
    ));
    insns.push((
        Inst::FpuRound {
            rd: writable_fpr(8),
            rn: fpr(12),
            op: FpuRoundMode::Minus64,
        },
        "B35F708C",
        "fidbr %f8, %f12, 7",
    ));
    insns.push((
        Inst::FpuRound {
            rd: writable_fpr(8),
            rn: fpr(12),
            op: FpuRoundMode::Plus32,
        },
        "B357608C",
        "fiebr %f8, %f12, 6",
    ));
    insns.push((
        Inst::FpuRound {
            rd: writable_fpr(8),
            rn: fpr(12),
            op: FpuRoundMode::Plus64,
        },
        "B35F608C",
        "fidbr %f8, %f12, 6",
    ));
    insns.push((
        Inst::FpuRound {
            rd: writable_fpr(8),
            rn: fpr(12),
            op: FpuRoundMode::Zero32,
        },
        "B357508C",
        "fiebr %f8, %f12, 5",
    ));
    insns.push((
        Inst::FpuRound {
            rd: writable_fpr(8),
            rn: fpr(12),
            op: FpuRoundMode::Zero64,
        },
        "B35F508C",
        "fidbr %f8, %f12, 5",
    ));
    insns.push((
        Inst::FpuRound {
            rd: writable_fpr(8),
            rn: fpr(12),
            op: FpuRoundMode::Nearest32,
        },
        "B357408C",
        "fiebr %f8, %f12, 4",
    ));
    insns.push((
        Inst::FpuRound {
            rd: writable_fpr(8),
            rn: fpr(12),
            op: FpuRoundMode::Nearest64,
        },
        "B35F408C",
        "fidbr %f8, %f12, 4",
    ));

    insns.push((
        Inst::FpuVecRRR {
            fpu_op: FPUOp2::Max32,
            rd: writable_fpr(4),
            rn: fpr(6),
            rm: fpr(8),
        },
        "E746801820EF",
        "wfmaxsb %f4, %f6, %f8, 1",
    ));
    insns.push((
        Inst::FpuVecRRR {
            fpu_op: FPUOp2::Max64,
            rd: writable_fpr(4),
            rn: fpr(6),
            rm: fpr(8),
        },
        "E746801830EF",
        "wfmaxdb %f4, %f6, %f8, 1",
    ));
    insns.push((
        Inst::FpuVecRRR {
            fpu_op: FPUOp2::Min32,
            rd: writable_fpr(4),
            rn: fpr(6),
            rm: fpr(8),
        },
        "E746801820EE",
        "wfminsb %f4, %f6, %f8, 1",
    ));
    insns.push((
        Inst::FpuVecRRR {
            fpu_op: FPUOp2::Min64,
            rd: writable_fpr(4),
            rn: fpr(6),
            rm: fpr(8),
        },
        "E746801830EE",
        "wfmindb %f4, %f6, %f8, 1",
    ));

    let flags = settings::Flags::new(settings::builder());

    use crate::settings::Configurable;
    let mut isa_flag_builder = s390x_settings::builder();
    isa_flag_builder.enable("arch13").unwrap();
    let isa_flags = s390x_settings::Flags::new(&flags, isa_flag_builder);

    let rru = create_reg_universe(&flags);
    let emit_info = EmitInfo::new(flags, isa_flags);
    for (insn, expected_encoding, expected_printing) in insns {
        println!(
            "S390x: {:?}, {}, {}",
            insn, expected_encoding, expected_printing
        );

        // Check the printed text is as expected.
        let actual_printing = insn.show_rru(Some(&rru));
        assert_eq!(expected_printing, actual_printing);

        let mut buffer = MachBuffer::new();

        // Label 0 before the instruction.
        let label0 = buffer.get_label();
        buffer.bind_label(label0);

        // Emit the instruction.
        insn.emit(&mut buffer, &emit_info, &mut Default::default());

        // Label 1 after the instruction.
        let label1 = buffer.get_label();
        buffer.bind_label(label1);

        let buffer = buffer.finish();
        let actual_encoding = &buffer.stringify_code_bytes();
        assert_eq!(expected_encoding, actual_encoding);
    }
}
