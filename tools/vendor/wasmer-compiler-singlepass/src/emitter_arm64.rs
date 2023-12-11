pub use crate::arm64_decl::{ARM64Register, ArgumentRegisterAllocator, GPR, NEON};
use crate::codegen_error;
use crate::common_decl::Size;
use crate::location::Location as AbstractLocation;
pub use crate::location::{Multiplier, Reg};
pub use crate::machine::{Label, Offset};
use dynasm::dynasm;
pub use dynasmrt::aarch64::{encode_logical_immediate_32bit, encode_logical_immediate_64bit};
use dynasmrt::{
    aarch64::Aarch64Relocation, AssemblyOffset, DynamicLabel, DynasmApi, DynasmLabelApi,
    VecAssembler,
};
use wasmer_types::{
    CallingConvention, CompileError, CustomSection, CustomSectionProtection, FunctionBody,
    FunctionIndex, FunctionType, SectionBody, Type, VMOffsets,
};

type Assembler = VecAssembler<Aarch64Relocation>;

/// Force `dynasm!` to use the correct arch (aarch64) when cross-compiling.
/// `dynasm!` proc-macro tries to auto-detect it by default by looking at the
/// `target_arch`, but it sees the `target_arch` of the proc-macro itself, which
/// is always equal to host, even when cross-compiling.
macro_rules! dynasm {
    ($a:expr ; $($tt:tt)*) => {
        dynasm::dynasm!(
            $a
            ; .arch aarch64
            ; $($tt)*
        )
    };
}

pub type Location = AbstractLocation<GPR, NEON>;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[allow(dead_code)]
#[repr(u8)]
pub enum Condition {
    // meaning for cmp or sub
    /// Equal
    Eq = 0,
    /// Not equal
    Ne = 1,
    /// Unsigned higher or same (or carry set)
    Cs = 2,
    /// Unsigned lower (or carry clear)
    Cc = 3,
    /// Negative. The mnemonic stands for "minus"
    Mi = 4,
    /// Positive or zero. The mnemonic stands for "plus"
    Pl = 5,
    /// Signed overflow. The mnemonic stands for "V set"
    Vs = 6,
    /// No signed overflow. The mnemonic stands for "V clear"
    Vc = 7,
    /// Unsigned higher
    Hi = 8,
    /// Unsigned lower or same
    Ls = 9,
    /// Signed greater than or equal
    Ge = 10,
    /// Signed less than
    Lt = 11,
    /// Signed greater than
    Gt = 12,
    /// Signed less than or equal
    Le = 13,
    /// Always executed
    Al = 14,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[allow(dead_code, clippy::upper_case_acronyms)]
pub enum NeonOrMemory {
    NEON(NEON),
    Memory(GPR, i32),
}

#[derive(Copy, Clone, Debug)]
#[allow(dead_code, clippy::upper_case_acronyms)]
pub enum GPROrMemory {
    GPR(GPR),
    Memory(GPR, i32),
}

pub trait EmitterARM64 {
    fn get_label(&mut self) -> Label;
    fn get_offset(&self) -> Offset;
    fn get_jmp_instr_size(&self) -> u8;

    fn finalize_function(&mut self);

    fn emit_str(&mut self, sz: Size, reg: Location, addr: Location) -> Result<(), CompileError>;
    fn emit_ldr(&mut self, sz: Size, reg: Location, addr: Location) -> Result<(), CompileError>;
    fn emit_stur(
        &mut self,
        sz: Size,
        reg: Location,
        addr: GPR,
        offset: i32,
    ) -> Result<(), CompileError>;
    fn emit_ldur(
        &mut self,
        sz: Size,
        reg: Location,
        addr: GPR,
        offset: i32,
    ) -> Result<(), CompileError>;
    fn emit_strdb(
        &mut self,
        sz: Size,
        reg: Location,
        addr: GPR,
        offset: u32,
    ) -> Result<(), CompileError>;
    fn emit_stria(
        &mut self,
        sz: Size,
        reg: Location,
        addr: GPR,
        offset: u32,
    ) -> Result<(), CompileError>;
    fn emit_ldria(
        &mut self,
        sz: Size,
        reg: Location,
        addr: GPR,
        offset: u32,
    ) -> Result<(), CompileError>;
    fn emit_stpdb(
        &mut self,
        sz: Size,
        reg1: Location,
        reg2: Location,
        addr: GPR,
        offset: u32,
    ) -> Result<(), CompileError>;
    fn emit_ldpia(
        &mut self,
        sz: Size,
        reg1: Location,
        reg2: Location,
        addr: GPR,
        offset: u32,
    ) -> Result<(), CompileError>;

    fn emit_ldrb(&mut self, sz: Size, reg: Location, dst: Location) -> Result<(), CompileError>;
    fn emit_ldrh(&mut self, sz: Size, reg: Location, dst: Location) -> Result<(), CompileError>;
    fn emit_ldrsb(&mut self, sz: Size, reg: Location, dst: Location) -> Result<(), CompileError>;
    fn emit_ldrsh(&mut self, sz: Size, reg: Location, dst: Location) -> Result<(), CompileError>;
    fn emit_ldrsw(&mut self, sz: Size, reg: Location, dst: Location) -> Result<(), CompileError>;
    fn emit_strb(&mut self, sz: Size, reg: Location, dst: Location) -> Result<(), CompileError>;
    fn emit_strh(&mut self, sz: Size, reg: Location, dst: Location) -> Result<(), CompileError>;

    fn emit_ldaxr(&mut self, sz: Size, reg: Location, dst: Location) -> Result<(), CompileError>;
    fn emit_ldaxrb(&mut self, sz: Size, reg: Location, dst: Location) -> Result<(), CompileError>;
    fn emit_ldaxrh(&mut self, sz: Size, reg: Location, dst: Location) -> Result<(), CompileError>;
    fn emit_stlxr(
        &mut self,
        sz: Size,
        status: Location,
        reg: Location,
        dst: Location,
    ) -> Result<(), CompileError>;
    fn emit_stlxrb(
        &mut self,
        sz: Size,
        status: Location,
        reg: Location,
        dst: Location,
    ) -> Result<(), CompileError>;
    fn emit_stlxrh(
        &mut self,
        sz: Size,
        status: Location,
        reg: Location,
        dst: Location,
    ) -> Result<(), CompileError>;

    fn emit_mov(&mut self, sz: Size, src: Location, dst: Location) -> Result<(), CompileError>;

    fn emit_movn(&mut self, sz: Size, reg: Location, val: u32) -> Result<(), CompileError>;
    fn emit_movz(&mut self, reg: Location, val: u32) -> Result<(), CompileError>;
    fn emit_movk(&mut self, reg: Location, val: u32, shift: u32) -> Result<(), CompileError>;

    fn emit_mov_imm(&mut self, dst: Location, val: u64) -> Result<(), CompileError>;

