//! ARM64 structures.

use crate::common_decl::{MachineState, MachineValue, RegisterIndex};
use crate::location::CombinedRegister;
use crate::location::Reg as AbstractReg;
use std::collections::BTreeMap;
use std::slice::Iter;
use wasmer_types::{CallingConvention, Type};

/// General-purpose registers.
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[allow(clippy::upper_case_acronyms)]
pub enum GPR {
    X0 = 0,
    X1 = 1,
    X2 = 2,
    X3 = 3,
    X4 = 4,
    X5 = 5,
    X6 = 6,
    X7 = 7,
    X8 = 8,
    X9 = 9,
    X10 = 10,
    X11 = 11,
    X12 = 12,
    X13 = 13,
    X14 = 14,
    X15 = 15,
    X16 = 16,
    X17 = 17,
    X18 = 18,
    X19 = 19,
    X20 = 20,
    X21 = 21,
    X22 = 22,
    X23 = 23,
    X24 = 24,
    X25 = 25,
    X26 = 26,
    X27 = 27,
    X28 = 28,
    X29 = 29,
    X30 = 30,
    XzrSp = 31,
}

/// NEON registers.
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
pub enum NEON {
    V0 = 0,
    V1 = 1,
    V2 = 2,
    V3 = 3,
    V4 = 4,
    V5 = 5,
    V6 = 6,
    V7 = 7,
    V8 = 8,
    V9 = 9,
    V10 = 10,
    V11 = 11,
    V12 = 12,
    V13 = 13,
    V14 = 14,
    V15 = 15,
    V16 = 16,
    V17 = 17,
    V18 = 18,
    V19 = 19,
    V20 = 20,
    V21 = 21,
    V22 = 22,
    V23 = 23,
    V24 = 24,
    V25 = 25,
    V26 = 26,
    V27 = 27,
    V28 = 28,
    V29 = 29,
    V30 = 30,
    V31 = 31,
}

impl AbstractReg for GPR {
    fn is_callee_save(self) -> bool {
        self as usize > 18
    }
    fn is_reserved(self) -> bool {
        !matches!(self.into_index(), 0..=16 | 19..=27)
    }
    fn into_index(self) -> usize {
        self as usize
    }
    fn from_index(n: usize) -> Result<GPR, ()> {
        match n {
            0..=31 => Ok(*GPR::iterator().nth(n).unwrap()),
            _ => Err(()),
        }
    }
    fn iterator() -> Iter<'static, GPR> {
        static GPRS: [GPR; 32] = [
            GPR::X0,
            GPR::X1,
            GPR::X2,
            GPR::X3,
            GPR::X4,
            GPR::X5,
            GPR::X6,
            GPR::X7,
            GPR::X8,
            GPR::X9,
            GPR::X10,
            GPR::X11,
            GPR::X12,
            GPR::X13,
            GPR::X14,
            GPR::X15,
            GPR::X16,
            GPR::X17,
            GPR::X18,
            GPR::X19,
            GPR::X20,
            GPR::X21,
            GPR::X22,
            GPR::X23,
            GPR::X24,
            GPR::X25,
            GPR::X26,
            GPR::X27,
            GPR::X28,
            GPR::X29,
            GPR::X30,
            GPR::XzrSp,
        ];
        GPRS.iter()
    }
    fn to_dwarf(self) -> u16 {
        self.into_index() as u16
    }
}

