//! Unwind information for Windows x64 ABI.

use regalloc::{Reg, RegClass};

pub(crate) struct RegisterMapper;

impl crate::isa::unwind::winx64::RegisterMapper<Reg> for RegisterMapper {
    fn map(reg: Reg) -> crate::isa::unwind::winx64::MappedRegister {
        use crate::isa::unwind::winx64::MappedRegister;
        match reg.get_class() {
            RegClass::I64 => MappedRegister::Int(reg.get_hw_encoding()),
            RegClass::V128 => MappedRegister::Xmm(reg.get_hw_encoding()),
            _ => unreachable!(),
        }
    }
}