    fn emit_add(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError>;
    fn emit_sub(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError>;
    fn emit_mul(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError>;
    fn emit_adds(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError>;
    fn emit_subs(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError>;

    fn emit_add_lsl(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        lsl: u32,
        dst: Location,
    ) -> Result<(), CompileError>;

    fn emit_cmp(&mut self, sz: Size, src: Location, dst: Location) -> Result<(), CompileError>;
    fn emit_tst(&mut self, sz: Size, src: Location, dst: Location) -> Result<(), CompileError>;

    fn emit_lsl(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError>;
    fn emit_lsr(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError>;
    fn emit_asr(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError>;
    fn emit_ror(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError>;

    fn emit_or(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError>;
    fn emit_and(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError>;
    fn emit_eor(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError>;

    fn emit_bfc(
        &mut self,
        se: Size,
        lsb: u32,
        width: u32,
        dst: Location,
    ) -> Result<(), CompileError>;
    fn emit_bfi(
        &mut self,
        se: Size,
        src: Location,
        lsb: u32,
        width: u32,
        dst: Location,
    ) -> Result<(), CompileError>;

    fn emit_udiv(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError>;
    fn emit_sdiv(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError>;
    /// msub : c - a*b -> dst
    fn emit_msub(
        &mut self,
        sz: Size,
        a: Location,
        b: Location,
        c: Location,
        dst: Location,
    ) -> Result<(), CompileError>;

    fn emit_sxtb(&mut self, sz: Size, src: Location, dst: Location) -> Result<(), CompileError>;
    fn emit_sxth(&mut self, sz: Size, src: Location, dst: Location) -> Result<(), CompileError>;
    fn emit_sxtw(&mut self, sz: Size, src: Location, dst: Location) -> Result<(), CompileError>;
    fn emit_uxtb(&mut self, sz: Size, src: Location, dst: Location) -> Result<(), CompileError>;
    fn emit_uxth(&mut self, sz: Size, src: Location, dst: Location) -> Result<(), CompileError>;

    fn emit_cset(&mut self, sz: Size, dst: Location, cond: Condition) -> Result<(), CompileError>;
    fn emit_csetm(&mut self, sz: Size, dst: Location, cond: Condition) -> Result<(), CompileError>;
    fn emit_cinc(
        &mut self,
        sz: Size,
        src: Location,
        dst: Location,
        cond: Condition,
    ) -> Result<(), CompileError>;
    fn emit_clz(&mut self, sz: Size, src: Location, dst: Location) -> Result<(), CompileError>;
    fn emit_rbit(&mut self, sz: Size, src: Location, dst: Location) -> Result<(), CompileError>;

    fn emit_label(&mut self, label: Label) -> Result<(), CompileError>;
    fn emit_load_label(&mut self, reg: GPR, label: Label) -> Result<(), CompileError>;
    fn emit_b_label(&mut self, label: Label) -> Result<(), CompileError>;
    fn emit_cbz_label(&mut self, sz: Size, reg: Location, label: Label)
        -> Result<(), CompileError>;
    fn emit_cbnz_label(
        &mut self,
        sz: Size,
        reg: Location,
        label: Label,
    ) -> Result<(), CompileError>;
    fn emit_tbz_label(
        &mut self,
        sz: Size,
        reg: Location,
        n: u32,
        label: Label,
    ) -> Result<(), CompileError>;
    fn emit_tbnz_label(
        &mut self,
        sz: Size,
        reg: Location,
        n: u32,
        label: Label,
    ) -> Result<(), CompileError>;
    fn emit_bcond_label(&mut self, condition: Condition, label: Label) -> Result<(), CompileError>;
    fn emit_bcond_label_far(
        &mut self,
        condition: Condition,
        label: Label,
    ) -> Result<(), CompileError>;
    fn emit_b_register(&mut self, reg: GPR) -> Result<(), CompileError>;
    fn emit_call_label(&mut self, label: Label) -> Result<(), CompileError>;
    fn emit_call_register(&mut self, reg: GPR) -> Result<(), CompileError>;
    fn emit_ret(&mut self) -> Result<(), CompileError>;

    fn emit_udf(&mut self, payload: u16) -> Result<(), CompileError>;
    fn emit_dmb(&mut self) -> Result<(), CompileError>;
    fn emit_brk(&mut self) -> Result<(), CompileError>;

    fn emit_fcmp(&mut self, sz: Size, src1: Location, src2: Location) -> Result<(), CompileError>;
    fn emit_fneg(&mut self, sz: Size, src: Location, dst: Location) -> Result<(), CompileError>;
    fn emit_fsqrt(&mut self, sz: Size, src: Location, dst: Location) -> Result<(), CompileError>;

    fn emit_fadd(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError>;
    fn emit_fsub(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError>;
    fn emit_fmul(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError>;
    fn emit_fdiv(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError>;

    fn emit_fmin(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError>;
    fn emit_fmax(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError>;

    fn emit_frintz(&mut self, sz: Size, src: Location, dst: Location) -> Result<(), CompileError>;
    fn emit_frintn(&mut self, sz: Size, src: Location, dst: Location) -> Result<(), CompileError>;
    fn emit_frintm(&mut self, sz: Size, src: Location, dst: Location) -> Result<(), CompileError>;
    fn emit_frintp(&mut self, sz: Size, src: Location, dst: Location) -> Result<(), CompileError>;

    fn emit_scvtf(
        &mut self,
        sz_in: Size,
        src: Location,
        sz_out: Size,
        dst: Location,
    ) -> Result<(), CompileError>;
    fn emit_ucvtf(
        &mut self,
        sz_in: Size,
        src: Location,
        sz_out: Size,
        dst: Location,
    ) -> Result<(), CompileError>;
    fn emit_fcvt(&mut self, sz_in: Size, src: Location, dst: Location) -> Result<(), CompileError>;
    fn emit_fcvtzs(
        &mut self,
        sz_in: Size,
        src: Location,
        sz_out: Size,
        dst: Location,
    ) -> Result<(), CompileError>;
    fn emit_fcvtzu(
        &mut self,
        sz_in: Size,
        src: Location,
        sz_out: Size,
        dst: Location,
    ) -> Result<(), CompileError>;

    fn emit_read_fpcr(&mut self, reg: GPR) -> Result<(), CompileError>;
    fn emit_write_fpcr(&mut self, reg: GPR) -> Result<(), CompileError>;
    fn emit_read_fpsr(&mut self, reg: GPR) -> Result<(), CompileError>;
    fn emit_write_fpsr(&mut self, reg: GPR) -> Result<(), CompileError>;

    fn arch_supports_canonicalize_nan(&self) -> bool {
        true
    }

    fn arch_requires_indirect_call_trampoline(&self) -> bool {
        false
    }

    fn arch_emit_indirect_call_with_trampoline(
        &mut self,
        _loc: Location,
    ) -> Result<(), CompileError> {
        codegen_error!("singlepass arch_emit_indirect_call_with_trampoline unimplemented")
    }
}

impl EmitterARM64 for Assembler {
    fn get_label(&mut self) -> DynamicLabel {
        self.new_dynamic_label()
    }

    fn get_offset(&self) -> AssemblyOffset {
        self.offset()
    }

    fn get_jmp_instr_size(&self) -> u8 {
        4 // relative jump, not full 32bits capable
    }

    fn finalize_function(&mut self) {
        dynasm!(
            self
            ; const_neg_one_32:
            ; .word -1
            ; const_zero_32:
            ; .word 0
            ; const_pos_one_32:
            ; .word 1
        );
    }

    fn emit_str(&mut self, sz: Size, reg: Location, addr: Location) -> Result<(), CompileError> {
        match (sz, reg, addr) {
            (Size::S64, Location::GPR(reg), Location::Memory(addr, disp)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                let disp = disp as u32;
                assert!((disp & 0x7) == 0 && (disp < 0x8000));
                dynasm!(self ; str X(reg), [X(addr), disp]);
            }
            (Size::S32, Location::GPR(reg), Location::Memory(addr, disp)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                let disp = disp as u32;
                assert!((disp & 0x3) == 0 && (disp < 0x4000));
                dynasm!(self ; str W(reg), [X(addr), disp]);
            }
            (Size::S16, Location::GPR(reg), Location::Memory(addr, disp)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                let disp = disp as u32;
                assert!((disp & 0x1) == 0 && (disp < 0x2000));
                dynasm!(self ; strh W(reg), [X(addr), disp]);
            }
            (Size::S8, Location::GPR(reg), Location::Memory(addr, disp)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                let disp = disp as u32;
                assert!(disp < 0x1000);
                dynasm!(self ; strb W(reg), [X(addr), disp]);
            }
            (Size::S64, Location::SIMD(reg), Location::Memory(addr, disp)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                let disp = disp as u32;
                assert!((disp & 0x7) == 0 && (disp < 0x8000));
                dynasm!(self ; str D(reg), [X(addr), disp]);
            }
            (Size::S32, Location::SIMD(reg), Location::Memory(addr, disp)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                let disp = disp as u32;
                assert!((disp & 0x3) == 0 && (disp < 0x4000));
                dynasm!(self ; str S(reg), [X(addr), disp]);
            }
            (Size::S64, Location::GPR(reg), Location::Memory2(addr, r2, mult, offs)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                let r2 = r2.into_index() as u32;
                assert!(offs == 0);
                let mult = mult as u32;
                match mult {
                    0 => dynasm!(self ; str X(reg), [X(addr)]),
                    1 => dynasm!(self ; str X(reg), [X(addr), X(r2)]),
                    _ => dynasm!(self ; str X(reg), [X(addr), X(r2), LSL mult]),
                };
            }
            (Size::S32, Location::GPR(reg), Location::Memory2(addr, r2, mult, offs)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                let r2 = r2.into_index() as u32;
                assert!(offs == 0);
                let mult = mult as u32;
                match mult {
                    0 => dynasm!(self ; str W(reg), [X(addr)]),
                    1 => dynasm!(self ; str W(reg), [X(addr), X(r2)]),
                    _ => dynasm!(self ; str W(reg), [X(addr), X(r2), LSL mult]),
                };
            }
            _ => codegen_error!("singlepass can't emit STR {:?}, {:?}, {:?}", sz, reg, addr),
        }
        Ok(())
    }
    fn emit_ldr(&mut self, sz: Size, reg: Location, addr: Location) -> Result<(), CompileError> {
        match (sz, reg, addr) {
            (Size::S64, Location::GPR(reg), Location::Memory(addr, disp)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                assert!((disp & 0x7) == 0 && (disp < 0x8000));
                let disp = disp as u32;
                dynasm!(self ; ldr X(reg), [X(addr), disp]);
            }
            (Size::S32, Location::GPR(reg), Location::Memory(addr, disp)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                assert!((disp & 0x3) == 0 && (disp < 0x4000));
                let disp = disp as u32;
                dynasm!(self ; ldr W(reg), [X(addr), disp]);
            }
            (Size::S16, Location::GPR(reg), Location::Memory(addr, disp)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                assert!((disp & 0x1 == 0) && (disp < 0x2000));
                let disp = disp as u32;
                dynasm!(self ; ldrh W(reg), [X(addr), disp]);
            }
            (Size::S8, Location::GPR(reg), Location::Memory(addr, disp)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                assert!(disp < 0x1000);
                let disp = disp as u32;
                dynasm!(self ; ldrb W(reg), [X(addr), disp]);
            }
            (Size::S64, Location::GPR(reg), Location::Memory2(addr, r2, mult, offs)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                let r2 = r2.into_index() as u32;
                assert!(offs == 0);
                let mult = mult as u32;
                match mult {
                    0 => dynasm!(self ; ldr X(reg), [X(addr)]),
                    1 => dynasm!(self ; ldr X(reg), [X(addr), X(r2)]),
                    _ => dynasm!(self ; ldr X(reg), [X(addr), X(r2), LSL mult]),
                };
            }
            (Size::S32, Location::GPR(reg), Location::Memory2(addr, r2, mult, offs)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                let r2 = r2.into_index() as u32;
                assert!(offs == 0);
                let mult = mult as u32;
                match mult {
                    0 => dynasm!(self ; ldr W(reg), [X(addr)]),
                    1 => dynasm!(self ; ldr W(reg), [X(addr), X(r2)]),
                    _ => dynasm!(self ; ldr W(reg), [X(addr), X(r2), LSL mult]),
                };
            }
            (Size::S64, Location::SIMD(reg), Location::Memory(addr, disp)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                let disp = disp as u32;
                assert!((disp & 0x7) == 0 && (disp < 0x8000));
                dynasm!(self ; ldr D(reg), [X(addr), disp]);
            }
            (Size::S32, Location::SIMD(reg), Location::Memory(addr, disp)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                let disp = disp as u32;
                assert!((disp & 0x3) == 0 && (disp < 0x4000));
                dynasm!(self ; ldr S(reg), [X(addr), disp]);
            }
            (Size::S64, Location::SIMD(reg), Location::Memory2(addr, r2, mult, offs)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                let r2 = r2.into_index() as u32;
                assert!(offs == 0);
                let mult = mult as u32;
                match mult {
                    0 => dynasm!(self ; ldr D(reg), [X(addr)]),
                    1 => dynasm!(self ; ldr D(reg), [X(addr), X(r2)]),
                    _ => dynasm!(self ; ldr D(reg), [X(addr), X(r2), LSL mult]),
                };
            }
            (Size::S32, Location::SIMD(reg), Location::Memory2(addr, r2, mult, offs)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                let r2 = r2.into_index() as u32;
                assert!(offs == 0);
                let mult = mult as u32;
                match mult {
                    0 => dynasm!(self ; ldr S(reg), [X(addr)]),
                    1 => dynasm!(self ; ldr S(reg), [X(addr), X(r2)]),
                    _ => dynasm!(self ; ldr S(reg), [X(addr), X(r2), LSL mult]),
                };
            }
            _ => codegen_error!("singlepass can't emit LDR {:?}, {:?}, {:?}", sz, reg, addr),
        }
        Ok(())
    }
    fn emit_stur(
        &mut self,
        sz: Size,
        reg: Location,
        addr: GPR,
        offset: i32,
    ) -> Result<(), CompileError> {
        assert!((-255..=255).contains(&offset));
        match (sz, reg) {
            (Size::S64, Location::GPR(reg)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                dynasm!(self ; stur X(reg), [X(addr), offset]);
            }
            (Size::S32, Location::GPR(reg)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                dynasm!(self ; stur W(reg), [X(addr), offset]);
            }
            (Size::S64, Location::SIMD(reg)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                dynasm!(self ; stur D(reg), [X(addr), offset]);
            }
            (Size::S32, Location::SIMD(reg)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                dynasm!(self ; stur S(reg), [X(addr), offset]);
            }
            _ => codegen_error!(
                "singlepass can't emit STUR {:?}, {:?}, {:?}, {:?}",
                sz,
                reg,
                addr,
                offset
            ),
        }
        Ok(())
    }
    fn emit_ldur(
        &mut self,
        sz: Size,
        reg: Location,
        addr: GPR,
        offset: i32,
    ) -> Result<(), CompileError> {
        assert!((-255..=255).contains(&offset));
        match (sz, reg) {
            (Size::S64, Location::GPR(reg)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                dynasm!(self ; ldur X(reg), [X(addr), offset]);
            }
            (Size::S32, Location::GPR(reg)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                dynasm!(self ; ldur W(reg), [X(addr), offset]);
            }
            (Size::S64, Location::SIMD(reg)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                dynasm!(self ; ldur D(reg), [X(addr), offset]);
            }
            (Size::S32, Location::SIMD(reg)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                dynasm!(self ; ldur S(reg), [X(addr), offset]);
            }
            _ => codegen_error!(
                "singlepass can't emit LDUR {:?}, {:?}, {:?}, {:?}",
                sz,
                reg,
                addr,
                offset
            ),
        }
        Ok(())
    }

    fn emit_strdb(
        &mut self,
        sz: Size,
        reg: Location,
        addr: GPR,
        offset: u32,
    ) -> Result<(), CompileError> {
        assert!(offset <= 255);
        match (sz, reg) {
            (Size::S64, Location::GPR(reg)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                dynasm!(self ; str X(reg), [X(addr), -(offset as i32)]!);
            }
            (Size::S64, Location::SIMD(reg)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                dynasm!(self ; str D(reg), [X(addr), -(offset as i32)]!);
            }
            _ => codegen_error!("singlepass can't emit STRDB"),
        }
        Ok(())
    }
    fn emit_stria(
        &mut self,
        sz: Size,
        reg: Location,
        addr: GPR,
        offset: u32,
    ) -> Result<(), CompileError> {
        assert!(offset <= 255);
        match (sz, reg) {
            (Size::S64, Location::GPR(reg)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                dynasm!(self ; str X(reg), [X(addr)], (offset as i32));
            }
            (Size::S64, Location::SIMD(reg)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                dynasm!(self ; str D(reg), [X(addr)], (offset as i32));
            }
            _ => codegen_error!("singlepass can't emit STRIA"),
        }
        Ok(())
    }
    fn emit_ldria(
        &mut self,
        sz: Size,
        reg: Location,
        addr: GPR,
        offset: u32,
    ) -> Result<(), CompileError> {
        assert!(offset <= 255);
        match (sz, reg) {
            (Size::S64, Location::GPR(reg)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                dynasm!(self ; ldr X(reg), [X(addr)], offset);
            }
            (Size::S64, Location::SIMD(reg)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                dynasm!(self ; ldr D(reg), [X(addr)], offset);
            }
            _ => codegen_error!("singlepass can't emit LDRIA"),
        }
        Ok(())
    }

    fn emit_stpdb(
        &mut self,
        sz: Size,
        reg1: Location,
        reg2: Location,
        addr: GPR,
        offset: u32,
    ) -> Result<(), CompileError> {
        assert!(offset <= 255);
        match (sz, reg1, reg2) {
            (Size::S64, Location::GPR(reg1), Location::GPR(reg2)) => {
                let reg1 = reg1.into_index() as u32;
                let reg2 = reg2.into_index() as u32;
                let addr = addr.into_index() as u32;
                dynasm!(self ; stp X(reg1), X(reg2), [X(addr), -(offset as i32)]!);
            }
            _ => codegen_error!("singlepass can't emit STPDB"),
        }
        Ok(())
    }
    fn emit_ldpia(
        &mut self,
        sz: Size,
        reg1: Location,
        reg2: Location,
        addr: GPR,
        offset: u32,
    ) -> Result<(), CompileError> {
        assert!(offset <= 255);
        match (sz, reg1, reg2) {
            (Size::S64, Location::GPR(reg1), Location::GPR(reg2)) => {
                let reg1 = reg1.into_index() as u32;
                let reg2 = reg2.into_index() as u32;
                let addr = addr.into_index() as u32;
                dynasm!(self ; ldp X(reg1), X(reg2), [X(addr)], offset);
            }
            _ => codegen_error!("singlepass can't emit LDPIA"),
        }
        Ok(())
    }

    fn emit_ldrb(&mut self, _sz: Size, reg: Location, dst: Location) -> Result<(), CompileError> {
        match (reg, dst) {
            (Location::GPR(reg), Location::Memory(addr, offset)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                let offset = offset as u32;
                assert!(offset < 0x1000);
                dynasm!(self ; ldrb W(reg), [X(addr), offset]);
            }
            (Location::GPR(reg), Location::Memory2(addr, r2, mult, offs)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                let r2 = r2.into_index() as u32;
                assert!(offs == 0);
                let mult = mult as u32;
                match mult {
                    0 => dynasm!(self ; ldrb W(reg), [X(addr)]),
                    1 => dynasm!(self ; ldrb W(reg), [X(addr), X(r2)]),
                    _ => dynasm!(self ; ldrb W(reg), [X(addr), X(r2), LSL mult]),
                };
            }
            _ => codegen_error!("singlepass can't emit LDRB {:?}, {:?}", reg, dst),
        }
        Ok(())
    }
    fn emit_ldrh(&mut self, _sz: Size, reg: Location, dst: Location) -> Result<(), CompileError> {
        match (reg, dst) {
            (Location::GPR(reg), Location::Memory(addr, offset)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                let offset = offset as u32;
                assert!((offset & 1 == 0) && (offset < 0x2000));
                dynasm!(self ; ldrh W(reg), [X(addr), offset]);
            }
            (Location::GPR(reg), Location::Memory2(addr, r2, mult, offs)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                let r2 = r2.into_index() as u32;
                assert!(offs == 0);
                let mult = mult as u32;
                match mult {
                    0 => dynasm!(self ; ldrh W(reg), [X(addr)]),
                    1 => dynasm!(self ; ldrh W(reg), [X(addr), X(r2)]),
                    _ => dynasm!(self ; ldrh W(reg), [X(addr), X(r2), LSL mult]),
                };
            }
            _ => codegen_error!("singlepass can't emit LDRH {:?}, {:?}", reg, dst),
        }
        Ok(())
    }
    fn emit_ldrsb(&mut self, sz: Size, reg: Location, dst: Location) -> Result<(), CompileError> {
        match (sz, reg, dst) {
            (Size::S64, Location::GPR(reg), Location::Memory(addr, offset)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                let offset = offset as u32;
                assert!(offset < 0x1000);
                dynasm!(self ; ldrsb X(reg), [X(addr), offset]);
            }
            (Size::S32, Location::GPR(reg), Location::Memory(addr, offset)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                let offset = offset as u32;
                assert!(offset < 0x1000);
                dynasm!(self ; ldrsb W(reg), [X(addr), offset]);
            }
            (Size::S64, Location::GPR(reg), Location::Memory2(addr, r2, mult, offs)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                let r2 = r2.into_index() as u32;
                assert!(offs == 0);
                let mult = mult as u32;
                match mult {
                    0 => dynasm!(self ; ldrsb X(reg), [X(addr)]),
                    1 => dynasm!(self ; ldrsb X(reg), [X(addr), X(r2)]),
                    _ => dynasm!(self ; ldrsb X(reg), [X(addr), X(r2), LSL mult]),
                };
            }
            (Size::S32, Location::GPR(reg), Location::Memory2(addr, r2, mult, offs)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                let r2 = r2.into_index() as u32;
                assert!(offs == 0);
                let mult = mult as u32;
                match mult {
                    0 => dynasm!(self ; ldrsb W(reg), [X(addr)]),
                    1 => dynasm!(self ; ldrsb W(reg), [X(addr), X(r2)]),
                    _ => dynasm!(self ; ldrsb W(reg), [X(addr), X(r2), LSL mult]),
                };
            }
            _ => codegen_error!("singlepass can't emit LDRSB {:?}, {:?}, {:?}", sz, reg, dst),
        }
        Ok(())
    }
    fn emit_ldrsh(&mut self, sz: Size, reg: Location, dst: Location) -> Result<(), CompileError> {
        match (sz, reg, dst) {
            (Size::S64, Location::GPR(reg), Location::Memory(addr, offset)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                let offset = offset as u32;
                assert!((offset & 1 == 0) && (offset < 0x2000));
                dynasm!(self ; ldrsh X(reg), [X(addr), offset]);
            }
            (Size::S32, Location::GPR(reg), Location::Memory(addr, offset)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                let offset = offset as u32;
                assert!((offset & 1 == 0) && (offset < 0x2000));
                dynasm!(self ; ldrsh W(reg), [X(addr), offset]);
            }
            (Size::S64, Location::GPR(reg), Location::Memory2(addr, r2, mult, offs)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                let r2 = r2.into_index() as u32;
                assert!(offs == 0);
                let mult = mult as u32;
                match mult {
                    0 => dynasm!(self ; ldrsh X(reg), [X(addr)]),
                    1 => dynasm!(self ; ldrsh X(reg), [X(addr), X(r2)]),
                    _ => dynasm!(self ; ldrsh X(reg), [X(addr), X(r2), LSL mult]),
                };
            }
            (Size::S32, Location::GPR(reg), Location::Memory2(addr, r2, mult, offs)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                let r2 = r2.into_index() as u32;
                assert!(offs == 0);
                let mult = mult as u32;
                match mult {
                    0 => dynasm!(self ; ldrsh W(reg), [X(addr)]),
                    1 => dynasm!(self ; ldrsh W(reg), [X(addr), X(r2)]),
                    _ => dynasm!(self ; ldrsh W(reg), [X(addr), X(r2), LSL mult]),
                };
            }
            _ => codegen_error!("singlepass can't emit LDRSH {:?}, {:?}, {:?}", sz, reg, dst),
        }
        Ok(())
    }
    fn emit_ldrsw(&mut self, sz: Size, reg: Location, dst: Location) -> Result<(), CompileError> {
        match (sz, reg, dst) {
            (Size::S64, Location::GPR(reg), Location::Memory(addr, offset)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                let offset = offset as u32;
                assert!((offset & 3 == 0) && (offset < 0x4000));
                dynasm!(self ; ldrsw X(reg), [X(addr), offset]);
            }
            (Size::S64, Location::GPR(reg), Location::Memory2(addr, r2, mult, offs)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                let r2 = r2.into_index() as u32;
                assert!(offs == 0);
                let mult = mult as u32;
                match mult {
                    0 => dynasm!(self ; ldrsw X(reg), [X(addr)]),
                    1 => dynasm!(self ; ldrsw X(reg), [X(addr), X(r2)]),
                    _ => dynasm!(self ; ldrsw X(reg), [X(addr), X(r2), LSL mult]),
                };
            }
            _ => codegen_error!("singlepass can't emit LDRSW {:?}, {:?}, {:?}", sz, reg, dst),
        }
        Ok(())
    }
    fn emit_strb(&mut self, _sz: Size, reg: Location, dst: Location) -> Result<(), CompileError> {
        match (reg, dst) {
            (Location::GPR(reg), Location::Memory(addr, offset)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                let offset = offset as u32;
                assert!(offset < 0x1000);
                dynasm!(self ; strb W(reg), [X(addr), offset]);
            }
            (Location::GPR(reg), Location::Memory2(addr, r2, mult, offs)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                let r2 = r2.into_index() as u32;
                assert!(offs == 0);
                let mult = mult as u32;
                match mult {
                    0 => dynasm!(self ; strb W(reg), [X(addr)]),
                    1 => dynasm!(self ; strb W(reg), [X(addr), X(r2)]),
                    _ => dynasm!(self ; strb W(reg), [X(addr), X(r2), LSL mult]),
                };
            }
            _ => codegen_error!("singlepass can't emit STRB {:?}, {:?}", reg, dst),
        }
        Ok(())
    }
    fn emit_strh(&mut self, _sz: Size, reg: Location, dst: Location) -> Result<(), CompileError> {
        match (reg, dst) {
            (Location::GPR(reg), Location::Memory(addr, offset)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                let offset = offset as u32;
                assert!((offset & 1 == 0) && (offset < 0x2000));
                dynasm!(self ; strh W(reg), [X(addr), offset]);
            }
            (Location::GPR(reg), Location::Memory2(addr, r2, mult, offs)) => {
                let reg = reg.into_index() as u32;
                let addr = addr.into_index() as u32;
                let r2 = r2.into_index() as u32;
                assert!(offs == 0);
                let mult = mult as u32;
                match mult {
                    0 => dynasm!(self ; strh W(reg), [X(addr)]),
                    1 => dynasm!(self ; strh W(reg), [X(addr), X(r2)]),
                    _ => dynasm!(self ; strh W(reg), [X(addr), X(r2), LSL mult]),
                };
            }
            _ => codegen_error!("singlepass can't emit STRH {:?}, {:?}", reg, dst),
        }
        Ok(())
    }

    fn emit_ldaxr(&mut self, sz: Size, reg: Location, dst: Location) -> Result<(), CompileError> {
        match (sz, reg, dst) {
            (Size::S32, Location::GPR(reg), Location::GPR(dst)) => {
                let reg = reg.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; ldaxr W(reg), [X(dst)]);
            }
            (Size::S64, Location::GPR(reg), Location::GPR(dst)) => {
                let reg = reg.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; ldaxr X(reg), [X(dst)]);
            }
            _ => codegen_error!("singlepass can't emit LDAXR {:?}, {:?}", reg, dst),
        }
        Ok(())
    }
    fn emit_ldaxrb(&mut self, _sz: Size, reg: Location, dst: Location) -> Result<(), CompileError> {
        match (reg, dst) {
            (Location::GPR(reg), Location::GPR(dst)) => {
                let reg = reg.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; ldaxrb W(reg), [X(dst)]);
            }
            _ => codegen_error!("singlepass can't emit LDAXRB {:?}, {:?}", reg, dst),
        }
        Ok(())
    }
    fn emit_ldaxrh(&mut self, _sz: Size, reg: Location, dst: Location) -> Result<(), CompileError> {
        match (reg, dst) {
            (Location::GPR(reg), Location::GPR(dst)) => {
                let reg = reg.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; ldaxrh W(reg), [X(dst)]);
            }
            _ => codegen_error!("singlepass can't emit LDAXRH {:?}, {:?}", reg, dst),
        }
        Ok(())
    }
    fn emit_stlxr(
        &mut self,
        sz: Size,
        status: Location,
        reg: Location,
        dst: Location,
    ) -> Result<(), CompileError> {
        match (sz, status, reg, dst) {
            (Size::S32, Location::GPR(status), Location::GPR(reg), Location::GPR(dst)) => {
                let reg = reg.into_index() as u32;
                let dst = dst.into_index() as u32;
                let status = status.into_index() as u32;
                dynasm!(self ; stlxr W(status), W(reg), [X(dst)]);
            }
            (Size::S64, Location::GPR(status), Location::GPR(reg), Location::GPR(dst)) => {
                let reg = reg.into_index() as u32;
                let dst = dst.into_index() as u32;
                let status = status.into_index() as u32;
                dynasm!(self ; stlxr W(status), X(reg), [X(dst)]);
            }
            _ => codegen_error!("singlepass can't emit STLXR {:?}, {:?}", reg, dst),
        }
        Ok(())
    }
    fn emit_stlxrb(
        &mut self,
        _sz: Size,
        status: Location,
        reg: Location,
        dst: Location,
    ) -> Result<(), CompileError> {
        match (status, reg, dst) {
            (Location::GPR(status), Location::GPR(reg), Location::GPR(dst)) => {
                let reg = reg.into_index() as u32;
                let dst = dst.into_index() as u32;
                let status = status.into_index() as u32;
                dynasm!(self ; stlxrb W(status), W(reg), [X(dst)]);
            }
            _ => codegen_error!("singlepass can't emit STLXRB {:?}, {:?}", reg, dst),
        }
        Ok(())
    }
    fn emit_stlxrh(
        &mut self,
        _sz: Size,
        status: Location,
        reg: Location,
        dst: Location,
    ) -> Result<(), CompileError> {
        match (status, reg, dst) {
            (Location::GPR(status), Location::GPR(reg), Location::GPR(dst)) => {
                let reg = reg.into_index() as u32;
                let dst = dst.into_index() as u32;
                let status = status.into_index() as u32;
                dynasm!(self ; stlxrh W(status), W(reg), [X(dst)]);
            }
            _ => codegen_error!("singlepass can't emit STLXRH {:?}, {:?}", reg, dst),
        }
        Ok(())
    }

    fn emit_mov(&mut self, sz: Size, src: Location, dst: Location) -> Result<(), CompileError> {
        match (sz, src, dst) {
            (Size::S64, Location::GPR(src), Location::GPR(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; mov X(dst), X(src));
            }
            (Size::S32, Location::GPR(src), Location::GPR(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; mov W(dst), W(src));
            }
            (Size::S64, Location::SIMD(src), Location::SIMD(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; mov V(dst).D[0], V(src).D[0]);
            }
            (Size::S32, Location::SIMD(src), Location::SIMD(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; mov V(dst).S[0], V(src).S[0]);
            }
            (Size::S64, Location::GPR(src), Location::SIMD(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; mov V(dst).D[0], X(src));
            }
            (Size::S32, Location::GPR(src), Location::SIMD(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; mov V(dst).S[0], W(src));
            }
            (Size::S64, Location::SIMD(src), Location::GPR(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; mov X(dst), V(src).D[0]);
            }
            (Size::S32, Location::SIMD(src), Location::GPR(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; mov W(dst), V(src).S[0]);
            }
            (Size::S32, Location::Imm32(val), Location::GPR(dst)) => {
                let dst = dst.into_index() as u32;
                if val < 0x1000 {
                    dynasm!(self ; mov W(dst), val as u64);
                } else if encode_logical_immediate_32bit(val as _).is_some() {
                    dynasm!(self ; orr W(dst), wzr, val);
                } else {
                    codegen_error!("singlepass can't emit MOV S32 {}, {:?}", val, dst);
                }
            }
            (Size::S64, Location::Imm32(val), Location::GPR(dst)) => {
                let dst = dst.into_index() as u32;
                if val < 0x1000 {
                    dynasm!(self ; mov W(dst), val as u64);
                } else if encode_logical_immediate_64bit(val as _).is_some() {
                    dynasm!(self ; orr X(dst), xzr, val as u64);
                } else {
                    codegen_error!("singlepass can't emit MOV S64 {}, {:?}", val, dst);
                }
            }
            (Size::S64, Location::Imm64(val), Location::GPR(dst)) => {
                let dst = dst.into_index() as u32;
                if val < 0x1000 {
                    dynasm!(self ; mov W(dst), val as u64);
                } else if encode_logical_immediate_64bit(val as _).is_some() {
                    dynasm!(self ; orr X(dst), xzr, val as u64);
                } else {
                    codegen_error!("singleplasse can't emit MOV S64 {}, {:?}", val, dst);
                }
            }
            _ => codegen_error!("singlepass can't emit MOV {:?}, {:?}, {:?}", sz, src, dst),
        }
        Ok(())
    }

    fn emit_movn(&mut self, sz: Size, reg: Location, val: u32) -> Result<(), CompileError> {
        match (sz, reg) {
            (Size::S32, Location::GPR(reg)) => {
                let reg = reg.into_index() as u32;
                dynasm!(self ; movn W(reg), val);
            }
            (Size::S64, Location::GPR(reg)) => {
                let reg = reg.into_index() as u32;
                dynasm!(self ; movn X(reg), val);
            }
            _ => codegen_error!("singlepass can't emit MOVN"),
        }
        Ok(())
    }
    fn emit_movz(&mut self, reg: Location, val: u32) -> Result<(), CompileError> {
        match reg {
            Location::GPR(reg) => {
                let reg = reg.into_index() as u32;
                dynasm!(self ; movz W(reg), val);
            }
            _ => codegen_error!("singlepass can't emit MOVZ"),
        }
        Ok(())
    }
    fn emit_movk(&mut self, reg: Location, val: u32, shift: u32) -> Result<(), CompileError> {
        match reg {
            Location::GPR(reg) => {
                let reg = reg.into_index() as u32;
                dynasm!(self ; movk X(reg), val, LSL shift);
            }
            _ => codegen_error!("singlepass can't emit MOVK"),
        }
        Ok(())
    }

    fn emit_mov_imm(&mut self, dst: Location, val: u64) -> Result<(), CompileError> {
        match dst {
            Location::GPR(dst) => {
                let dst = dst.into_index() as u32;
                let offset = val.trailing_zeros() & 48;
                let masked = 0xffff & (val >> offset);
                if (masked << offset) == val {
                    dynasm!(self ; movz X(dst), masked as u32, LSL offset);
                } else if val >> 16 == 0xffff_ffff_ffff {
                    let val: u16 = !((val & 0xffff) as u16);
                    dynasm!(self ; movn X(dst), val as u32);
                } else if val >> 16 == 0xffff {
                    let val: u16 = !((val & 0xffff) as u16);
                    dynasm!(self ; movn W(dst), val as u32);
                } else {
                    dynasm!(self ; movz W(dst), (val&0xffff) as u32);
                    let val = val >> 16;
                    if val != 0 {
                        dynasm!(self ; movk X(dst), (val&0xffff) as u32, LSL 16);
                        let val = val >> 16;
                        if val != 0 {
                            dynasm!(self ; movk X(dst), (val&0xffff) as u32, LSL 32);
                            let val = val >> 16;
                            if val != 0 {
                                dynasm!(self ; movk X(dst), (val&0xffff) as u32, LSL 48);
                            }
                        }
                    }
                }
            }
            _ => codegen_error!("singlepass can't emit MOVW {:?}", dst),
        }
        Ok(())
    }

    fn emit_add(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError> {
        match (sz, src1, src2, dst) {
            (Size::S64, Location::GPR(src1), Location::GPR(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; add X(dst), X(src1), X(src2), UXTX);
            }
            (Size::S32, Location::GPR(src1), Location::GPR(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; add W(dst), W(src1), W(src2), UXTX);
            }
            (Size::S64, Location::GPR(src1), Location::Imm8(imm), Location::GPR(dst))
            | (Size::S64, Location::Imm8(imm), Location::GPR(src1), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; add X(dst), X(src1), imm as u32);
            }
            (Size::S64, Location::GPR(src1), Location::Imm32(imm), Location::GPR(dst))
            | (Size::S64, Location::Imm32(imm), Location::GPR(src1), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let dst = dst.into_index() as u32;
                if imm >= 0x1000 {
                    unreachable!();
                }
                dynasm!(self ; add X(dst), X(src1), imm);
            }
            (Size::S64, Location::GPR(src1), Location::Imm64(imm), Location::GPR(dst))
            | (Size::S64, Location::Imm64(imm), Location::GPR(src1), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let dst = dst.into_index() as u32;
                if imm >= 0x1000 {
                    unreachable!();
                }
                let imm = imm as u32;
                dynasm!(self ; add X(dst), X(src1), imm);
            }
            (Size::S32, Location::GPR(src1), Location::Imm8(imm), Location::GPR(dst))
            | (Size::S32, Location::Imm8(imm), Location::GPR(src1), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; add W(dst), W(src1), imm as u32);
            }
            (Size::S32, Location::GPR(src1), Location::Imm32(imm), Location::GPR(dst))
            | (Size::S32, Location::Imm32(imm), Location::GPR(src1), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let dst = dst.into_index() as u32;
                if imm >= 0x1000 {
                    unreachable!();
                }
                dynasm!(self ; add W(dst), W(src1), imm);
            }
            _ => codegen_error!(
                "singlepass can't emit ADD {:?} {:?} {:?} {:?}",
                sz,
                src1,
                src2,
                dst
            ),
        }
        Ok(())
    }
    fn emit_sub(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError> {
        match (sz, src1, src2, dst) {
            (Size::S64, Location::GPR(src1), Location::GPR(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; sub X(dst), X(src1), X(src2), UXTX);
            }
            (Size::S32, Location::GPR(src1), Location::GPR(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; sub W(dst), W(src1), W(src2), UXTX);
            }
            (Size::S64, Location::GPR(src1), Location::Imm8(imm), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; sub X(dst), X(src1), imm as u32);
            }
            (Size::S32, Location::GPR(src1), Location::Imm8(imm), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; sub W(dst), W(src1), imm as u32);
            }
            (Size::S32, Location::GPR(src1), Location::Imm32(imm), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let dst = dst.into_index() as u32;
                if imm >= 0x1000 {
                    unreachable!();
                }
                dynasm!(self ; sub W(dst), W(src1), imm);
            }
            (Size::S64, Location::GPR(src1), Location::Imm32(imm), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let dst = dst.into_index() as u32;
                if imm >= 0x1000 {
                    unreachable!();
                }
                dynasm!(self ; sub X(dst), X(src1), imm);
            }
            (Size::S64, Location::GPR(src1), Location::Imm64(imm), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let dst = dst.into_index() as u32;
                if imm >= 0x1000 {
                    unreachable!();
                }
                dynasm!(self ; sub X(dst), X(src1), imm as u32);
            }
            _ => codegen_error!(
                "singlepass can't emit SUB {:?} {:?} {:?} {:?}",
                sz,
                src1,
                src2,
                dst
            ),
        }
        Ok(())
    }
    fn emit_mul(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError> {
        match (sz, src1, src2, dst) {
            (Size::S64, Location::GPR(src1), Location::GPR(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; mul X(dst), X(src1), X(src2));
            }
            (Size::S32, Location::GPR(src1), Location::GPR(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; mul W(dst), W(src1), W(src2));
            }
            _ => codegen_error!(
                "singlepass can't emit MUL {:?} {:?} {:?} {:?}",
                sz,
                src1,
                src2,
                dst
            ),
        }
        Ok(())
    }
    fn emit_adds(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError> {
        match (sz, src1, src2, dst) {
            (Size::S64, Location::GPR(src1), Location::GPR(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; adds X(dst), X(src1), X(src2));
            }
            (Size::S32, Location::GPR(src1), Location::GPR(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; adds W(dst), W(src1), W(src2));
            }
            (Size::S64, Location::GPR(src1), Location::Imm8(imm), Location::GPR(dst))
            | (Size::S64, Location::Imm8(imm), Location::GPR(src1), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; adds X(dst), X(src1), imm as u32);
            }
            (Size::S64, Location::GPR(src1), Location::Imm32(imm), Location::GPR(dst))
            | (Size::S64, Location::Imm32(imm), Location::GPR(src1), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let dst = dst.into_index() as u32;
                if imm >= 0x1000 {
                    codegen_error!("singlepass ADD.S with imm too large {}", imm);
                }
                dynasm!(self ; adds X(dst), X(src1), imm);
            }
            (Size::S32, Location::GPR(src1), Location::Imm8(imm), Location::GPR(dst))
            | (Size::S32, Location::Imm8(imm), Location::GPR(src1), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; adds W(dst), W(src1), imm as u32);
            }
            (Size::S32, Location::GPR(src1), Location::Imm32(imm), Location::GPR(dst))
            | (Size::S32, Location::Imm32(imm), Location::GPR(src1), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let dst = dst.into_index() as u32;
                if imm >= 0x1000 {
                    codegen_error!("singlepass ADD.S with imm too large {}", imm);
                }
                dynasm!(self ; adds W(dst), W(src1), imm);
            }
            _ => codegen_error!(
                "singlepass can't emit ADD.S {:?} {:?} {:?} {:?}",
                sz,
                src1,
                src2,
                dst
            ),
        }
        Ok(())
    }
    fn emit_subs(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError> {
        match (sz, src1, src2, dst) {
            (Size::S64, Location::GPR(src1), Location::GPR(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; subs X(dst), X(src1), X(src2));
            }
            (Size::S32, Location::GPR(src1), Location::GPR(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; subs W(dst), W(src1), W(src2));
            }
            (Size::S64, Location::GPR(src1), Location::Imm8(imm), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; subs X(dst), X(src1), imm as u32);
            }
            (Size::S32, Location::GPR(src1), Location::Imm8(imm), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; subs W(dst), W(src1), imm as u32);
            }
            _ => codegen_error!(
                "singlepass can't emit SUB.S {:?} {:?} {:?} {:?}",
                sz,
                src1,
                src2,
                dst
            ),
        }
        Ok(())
    }
    fn emit_add_lsl(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        lsl: u32,
        dst: Location,
    ) -> Result<(), CompileError> {
        match (sz, src1, src2, dst) {
            (Size::S64, Location::GPR(src1), Location::GPR(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; add X(dst), X(src1), X(src2), LSL lsl);
            }
            _ => codegen_error!(
                "singlepass can't emit LSL {:?} {:?} {:?} {:?} LSL {:?}",
                sz,
                src1,
                src2,
                dst,
                lsl
            ),
        }
        Ok(())
    }

    fn emit_cmp(&mut self, sz: Size, src: Location, dst: Location) -> Result<(), CompileError> {
        match (sz, src, dst) {
            (Size::S64, Location::GPR(src), Location::GPR(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; cmp X(dst), X(src));
            }
            (Size::S32, Location::GPR(src), Location::GPR(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; cmp W(dst), W(src));
            }
            (Size::S64, Location::Imm8(imm), Location::GPR(dst)) => {
                let dst = dst.into_index() as u32;
                dynasm!(self ; cmp X(dst), imm as u32);
            }
            (Size::S64, Location::Imm32(imm), Location::GPR(dst)) => {
                let dst = dst.into_index() as u32;
                if imm >= 0x1000 {
                    codegen_error!("singlepass CMP with imm too large {}", imm);
                }
                dynasm!(self ; cmp X(dst), imm as u32);
            }
            (Size::S64, Location::Imm64(imm), Location::GPR(dst)) => {
                let dst = dst.into_index() as u32;
                if imm >= 0x1000 {
                    codegen_error!("singlepass CMP with imm too large {}", imm);
                }
                dynasm!(self ; cmp X(dst), imm as u32);
            }
            (Size::S32, Location::Imm8(imm), Location::GPR(dst)) => {
                let dst = dst.into_index() as u32;
                dynasm!(self ; cmp W(dst), imm as u32);
            }
            (Size::S32, Location::Imm32(imm), Location::GPR(dst)) => {
                let dst = dst.into_index() as u32;
                if imm >= 0x1000 {
                    codegen_error!("singlepass CMP with imm too large {}", imm);
                }
                dynasm!(self ; cmp W(dst), imm as u32);
            }
            _ => codegen_error!("singlepass can't emit CMP {:?} {:?} {:?}", sz, src, dst),
        }
        Ok(())
    }

    fn emit_tst(&mut self, sz: Size, src: Location, dst: Location) -> Result<(), CompileError> {
        match (sz, src, dst) {
            (Size::S64, Location::GPR(src), Location::GPR(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; tst X(dst), X(src));
            }
            (Size::S64, Location::Imm32(imm), Location::GPR(dst)) => {
                let dst = dst.into_index() as u32;
                if encode_logical_immediate_64bit(imm as u64).is_none() {
                    codegen_error!("singlepass TST with incompatible imm {}", imm);
                }
                dynasm!(self ; tst X(dst), imm as u64);
            }
            (Size::S64, Location::Imm64(imm), Location::GPR(dst)) => {
                let dst = dst.into_index() as u32;
                if encode_logical_immediate_64bit(imm as u64).is_none() {
                    codegen_error!("singlepass TST with incompatible imm {}", imm);
                }
                dynasm!(self ; tst X(dst), imm as u64);
            }
            (Size::S32, Location::GPR(src), Location::GPR(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; tst W(dst), W(src));
            }
            (Size::S32, Location::Imm32(imm), Location::GPR(dst)) => {
                let dst = dst.into_index() as u32;
                if encode_logical_immediate_64bit(imm as u64).is_none() {
                    codegen_error!("singlepass TST with incompatible imm {}", imm);
                }
                dynasm!(self ; tst W(dst), imm);
            }
            _ => codegen_error!("singlepass can't emit TST"),
        }
        Ok(())
    }

    fn emit_lsl(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError> {
        match (sz, src1, src2, dst) {
            (Size::S64, Location::GPR(src1), Location::GPR(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; lsl X(dst), X(src1), X(src2));
            }
            (Size::S64, Location::GPR(src1), Location::Imm32(imm), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                if imm > 63 {
                    codegen_error!("singlepass LSL with incompatible imm {}", imm);
                }
                let imm = imm as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; lsl X(dst), X(src1), imm);
            }
            (Size::S32, Location::GPR(src1), Location::GPR(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; lsl W(dst), W(src1), W(src2));
            }
            (Size::S64, Location::GPR(src1), Location::Imm8(imm), Location::GPR(dst))
            | (Size::S64, Location::Imm8(imm), Location::GPR(src1), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let dst = dst.into_index() as u32;
                if imm > 63 {
                    codegen_error!("singlepass LSL with incompatible imm {}", imm);
                }
                dynasm!(self ; lsl X(dst), X(src1), imm as u32);
            }
            (Size::S64, Location::GPR(src1), Location::Imm64(imm), Location::GPR(dst))
            | (Size::S64, Location::Imm64(imm), Location::GPR(src1), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let dst = dst.into_index() as u32;
                if imm > 63 {
                    codegen_error!("singlepass LSL with incompatible imm {}", imm);
                }
                dynasm!(self ; lsl X(dst), X(src1), imm as u32);
            }
            (Size::S32, Location::GPR(src1), Location::Imm8(imm), Location::GPR(dst))
            | (Size::S32, Location::Imm8(imm), Location::GPR(src1), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let dst = dst.into_index() as u32;
                if imm > 31 {
                    codegen_error!("singlepass LSL with incompatible imm {}", imm);
                }
                dynasm!(self ; lsl W(dst), W(src1), imm as u32);
            }
            (Size::S32, Location::GPR(src1), Location::Imm32(imm), Location::GPR(dst))
            | (Size::S32, Location::Imm32(imm), Location::GPR(src1), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let dst = dst.into_index() as u32;
                if imm > 31 {
                    codegen_error!("singlepass LSL with incompatible imm {}", imm);
                }
                dynasm!(self ; lsl W(dst), W(src1), imm as u32);
            }
            _ => codegen_error!(
                "singlepass can't emit LSL {:?} {:?} {:?} {:?}",
                sz,
                src1,
                src2,
                dst
            ),
        }
        Ok(())
    }
    fn emit_asr(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError> {
        match (sz, src1, src2, dst) {
            (Size::S64, Location::GPR(src1), Location::GPR(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; asr X(dst), X(src1), X(src2));
            }
            (Size::S64, Location::GPR(src1), Location::Imm32(imm), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let imm = imm as u32;
                let dst = dst.into_index() as u32;
                if imm == 0 || imm > 63 {
                    codegen_error!("singlepass ASR with incompatible imm {}", imm);
                }
                dynasm!(self ; asr X(dst), X(src1), imm);
            }
            (Size::S32, Location::GPR(src1), Location::GPR(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; asr W(dst), W(src1), W(src2));
            }
            (Size::S64, Location::GPR(src1), Location::Imm8(imm), Location::GPR(dst))
            | (Size::S64, Location::Imm8(imm), Location::GPR(src1), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let dst = dst.into_index() as u32;
                if imm == 0 || imm > 63 {
                    codegen_error!("singlepass ASR with incompatible imm {}", imm);
                }
                dynasm!(self ; asr X(dst), X(src1), imm as u32);
            }
            (Size::S64, Location::GPR(src1), Location::Imm64(imm), Location::GPR(dst))
            | (Size::S64, Location::Imm64(imm), Location::GPR(src1), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let dst = dst.into_index() as u32;
                if imm == 0 || imm > 63 {
                    codegen_error!("singlepass ASR with incompatible imm {}", imm);
                }
                dynasm!(self ; asr X(dst), X(src1), imm as u32);
            }
            (Size::S32, Location::GPR(src1), Location::Imm8(imm), Location::GPR(dst))
            | (Size::S32, Location::Imm8(imm), Location::GPR(src1), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let dst = dst.into_index() as u32;
                if imm == 0 || imm > 31 {
                    codegen_error!("singlepass ASR with incompatible imm {}", imm);
                }
                dynasm!(self ; asr W(dst), W(src1), imm as u32);
            }
            (Size::S32, Location::GPR(src1), Location::Imm32(imm), Location::GPR(dst))
            | (Size::S32, Location::Imm32(imm), Location::GPR(src1), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let dst = dst.into_index() as u32;
                if imm == 0 || imm > 31 {
                    codegen_error!("singlepass ASR with incompatible imm {}", imm);
                }
                dynasm!(self ; asr W(dst), W(src1), imm as u32);
            }
            _ => codegen_error!(
                "singlepass can't emit ASR {:?} {:?} {:?} {:?}",
                sz,
                src1,
                src2,
                dst
            ),
        }
        Ok(())
    }
    fn emit_lsr(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError> {
        match (sz, src1, src2, dst) {
            (Size::S64, Location::GPR(src1), Location::GPR(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; lsr X(dst), X(src1), X(src2));
            }
            (Size::S64, Location::GPR(src1), Location::Imm32(imm), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let imm = imm as u32;
                let dst = dst.into_index() as u32;
                if imm == 0 || imm > 63 {
                    codegen_error!("singlepass LSR with incompatible imm {}", imm);
                }
                dynasm!(self ; lsr X(dst), X(src1), imm);
            }
            (Size::S32, Location::GPR(src1), Location::GPR(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; lsr W(dst), W(src1), W(src2));
            }
            (Size::S64, Location::GPR(src1), Location::Imm8(imm), Location::GPR(dst))
            | (Size::S64, Location::Imm8(imm), Location::GPR(src1), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let dst = dst.into_index() as u32;
                if imm == 0 || imm > 63 {
                    codegen_error!("singlepass LSR with incompatible imm {}", imm);
                }
                dynasm!(self ; lsr X(dst), X(src1), imm as u32);
            }
            (Size::S64, Location::GPR(src1), Location::Imm64(imm), Location::GPR(dst))
            | (Size::S64, Location::Imm64(imm), Location::GPR(src1), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let dst = dst.into_index() as u32;
                if imm == 0 || imm > 63 {
                    codegen_error!("singlepass LSR with incompatible imm {}", imm);
                }
                dynasm!(self ; lsr X(dst), X(src1), imm as u32);
            }
            (Size::S32, Location::GPR(src1), Location::Imm8(imm), Location::GPR(dst))
            | (Size::S32, Location::Imm8(imm), Location::GPR(src1), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let dst = dst.into_index() as u32;
                if imm == 0 || imm > 31 {
                    codegen_error!("singlepass LSR with incompatible imm {}", imm);
                }
                dynasm!(self ; lsr W(dst), W(src1), imm as u32);
            }
            (Size::S32, Location::GPR(src1), Location::Imm32(imm), Location::GPR(dst))
            | (Size::S32, Location::Imm32(imm), Location::GPR(src1), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let dst = dst.into_index() as u32;
                if imm == 0 || imm > 31 {
                    codegen_error!("singlepass LSR with incompatible imm {}", imm);
                }
                dynasm!(self ; lsr W(dst), W(src1), imm as u32);
            }
            _ => codegen_error!(
                "singlepass can't emit LSR {:?} {:?} {:?} {:?}",
                sz,
                src1,
                src2,
                dst
            ),
        }
        Ok(())
    }
    fn emit_ror(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError> {
        match (sz, src1, src2, dst) {
            (Size::S64, Location::GPR(src1), Location::GPR(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; ror X(dst), X(src1), X(src2));
            }
            (Size::S64, Location::GPR(src1), Location::Imm32(imm), Location::GPR(dst))
            | (Size::S64, Location::Imm32(imm), Location::GPR(src1), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let imm = imm as u32;
                let dst = dst.into_index() as u32;
                if imm == 0 || imm > 63 {
                    codegen_error!("singlepass ROR with incompatible imm {}", imm);
                }
                dynasm!(self ; ror X(dst), X(src1), imm);
            }
            (Size::S64, Location::GPR(src1), Location::Imm64(imm), Location::GPR(dst))
            | (Size::S64, Location::Imm64(imm), Location::GPR(src1), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let imm = imm as u32;
                let dst = dst.into_index() as u32;
                if imm == 0 || imm > 63 {
                    codegen_error!("singlepass ROR with incompatible imm {}", imm);
                }
                dynasm!(self ; ror X(dst), X(src1), imm);
            }
            (Size::S32, Location::GPR(src1), Location::Imm32(imm), Location::GPR(dst))
            | (Size::S32, Location::Imm32(imm), Location::GPR(src1), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let dst = dst.into_index() as u32;
                if imm == 0 || imm > 31 {
                    codegen_error!("singlepass ROR with incompatible imm {}", imm);
                }
                dynasm!(self ; ror W(dst), W(src1), imm as u32);
            }
            (Size::S32, Location::GPR(src1), Location::GPR(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; ror W(dst), W(src1), W(src2));
            }
            (Size::S64, Location::GPR(src1), Location::Imm8(imm), Location::GPR(dst))
            | (Size::S64, Location::Imm8(imm), Location::GPR(src1), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let dst = dst.into_index() as u32;
                if imm == 0 || imm > 63 {
                    codegen_error!("singlepass ROR with incompatible imm {}", imm);
                }
                dynasm!(self ; ror X(dst), X(src1), imm as u32);
            }
            (Size::S32, Location::GPR(src1), Location::Imm8(imm), Location::GPR(dst))
            | (Size::S32, Location::Imm8(imm), Location::GPR(src1), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let dst = dst.into_index() as u32;
                if imm == 0 || imm > 31 {
                    codegen_error!("singlepass ROR with incompatible imm {}", imm);
                }
                dynasm!(self ; ror W(dst), W(src1), imm as u32);
            }
            _ => codegen_error!(
                "singlepass can't emit ROR {:?} {:?} {:?} {:?}",
                sz,
                src1,
                src2,
                dst
            ),
        }
        Ok(())
    }

    fn emit_or(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError> {
        match (sz, src1, src2, dst) {
            (Size::S64, Location::GPR(src1), Location::GPR(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; orr X(dst), X(src1), X(src2));
            }
            (Size::S64, Location::GPR(src1), Location::Imm64(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2 as u64;
                let dst = dst.into_index() as u32;
                if encode_logical_immediate_64bit(src2 as u64).is_none() {
                    codegen_error!("singlepass OR with incompatible imm {}", src2);
                }
                dynasm!(self ; orr X(dst), X(src1), src2);
            }
            (Size::S32, Location::GPR(src1), Location::Imm32(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2 as u32;
                let dst = dst.into_index() as u32;
                if encode_logical_immediate_32bit(src2).is_none() {
                    codegen_error!("singlepass OR with incompatible imm {}", src2);
                }
                dynasm!(self ; orr W(dst), W(src1), src2);
            }
            (Size::S32, Location::GPR(src1), Location::GPR(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; orr W(dst), W(src1), W(src2));
            }
            _ => codegen_error!(
                "singlepass can't emit OR {:?} {:?} {:?} {:?}",
                sz,
                src1,
                src2,
                dst
            ),
        }
        Ok(())
    }
    fn emit_and(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError> {
        match (sz, src1, src2, dst) {
            (Size::S64, Location::GPR(src1), Location::GPR(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; and X(dst), X(src1), X(src2));
            }
            (Size::S64, Location::GPR(src1), Location::Imm64(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2 as u64;
                let dst = dst.into_index() as u32;
                if encode_logical_immediate_64bit(src2 as u64).is_none() {
                    codegen_error!("singlepass AND with incompatible imm {}", src2);
                }
                dynasm!(self ; and X(dst), X(src1), src2);
            }
            (Size::S32, Location::GPR(src1), Location::Imm32(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2 as u32;
                let dst = dst.into_index() as u32;
                if encode_logical_immediate_32bit(src2).is_none() {
                    codegen_error!("singlepass AND with incompatible imm {}", src2);
                }
                dynasm!(self ; and W(dst), W(src1), src2);
            }
            (Size::S32, Location::GPR(src1), Location::GPR(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; and W(dst), W(src1), W(src2));
            }
            _ => codegen_error!(
                "singlepass can't emit AND {:?} {:?} {:?} {:?}",
                sz,
                src1,
                src2,
                dst
            ),
        }
        Ok(())
    }
    fn emit_eor(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError> {
        match (sz, src1, src2, dst) {
            (Size::S64, Location::GPR(src1), Location::GPR(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; eor X(dst), X(src1), X(src2));
            }
            (Size::S64, Location::GPR(src1), Location::Imm64(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2 as u64;
                let dst = dst.into_index() as u32;
                if encode_logical_immediate_64bit(src2 as u64).is_none() {
                    codegen_error!("singlepass EOR with incompatible imm {}", src2);
                }
                dynasm!(self ; eor X(dst), X(src1), src2);
            }
            (Size::S32, Location::GPR(src1), Location::Imm32(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2 as u32;
                let dst = dst.into_index() as u32;
                if encode_logical_immediate_32bit(src2).is_none() {
                    codegen_error!("singlepass EOR with incompatible imm {}", src2);
                }
                dynasm!(self ; eor W(dst), W(src1), src2);
            }
            (Size::S32, Location::GPR(src1), Location::GPR(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; eor W(dst), W(src1), W(src2));
            }
            _ => codegen_error!(
                "singlepass can't emit EOR {:?} {:?} {:?} {:?}",
                sz,
                src1,
                src2,
                dst
            ),
        }
        Ok(())
    }

    fn emit_bfc(
        &mut self,
        sz: Size,
        lsb: u32,
        width: u32,
        dst: Location,
    ) -> Result<(), CompileError> {
        match (sz, dst) {
            (Size::S32, Location::GPR(dst)) => {
                dynasm!(self ; bfc W(dst as u32), lsb, width);
            }
            (Size::S64, Location::GPR(dst)) => {
                dynasm!(self ; bfc X(dst as u32), lsb, width);
            }
            _ => codegen_error!("singlepass can't emit BFC"),
        }
        Ok(())
    }
    fn emit_bfi(
        &mut self,
        sz: Size,
        src: Location,
        lsb: u32,
        width: u32,
        dst: Location,
    ) -> Result<(), CompileError> {
        match (sz, src, dst) {
            (Size::S32, Location::GPR(src), Location::GPR(dst)) => {
                dynasm!(self ; bfi W(dst as u32), W(src as u32), lsb, width);
            }
            (Size::S64, Location::GPR(src), Location::GPR(dst)) => {
                dynasm!(self ; bfi X(dst as u32), X(src as u32), lsb, width);
            }
            _ => codegen_error!("singlepass can't emit BFI"),
        }
        Ok(())
    }

    fn emit_udiv(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError> {
        match (sz, src1, src2, dst) {
            (Size::S32, Location::GPR(src1), Location::GPR(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; udiv W(dst), W(src1), W(src2));
            }
            (Size::S64, Location::GPR(src1), Location::GPR(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; udiv X(dst), X(src1), X(src2));
            }
            _ => codegen_error!(
                "singlepass can't emit UDIV {:?} {:?} {:?} {:?}",
                sz,
                src1,
                src2,
                dst
            ),
        }
        Ok(())
    }
    fn emit_sdiv(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError> {
        match (sz, src1, src2, dst) {
            (Size::S32, Location::GPR(src1), Location::GPR(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; sdiv W(dst), W(src1), W(src2));
            }
            (Size::S64, Location::GPR(src1), Location::GPR(src2), Location::GPR(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; sdiv X(dst), X(src1), X(src2));
            }
            _ => codegen_error!(
                "singlepass can't emit UDIV {:?} {:?} {:?} {:?}",
                sz,
                src1,
                src2,
                dst
            ),
        }
        Ok(())
    }

    /// msub : c - a*b -> dst
    fn emit_msub(
        &mut self,
        sz: Size,
        a: Location,
        b: Location,
        c: Location,
        dst: Location,
    ) -> Result<(), CompileError> {
        match (sz, a, b, c, dst) {
            (
                Size::S32,
                Location::GPR(a),
                Location::GPR(b),
                Location::GPR(c),
                Location::GPR(dst),
            ) => {
                let a = a.into_index() as u32;
                let b = b.into_index() as u32;
                let c = c.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; msub W(dst), W(a), W(b), W(c));
            }
            (
                Size::S64,
                Location::GPR(a),
                Location::GPR(b),
                Location::GPR(c),
                Location::GPR(dst),
            ) => {
                let a = a.into_index() as u32;
                let b = b.into_index() as u32;
                let c = c.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; msub X(dst), X(a), X(b), X(c));
            }
            _ => codegen_error!(
                "singlepass can't emit msub {:?} {:?} {:?} {:?} {:?}",
                sz,
                a,
                b,
                c,
                dst
            ),
        }
        Ok(())
    }

    fn emit_sxtb(&mut self, sz: Size, src: Location, dst: Location) -> Result<(), CompileError> {
        match (sz, src, dst) {
            (Size::S32, Location::GPR(src), Location::GPR(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; sxtb W(dst), W(src));
            }
            (Size::S64, Location::GPR(src), Location::GPR(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; sxtb X(dst), W(src));
            }
            _ => codegen_error!("singlepass can't emit SXTB {:?} {:?} {:?}", sz, src, dst),
        }
        Ok(())
    }
    fn emit_sxth(&mut self, sz: Size, src: Location, dst: Location) -> Result<(), CompileError> {
        match (sz, src, dst) {
            (Size::S32, Location::GPR(src), Location::GPR(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; sxth W(dst), W(src));
            }
            (Size::S64, Location::GPR(src), Location::GPR(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; sxth X(dst), W(src));
            }
            _ => codegen_error!("singlepass can't emit SXTH {:?} {:?} {:?}", sz, src, dst),
        }
        Ok(())
    }
    fn emit_sxtw(&mut self, _sz: Size, src: Location, dst: Location) -> Result<(), CompileError> {
        match (src, dst) {
            (Location::GPR(src), Location::GPR(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; sxtw X(dst), W(src));
            }
            _ => codegen_error!("singlepass can't emit SXTW {:?} {:?}", src, dst),
        }
        Ok(())
    }
    fn emit_uxtb(&mut self, _sz: Size, src: Location, dst: Location) -> Result<(), CompileError> {
        match (src, dst) {
            (Location::GPR(src), Location::GPR(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; uxtb W(dst), W(src));
            }
            _ => codegen_error!("singlepass can't emit UXTB {:?} {:?}", src, dst),
        }
        Ok(())
    }
    fn emit_uxth(&mut self, _sz: Size, src: Location, dst: Location) -> Result<(), CompileError> {
        match (src, dst) {
            (Location::GPR(src), Location::GPR(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; uxth W(dst), W(src));
            }
            _ => codegen_error!("singlepass can't emit UXTH {:?} {:?}", src, dst),
        }
        Ok(())
    }

    fn emit_cset(&mut self, sz: Size, dst: Location, cond: Condition) -> Result<(), CompileError> {
        match (sz, dst) {
            (Size::S32, Location::GPR(reg)) => {
                let reg = reg as u32;
                match cond {
                    Condition::Eq => dynasm!(self ; cset W(reg), eq),
                    Condition::Ne => dynasm!(self ; cset W(reg), ne),
                    Condition::Cs => dynasm!(self ; cset W(reg), cs),
                    Condition::Cc => dynasm!(self ; cset W(reg), cc),
                    Condition::Mi => dynasm!(self ; cset W(reg), mi),
                    Condition::Pl => dynasm!(self ; cset W(reg), pl),
                    Condition::Vs => dynasm!(self ; cset W(reg), vs),
                    Condition::Vc => dynasm!(self ; cset W(reg), vc),
                    Condition::Hi => dynasm!(self ; cset W(reg), hi),
                    Condition::Ls => dynasm!(self ; cset W(reg), ls),
                    Condition::Ge => dynasm!(self ; cset W(reg), ge),
                    Condition::Lt => dynasm!(self ; cset W(reg), lt),
                    Condition::Gt => dynasm!(self ; cset W(reg), gt),
                    Condition::Le => dynasm!(self ; cset W(reg), le),
                    Condition::Al => dynasm!(self ; cset W(reg), al),
                }
            }
            (Size::S64, Location::GPR(reg)) => {
                let reg = reg as u32;
                match cond {
                    Condition::Eq => dynasm!(self ; cset X(reg), eq),
                    Condition::Ne => dynasm!(self ; cset X(reg), ne),
                    Condition::Cs => dynasm!(self ; cset X(reg), cs),
                    Condition::Cc => dynasm!(self ; cset X(reg), cc),
                    Condition::Mi => dynasm!(self ; cset X(reg), mi),
                    Condition::Pl => dynasm!(self ; cset X(reg), pl),
                    Condition::Vs => dynasm!(self ; cset X(reg), vs),
                    Condition::Vc => dynasm!(self ; cset X(reg), vc),
                    Condition::Hi => dynasm!(self ; cset X(reg), hi),
                    Condition::Ls => dynasm!(self ; cset X(reg), ls),
                    Condition::Ge => dynasm!(self ; cset X(reg), ge),
                    Condition::Lt => dynasm!(self ; cset X(reg), lt),
                    Condition::Gt => dynasm!(self ; cset X(reg), gt),
                    Condition::Le => dynasm!(self ; cset X(reg), le),
                    Condition::Al => dynasm!(self ; cset X(reg), al),
                }
            }
            _ => codegen_error!("singlepass can't emit CSET {:?} {:?} {:?}", sz, dst, cond),
        }
        Ok(())
    }
    fn emit_csetm(&mut self, sz: Size, dst: Location, cond: Condition) -> Result<(), CompileError> {
        match (sz, dst) {
            (Size::S32, Location::GPR(reg)) => {
                let reg = reg as u32;
                match cond {
                    Condition::Eq => dynasm!(self ; csetm W(reg), eq),
                    Condition::Ne => dynasm!(self ; csetm W(reg), ne),
                    Condition::Cs => dynasm!(self ; csetm W(reg), cs),
                    Condition::Cc => dynasm!(self ; csetm W(reg), cc),
                    Condition::Mi => dynasm!(self ; csetm W(reg), mi),
                    Condition::Pl => dynasm!(self ; csetm W(reg), pl),
                    Condition::Vs => dynasm!(self ; csetm W(reg), vs),
                    Condition::Vc => dynasm!(self ; csetm W(reg), vc),
                    Condition::Hi => dynasm!(self ; csetm W(reg), hi),
                    Condition::Ls => dynasm!(self ; csetm W(reg), ls),
                    Condition::Ge => dynasm!(self ; csetm W(reg), ge),
                    Condition::Lt => dynasm!(self ; csetm W(reg), lt),
                    Condition::Gt => dynasm!(self ; csetm W(reg), gt),
                    Condition::Le => dynasm!(self ; csetm W(reg), le),
                    Condition::Al => dynasm!(self ; csetm W(reg), al),
                }
            }
            (Size::S64, Location::GPR(reg)) => {
                let reg = reg as u32;
                match cond {
                    Condition::Eq => dynasm!(self ; csetm X(reg), eq),
                    Condition::Ne => dynasm!(self ; csetm X(reg), ne),
                    Condition::Cs => dynasm!(self ; csetm X(reg), cs),
                    Condition::Cc => dynasm!(self ; csetm X(reg), cc),
                    Condition::Mi => dynasm!(self ; csetm X(reg), mi),
                    Condition::Pl => dynasm!(self ; csetm X(reg), pl),
                    Condition::Vs => dynasm!(self ; csetm X(reg), vs),
                    Condition::Vc => dynasm!(self ; csetm X(reg), vc),
                    Condition::Hi => dynasm!(self ; csetm X(reg), hi),
                    Condition::Ls => dynasm!(self ; csetm X(reg), ls),
                    Condition::Ge => dynasm!(self ; csetm X(reg), ge),
                    Condition::Lt => dynasm!(self ; csetm X(reg), lt),
                    Condition::Gt => dynasm!(self ; csetm X(reg), gt),
                    Condition::Le => dynasm!(self ; csetm X(reg), le),
                    Condition::Al => dynasm!(self ; csetm X(reg), al),
                }
            }
            _ => codegen_error!("singlepass can't emit CSETM {:?} {:?} {:?}", sz, dst, cond),
        }
        Ok(())
    }
    fn emit_cinc(
        &mut self,
        sz: Size,
        src: Location,
        dst: Location,
        cond: Condition,
    ) -> Result<(), CompileError> {
        match (sz, src, dst) {
            (Size::S32, Location::GPR(src), Location::GPR(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                match cond {
                    Condition::Eq => dynasm!(self ; cinc W(dst), W(src), eq),
                    Condition::Ne => dynasm!(self ; cinc W(dst), W(src), ne),
                    Condition::Cs => dynasm!(self ; cinc W(dst), W(src), cs),
                    Condition::Cc => dynasm!(self ; cinc W(dst), W(src), cc),
                    Condition::Mi => dynasm!(self ; cinc W(dst), W(src), mi),
                    Condition::Pl => dynasm!(self ; cinc W(dst), W(src), pl),
                    Condition::Vs => dynasm!(self ; cinc W(dst), W(src), vs),
                    Condition::Vc => dynasm!(self ; cinc W(dst), W(src), vc),
                    Condition::Hi => dynasm!(self ; cinc W(dst), W(src), hi),
                    Condition::Ls => dynasm!(self ; cinc W(dst), W(src), ls),
                    Condition::Ge => dynasm!(self ; cinc W(dst), W(src), ge),
                    Condition::Lt => dynasm!(self ; cinc W(dst), W(src), lt),
                    Condition::Gt => dynasm!(self ; cinc W(dst), W(src), gt),
                    Condition::Le => dynasm!(self ; cinc W(dst), W(src), le),
                    Condition::Al => dynasm!(self ; cinc W(dst), W(src), al),
                };
            }
            (Size::S64, Location::GPR(src), Location::GPR(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                match cond {
                    Condition::Eq => dynasm!(self ; cinc X(src), X(dst), eq),
                    Condition::Ne => dynasm!(self ; cinc X(src), X(dst), ne),
                    Condition::Cs => dynasm!(self ; cinc X(src), X(dst), cs),
                    Condition::Cc => dynasm!(self ; cinc X(src), X(dst), cc),
                    Condition::Mi => dynasm!(self ; cinc X(src), X(dst), mi),
                    Condition::Pl => dynasm!(self ; cinc X(src), X(dst), pl),
                    Condition::Vs => dynasm!(self ; cinc X(src), X(dst), vs),
                    Condition::Vc => dynasm!(self ; cinc X(src), X(dst), vc),
                    Condition::Hi => dynasm!(self ; cinc X(src), X(dst), hi),
                    Condition::Ls => dynasm!(self ; cinc X(src), X(dst), ls),
                    Condition::Ge => dynasm!(self ; cinc X(src), X(dst), ge),
                    Condition::Lt => dynasm!(self ; cinc X(src), X(dst), lt),
                    Condition::Gt => dynasm!(self ; cinc X(src), X(dst), gt),
                    Condition::Le => dynasm!(self ; cinc X(src), X(dst), le),
                    Condition::Al => dynasm!(self ; cinc X(src), X(dst), al),
                };
            }
            _ => codegen_error!("singlepass can't emit CINC"),
        }
        Ok(())
    }

    fn emit_clz(&mut self, sz: Size, src: Location, dst: Location) -> Result<(), CompileError> {
        match (sz, src, dst) {
            (Size::S64, Location::GPR(src), Location::GPR(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; clz X(dst), X(src));
            }
            (Size::S32, Location::GPR(src), Location::GPR(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; clz W(dst), W(src));
            }
            _ => codegen_error!("singlepass can't emit CLS {:?} {:?} {:?}", sz, src, dst),
        }
        Ok(())
    }
    fn emit_rbit(&mut self, sz: Size, src: Location, dst: Location) -> Result<(), CompileError> {
        match (sz, src, dst) {
            (Size::S64, Location::GPR(src), Location::GPR(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; rbit X(dst), X(src));
            }
            (Size::S32, Location::GPR(src), Location::GPR(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; rbit W(dst), W(src));
            }
            _ => codegen_error!("singlepass can't emit CLS {:?} {:?} {:?}", sz, src, dst),
        }
        Ok(())
    }

    fn emit_label(&mut self, label: Label) -> Result<(), CompileError> {
        dynasm!(self ; => label);
        Ok(())
    }
    fn emit_load_label(&mut self, reg: GPR, label: Label) -> Result<(), CompileError> {
        let reg = reg.into_index() as u32;
        dynasm!(self ; adr X(reg), =>label);
        Ok(())
    }

    fn emit_b_label(&mut self, label: Label) -> Result<(), CompileError> {
        dynasm!(self ; b =>label);
        Ok(())
    }
    fn emit_cbz_label(
        &mut self,
        sz: Size,
        reg: Location,
        label: Label,
    ) -> Result<(), CompileError> {
        match (sz, reg) {
            (Size::S32, Location::GPR(reg)) => {
                let reg = reg.into_index() as u32;
                dynasm!(self ; cbz W(reg), =>label);
            }
            (Size::S64, Location::GPR(reg)) => {
                let reg = reg.into_index() as u32;
                dynasm!(self ; cbz X(reg), =>label);
            }
            _ => codegen_error!("singlepass can't emit CBZ {:?} {:?} {:?}", sz, reg, label),
        }
        Ok(())
    }
    fn emit_cbnz_label(
        &mut self,
        sz: Size,
        reg: Location,
        label: Label,
    ) -> Result<(), CompileError> {
        match (sz, reg) {
            (Size::S32, Location::GPR(reg)) => {
                let reg = reg.into_index() as u32;
                dynasm!(self ; cbnz W(reg), =>label);
            }
            (Size::S64, Location::GPR(reg)) => {
                let reg = reg.into_index() as u32;
                dynasm!(self ; cbnz X(reg), =>label);
            }
            _ => codegen_error!("singlepass can't emit CBNZ {:?} {:?} {:?}", sz, reg, label),
        }
        Ok(())
    }
    fn emit_tbz_label(
        &mut self,
        sz: Size,
        reg: Location,
        n: u32,
        label: Label,
    ) -> Result<(), CompileError> {
        match (sz, reg) {
            (Size::S32, Location::GPR(reg)) => {
                let reg = reg.into_index() as u32;
                dynasm!(self ; tbz W(reg), n, =>label);
            }
            (Size::S64, Location::GPR(reg)) => {
                let reg = reg.into_index() as u32;
                dynasm!(self ; tbz X(reg), n, =>label);
            }
            _ => codegen_error!(
                "singlepass can't emit TBZ {:?} {:?} {:?} {:?}",
                sz,
                reg,
                n,
                label
            ),
        }
        Ok(())
    }
    fn emit_tbnz_label(
        &mut self,
        sz: Size,
        reg: Location,
        n: u32,
        label: Label,
    ) -> Result<(), CompileError> {
        match (sz, reg) {
            (Size::S32, Location::GPR(reg)) => {
                let reg = reg.into_index() as u32;
                dynasm!(self ; tbnz W(reg), n, =>label);
            }
            (Size::S64, Location::GPR(reg)) => {
                let reg = reg.into_index() as u32;
                dynasm!(self ; tbnz X(reg), n, =>label);
            }
            _ => codegen_error!(
                "singlepass can't emit TBNZ {:?} {:?} {:?} {:?}",
                sz,
                reg,
                n,
                label
            ),
        }
        Ok(())
    }
    fn emit_bcond_label(&mut self, condition: Condition, label: Label) -> Result<(), CompileError> {
        match condition {
            Condition::Eq => dynasm!(self ; b.eq => label),
            Condition::Ne => dynasm!(self ; b.ne => label),
            Condition::Cs => dynasm!(self ; b.cs => label),
            Condition::Cc => dynasm!(self ; b.cc => label),
            Condition::Mi => dynasm!(self ; b.mi => label),
            Condition::Pl => dynasm!(self ; b.pl => label),
            Condition::Vs => dynasm!(self ; b.vs => label),
            Condition::Vc => dynasm!(self ; b.vc => label),
            Condition::Hi => dynasm!(self ; b.hi => label),
            Condition::Ls => dynasm!(self ; b.ls => label),
            Condition::Ge => dynasm!(self ; b.ge => label),
            Condition::Lt => dynasm!(self ; b.lt => label),
            Condition::Gt => dynasm!(self ; b.gt => label),
            Condition::Le => dynasm!(self ; b.le => label),
            Condition::Al => dynasm!(self ; b => label),
        }
        Ok(())
    }
    fn emit_bcond_label_far(
        &mut self,
        condition: Condition,
        label: Label,
    ) -> Result<(), CompileError> {
        let cont: Label = self.get_label();
        match condition {
            // if not condition than continue
            Condition::Eq => dynasm!(self ; b.ne => cont),
            Condition::Ne => dynasm!(self ; b.eq => cont),
            Condition::Cs => dynasm!(self ; b.cc => cont),
            Condition::Cc => dynasm!(self ; b.cs => cont),
            Condition::Mi => dynasm!(self ; b.pl => cont),
            Condition::Pl => dynasm!(self ; b.mi => cont),
            Condition::Vs => dynasm!(self ; b.vc => cont),
            Condition::Vc => dynasm!(self ; b.vs => cont),
            Condition::Hi => dynasm!(self ; b.ls => cont),
            Condition::Ls => dynasm!(self ; b.hi => cont),
            Condition::Ge => dynasm!(self ; b.lt => cont),
            Condition::Lt => dynasm!(self ; b.ge => cont),
            Condition::Gt => dynasm!(self ; b.le => cont),
            Condition::Le => dynasm!(self ; b.gt => cont),
            Condition::Al => { /*nothing*/ }
        }
        dynasm!(self ; b => label);
        self.emit_label(cont)?;
        Ok(())
    }
    fn emit_b_register(&mut self, reg: GPR) -> Result<(), CompileError> {
        dynasm!(self ; br X(reg.into_index() as u32));
        Ok(())
    }
    fn emit_call_label(&mut self, label: Label) -> Result<(), CompileError> {
        dynasm!(self ; bl =>label);
        Ok(())
    }
    fn emit_call_register(&mut self, reg: GPR) -> Result<(), CompileError> {
        dynasm!(self ; blr X(reg.into_index() as u32));
        Ok(())
    }
    fn emit_ret(&mut self) -> Result<(), CompileError> {
        dynasm!(self ; ret);
        Ok(())
    }

    fn emit_udf(&mut self, payload: u16) -> Result<(), CompileError> {
        dynasm!(self ; udf (payload as u32));
        Ok(())
    }
    fn emit_dmb(&mut self) -> Result<(), CompileError> {
        dynasm!(self ; dmb ish);
        Ok(())
    }
    fn emit_brk(&mut self) -> Result<(), CompileError> {
        dynasm!(self ; brk 0);
        Ok(())
    }

    fn emit_fcmp(&mut self, sz: Size, src1: Location, src2: Location) -> Result<(), CompileError> {
        match (sz, src1, src2) {
            (Size::S32, Location::SIMD(src1), Location::SIMD(src2)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                dynasm!(self ; fcmp S(src1), S(src2));
            }
            (Size::S64, Location::SIMD(src1), Location::SIMD(src2)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                dynasm!(self ; fcmp D(src1), D(src2));
            }
            _ => codegen_error!("singlepass can't emit FCMP {:?} {:?} {:?}", sz, src1, src2),
        }
        Ok(())
    }

    fn emit_fneg(&mut self, sz: Size, src: Location, dst: Location) -> Result<(), CompileError> {
        match (sz, src, dst) {
            (Size::S32, Location::SIMD(src), Location::SIMD(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; fneg S(dst), S(src));
            }
            (Size::S64, Location::SIMD(src), Location::SIMD(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; fneg D(dst), D(src));
            }
            _ => codegen_error!("singlepass can't emit FNEG {:?} {:?} {:?}", sz, src, dst),
        }
        Ok(())
    }
    fn emit_fsqrt(&mut self, sz: Size, src: Location, dst: Location) -> Result<(), CompileError> {
        match (sz, src, dst) {
            (Size::S32, Location::SIMD(src), Location::SIMD(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; fsqrt S(dst), S(src));
            }
            (Size::S64, Location::SIMD(src), Location::SIMD(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; fsqrt D(dst), D(src));
            }
            _ => codegen_error!("singlepass can't emit FSQRT {:?} {:?} {:?}", sz, src, dst),
        }
        Ok(())
    }

    fn emit_fadd(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError> {
        match (sz, src1, src2, dst) {
            (Size::S32, Location::SIMD(src1), Location::SIMD(src2), Location::SIMD(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; fadd S(dst), S(src1), S(src2));
            }
            (Size::S64, Location::SIMD(src1), Location::SIMD(src2), Location::SIMD(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; fadd D(dst), D(src1), D(src2));
            }
            _ => codegen_error!(
                "singlepass can't emit FADD {:?} {:?} {:?} {:?}",
                sz,
                src1,
                src2,
                dst
            ),
        }
        Ok(())
    }
    fn emit_fsub(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError> {
        match (sz, src1, src2, dst) {
            (Size::S32, Location::SIMD(src1), Location::SIMD(src2), Location::SIMD(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; fsub S(dst), S(src1), S(src2));
            }
            (Size::S64, Location::SIMD(src1), Location::SIMD(src2), Location::SIMD(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; fsub D(dst), D(src1), D(src2));
            }
            _ => codegen_error!(
                "singlepass can't emit FSUB {:?} {:?} {:?} {:?}",
                sz,
                src1,
                src2,
                dst
            ),
        }
        Ok(())
    }
    fn emit_fmul(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError> {
        match (sz, src1, src2, dst) {
            (Size::S32, Location::SIMD(src1), Location::SIMD(src2), Location::SIMD(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; fmul S(dst), S(src1), S(src2));
            }
            (Size::S64, Location::SIMD(src1), Location::SIMD(src2), Location::SIMD(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; fmul D(dst), D(src1), D(src2));
            }
            _ => codegen_error!(
                "singlepass can't emit FMUL {:?} {:?} {:?} {:?}",
                sz,
                src1,
                src2,
                dst
            ),
        }
        Ok(())
    }
    fn emit_fdiv(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError> {
        match (sz, src1, src2, dst) {
            (Size::S32, Location::SIMD(src1), Location::SIMD(src2), Location::SIMD(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; fdiv S(dst), S(src1), S(src2));
            }
            (Size::S64, Location::SIMD(src1), Location::SIMD(src2), Location::SIMD(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; fdiv D(dst), D(src1), D(src2));
            }
            _ => codegen_error!(
                "singlepass can't emit FDIV {:?} {:?} {:?} {:?}",
                sz,
                src1,
                src2,
                dst
            ),
        }
        Ok(())
    }

    fn emit_fmin(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError> {
        match (sz, src1, src2, dst) {
            (Size::S32, Location::SIMD(src1), Location::SIMD(src2), Location::SIMD(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; fmin S(dst), S(src1), S(src2));
            }
            (Size::S64, Location::SIMD(src1), Location::SIMD(src2), Location::SIMD(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; fmin D(dst), D(src1), D(src2));
            }
            _ => codegen_error!(
                "singlepass can't emit FMIN {:?} {:?} {:?} {:?}",
                sz,
                src1,
                src2,
                dst
            ),
        }
        Ok(())
    }
    fn emit_fmax(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
    ) -> Result<(), CompileError> {
        match (sz, src1, src2, dst) {
            (Size::S32, Location::SIMD(src1), Location::SIMD(src2), Location::SIMD(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; fmax S(dst), S(src1), S(src2));
            }
            (Size::S64, Location::SIMD(src1), Location::SIMD(src2), Location::SIMD(dst)) => {
                let src1 = src1.into_index() as u32;
                let src2 = src2.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; fmax D(dst), D(src1), D(src2));
            }
            _ => codegen_error!(
                "singlepass can't emit FMAX {:?} {:?} {:?} {:?}",
                sz,
                src1,
                src2,
                dst
            ),
        }
        Ok(())
    }

    fn emit_frintz(&mut self, sz: Size, src: Location, dst: Location) -> Result<(), CompileError> {
        match (sz, src, dst) {
            (Size::S32, Location::SIMD(src), Location::SIMD(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; frintz S(dst), S(src));
            }
            (Size::S64, Location::SIMD(src), Location::SIMD(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; frintz D(dst), D(src));
            }
            _ => codegen_error!("singlepass can't emit FRINTZ {:?} {:?} {:?}", sz, src, dst),
        }
        Ok(())
    }
    fn emit_frintn(&mut self, sz: Size, src: Location, dst: Location) -> Result<(), CompileError> {
        match (sz, src, dst) {
            (Size::S32, Location::SIMD(src), Location::SIMD(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; frintn S(dst), S(src));
            }
            (Size::S64, Location::SIMD(src), Location::SIMD(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; frintn D(dst), D(src));
            }
            _ => codegen_error!("singlepass can't emit FRINTN {:?} {:?} {:?}", sz, src, dst),
        }
        Ok(())
    }
    fn emit_frintm(&mut self, sz: Size, src: Location, dst: Location) -> Result<(), CompileError> {
        match (sz, src, dst) {
            (Size::S32, Location::SIMD(src), Location::SIMD(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; frintm S(dst), S(src));
            }
            (Size::S64, Location::SIMD(src), Location::SIMD(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; frintm D(dst), D(src));
            }
            _ => codegen_error!("singlepass can't emit FRINTM {:?} {:?} {:?}", sz, src, dst),
        }
        Ok(())
    }
    fn emit_frintp(&mut self, sz: Size, src: Location, dst: Location) -> Result<(), CompileError> {
        match (sz, src, dst) {
            (Size::S32, Location::SIMD(src), Location::SIMD(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; frintp S(dst), S(src));
            }
            (Size::S64, Location::SIMD(src), Location::SIMD(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; frintp D(dst), D(src));
            }
            _ => codegen_error!("singlepass can't emit FRINTP {:?} {:?} {:?}", sz, src, dst),
        }
        Ok(())
    }

    fn emit_scvtf(
        &mut self,
        sz_in: Size,
        src: Location,
        sz_out: Size,
        dst: Location,
    ) -> Result<(), CompileError> {
        match (sz_in, src, sz_out, dst) {
            (Size::S32, Location::GPR(src), Size::S32, Location::SIMD(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; scvtf S(dst), W(src));
            }
            (Size::S64, Location::GPR(src), Size::S32, Location::SIMD(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; scvtf S(dst), X(src));
            }
            (Size::S32, Location::GPR(src), Size::S64, Location::SIMD(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; scvtf D(dst), W(src));
            }
            (Size::S64, Location::GPR(src), Size::S64, Location::SIMD(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; scvtf D(dst), X(src));
            }
            _ => codegen_error!(
                "singlepass can't emit SCVTF {:?} {:?} {:?} {:?}",
                sz_in,
                src,
                sz_out,
                dst
            ),
        }
        Ok(())
    }
    fn emit_ucvtf(
        &mut self,
        sz_in: Size,
        src: Location,
        sz_out: Size,
        dst: Location,
    ) -> Result<(), CompileError> {
        match (sz_in, src, sz_out, dst) {
            (Size::S32, Location::GPR(src), Size::S32, Location::SIMD(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; ucvtf S(dst), W(src));
            }
            (Size::S64, Location::GPR(src), Size::S32, Location::SIMD(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; ucvtf S(dst), X(src));
            }
            (Size::S32, Location::GPR(src), Size::S64, Location::SIMD(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; ucvtf D(dst), W(src));
            }
            (Size::S64, Location::GPR(src), Size::S64, Location::SIMD(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; ucvtf D(dst), X(src));
            }
            _ => codegen_error!(
                "singlepass can't emit UCVTF {:?} {:?} {:?} {:?}",
                sz_in,
                src,
                sz_out,
                dst
            ),
        }
        Ok(())
    }
    fn emit_fcvt(&mut self, sz_in: Size, src: Location, dst: Location) -> Result<(), CompileError> {
        match (sz_in, src, dst) {
            (Size::S32, Location::SIMD(src), Location::SIMD(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; fcvt D(dst), S(src));
            }
            (Size::S64, Location::SIMD(src), Location::SIMD(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; fcvt S(dst), D(src));
            }
            _ => codegen_error!(
                "singlepass can't emit UCVTF {:?} {:?} {:?}",
                sz_in,
                src,
                dst
            ),
        }
        Ok(())
    }
    fn emit_fcvtzs(
        &mut self,
        sz_in: Size,
        src: Location,
        sz_out: Size,
        dst: Location,
    ) -> Result<(), CompileError> {
        match (sz_in, src, sz_out, dst) {
            (Size::S32, Location::SIMD(src), Size::S32, Location::GPR(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; fcvtzs W(dst), S(src));
            }
            (Size::S64, Location::SIMD(src), Size::S32, Location::GPR(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; fcvtzs W(dst), D(src));
            }
            (Size::S32, Location::SIMD(src), Size::S64, Location::GPR(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; fcvtzs X(dst), S(src));
            }
            (Size::S64, Location::SIMD(src), Size::S64, Location::GPR(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; fcvtzs X(dst), D(src));
            }
            _ => codegen_error!(
                "singlepass can't emit FCVTZS {:?} {:?} {:?} {:?}",
                sz_in,
                src,
                sz_out,
                dst
            ),
        }
        Ok(())
    }
    fn emit_fcvtzu(
        &mut self,
        sz_in: Size,
        src: Location,
        sz_out: Size,
        dst: Location,
    ) -> Result<(), CompileError> {
        match (sz_in, src, sz_out, dst) {
            (Size::S32, Location::SIMD(src), Size::S32, Location::GPR(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; fcvtzu W(dst), S(src));
            }
            (Size::S64, Location::SIMD(src), Size::S32, Location::GPR(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; fcvtzu W(dst), D(src));
            }
            (Size::S32, Location::SIMD(src), Size::S64, Location::GPR(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; fcvtzu X(dst), S(src));
            }
            (Size::S64, Location::SIMD(src), Size::S64, Location::GPR(dst)) => {
                let src = src.into_index() as u32;
                let dst = dst.into_index() as u32;
                dynasm!(self ; fcvtzu X(dst), D(src));
            }
            _ => codegen_error!(
                "singlepass can't emit FCVTZU {:?} {:?} {:?} {:?}",
                sz_in,
                src,
                sz_out,
                dst
            ),
        }
        Ok(())
    }

    // 1 011 0100 0100 000 => fpcr
    fn emit_read_fpcr(&mut self, reg: GPR) -> Result<(), CompileError> {
        dynasm!(self ; mrs X(reg as u32), 0b1_011_0100_0100_000);
        Ok(())
    }
    fn emit_write_fpcr(&mut self, reg: GPR) -> Result<(), CompileError> {
        dynasm!(self ; msr 0b1_011_0100_0100_000, X(reg as u32));
        Ok(())
    }
    // 1 011 0100 0100 001 => fpsr
    fn emit_read_fpsr(&mut self, reg: GPR) -> Result<(), CompileError> {
        dynasm!(self ; mrs X(reg as u32), 0b1_011_0100_0100_001);
        Ok(())
    }
    fn emit_write_fpsr(&mut self, reg: GPR) -> Result<(), CompileError> {
        dynasm!(self ; msr 0b1_011_0100_0100_001, X(reg as u32));
        Ok(())
    }
}

pub fn gen_std_trampoline_arm64(
    sig: &FunctionType,
    calling_convention: CallingConvention,
) -> Result<FunctionBody, CompileError> {
    let mut a = Assembler::new(0);

    let fptr = GPR::X27;
    let args = GPR::X28;

    dynasm!(a
        ; sub sp, sp, 32
        ; stp x29, x30, [sp]
        ; stp X(fptr as u32), X(args as u32), [sp, 16]
        ; mov x29, sp
        ; mov X(fptr as u32), x1
        ; mov X(args as u32), x2
    );

    let stack_args = sig.params().len().saturating_sub(7); //1st arg is ctx, not an actual arg
    let mut stack_offset = stack_args as u32 * 8;
    if stack_args > 0 {
        if stack_offset % 16 != 0 {
            stack_offset += 8;
            assert!(stack_offset % 16 == 0);
        }
        dynasm!(a ; sub sp, sp, stack_offset);
    }

    // Move arguments to their locations.
    // `callee_vmctx` is already in the first argument register, so no need to move.
    let mut caller_stack_offset: i32 = 0;
    for (i, param) in sig.params().iter().enumerate() {
        let sz = match *param {
            Type::I32 | Type::F32 => Size::S32,
            Type::I64 | Type::F64 => Size::S64,
            Type::ExternRef => Size::S64,
            Type::FuncRef => Size::S64,
            _ => codegen_error!(
                "singlepass unsupported param type for trampoline {:?}",
                *param
            ),
        };
        match i {
            0..=6 => {
                a.emit_ldr(
                    sz,
                    Location::GPR(GPR::from_index(i + 1).unwrap()),
                    Location::Memory(args, (i * 16) as i32),
                )?;
            }
            _ => {
                #[allow(clippy::single_match)]
                match calling_convention {
                    CallingConvention::AppleAarch64 => {
                        let sz = 1
                            << match sz {
                                Size::S8 => 0,
                                Size::S16 => 1,
                                Size::S32 => 2,
                                Size::S64 => 3,
                            };
                        // align first
                        if sz > 1 && caller_stack_offset & (sz - 1) != 0 {
                            caller_stack_offset = (caller_stack_offset + (sz - 1)) & !(sz - 1);
                        }
                    }
                    _ => (),
                };
                // using X16 as scratch reg
                a.emit_ldr(
                    sz,
                    Location::GPR(GPR::X16),
                    Location::Memory(args, (i * 16) as i32),
                )?;
                a.emit_str(
                    sz,
                    Location::GPR(GPR::X16),
                    Location::Memory(GPR::XzrSp, caller_stack_offset),
                )?;
                match calling_convention {
                    CallingConvention::AppleAarch64 => {
                        caller_stack_offset += 1
                            << match sz {
                                Size::S8 => 0,
                                Size::S16 => 1,
                                Size::S32 => 2,
                                Size::S64 => 3,
                            };
                    }
                    _ => {
                        caller_stack_offset += 8;
                    }
                }
            }
        }
    }

    dynasm!(a  ; blr X(fptr as u32));

    // Write return value.
    if !sig.results().is_empty() {
        a.emit_str(Size::S64, Location::GPR(GPR::X0), Location::Memory(args, 0))?;
    }

    // Restore stack.
    dynasm!(a
        ; ldp X(fptr as u32), X(args as u32), [x29, 16]
        ; ldp x29, x30, [x29]
        ; add sp, sp, 32 + stack_offset as u32
        ; ret
    );

    let mut body = a.finalize().unwrap();
    body.shrink_to_fit();
    Ok(FunctionBody {
        body,
        unwind_info: None,
    })
}
// Generates dynamic import function call trampoline for a function type.
pub fn gen_std_dynamic_import_trampoline_arm64(
    vmoffsets: &VMOffsets,
    sig: &FunctionType,
    calling_convention: CallingConvention,
) -> Result<FunctionBody, CompileError> {
    let mut a = Assembler::new(0);
    // Allocate argument array.
    let stack_offset: usize = 16 * std::cmp::max(sig.params().len(), sig.results().len());
    // Save LR and X26, as scratch register
    a.emit_stpdb(
        Size::S64,
        Location::GPR(GPR::X30),
        Location::GPR(GPR::X26),
        GPR::XzrSp,
        16,
    )?;

    if stack_offset != 0 {
        if stack_offset < 0x1000 {
            a.emit_sub(
                Size::S64,
                Location::GPR(GPR::XzrSp),
                Location::Imm32(stack_offset as _),
                Location::GPR(GPR::XzrSp),
            )?;
        } else {
            a.emit_mov_imm(Location::GPR(GPR::X26), stack_offset as u64)?;
            a.emit_sub(
                Size::S64,
                Location::GPR(GPR::XzrSp),
                Location::GPR(GPR::X26),
                Location::GPR(GPR::XzrSp),
            )?;
        }
    }

    // Copy arguments.
    if !sig.params().is_empty() {
        let mut argalloc = ArgumentRegisterAllocator::default();
        argalloc.next(Type::I64, calling_convention).unwrap(); // skip VMContext

        let mut stack_param_count: usize = 0;

        for (i, ty) in sig.params().iter().enumerate() {
            let source_loc = match argalloc.next(*ty, calling_convention) {
                Some(ARM64Register::GPR(gpr)) => Location::GPR(gpr),
                Some(ARM64Register::NEON(neon)) => Location::SIMD(neon),
                None => {
                    let sz = match calling_convention {
                        CallingConvention::AppleAarch64 => match *ty {
                            Type::I32 | Type::F32 => Size::S32,
                            _ => {
                                if stack_param_count & 7 != 0 {
                                    stack_param_count = (stack_param_count + 7) & !7;
                                };
                                Size::S64
                            }
                        },
                        _ => Size::S64,
                    };
                    a.emit_ldr(
                        sz,
                        Location::GPR(GPR::X26),
                        Location::Memory(GPR::XzrSp, (stack_offset + 16 + stack_param_count) as _),
                    )?;
                    stack_param_count += match sz {
                        Size::S32 => 4,
                        Size::S64 => 8,
                        _ => codegen_error!(
                            "singlepass unreachable in gen_std_dynamic_import_trampoline_arm64"
                        ),
                    };
                    Location::GPR(GPR::X26)
                }
            };
            a.emit_str(
                Size::S64,
                source_loc,
                Location::Memory(GPR::XzrSp, (i * 16) as _),
            )?;

            // Zero upper 64 bits.
            a.emit_str(
                Size::S64,
                Location::GPR(GPR::XzrSp),                       // XZR here
                Location::Memory(GPR::XzrSp, (i * 16 + 8) as _), // XSP here
            )?;
        }
    }

    #[allow(clippy::match_single_binding)]
    match calling_convention {
        _ => {
            // Load target address.
            let offset = vmoffsets.vmdynamicfunction_import_context_address();
            a.emit_ldur(Size::S64, Location::GPR(GPR::X26), GPR::X0, offset as i32)?;
            // Load values array.
            a.emit_add(
                Size::S64,
                Location::GPR(GPR::XzrSp),
                Location::Imm8(0),
                Location::GPR(GPR::X1),
            )?;
        }
    };

    // Call target.
    a.emit_call_register(GPR::X26)?;

    // Fetch return value.
    if !sig.results().is_empty() {
        assert_eq!(sig.results().len(), 1);
        a.emit_ldr(
            Size::S64,
            Location::GPR(GPR::X0),
            Location::Memory(GPR::XzrSp, 0),
        )?;
    }

    // Release values array.
    if stack_offset != 0 {
        if stack_offset < 0x1000 {
            a.emit_add(
                Size::S64,
                Location::GPR(GPR::XzrSp),
                Location::Imm32(stack_offset as _),
                Location::GPR(GPR::XzrSp),
            )?;
        } else {
            a.emit_mov_imm(Location::GPR(GPR::X26), stack_offset as u64)?;
            a.emit_add(
                Size::S64,
                Location::GPR(GPR::XzrSp),
                Location::GPR(GPR::X26),
                Location::GPR(GPR::XzrSp),
            )?;
        }
    }
    a.emit_ldpia(
        Size::S64,
        Location::GPR(GPR::X30),
        Location::GPR(GPR::X26),
        GPR::XzrSp,
        16,
    )?;

    // Return.
    a.emit_ret()?;

    let mut body = a.finalize().unwrap();
    body.shrink_to_fit();
    Ok(FunctionBody {
        body,
        unwind_info: None,
    })
}
// Singlepass calls import functions through a trampoline.
pub fn gen_import_call_trampoline_arm64(
    vmoffsets: &VMOffsets,
    index: FunctionIndex,
    sig: &FunctionType,
    calling_convention: CallingConvention,
) -> Result<CustomSection, CompileError> {
    let mut a = Assembler::new(0);

    // Singlepass internally treats all arguments as integers
    // For the standard System V calling convention requires
    //  floating point arguments to be passed in NEON registers.
    //  Translation is expensive, so only do it if needed.
    if sig
        .params()
        .iter()
        .any(|&x| x == Type::F32 || x == Type::F64)
    {
        #[allow(clippy::match_single_binding)]
        match calling_convention {
            _ => {
                // Allocate stack space for arguments.
                let stack_offset: i32 = if sig.params().len() > 7 {
                    7 * 8
                } else {
                    (sig.params().len() as i32) * 8
                };
                let stack_offset = if stack_offset & 15 != 0 {
                    stack_offset + 8
                } else {
                    stack_offset
                };
                if stack_offset > 0 {
                    if stack_offset < 0x1000 {
                        a.emit_sub(
                            Size::S64,
                            Location::GPR(GPR::XzrSp),
                            Location::Imm32(stack_offset as u32),
                            Location::GPR(GPR::XzrSp),
                        )?;
                    } else {
                        a.emit_mov_imm(Location::GPR(GPR::X16), stack_offset as u64)?;
                        a.emit_sub(
                            Size::S64,
                            Location::GPR(GPR::XzrSp),
                            Location::GPR(GPR::X16),
                            Location::GPR(GPR::XzrSp),
                        )?;
                    }
                }

                // Store all arguments to the stack to prevent overwrite.
                static PARAM_REGS: &[GPR] = &[
                    GPR::X1,
                    GPR::X2,
                    GPR::X3,
                    GPR::X4,
                    GPR::X5,
                    GPR::X6,
                    GPR::X7,
                ];
                let mut param_locations = vec![];
                /* Clippy is wrong about using `i` to index `PARAM_REGS` here. */
                #[allow(clippy::needless_range_loop)]
                for i in 0..sig.params().len() {
                    let loc = match i {
                        0..=6 => {
                            let loc = Location::Memory(GPR::XzrSp, (i * 8) as i32);
                            a.emit_str(Size::S64, Location::GPR(PARAM_REGS[i]), loc)?;
                            loc
                        }
                        _ => Location::Memory(GPR::XzrSp, stack_offset + ((i - 7) * 8) as i32),
                    };
                    param_locations.push(loc);
                }

                // Copy arguments.
                let mut caller_stack_offset: i32 = 0;
                let mut argalloc = ArgumentRegisterAllocator::default();
                argalloc.next(Type::I64, calling_convention).unwrap(); // skip VMContext
                for (i, ty) in sig.params().iter().enumerate() {
                    let prev_loc = param_locations[i];
                    let targ = match argalloc.next(*ty, calling_convention) {
                        Some(ARM64Register::GPR(gpr)) => Location::GPR(gpr),
                        Some(ARM64Register::NEON(neon)) => Location::SIMD(neon),
                        None => {
                            // No register can be allocated. Put this argument on the stack.
                            a.emit_ldr(Size::S64, Location::GPR(GPR::X16), prev_loc)?;
                            a.emit_str(
                                Size::S64,
                                Location::GPR(GPR::X16),
                                Location::Memory(GPR::XzrSp, stack_offset + caller_stack_offset),
                            )?;
                            caller_stack_offset += 8;
                            continue;
                        }
                    };
                    a.emit_ldr(Size::S64, targ, prev_loc)?;
                }

                // Restore stack pointer.
                if stack_offset > 0 {
                    if stack_offset < 0x1000 {
                        a.emit_add(
                            Size::S64,
                            Location::GPR(GPR::XzrSp),
                            Location::Imm32(stack_offset as u32),
                            Location::GPR(GPR::XzrSp),
                        )?;
                    } else {
                        a.emit_mov_imm(Location::GPR(GPR::X16), stack_offset as u64)?;
                        a.emit_add(
                            Size::S64,
                            Location::GPR(GPR::XzrSp),
                            Location::GPR(GPR::X16),
                            Location::GPR(GPR::XzrSp),
                        )?;
                    }
                }
            }
        }
    }

    // Emits a tail call trampoline that loads the address of the target import function
    // from Ctx and jumps to it.

    let offset = vmoffsets.vmctx_vmfunction_import(index);
    // for ldr, offset needs to be a multiple of 8, wich often is not
    // so use ldur, but then offset is limited to -255 .. +255. It will be positive here
    let offset =
        if (offset > 0 && offset < 0xF8) || (offset > 0 && offset < 0x7FF8 && (offset & 7) == 0) {
            offset
        } else {
            a.emit_mov_imm(Location::GPR(GPR::X16), (offset as i64) as u64)?;
            a.emit_add(
                Size::S64,
                Location::GPR(GPR::X0),
                Location::GPR(GPR::X16),
                Location::GPR(GPR::X0),
            )?;
            0
        };
    #[allow(clippy::match_single_binding)]
    match calling_convention {
        _ => {
            if (offset & 7) == 0 {
                a.emit_ldr(
                    Size::S64,
                    Location::GPR(GPR::X16),
                    Location::Memory(GPR::X0, offset as i32), // function pointer
                )?;
                a.emit_ldr(
                    Size::S64,
                    Location::GPR(GPR::X0),
                    Location::Memory(GPR::X0, offset as i32 + 8), // target vmctx
                )?;
            } else {
                a.emit_ldur(
                    Size::S64,
                    Location::GPR(GPR::X16),
                    GPR::X0,
                    offset as i32, // function pointer
                )?;
                a.emit_ldur(
                    Size::S64,
                    Location::GPR(GPR::X0),
                    GPR::X0,
                    offset as i32 + 8, // target vmctx
                )?;
            }
        }
    }
    a.emit_b_register(GPR::X16)?;

    let mut contents = a.finalize().unwrap();
    contents.shrink_to_fit();
    let section_body = SectionBody::new_with_vec(contents);

    Ok(CustomSection {
        protection: CustomSectionProtection::ReadExecute,
        bytes: section_body,
        relocations: vec![],
    })
}
