//! X64 structures.

#![allow(clippy::upper_case_acronyms)]
use crate::common_decl::{MachineState, MachineValue, RegisterIndex};
use crate::location::CombinedRegister;
use crate::location::Reg as AbstractReg;
use std::collections::BTreeMap;
use std::slice::Iter;
use wasmer_types::{CallingConvention, Type};

/// General-purpose registers.
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum GPR {
    RAX = 0,
    RCX = 1,
    RDX = 2,
    RBX = 3,
    RSP = 4,
    RBP = 5,
    RSI = 6,
    RDI = 7,
    R8 = 8,
    R9 = 9,
    R10 = 10,
    R11 = 11,
    R12 = 12,
    R13 = 13,
    R14 = 14,
    R15 = 15,
}

/// XMM registers.
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[allow(dead_code)]
pub enum XMM {
    XMM0 = 0,
    XMM1 = 1,
    XMM2 = 2,
    XMM3 = 3,
    XMM4 = 4,
    XMM5 = 5,
    XMM6 = 6,
    XMM7 = 7,
    XMM8 = 8,
    XMM9 = 9,
    XMM10 = 10,
    XMM11 = 11,
    XMM12 = 12,
    XMM13 = 13,
    XMM14 = 14,
    XMM15 = 15,
}

impl AbstractReg for GPR {
    fn is_callee_save(self) -> bool {
        const IS_CALLEE_SAVE: [bool; 16] = [
            false, false, false, true, true, true, false, false, false, false, false, false, true,
            true, true, true,
        ];
        IS_CALLEE_SAVE[self as usize]
    }
    fn is_reserved(self) -> bool {
        self == GPR::RSP || self == GPR::RBP || self == GPR::R10 || self == GPR::R15
    }
    fn into_index(self) -> usize {
        self as usize
    }
    fn from_index(n: usize) -> Result<GPR, ()> {
        match n {
            0..=15 => Ok(*GPR::iterator().nth(n).unwrap()),
            _ => Err(()),
        }
    }
    fn iterator() -> Iter<'static, GPR> {
        static GPRS: [GPR; 16] = [
            GPR::RAX,
            GPR::RCX,
            GPR::RDX,
            GPR::RBX,
            GPR::RSP,
            GPR::RBP,
            GPR::RSI,
            GPR::RDI,
            GPR::R8,
            GPR::R9,
            GPR::R10,
            GPR::R11,
            GPR::R12,
            GPR::R13,
            GPR::R14,
            GPR::R15,
        ];
        GPRS.iter()
    }
    fn to_dwarf(self) -> u16 {
        match self {
            GPR::RAX => 0,
            GPR::RDX => 1,
            GPR::RCX => 2,
            GPR::RBX => 3,
            GPR::RSI => 4,
            GPR::RDI => 5,
            GPR::RBP => 6,
            GPR::RSP => 7,
            GPR::R8 => 8,
            GPR::R9 => 9,
            GPR::R10 => 10,
            GPR::R11 => 11,
            GPR::R12 => 12,
            GPR::R13 => 13,
            GPR::R14 => 14,
            GPR::R15 => 15,
        }
    }
}

impl AbstractReg for XMM {
    fn is_callee_save(self) -> bool {
        const IS_CALLEE_SAVE: [bool; 16] = [
            false, false, false, false, false, false, false, false, true, true, true, true, true,
            true, true, true,
        ];
        IS_CALLEE_SAVE[self as usize]
    }
    fn is_reserved(self) -> bool {
        false
    }
    fn into_index(self) -> usize {
        self as usize
    }
    fn from_index(n: usize) -> Result<XMM, ()> {
        match n {
            0..=15 => Ok(*XMM::iterator().nth(n).unwrap()),
            _ => Err(()),
        }
    }
    fn iterator() -> Iter<'static, XMM> {
        static XMMS: [XMM; 16] = [
            XMM::XMM0,
            XMM::XMM1,
            XMM::XMM2,
            XMM::XMM3,
            XMM::XMM4,
            XMM::XMM5,
            XMM::XMM6,
            XMM::XMM7,
            XMM::XMM8,
            XMM::XMM9,
            XMM::XMM10,
            XMM::XMM11,
            XMM::XMM12,
            XMM::XMM13,
            XMM::XMM14,
            XMM::XMM15,
        ];
        XMMS.iter()
    }
    fn to_dwarf(self) -> u16 {
        self.into_index() as u16 + 17
    }
}

/// A machine register under the x86-64 architecture.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum X64Register {
    /// General-purpose registers.
    GPR(GPR),
    /// XMM (floating point/SIMD) registers.
    XMM(XMM),
}