impl AbstractReg for NEON {
    fn is_callee_save(self) -> bool {
        self as usize > 16
    }
    fn is_reserved(self) -> bool {
        false
    }
    fn into_index(self) -> usize {
        self as usize
    }
    fn from_index(n: usize) -> Result<NEON, ()> {
        match n {
            0..=31 => Ok(*NEON::iterator().nth(n).unwrap()),
            _ => Err(()),
        }
    }
    fn iterator() -> Iter<'static, NEON> {
        const NEONS: [NEON; 32] = [
            NEON::V0,
            NEON::V1,
            NEON::V2,
            NEON::V3,
            NEON::V4,
            NEON::V5,
            NEON::V6,
            NEON::V7,
            NEON::V8,
            NEON::V9,
            NEON::V10,
            NEON::V11,
            NEON::V12,
            NEON::V13,
            NEON::V14,
            NEON::V15,
            NEON::V16,
            NEON::V17,
            NEON::V18,
            NEON::V19,
            NEON::V20,
            NEON::V21,
            NEON::V22,
            NEON::V23,
            NEON::V24,
            NEON::V25,
            NEON::V26,
            NEON::V27,
            NEON::V28,
            NEON::V29,
            NEON::V30,
            NEON::V31,
        ];
        NEONS.iter()
    }
    fn to_dwarf(self) -> u16 {
        self.into_index() as u16 + 64
    }
}

/// A machine register under the x86-64 architecture.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub enum ARM64Register {
    /// General-purpose registers.
    GPR(GPR),
    /// NEON (floating point/SIMD) registers.
    NEON(NEON),
}

impl CombinedRegister for ARM64Register {
    /// Returns the index of the register.
    fn to_index(&self) -> RegisterIndex {
        match *self {
            ARM64Register::GPR(x) => RegisterIndex(x as usize),
            ARM64Register::NEON(x) => RegisterIndex(x as usize + 64),
        }
    }
    /// Convert from a GPR register
    fn from_gpr(x: u16) -> Self {
        ARM64Register::GPR(GPR::from_index(x as usize).unwrap())
    }
    /// Convert from an SIMD register
    fn from_simd(x: u16) -> Self {
        ARM64Register::NEON(NEON::from_index(x as usize).unwrap())
    }

    /// Converts a DWARF regnum to ARM64Register.
    fn _from_dwarf_regnum(x: u16) -> Option<ARM64Register> {
        Some(match x {
            0..=31 => ARM64Register::GPR(GPR::from_index(x as usize).unwrap()),
            64..=95 => ARM64Register::NEON(NEON::from_index(x as usize - 64).unwrap()),
            _ => return None,
        })
    }
}

/// An allocator that allocates registers for function arguments according to the System V ABI.
#[derive(Default)]
pub struct ArgumentRegisterAllocator {
    n_gprs: usize,
    n_neons: usize,
}

impl ArgumentRegisterAllocator {
    /// Allocates a register for argument type `ty`. Returns `None` if no register is available for this type.
    pub fn next(
        &mut self,
        ty: Type,
        calling_convention: CallingConvention,
    ) -> Option<ARM64Register> {
        match calling_convention {
            CallingConvention::SystemV | CallingConvention::AppleAarch64 => {
                static GPR_SEQ: &[GPR] = &[
                    GPR::X0,
                    GPR::X1,
                    GPR::X2,
                    GPR::X3,
                    GPR::X4,
                    GPR::X5,
                    GPR::X6,
                    GPR::X7,
                ];
                static NEON_SEQ: &[NEON] = &[
                    NEON::V0,
                    NEON::V1,
                    NEON::V2,
                    NEON::V3,
                    NEON::V4,
                    NEON::V5,
                    NEON::V6,
                    NEON::V7,
                ];
                match ty {
                    Type::I32 | Type::I64 => {
                        if self.n_gprs < GPR_SEQ.len() {
                            let gpr = GPR_SEQ[self.n_gprs];
                            self.n_gprs += 1;
                            Some(ARM64Register::GPR(gpr))
                        } else {
                            None
                        }
                    }
                    Type::F32 | Type::F64 => {
                        if self.n_neons < NEON_SEQ.len() {
                            let neon = NEON_SEQ[self.n_neons];
                            self.n_neons += 1;
                            Some(ARM64Register::NEON(neon))
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
            _ => unimplemented!(),
        }
    }
}

/// Create a new `MachineState` with default values.
pub fn new_machine_state() -> MachineState {
    MachineState {
        stack_values: vec![],
        register_values: vec![MachineValue::Undefined; 32 + 32],
        prev_frame: BTreeMap::new(),
        wasm_stack: vec![],
        wasm_inst_offset: std::usize::MAX,
    }
}