impl CombinedRegister for X64Register {
    /// Returns the index of the register.
    fn to_index(&self) -> RegisterIndex {
        match *self {
            X64Register::GPR(x) => RegisterIndex(x as usize),
            X64Register::XMM(x) => RegisterIndex(x as usize + 16),
        }
    }
    /// Convert from a GPR register
    fn from_gpr(x: u16) -> Self {
        X64Register::GPR(GPR::from_index(x as usize).unwrap())
    }
    /// Convert from an SIMD register
    fn from_simd(x: u16) -> Self {
        X64Register::XMM(XMM::from_index(x as usize).unwrap())
    }
    /* x86_64-abi-0.99.pdf
     * Register Name                    | Number | Abbreviation
     * General Purpose Register RAX     | 0      | %rax
     * General Purpose Register RDX     | 1      | %rdx
     * General Purpose Register RCX     | 2      | %rcx
     * General Purpose Register RBX     | 3      | %rbx
     * General Purpose Register RSI     | 4      | %rsi
     * General Purpose Register RDI     | 5      | %rdi
     * Frame Pointer Register   RBP     | 6      | %rbp
     * Stack Pointer Register   RSP     | 7      | %rsp
     * Extended Integer Registers 8-15  | 8-15   | %r8-%r15
     * Return Address RA                | 16     |
     * Vector Registers 0-7             | 17-24  | %xmm0-%xmm7
     * Extended Vector Registers 8-15   | 25-32  | %xmm8-%xmm15
     * Floating Point Registers 0-7     | 33-40  | %st0-%st7
     * MMX Registers 0-7                | 41-48  | %mm0-%mm7
     * Flag Register                    | 49     | %rFLAGS
     * Segment Register ES              | 50     | %es
     * Segment Register CS              | 51     | %cs
     * Segment Register SS              | 52     | %ss
     * Segment Register DS              | 53     | %ds
     * Segment Register FS              | 54     | %fs
     * Segment Register GS              | 55     | %gs
     * Reserved                         | 56-57  |
     * FS Base address                  | 58     | %fs.base
     * GS Base address                  | 59     | %gs.base
     * Reserved                         | 60-61  |
     * Task Register                    | 62     | %tr
     * LDT Register                     | 63     | %ldtr
     * 128-bit Media Control and Status | 64     | %mxcsr
     * x87 Control Word                 | 65     | %fcw
     * x87 Status Word                  | 66     | %fsw
     */
    /// Converts a DWARF regnum to X64Register.
    fn _from_dwarf_regnum(x: u16) -> Option<X64Register> {
        static DWARF_REGS: [GPR; 16] = [
            GPR::RAX,
            GPR::RDX,
            GPR::RCX,
            GPR::RBX,
            GPR::RSI,
            GPR::RDI,
            GPR::RBP,
            GPR::RSP,
            GPR::R8,
            GPR::R9,
            GPR::R10,
            GPR::R11,
            GPR::R12,
            GPR::R13,
            GPR::R14,
            GPR::R15,
        ];
        Some(match x {
            0..=15 => X64Register::GPR(DWARF_REGS[x as usize]),
            17..=24 => X64Register::XMM(XMM::from_index(x as usize - 17).unwrap()),
            _ => return None,
        })
    }
}

/// An allocator that allocates registers for function arguments according to the System V ABI.
#[derive(Default)]
pub struct ArgumentRegisterAllocator {
    n_gprs: usize,
    n_xmms: usize,
}

impl ArgumentRegisterAllocator {
    /// Allocates a register for argument type `ty`. Returns `None` if no register is available for this type.
    pub fn next(&mut self, ty: Type, calling_convention: CallingConvention) -> Option<X64Register> {
        match calling_convention {
            CallingConvention::WindowsFastcall => {
                static GPR_SEQ: &[GPR] = &[GPR::RCX, GPR::RDX, GPR::R8, GPR::R9];
                static XMM_SEQ: &[XMM] = &[XMM::XMM0, XMM::XMM1, XMM::XMM2, XMM::XMM3];
                let idx = self.n_gprs + self.n_xmms;
                match ty {
                    Type::I32 | Type::I64 => {
                        if idx < 4 {
                            let gpr = GPR_SEQ[idx];
                            self.n_gprs += 1;
                            Some(X64Register::GPR(gpr))
                        } else {
                            None
                        }
                    }
                    Type::F32 | Type::F64 => {
                        if idx < 4 {
                            let xmm = XMM_SEQ[idx];
                            self.n_xmms += 1;
                            Some(X64Register::XMM(xmm))
                        } else {
                            None
                        }
                    }
                    _ => todo!(
                        "ArgumentRegisterAllocator::next: Unsupported type: {:?}",
                        ty
                    ),
                }
            }
            _ => {
                static GPR_SEQ: &[GPR] =
                    &[GPR::RDI, GPR::RSI, GPR::RDX, GPR::RCX, GPR::R8, GPR::R9];
                static XMM_SEQ: &[XMM] = &[
                    XMM::XMM0,
                    XMM::XMM1,
                    XMM::XMM2,
                    XMM::XMM3,
                    XMM::XMM4,
                    XMM::XMM5,
                    XMM::XMM6,
                    XMM::XMM7,
                ];
                match ty {
                    Type::I32 | Type::I64 => {
                        if self.n_gprs < GPR_SEQ.len() {
                            let gpr = GPR_SEQ[self.n_gprs];
                            self.n_gprs += 1;
                            Some(X64Register::GPR(gpr))
                        } else {
                            None
                        }
                    }
                    Type::F32 | Type::F64 => {
                        if self.n_xmms < XMM_SEQ.len() {
                            let xmm = XMM_SEQ[self.n_xmms];
                            self.n_xmms += 1;
                            Some(X64Register::XMM(xmm))
                        } else {
                            None
                        }
                    }
                    _ => todo!(
                        "ArgumentRegisterAllocator::next: Unsupported type: {:?}",
                        ty
                    ),
                }
            }
        }
    }
}

/// Create a new `MachineState` with default values.
pub fn new_machine_state() -> MachineState {
    MachineState {
        stack_values: vec![],
        register_values: vec![MachineValue::Undefined; 16 + 8],
        prev_frame: BTreeMap::new(),
        wasm_stack: vec![],
        wasm_inst_offset: std::usize::MAX,
    }
}
