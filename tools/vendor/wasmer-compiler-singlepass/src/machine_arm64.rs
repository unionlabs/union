use crate::arm64_decl::new_machine_state;
use crate::arm64_decl::{GPR, NEON};
use crate::codegen_error;
use crate::common_decl::*;
use crate::emitter_arm64::*;
use crate::location::Location as AbstractLocation;
use crate::location::Reg;
use crate::machine::*;
use crate::unwind::{UnwindInstructions, UnwindOps};
use dynasmrt::{aarch64::Aarch64Relocation, VecAssembler};
#[cfg(feature = "unwind")]
use gimli::{write::CallFrameInstruction, AArch64};
use wasmer_compiler::wasmparser::ValType as WpType;
use wasmer_types::{
    CallingConvention, CompileError, CustomSection, FunctionBody, FunctionIndex, FunctionType,
    InstructionAddressMap, Relocation, RelocationKind, RelocationTarget, SourceLoc, TrapCode,
    TrapInformation, VMOffsets,
};

type Assembler = VecAssembler<Aarch64Relocation>;
type Location = AbstractLocation<GPR, NEON>;

#[cfg(feature = "unwind")]
fn dwarf_index(reg: u16) -> gimli::Register {
    static DWARF_GPR: [gimli::Register; 32] = [
        AArch64::X0,
        AArch64::X1,
        AArch64::X2,
        AArch64::X3,
        AArch64::X4,
        AArch64::X5,
        AArch64::X6,
        AArch64::X7,
        AArch64::X8,
        AArch64::X9,
        AArch64::X10,
        AArch64::X11,
        AArch64::X12,
        AArch64::X13,
        AArch64::X14,
        AArch64::X15,
        AArch64::X16,
        AArch64::X17,
        AArch64::X18,
        AArch64::X19,
        AArch64::X20,
        AArch64::X21,
        AArch64::X22,
        AArch64::X23,
        AArch64::X24,
        AArch64::X25,
        AArch64::X26,
        AArch64::X27,
        AArch64::X28,
        AArch64::X29,
        AArch64::X30,
        AArch64::SP,
    ];
    static DWARF_NEON: [gimli::Register; 32] = [
        AArch64::V0,
        AArch64::V1,
        AArch64::V2,
        AArch64::V3,
        AArch64::V4,
        AArch64::V5,
        AArch64::V6,
        AArch64::V7,
        AArch64::V8,
        AArch64::V9,
        AArch64::V10,
        AArch64::V11,
        AArch64::V12,
        AArch64::V13,
        AArch64::V14,
        AArch64::V15,
        AArch64::V16,
        AArch64::V17,
        AArch64::V18,
        AArch64::V19,
        AArch64::V20,
        AArch64::V21,
        AArch64::V22,
        AArch64::V23,
        AArch64::V24,
        AArch64::V25,
        AArch64::V26,
        AArch64::V27,
        AArch64::V28,
        AArch64::V29,
        AArch64::V30,
        AArch64::V31,
    ];
    match reg {
        0..=31 => DWARF_GPR[reg as usize],
        64..=95 => DWARF_NEON[reg as usize - 64],
        _ => panic!("Unknown register index {}", reg),
    }
}

pub struct MachineARM64 {
    assembler: Assembler,
    used_gprs: u32,
    used_simd: u32,
    trap_table: TrapTable,
    /// Map from byte offset into wasm function to range of native instructions.
    ///
    // Ordered by increasing InstructionAddressMap::srcloc.
    instructions_address_map: Vec<InstructionAddressMap>,
    /// The source location for the current operator.
    src_loc: u32,
    /// is last push on a 8byte multiple or 16bytes?
    pushed: bool,
    /// Vector of unwind operations with offset
    unwind_ops: Vec<(usize, UnwindOps)>,
}

#[allow(dead_code)]
#[derive(PartialEq)]
enum ImmType {
    None,
    NoneXzr,
    Bits8,
    Bits12,
    Shift32,
    Shift32No0,
    Shift64,
    Shift64No0,
    Logical32,
    Logical64,
    UnscaledOffset,
    OffsetByte,
    OffsetHWord,
    OffsetWord,
    OffsetDWord,
}

#[allow(dead_code)]
impl MachineARM64 {
    pub fn new() -> Self {
        MachineARM64 {
            assembler: Assembler::new(0),
            used_gprs: 0,
            used_simd: 0,
            trap_table: TrapTable::default(),
            instructions_address_map: vec![],
            src_loc: 0,
            pushed: false,
            unwind_ops: vec![],
        }
    }
    fn compatible_imm(&self, imm: i64, ty: ImmType) -> bool {
        match ty {
            ImmType::None => false,
            ImmType::NoneXzr => false,
            ImmType::Bits8 => (0..256).contains(&imm),
            ImmType::Bits12 => (0..0x1000).contains(&imm),
            ImmType::Shift32 => (0..32).contains(&imm),
            ImmType::Shift32No0 => (1..32).contains(&imm),
            ImmType::Shift64 => (0..64).contains(&imm),
            ImmType::Shift64No0 => (1..64).contains(&imm),
            ImmType::Logical32 => encode_logical_immediate_32bit(imm as u32).is_some(),
            ImmType::Logical64 => encode_logical_immediate_64bit(imm as u64).is_some(),
            ImmType::UnscaledOffset => (imm > -256) && (imm < 256),
            ImmType::OffsetByte => (0..0x1000).contains(&imm),
            ImmType::OffsetHWord => (imm & 1 == 0) && (0..0x2000).contains(&imm),
            ImmType::OffsetWord => (imm & 3 == 0) && (0..0x4000).contains(&imm),
            ImmType::OffsetDWord => (imm & 7 == 0) && (0..0x8000).contains(&imm),
        }
    }

    fn location_to_reg(
        &mut self,
        sz: Size,
        src: Location,
        temps: &mut Vec<GPR>,
        allow_imm: ImmType,
        read_val: bool,
        wanted: Option<GPR>,
    ) -> Result<Location, CompileError> {
        match src {
            Location::GPR(_) | Location::SIMD(_) => Ok(src),
            Location::Imm8(val) => {
                if allow_imm == ImmType::NoneXzr && val == 0 {
                    Ok(Location::GPR(GPR::XzrSp))
                } else if self.compatible_imm(val as i64, allow_imm) {
                    Ok(src)
                } else {
                    let tmp = if let Some(wanted) = wanted {
                        wanted
                    } else {
                        let tmp = self.acquire_temp_gpr().ok_or_else(|| {
                            CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                        })?;
                        temps.push(tmp);
                        tmp
                    };
                    self.assembler
                        .emit_mov_imm(Location::GPR(tmp), val as u64)?;
                    Ok(Location::GPR(tmp))
                }
            }
            Location::Imm32(val) => {
                if allow_imm == ImmType::NoneXzr && val == 0 {
                    Ok(Location::GPR(GPR::XzrSp))
                } else if self.compatible_imm(val as i64, allow_imm) {
                    Ok(src)
                } else {
                    let tmp = if let Some(wanted) = wanted {
                        wanted
                    } else {
                        let tmp = self.acquire_temp_gpr().ok_or_else(|| {
                            CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                        })?;
                        temps.push(tmp);
                        tmp
                    };
                    self.assembler
                        .emit_mov_imm(Location::GPR(tmp), (val as i64) as u64)?;
                    Ok(Location::GPR(tmp))
                }
            }
            Location::Imm64(val) => {
                if allow_imm == ImmType::NoneXzr && val == 0 {
                    Ok(Location::GPR(GPR::XzrSp))
                } else if self.compatible_imm(val as i64, allow_imm) {
                    Ok(src)
                } else {
                    let tmp = if let Some(wanted) = wanted {
                        wanted
                    } else {
                        let tmp = self.acquire_temp_gpr().ok_or_else(|| {
                            CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                        })?;
                        temps.push(tmp);
                        tmp
                    };
                    self.assembler
                        .emit_mov_imm(Location::GPR(tmp), val as u64)?;
                    Ok(Location::GPR(tmp))
                }
            }
            Location::Memory(reg, val) => {
                let tmp = if let Some(wanted) = wanted {
                    wanted
                } else {
                    let tmp = self.acquire_temp_gpr().ok_or_else(|| {
                        CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                    })?;
                    temps.push(tmp);
                    tmp
                };
                if read_val {
                    let offsize = match sz {
                        Size::S8 => ImmType::OffsetByte,
                        Size::S16 => ImmType::OffsetHWord,
                        Size::S32 => ImmType::OffsetWord,
                        Size::S64 => ImmType::OffsetDWord,
                    };
                    if sz == Size::S8 {
                        if self.compatible_imm(val as i64, offsize) {
                            self.assembler.emit_ldrb(
                                sz,
                                Location::GPR(tmp),
                                Location::Memory(reg, val as _),
                            )?;
                        } else {
                            if reg == tmp {
                                codegen_error!("singlepass reg==tmp unreachable");
                            }
                            self.assembler
                                .emit_mov_imm(Location::GPR(tmp), (val as i64) as u64)?;
                            self.assembler.emit_ldrb(
                                sz,
                                Location::GPR(tmp),
                                Location::Memory2(reg, tmp, Multiplier::One, 0),
                            )?;
                        }
                    } else if sz == Size::S16 {
                        if self.compatible_imm(val as i64, offsize) {
                            self.assembler.emit_ldrh(
                                sz,
                                Location::GPR(tmp),
                                Location::Memory(reg, val as _),
                            )?;
                        } else {
                            if reg == tmp {
                                codegen_error!("singlepass reg==tmp unreachable");
                            }
                            self.assembler
                                .emit_mov_imm(Location::GPR(tmp), (val as i64) as u64)?;
                            self.assembler.emit_ldrh(
                                sz,
                                Location::GPR(tmp),
                                Location::Memory2(reg, tmp, Multiplier::One, 0),
                            )?;
                        }
                    } else if self.compatible_imm(val as i64, offsize) {
                        self.assembler.emit_ldr(
                            sz,
                            Location::GPR(tmp),
                            Location::Memory(reg, val as _),
                        )?;
                    } else if self.compatible_imm(val as i64, ImmType::UnscaledOffset) {
                        self.assembler.emit_ldur(sz, Location::GPR(tmp), reg, val)?;
                    } else {
                        if reg == tmp {
                            codegen_error!("singlepass reg == tmp unreachable");
                        }
                        self.assembler
                            .emit_mov_imm(Location::GPR(tmp), (val as i64) as u64)?;
                        self.assembler.emit_ldr(
                            sz,
                            Location::GPR(tmp),
                            Location::Memory2(reg, tmp, Multiplier::One, 0),
                        )?;
                    }
                }
                Ok(Location::GPR(tmp))
            }
            _ => codegen_error!("singlepass can't emit location_to_reg {:?} {:?}", sz, src),
        }
    }
    fn location_to_neon(
        &mut self,
        sz: Size,
        src: Location,
        temps: &mut Vec<NEON>,
        allow_imm: ImmType,
        read_val: bool,
    ) -> Result<Location, CompileError> {
        match src {
            Location::SIMD(_) => Ok(src),
            Location::GPR(_) => {
                let tmp = self.acquire_temp_simd().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp simd".to_owned())
                })?;
                temps.push(tmp);
                if read_val {
                    self.assembler.emit_mov(sz, src, Location::SIMD(tmp))?;
                }
                Ok(Location::SIMD(tmp))
            }
            Location::Imm8(val) => {
                if self.compatible_imm(val as i64, allow_imm) {
                    Ok(src)
                } else {
                    let gpr = self.acquire_temp_gpr().ok_or_else(|| {
                        CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                    })?;
                    let tmp = self.acquire_temp_simd().ok_or_else(|| {
                        CompileError::Codegen("singlepass cannot acquire temp simd".to_owned())
                    })?;
                    temps.push(tmp);
                    self.assembler
                        .emit_mov_imm(Location::GPR(gpr), val as u64)?;
                    self.assembler
                        .emit_mov(sz, Location::GPR(gpr), Location::SIMD(tmp))?;
                    self.release_gpr(gpr);
                    Ok(Location::SIMD(tmp))
                }
            }
            Location::Imm32(val) => {
                if self.compatible_imm(val as i64, allow_imm) {
                    Ok(src)
                } else {
                    let gpr = self.acquire_temp_gpr().ok_or_else(|| {
                        CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                    })?;
                    let tmp = self.acquire_temp_simd().ok_or_else(|| {
                        CompileError::Codegen("singlepass cannot acquire temp simd".to_owned())
                    })?;
                    temps.push(tmp);
                    self.assembler
                        .emit_mov_imm(Location::GPR(gpr), (val as i64) as u64)?;
                    self.assembler
                        .emit_mov(sz, Location::GPR(gpr), Location::SIMD(tmp))?;
                    self.release_gpr(gpr);
                    Ok(Location::SIMD(tmp))
                }
            }
            Location::Imm64(val) => {
                if self.compatible_imm(val as i64, allow_imm) {
                    Ok(src)
                } else {
                    let gpr = self.acquire_temp_gpr().ok_or_else(|| {
                        CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                    })?;
                    let tmp = self.acquire_temp_simd().ok_or_else(|| {
                        CompileError::Codegen("singlepass cannot acquire temp simd".to_owned())
                    })?;
                    temps.push(tmp);
                    self.assembler
                        .emit_mov_imm(Location::GPR(gpr), val as u64)?;
                    self.assembler
                        .emit_mov(sz, Location::GPR(gpr), Location::SIMD(tmp))?;
                    self.release_gpr(gpr);
                    Ok(Location::SIMD(tmp))
                }
            }
            Location::Memory(reg, val) => {
                let tmp = self.acquire_temp_simd().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp simd".to_owned())
                })?;
                temps.push(tmp);
                if read_val {
                    let offsize = if sz == Size::S32 {
                        ImmType::OffsetWord
                    } else {
                        ImmType::OffsetDWord
                    };
                    if self.compatible_imm(val as i64, offsize) {
                        self.assembler.emit_ldr(
                            sz,
                            Location::SIMD(tmp),
                            Location::Memory(reg, val as _),
                        )?;
                    } else if self.compatible_imm(val as i64, ImmType::UnscaledOffset) {
                        self.assembler
                            .emit_ldur(sz, Location::SIMD(tmp), reg, val)?;
                    } else {
                        let gpr = self.acquire_temp_gpr().ok_or_else(|| {
                            CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                        })?;
                        self.assembler
                            .emit_mov_imm(Location::GPR(gpr), (val as i64) as u64)?;
                        self.assembler.emit_ldr(
                            sz,
                            Location::SIMD(tmp),
                            Location::Memory2(reg, gpr, Multiplier::One, 0),
                        )?;
                        self.release_gpr(gpr);
                    }
                }
                Ok(Location::SIMD(tmp))
            }
            _ => codegen_error!("singlepass can't emit location_to_neon {:?} {:?}", sz, src),
        }
    }

    fn emit_relaxed_binop(
        &mut self,
        op: fn(&mut Assembler, Size, Location, Location) -> Result<(), CompileError>,
        sz: Size,
        src: Location,
        dst: Location,
        putback: bool,
    ) -> Result<(), CompileError> {
        let mut temps = vec![];
        let src_imm = if putback {
            ImmType::None
        } else {
            ImmType::Bits12
        };
        let src = self.location_to_reg(sz, src, &mut temps, src_imm, true, None)?;
        let dest = self.location_to_reg(sz, dst, &mut temps, ImmType::None, !putback, None)?;
        op(&mut self.assembler, sz, src, dest)?;
        if dst != dest && putback {
            self.move_location(sz, dest, dst)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn emit_relaxed_binop_neon(
        &mut self,
        op: fn(&mut Assembler, Size, Location, Location) -> Result<(), CompileError>,
        sz: Size,
        src: Location,
        dst: Location,
        putback: bool,
    ) -> Result<(), CompileError> {
        let mut temps = vec![];
        let src = self.location_to_neon(sz, src, &mut temps, ImmType::None, true)?;
        let dest = self.location_to_neon(sz, dst, &mut temps, ImmType::None, !putback)?;
        op(&mut self.assembler, sz, src, dest)?;
        if dst != dest && putback {
            self.move_location(sz, dest, dst)?;
        }
        for r in temps {
            self.release_simd(r);
        }
        Ok(())
    }
    fn emit_relaxed_binop3(
        &mut self,
        op: fn(&mut Assembler, Size, Location, Location, Location) -> Result<(), CompileError>,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
        allow_imm: ImmType,
    ) -> Result<(), CompileError> {
        let mut temps = vec![];
        let src1 = self.location_to_reg(sz, src1, &mut temps, ImmType::None, true, None)?;
        let src2 = self.location_to_reg(sz, src2, &mut temps, allow_imm, true, None)?;
        let dest = self.location_to_reg(sz, dst, &mut temps, ImmType::None, false, None)?;
        op(&mut self.assembler, sz, src1, src2, dest)?;
        if dst != dest {
            self.move_location(sz, dest, dst)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn emit_relaxed_binop3_neon(
        &mut self,
        op: fn(&mut Assembler, Size, Location, Location, Location) -> Result<(), CompileError>,
        sz: Size,
        src1: Location,
        src2: Location,
        dst: Location,
        allow_imm: ImmType,
    ) -> Result<(), CompileError> {
        let mut temps = vec![];
        let src1 = self.location_to_neon(sz, src1, &mut temps, ImmType::None, true)?;
        let src2 = self.location_to_neon(sz, src2, &mut temps, allow_imm, true)?;
        let dest = self.location_to_neon(sz, dst, &mut temps, ImmType::None, false)?;
        op(&mut self.assembler, sz, src1, src2, dest)?;
        if dst != dest {
            self.move_location(sz, dest, dst)?;
        }
        for r in temps {
            self.release_simd(r);
        }
        Ok(())
    }
    fn emit_relaxed_ldr64(
        &mut self,
        sz: Size,
        dst: Location,
        src: Location,
    ) -> Result<(), CompileError> {
        let mut temps = vec![];
        let dest = self.location_to_reg(sz, dst, &mut temps, ImmType::None, false, None)?;
        match src {
            Location::Memory(addr, offset) => {
                if self.compatible_imm(offset as i64, ImmType::OffsetDWord) {
                    self.assembler.emit_ldr(Size::S64, dest, src)?;
                } else if self.compatible_imm(offset as i64, ImmType::UnscaledOffset) {
                    self.assembler.emit_ldur(Size::S64, dest, addr, offset)?;
                } else {
                    let tmp = self.acquire_temp_gpr().ok_or_else(|| {
                        CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                    })?;
                    self.assembler
                        .emit_mov_imm(Location::GPR(tmp), (offset as i64) as u64)?;
                    self.assembler.emit_ldr(
                        Size::S64,
                        dest,
                        Location::Memory2(addr, tmp, Multiplier::One, 0),
                    )?;
                    temps.push(tmp);
                }
            }
            _ => codegen_error!("singplass emit_relaxed_ldr64 unreachable"),
        }
        if dst != dest {
            self.move_location(sz, dest, dst)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn emit_relaxed_ldr32(
        &mut self,
        sz: Size,
        dst: Location,
        src: Location,
    ) -> Result<(), CompileError> {
        let mut temps = vec![];
        let dest = self.location_to_reg(sz, dst, &mut temps, ImmType::None, false, None)?;
        match src {
            Location::Memory(addr, offset) => {
                if self.compatible_imm(offset as i64, ImmType::OffsetWord) {
                    self.assembler.emit_ldr(Size::S32, dest, src)?;
                } else if self.compatible_imm(offset as i64, ImmType::UnscaledOffset) {
                    self.assembler.emit_ldur(Size::S32, dest, addr, offset)?;
                } else {
                    let tmp = self.acquire_temp_gpr().ok_or_else(|| {
                        CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                    })?;
                    self.assembler
                        .emit_mov_imm(Location::GPR(tmp), (offset as i64) as u64)?;
                    self.assembler.emit_ldr(
                        Size::S32,
                        dest,
                        Location::Memory2(addr, tmp, Multiplier::One, 0),
                    )?;
                    temps.push(tmp);
                }
            }
            _ => codegen_error!("singlepass emit_relaxed_ldr32 unreachable"),
        }
        if dst != dest {
            self.move_location(sz, dest, dst)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn emit_relaxed_ldr32s(
        &mut self,
        sz: Size,
        dst: Location,
        src: Location,
    ) -> Result<(), CompileError> {
        let mut temps = vec![];
        let dest = self.location_to_reg(sz, dst, &mut temps, ImmType::None, false, None)?;
        match src {
            Location::Memory(addr, offset) => {
                if self.compatible_imm(offset as i64, ImmType::OffsetWord) {
                    self.assembler.emit_ldrsw(Size::S64, dest, src)?;
                } else {
                    let tmp = self.acquire_temp_gpr().ok_or_else(|| {
                        CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                    })?;
                    self.assembler
                        .emit_mov_imm(Location::GPR(tmp), (offset as i64) as u64)?;
                    self.assembler.emit_ldrsw(
                        Size::S64,
                        dest,
                        Location::Memory2(addr, tmp, Multiplier::One, 0),
                    )?;
                    temps.push(tmp);
                }
            }
            _ => codegen_error!("singplepass emit_relaxed_ldr32s unreachable"),
        }
        if dst != dest {
            self.move_location(sz, dest, dst)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn emit_relaxed_ldr16(
        &mut self,
        sz: Size,
        dst: Location,
        src: Location,
    ) -> Result<(), CompileError> {
        let mut temps = vec![];
        let dest = self.location_to_reg(sz, dst, &mut temps, ImmType::None, false, None)?;
        match src {
            Location::Memory(addr, offset) => {
                if self.compatible_imm(offset as i64, ImmType::OffsetHWord) {
                    self.assembler.emit_ldrh(Size::S32, dest, src)?;
                } else {
                    let tmp = self.acquire_temp_gpr().ok_or_else(|| {
                        CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                    })?;
                    self.assembler
                        .emit_mov_imm(Location::GPR(tmp), (offset as i64) as u64)?;
                    self.assembler.emit_ldrh(
                        Size::S32,
                        dest,
                        Location::Memory2(addr, tmp, Multiplier::One, 0),
                    )?;
                    temps.push(tmp);
                }
            }
            _ => codegen_error!("singlpass emit_relaxed_ldr16 unreachable"),
        }
        if dst != dest {
            self.move_location(sz, dest, dst)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn emit_relaxed_ldr16s(
        &mut self,
        sz: Size,
        dst: Location,
        src: Location,
    ) -> Result<(), CompileError> {
        let mut temps = vec![];
        let dest = self.location_to_reg(sz, dst, &mut temps, ImmType::None, false, None)?;
        match src {
            Location::Memory(addr, offset) => {
                if self.compatible_imm(offset as i64, ImmType::OffsetHWord) {
                    self.assembler.emit_ldrsh(sz, dest, src)?;
                } else {
                    let tmp = self.acquire_temp_gpr().ok_or_else(|| {
                        CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                    })?;
                    self.assembler
                        .emit_mov_imm(Location::GPR(tmp), (offset as i64) as u64)?;
                    self.assembler.emit_ldrsh(
                        sz,
                        dest,
                        Location::Memory2(addr, tmp, Multiplier::One, 0),
                    )?;
                    temps.push(tmp);
                }
            }
            _ => codegen_error!("singlepass emit_relaxed_ldr16s unreachable"),
        }
        if dst != dest {
            self.move_location(sz, dest, dst)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn emit_relaxed_ldr8(
        &mut self,
        sz: Size,
        dst: Location,
        src: Location,
    ) -> Result<(), CompileError> {
        let mut temps = vec![];
        let dest = self.location_to_reg(sz, dst, &mut temps, ImmType::None, false, None)?;
        match src {
            Location::Memory(addr, offset) => {
                if self.compatible_imm(offset as i64, ImmType::OffsetByte) {
                    self.assembler.emit_ldrb(Size::S32, dest, src)?;
                } else {
                    let tmp = self.acquire_temp_gpr().ok_or_else(|| {
                        CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                    })?;
                    self.assembler
                        .emit_mov_imm(Location::GPR(tmp), (offset as i64) as u64)?;
                    self.assembler.emit_ldrb(
                        Size::S32,
                        dest,
                        Location::Memory2(addr, tmp, Multiplier::One, 0),
                    )?;
                    temps.push(tmp);
                }
            }
            _ => codegen_error!("singplepass emit_relaxed_ldr8 unreachable"),
        }
        if dst != dest {
            self.move_location(sz, dest, dst)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn emit_relaxed_ldr8s(
        &mut self,
        sz: Size,
        dst: Location,
        src: Location,
    ) -> Result<(), CompileError> {
        let mut temps = vec![];
        let dest = self.location_to_reg(sz, dst, &mut temps, ImmType::None, false, None)?;
        match src {
            Location::Memory(addr, offset) => {
                if self.compatible_imm(offset as i64, ImmType::OffsetByte) {
                    self.assembler.emit_ldrsb(sz, dest, src)?;
                } else {
                    let tmp = self.acquire_temp_gpr().ok_or_else(|| {
                        CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                    })?;
                    self.assembler
                        .emit_mov_imm(Location::GPR(tmp), (offset as i64) as u64)?;
                    self.assembler.emit_ldrsb(
                        sz,
                        dest,
                        Location::Memory2(addr, tmp, Multiplier::One, 0),
                    )?;
                    temps.push(tmp);
                }
            }
            _ => codegen_error!("singlepass emit_relaxed_ldr8s unreachable"),
        }
        if dst != dest {
            self.move_location(sz, dest, dst)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn emit_relaxed_str64(&mut self, dst: Location, src: Location) -> Result<(), CompileError> {
        let mut temps = vec![];
        let dst = self.location_to_reg(Size::S64, dst, &mut temps, ImmType::NoneXzr, true, None)?;
        match src {
            Location::Memory(addr, offset) => {
                if self.compatible_imm(offset as i64, ImmType::OffsetDWord) {
                    self.assembler.emit_str(Size::S64, dst, src)?;
                } else if self.compatible_imm(offset as i64, ImmType::UnscaledOffset) {
                    self.assembler.emit_stur(Size::S64, dst, addr, offset)?;
                } else {
                    let tmp = self.acquire_temp_gpr().ok_or_else(|| {
                        CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                    })?;
                    self.assembler
                        .emit_mov_imm(Location::GPR(tmp), (offset as i64) as u64)?;
                    self.assembler.emit_str(
                        Size::S64,
                        dst,
                        Location::Memory2(addr, tmp, Multiplier::One, 0),
                    )?;
                    temps.push(tmp);
                }
            }
            _ => codegen_error!("singlepass can't emit str64 {:?} {:?}", dst, src),
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn emit_relaxed_str32(&mut self, dst: Location, src: Location) -> Result<(), CompileError> {
        let mut temps = vec![];
        let dst = self.location_to_reg(Size::S64, dst, &mut temps, ImmType::NoneXzr, true, None)?;
        match src {
            Location::Memory(addr, offset) => {
                if self.compatible_imm(offset as i64, ImmType::OffsetWord) {
                    self.assembler.emit_str(Size::S32, dst, src)?;
                } else if self.compatible_imm(offset as i64, ImmType::UnscaledOffset) {
                    self.assembler.emit_stur(Size::S32, dst, addr, offset)?;
                } else {
                    let tmp = self.acquire_temp_gpr().ok_or_else(|| {
                        CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                    })?;
                    self.assembler
                        .emit_mov_imm(Location::GPR(tmp), (offset as i64) as u64)?;
                    self.assembler.emit_str(
                        Size::S32,
                        dst,
                        Location::Memory2(addr, tmp, Multiplier::One, 0),
                    )?;
                    temps.push(tmp);
                }
            }
            _ => codegen_error!("singplepass emit_relaxed_str32 unreachable"),
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn emit_relaxed_str16(&mut self, dst: Location, src: Location) -> Result<(), CompileError> {
        let mut temps = vec![];
        let dst = self.location_to_reg(Size::S64, dst, &mut temps, ImmType::NoneXzr, true, None)?;
        match src {
            Location::Memory(addr, offset) => {
                if self.compatible_imm(offset as i64, ImmType::OffsetHWord) {
                    self.assembler.emit_strh(Size::S32, dst, src)?;
                } else {
                    let tmp = self.acquire_temp_gpr().ok_or_else(|| {
                        CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                    })?;
                    self.assembler
                        .emit_mov_imm(Location::GPR(tmp), (offset as i64) as u64)?;
                    self.assembler.emit_strh(
                        Size::S32,
                        dst,
                        Location::Memory2(addr, tmp, Multiplier::One, 0),
                    )?;
                    temps.push(tmp);
                }
            }
            _ => codegen_error!("singlepass emit_relaxed_str16 unreachable"),
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn emit_relaxed_str8(&mut self, dst: Location, src: Location) -> Result<(), CompileError> {
        let mut temps = vec![];
        let dst = self.location_to_reg(Size::S64, dst, &mut temps, ImmType::NoneXzr, true, None)?;
        match src {
            Location::Memory(addr, offset) => {
                if self.compatible_imm(offset as i64, ImmType::OffsetByte) {
                    self.assembler
                        .emit_strb(Size::S32, dst, Location::Memory(addr, offset))?;
                } else {
                    let tmp = self.acquire_temp_gpr().ok_or_else(|| {
                        CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                    })?;
                    self.assembler
                        .emit_mov_imm(Location::GPR(tmp), (offset as i64) as u64)?;
                    self.assembler.emit_strb(
                        Size::S32,
                        dst,
                        Location::Memory2(addr, tmp, Multiplier::One, 0),
                    )?;
                    temps.push(tmp);
                }
            }
            _ => codegen_error!("singlepass emit_relaxed_str8 unreachable"),
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    /// I64 comparison with.
    fn emit_cmpop_i64_dynamic_b(
        &mut self,
        c: Condition,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        match ret {
            Location::GPR(_) => {
                self.emit_relaxed_cmp(Size::S64, loc_b, loc_a)?;
                self.assembler.emit_cset(Size::S32, ret, c)?;
            }
            Location::Memory(_, _) => {
                let tmp = self.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                self.emit_relaxed_cmp(Size::S64, loc_b, loc_a)?;
                self.assembler.emit_cset(Size::S32, Location::GPR(tmp), c)?;
                self.move_location(Size::S32, Location::GPR(tmp), ret)?;
                self.release_gpr(tmp);
            }
            _ => {
                codegen_error!("singlepass emit_compop_i64_dynamic_b unreachable");
            }
        }
        Ok(())
    }
    /// I32 comparison with.
    fn emit_cmpop_i32_dynamic_b(
        &mut self,
        c: Condition,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        match ret {
            Location::GPR(_) => {
                self.emit_relaxed_cmp(Size::S32, loc_b, loc_a)?;
                self.assembler.emit_cset(Size::S32, ret, c)?;
            }
            Location::Memory(_, _) => {
                let tmp = self.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                self.emit_relaxed_cmp(Size::S32, loc_b, loc_a)?;
                self.assembler.emit_cset(Size::S32, Location::GPR(tmp), c)?;
                self.move_location(Size::S32, Location::GPR(tmp), ret)?;
                self.release_gpr(tmp);
            }
            _ => {
                codegen_error!("singlepass emit_cmpop_i32_dynamic_b unreachable");
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn memory_op<F: FnOnce(&mut Self, GPR) -> Result<(), CompileError>>(
        &mut self,
        addr: Location,
        memarg: &MemArg,
        check_alignment: bool,
        value_size: usize,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
        cb: F,
    ) -> Result<(), CompileError> {
        let tmp_addr = self.acquire_temp_gpr().ok_or_else(|| {
            CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
        })?;

        // Reusing `tmp_addr` for temporary indirection here, since it's not used before the last reference to `{base,bound}_loc`.
        let (base_loc, bound_loc) = if imported_memories {
            // Imported memories require one level of indirection.
            self.emit_relaxed_binop(
                Assembler::emit_mov,
                Size::S64,
                Location::Memory(self.get_vmctx_reg(), offset),
                Location::GPR(tmp_addr),
                true,
            )?;
            (Location::Memory(tmp_addr, 0), Location::Memory(tmp_addr, 8))
        } else {
            (
                Location::Memory(self.get_vmctx_reg(), offset),
                Location::Memory(self.get_vmctx_reg(), offset + 8),
            )
        };

        let tmp_base = self.acquire_temp_gpr().ok_or_else(|| {
            CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
        })?;
        let tmp_bound = self.acquire_temp_gpr().ok_or_else(|| {
            CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
        })?;

        // Load base into temporary register.
        self.emit_relaxed_ldr64(Size::S64, Location::GPR(tmp_base), base_loc)?;

        // Load bound into temporary register, if needed.
        if need_check {
            self.emit_relaxed_ldr64(Size::S64, Location::GPR(tmp_bound), bound_loc)?;

            // Wasm -> Effective.
            // Assuming we never underflow - should always be true on Linux/macOS and Windows >=8,
            // since the first page from 0x0 to 0x1000 is not accepted by mmap.
            self.assembler.emit_add(
                Size::S64,
                Location::GPR(tmp_bound),
                Location::GPR(tmp_base),
                Location::GPR(tmp_bound),
            )?;
            if self.compatible_imm(value_size as _, ImmType::Bits12) {
                self.assembler.emit_sub(
                    Size::S64,
                    Location::GPR(tmp_bound),
                    Location::Imm32(value_size as _),
                    Location::GPR(tmp_bound),
                )?;
            } else {
                let tmp2 = self.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                self.assembler
                    .emit_mov_imm(Location::GPR(tmp2), value_size as u64)?;
                self.assembler.emit_sub(
                    Size::S64,
                    Location::GPR(tmp_bound),
                    Location::GPR(tmp2),
                    Location::GPR(tmp_bound),
                )?;
                self.release_gpr(tmp2);
            }
        }

        // Load effective address.
        // `base_loc` and `bound_loc` becomes INVALID after this line, because `tmp_addr`
        // might be reused.
        self.move_location(Size::S32, addr, Location::GPR(tmp_addr))?;

        // Add offset to memory address.
        if memarg.offset != 0 {
            if self.compatible_imm(memarg.offset as _, ImmType::Bits12) {
                self.assembler.emit_adds(
                    Size::S32,
                    Location::Imm32(memarg.offset as u32),
                    Location::GPR(tmp_addr),
                    Location::GPR(tmp_addr),
                )?;
            } else {
                let tmp = self.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                self.assembler
                    .emit_mov_imm(Location::GPR(tmp), memarg.offset as _)?;
                self.assembler.emit_adds(
                    Size::S32,
                    Location::GPR(tmp_addr),
                    Location::GPR(tmp),
                    Location::GPR(tmp_addr),
                )?;
                self.release_gpr(tmp);
            }

            // Trap if offset calculation overflowed.
            self.assembler
                .emit_bcond_label_far(Condition::Cs, heap_access_oob)?;
        }

        // Wasm linear memory -> real memory
        self.assembler.emit_add(
            Size::S64,
            Location::GPR(tmp_base),
            Location::GPR(tmp_addr),
            Location::GPR(tmp_addr),
        )?;

        if need_check {
            // Trap if the end address of the requested area is above that of the linear memory.
            self.assembler.emit_cmp(
                Size::S64,
                Location::GPR(tmp_bound),
                Location::GPR(tmp_addr),
            )?;

            // `tmp_bound` is inclusive. So trap only if `tmp_addr > tmp_bound`.
            self.assembler
                .emit_bcond_label_far(Condition::Hi, heap_access_oob)?;
        }

        self.release_gpr(tmp_bound);
        self.release_gpr(tmp_base);

        let align = value_size as u32;
        if check_alignment && align != 1 {
            self.assembler.emit_tst(
                Size::S64,
                Location::Imm32(align - 1),
                Location::GPR(tmp_addr),
            )?;
            self.assembler
                .emit_bcond_label_far(Condition::Ne, unaligned_atomic)?;
        }
        let begin = self.assembler.get_offset().0;
        cb(self, tmp_addr)?;
        let end = self.assembler.get_offset().0;
        self.mark_address_range_with_trap_code(TrapCode::HeapAccessOutOfBounds, begin, end);

        self.release_gpr(tmp_addr);
        Ok(())
    }

    /*fn emit_compare_and_swap<F: FnOnce(&mut Self, GPR, GPR)>(
        &mut self,
        _loc: Location,
        _target: Location,
        _ret: Location,
        _memarg: &MemArg,
        _value_size: usize,
        _memory_sz: Size,
        _stack_sz: Size,
        _need_check: bool,
        _imported_memories: bool,
        _offset: i32,
        _heap_access_oob: Label,
        _unaligned_atomic: Label,
        _cb: F,
    ) {
        unimplemented!();
    }*/

    fn offset_is_ok(&self, size: Size, offset: i32) -> bool {
        if offset < 0 {
            return false;
        }
        let shift = match size {
            Size::S8 => 0,
            Size::S16 => 1,
            Size::S32 => 2,
            Size::S64 => 3,
        };
        if offset >= 0x1000 << shift {
            return false;
        }
        if (offset & ((1 << shift) - 1)) != 0 {
            return false;
        }
        true
    }

    fn emit_push(&mut self, sz: Size, src: Location) -> Result<(), CompileError> {
        match (sz, src) {
            (Size::S64, Location::GPR(_)) | (Size::S64, Location::SIMD(_)) => {
                let offset = if self.pushed {
                    0
                } else {
                    self.assembler.emit_sub(
                        Size::S64,
                        Location::GPR(GPR::XzrSp),
                        Location::Imm8(16),
                        Location::GPR(GPR::XzrSp),
                    )?;
                    8
                };
                self.assembler
                    .emit_stur(Size::S64, src, GPR::XzrSp, offset)?;
                self.pushed = !self.pushed;
            }
            (Size::S64, _) => {
                let mut temps = vec![];
                let src = self.location_to_reg(sz, src, &mut temps, ImmType::None, true, None)?;
                let offset = if self.pushed {
                    0
                } else {
                    self.assembler.emit_sub(
                        Size::S64,
                        Location::GPR(GPR::XzrSp),
                        Location::Imm8(16),
                        Location::GPR(GPR::XzrSp),
                    )?;
                    8
                };
                self.assembler
                    .emit_stur(Size::S64, src, GPR::XzrSp, offset)?;
                self.pushed = !self.pushed;
                for r in temps {
                    self.release_gpr(r);
                }
            }
            _ => codegen_error!("singlepass can't emit PUSH {:?} {:?}", sz, src),
        }
        Ok(())
    }
    fn emit_double_push(
        &mut self,
        sz: Size,
        src1: Location,
        src2: Location,
    ) -> Result<(), CompileError> {
        if !self.pushed {
            match (sz, src1, src2) {
                (Size::S64, Location::GPR(_), Location::GPR(_)) => {
                    self.assembler
                        .emit_stpdb(Size::S64, src1, src2, GPR::XzrSp, 16)?;
                }
                _ => {
                    self.emit_push(sz, src1)?;
                    self.emit_push(sz, src2)?;
                }
            }
        } else {
            self.emit_push(sz, src1)?;
            self.emit_push(sz, src2)?;
        }
        Ok(())
    }
    fn emit_pop(&mut self, sz: Size, dst: Location) -> Result<(), CompileError> {
        match (sz, dst) {
            (Size::S64, Location::GPR(_)) | (Size::S64, Location::SIMD(_)) => {
                let offset = if self.pushed { 8 } else { 0 };
                self.assembler
                    .emit_ldur(Size::S64, dst, GPR::XzrSp, offset)?;
                if self.pushed {
                    self.assembler.emit_add(
                        Size::S64,
                        Location::GPR(GPR::XzrSp),
                        Location::Imm8(16),
                        Location::GPR(GPR::XzrSp),
                    )?;
                }
                self.pushed = !self.pushed;
            }
            _ => codegen_error!("singlepass can't emit PUSH {:?} {:?}", sz, dst),
        }
        Ok(())
    }
    fn emit_double_pop(
        &mut self,
        sz: Size,
        dst1: Location,
        dst2: Location,
    ) -> Result<(), CompileError> {
        if !self.pushed {
            match (sz, dst1, dst2) {
                (Size::S64, Location::GPR(_), Location::GPR(_)) => {
                    self.assembler
                        .emit_ldpia(Size::S64, dst1, dst2, GPR::XzrSp, 16)?;
                }
                _ => {
                    self.emit_pop(sz, dst2)?;
                    self.emit_pop(sz, dst1)?;
                }
            }
        } else {
            self.emit_pop(sz, dst2)?;
            self.emit_pop(sz, dst1)?;
        }
        Ok(())
    }

    fn set_default_nan(&mut self, temps: &mut Vec<GPR>) -> Result<GPR, CompileError> {
        // temporarly set FPCR to DefaultNan
        let old_fpcr = self.acquire_temp_gpr().ok_or_else(|| {
            CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
        })?;
        temps.push(old_fpcr);
        self.assembler.emit_read_fpcr(old_fpcr)?;
        let new_fpcr = self.acquire_temp_gpr().ok_or_else(|| {
            CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
        })?;
        temps.push(new_fpcr);
        let tmp = self.acquire_temp_gpr().ok_or_else(|| {
            CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
        })?;
        temps.push(tmp);
        self.assembler
            .emit_mov(Size::S32, Location::Imm32(1), Location::GPR(tmp))?;
        self.assembler
            .emit_mov(Size::S64, Location::GPR(old_fpcr), Location::GPR(new_fpcr))?;
        // DN is bit 25 of FPCR
        self.assembler.emit_bfi(
            Size::S64,
            Location::GPR(tmp),
            25,
            1,
            Location::GPR(new_fpcr),
        )?;
        self.assembler.emit_write_fpcr(new_fpcr)?;
        Ok(old_fpcr)
    }
    fn set_trap_enabled(&mut self, temps: &mut Vec<GPR>) -> Result<GPR, CompileError> {
        // temporarly set FPCR to DefaultNan
        let old_fpcr = self.acquire_temp_gpr().ok_or_else(|| {
            CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
        })?;
        temps.push(old_fpcr);
        self.assembler.emit_read_fpcr(old_fpcr)?;
        let new_fpcr = self.acquire_temp_gpr().ok_or_else(|| {
            CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
        })?;
        temps.push(new_fpcr);
        self.assembler
            .emit_mov(Size::S64, Location::GPR(old_fpcr), Location::GPR(new_fpcr))?;
        // IOE is bit 8 of FPCR
        self.assembler
            .emit_bfc(Size::S64, 8, 1, Location::GPR(new_fpcr))?;
        self.assembler.emit_write_fpcr(new_fpcr)?;
        Ok(old_fpcr)
    }
    fn restore_fpcr(&mut self, old_fpcr: GPR) -> Result<(), CompileError> {
        self.assembler.emit_write_fpcr(old_fpcr)
    }

    fn reset_exception_fpsr(&mut self) -> Result<(), CompileError> {
        // reset exception count in FPSR
        let fpsr = self.acquire_temp_gpr().ok_or_else(|| {
            CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
        })?;
        self.assembler.emit_read_fpsr(fpsr)?;
        // IOC is 0
        self.assembler
            .emit_bfc(Size::S64, 0, 1, Location::GPR(fpsr))?;
        self.assembler.emit_write_fpsr(fpsr)?;
        self.release_gpr(fpsr);
        Ok(())
    }
    fn read_fpsr(&mut self) -> Result<GPR, CompileError> {
        let fpsr = self.acquire_temp_gpr().ok_or_else(|| {
            CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
        })?;
        self.assembler.emit_read_fpsr(fpsr)?;
        Ok(fpsr)
    }

    fn trap_float_convertion_errors(
        &mut self,
        old_fpcr: GPR,
        sz: Size,
        f: Location,
        temps: &mut Vec<GPR>,
    ) -> Result<(), CompileError> {
        let trap_badconv = self.assembler.get_label();
        let end = self.assembler.get_label();

        let fpsr = self.read_fpsr()?;
        temps.push(fpsr);
        // no trap, than all good
        self.assembler
            .emit_tbz_label(Size::S32, Location::GPR(fpsr), 0, end)?;
        // now need to check if it's overflow or NaN
        self.assembler
            .emit_bfc(Size::S64, 0, 4, Location::GPR(fpsr))?;
        self.restore_fpcr(old_fpcr)?;
        self.assembler.emit_fcmp(sz, f, f)?;
        self.assembler
            .emit_bcond_label(Condition::Vs, trap_badconv)?;
        // fallthru: trap_overflow
        self.emit_illegal_op_internal(TrapCode::IntegerOverflow)?;

        self.emit_label(trap_badconv)?;
        self.emit_illegal_op_internal(TrapCode::BadConversionToInteger)?;

        self.emit_label(end)?;
        self.restore_fpcr(old_fpcr)
    }

    fn used_gprs_contains(&self, r: &GPR) -> bool {
        self.used_gprs & (1 << r.into_index()) != 0
    }
    fn used_simd_contains(&self, r: &NEON) -> bool {
        self.used_simd & (1 << r.into_index()) != 0
    }
    fn used_gprs_insert(&mut self, r: GPR) {
        self.used_gprs |= 1 << r.into_index();
    }
    fn used_simd_insert(&mut self, r: NEON) {
        self.used_simd |= 1 << r.into_index();
    }
    fn used_gprs_remove(&mut self, r: &GPR) -> bool {
        let ret = self.used_gprs_contains(r);
        self.used_gprs &= !(1 << r.into_index());
        ret
    }
    fn used_simd_remove(&mut self, r: &NEON) -> bool {
        let ret = self.used_simd_contains(r);
        self.used_simd &= !(1 << r.into_index());
        ret
    }
    fn emit_unwind_op(&mut self, op: UnwindOps) {
        self.unwind_ops.push((self.get_offset().0, op));
    }
    fn emit_illegal_op_internal(&mut self, trap: TrapCode) -> Result<(), CompileError> {
        self.assembler.emit_udf(0xc0 | (trap as u8) as u16)
    }
}

impl Machine for MachineARM64 {
    type GPR = GPR;
    type SIMD = NEON;
    fn assembler_get_offset(&self) -> Offset {
        self.assembler.get_offset()
    }
    fn index_from_gpr(&self, x: GPR) -> RegisterIndex {
        RegisterIndex(x as usize)
    }
    fn index_from_simd(&self, x: NEON) -> RegisterIndex {
        RegisterIndex(x as usize + 32)
    }

    fn get_vmctx_reg(&self) -> GPR {
        GPR::X28
    }

    fn get_used_gprs(&self) -> Vec<GPR> {
        GPR::iterator()
            .filter(|x| self.used_gprs & (1 << x.into_index()) != 0)
            .cloned()
            .collect()
    }

    fn get_used_simd(&self) -> Vec<NEON> {
        NEON::iterator()
            .filter(|x| self.used_simd & (1 << x.into_index()) != 0)
            .cloned()
            .collect()
    }

    fn pick_gpr(&self) -> Option<GPR> {
        use GPR::*;
        static REGS: &[GPR] = &[X9, X10, X11, X12, X13, X14, X15];
        for r in REGS {
            if !self.used_gprs_contains(r) {
                return Some(*r);
            }
        }
        None
    }

    // Picks an unused general purpose register for internal temporary use.
    fn pick_temp_gpr(&self) -> Option<GPR> {
        use GPR::*;
        static REGS: &[GPR] = &[X8, X7, X6, X5, X4, X3, X2, X1];
        for r in REGS {
            if !self.used_gprs_contains(r) {
                return Some(*r);
            }
        }
        None
    }

    fn acquire_temp_gpr(&mut self) -> Option<GPR> {
        let gpr = self.pick_temp_gpr();
        if let Some(x) = gpr {
            self.used_gprs_insert(x);
        }
        gpr
    }

    fn release_gpr(&mut self, gpr: GPR) {
        assert!(self.used_gprs_remove(&gpr));
    }

    fn reserve_unused_temp_gpr(&mut self, gpr: GPR) -> GPR {
        assert!(!self.used_gprs_contains(&gpr));
        self.used_gprs_insert(gpr);
        gpr
    }

    fn reserve_gpr(&mut self, gpr: GPR) {
        self.used_gprs_insert(gpr);
    }

    fn push_used_gpr(&mut self, used_gprs: &[GPR]) -> Result<usize, CompileError> {
        if used_gprs.len() % 2 == 1 {
            self.emit_push(Size::S64, Location::GPR(GPR::XzrSp))?;
        }
        for r in used_gprs.iter() {
            self.emit_push(Size::S64, Location::GPR(*r))?;
        }
        Ok(((used_gprs.len() + 1) / 2) * 16)
    }
    fn pop_used_gpr(&mut self, used_gprs: &[GPR]) -> Result<(), CompileError> {
        for r in used_gprs.iter().rev() {
            self.emit_pop(Size::S64, Location::GPR(*r))?;
        }
        if used_gprs.len() % 2 == 1 {
            self.emit_pop(Size::S64, Location::GPR(GPR::XzrSp))?;
        }
        Ok(())
    }

    // Picks an unused NEON register.
    fn pick_simd(&self) -> Option<NEON> {
        use NEON::*;
        static REGS: &[NEON] = &[V8, V9, V10, V11, V12];
        for r in REGS {
            if !self.used_simd_contains(r) {
                return Some(*r);
            }
        }
        None
    }

    // Picks an unused NEON register for internal temporary use.
    fn pick_temp_simd(&self) -> Option<NEON> {
        use NEON::*;
        static REGS: &[NEON] = &[V0, V1, V2, V3, V4, V5, V6, V7];
        for r in REGS {
            if !self.used_simd_contains(r) {
                return Some(*r);
            }
        }
        None
    }

    // Acquires a temporary NEON register.
    fn acquire_temp_simd(&mut self) -> Option<NEON> {
        let simd = self.pick_temp_simd();
        if let Some(x) = simd {
            self.used_simd_insert(x);
        }
        simd
    }

    fn reserve_simd(&mut self, simd: NEON) {
        self.used_simd_insert(simd);
    }

    // Releases a temporary NEON register.
    fn release_simd(&mut self, simd: NEON) {
        assert!(self.used_simd_remove(&simd));
    }

    fn push_used_simd(&mut self, used_neons: &[NEON]) -> Result<usize, CompileError> {
        let stack_adjust = if used_neons.len() & 1 == 1 {
            (used_neons.len() * 8) as u32 + 8
        } else {
            (used_neons.len() * 8) as u32
        };
        self.adjust_stack(stack_adjust)?;

        for (i, r) in used_neons.iter().enumerate() {
            self.assembler.emit_str(
                Size::S64,
                Location::SIMD(*r),
                Location::Memory(GPR::XzrSp, (i * 8) as i32),
            )?;
        }
        Ok(stack_adjust as usize)
    }
    fn pop_used_simd(&mut self, used_neons: &[NEON]) -> Result<(), CompileError> {
        for (i, r) in used_neons.iter().enumerate() {
            self.assembler.emit_ldr(
                Size::S64,
                Location::SIMD(*r),
                Location::Memory(GPR::XzrSp, (i * 8) as i32),
            )?;
        }
        let stack_adjust = if used_neons.len() & 1 == 1 {
            (used_neons.len() * 8) as u32 + 8
        } else {
            (used_neons.len() * 8) as u32
        };
        self.assembler.emit_add(
            Size::S64,
            Location::GPR(GPR::XzrSp),
            Location::Imm32(stack_adjust as _),
            Location::GPR(GPR::XzrSp),
        )
    }

    /// Set the source location of the Wasm to the given offset.
    fn set_srcloc(&mut self, offset: u32) {
        self.src_loc = offset;
    }
    /// Marks each address in the code range emitted by `f` with the trap code `code`.
    fn mark_address_range_with_trap_code(&mut self, code: TrapCode, begin: usize, end: usize) {
        for i in begin..end {
            self.trap_table.offset_to_code.insert(i, code);
        }
        self.mark_instruction_address_end(begin);
    }

    /// Marks one address as trappable with trap code `code`.
    fn mark_address_with_trap_code(&mut self, code: TrapCode) {
        let offset = self.assembler.get_offset().0;
        self.trap_table.offset_to_code.insert(offset, code);
        self.mark_instruction_address_end(offset);
    }
    /// Marks the instruction as trappable with trap code `code`. return "begin" offset
    fn mark_instruction_with_trap_code(&mut self, code: TrapCode) -> usize {
        let offset = self.assembler.get_offset().0;
        self.trap_table.offset_to_code.insert(offset, code);
        offset
    }
    /// Pushes the instruction to the address map, calculating the offset from a
    /// provided beginning address.
    fn mark_instruction_address_end(&mut self, begin: usize) {
        self.instructions_address_map.push(InstructionAddressMap {
            srcloc: SourceLoc::new(self.src_loc),
            code_offset: begin,
            code_len: self.assembler.get_offset().0 - begin,
        });
    }

    /// Insert a StackOverflow (at offset 0)
    fn insert_stackoverflow(&mut self) {
        let offset = 0;
        self.trap_table
            .offset_to_code
            .insert(offset, TrapCode::StackOverflow);
        self.mark_instruction_address_end(offset);
    }

    /// Get all current TrapInformation
    fn collect_trap_information(&self) -> Vec<TrapInformation> {
        self.trap_table
            .offset_to_code
            .clone()
            .into_iter()
            .map(|(offset, code)| TrapInformation {
                code_offset: offset as u32,
                trap_code: code,
            })
            .collect()
    }

    fn instructions_address_map(&self) -> Vec<InstructionAddressMap> {
        self.instructions_address_map.clone()
    }

    // Return a rounded stack adjustement value (must be multiple of 16bytes on ARM64 for example)
    fn round_stack_adjust(&self, value: usize) -> usize {
        if value & 0xf != 0 {
            ((value >> 4) + 1) << 4
        } else {
            value
        }
    }

    // Memory location for a local on the stack
    fn local_on_stack(&mut self, stack_offset: i32) -> Location {
        Location::Memory(GPR::X29, -stack_offset)
    }

    // Adjust stack for locals
    fn adjust_stack(&mut self, delta_stack_offset: u32) -> Result<(), CompileError> {
        let delta = if self.compatible_imm(delta_stack_offset as _, ImmType::Bits12) {
            Location::Imm32(delta_stack_offset as _)
        } else {
            let tmp = GPR::X17;
            self.assembler
                .emit_mov_imm(Location::GPR(tmp), delta_stack_offset as u64)?;
            Location::GPR(tmp)
        };
        self.assembler.emit_sub(
            Size::S64,
            Location::GPR(GPR::XzrSp),
            delta,
            Location::GPR(GPR::XzrSp),
        )
    }
    // restore stack
    fn restore_stack(&mut self, delta_stack_offset: u32) -> Result<(), CompileError> {
        let delta = if self.compatible_imm(delta_stack_offset as _, ImmType::Bits12) {
            Location::Imm32(delta_stack_offset as _)
        } else {
            let tmp = GPR::X17;
            self.assembler
                .emit_mov_imm(Location::GPR(tmp), delta_stack_offset as u64)?;
            Location::GPR(tmp)
        };
        self.assembler.emit_add(
            Size::S64,
            Location::GPR(GPR::XzrSp),
            delta,
            Location::GPR(GPR::XzrSp),
        )
    }
    fn pop_stack_locals(&mut self, delta_stack_offset: u32) -> Result<(), CompileError> {
        let real_delta = if delta_stack_offset & 15 != 0 {
            delta_stack_offset + 8
        } else {
            delta_stack_offset
        };
        let delta = if self.compatible_imm(real_delta as i64, ImmType::Bits12) {
            Location::Imm32(real_delta as _)
        } else {
            let tmp = GPR::X17;
            self.assembler
                .emit_mov_imm(Location::GPR(tmp), real_delta as u64)?;
            Location::GPR(tmp)
        };
        self.assembler.emit_add(
            Size::S64,
            Location::GPR(GPR::XzrSp),
            delta,
            Location::GPR(GPR::XzrSp),
        )
    }
    // push a value on the stack for a native call
    fn move_location_for_native(
        &mut self,
        size: Size,
        loc: Location,
        dest: Location,
    ) -> Result<(), CompileError> {
        match loc {
            Location::Imm64(_)
            | Location::Imm32(_)
            | Location::Imm8(_)
            | Location::Memory(_, _)
            | Location::Memory2(_, _, _, _) => {
                self.move_location(size, loc, Location::GPR(GPR::X17))?;
                self.move_location(size, Location::GPR(GPR::X17), dest)
            }
            _ => self.move_location(size, loc, dest),
        }
    }

    // Zero a location that is 32bits
    fn zero_location(&mut self, size: Size, location: Location) -> Result<(), CompileError> {
        self.move_location(size, Location::GPR(GPR::XzrSp), location)
    }

    // GPR Reg used for local pointer on the stack
    fn local_pointer(&self) -> GPR {
        GPR::X29
    }

    // Determine whether a local should be allocated on the stack.
    fn is_local_on_stack(&self, idx: usize) -> bool {
        idx > 7
    }

    // Determine a local's location.
    fn get_local_location(&self, idx: usize, callee_saved_regs_size: usize) -> Location {
        // Use callee-saved registers for the first locals.
        match idx {
            0 => Location::GPR(GPR::X19),
            1 => Location::GPR(GPR::X20),
            2 => Location::GPR(GPR::X21),
            3 => Location::GPR(GPR::X22),
            4 => Location::GPR(GPR::X23),
            5 => Location::GPR(GPR::X24),
            6 => Location::GPR(GPR::X25),
            7 => Location::GPR(GPR::X26),
            _ => Location::Memory(GPR::X29, -(((idx - 7) * 8 + callee_saved_regs_size) as i32)),
        }
    }
    // Move a local to the stack
    fn move_local(&mut self, stack_offset: i32, location: Location) -> Result<(), CompileError> {
        if stack_offset < 256 {
            self.assembler
                .emit_stur(Size::S64, location, GPR::X29, -stack_offset)?;
        } else {
            let tmp = GPR::X17;
            if stack_offset < 0x1_0000 {
                self.assembler
                    .emit_mov_imm(Location::GPR(tmp), (-stack_offset as i64) as u64)?;
                self.assembler.emit_str(
                    Size::S64,
                    location,
                    Location::Memory2(GPR::X29, tmp, Multiplier::One, 0),
                )?;
            } else {
                self.assembler
                    .emit_mov_imm(Location::GPR(tmp), (stack_offset as i64) as u64)?;
                self.assembler.emit_sub(
                    Size::S64,
                    Location::GPR(GPR::X29),
                    Location::GPR(tmp),
                    Location::GPR(tmp),
                )?;
                self.assembler
                    .emit_str(Size::S64, location, Location::GPR(tmp))?;
            }
        }
        match location {
            Location::GPR(x) => self.emit_unwind_op(UnwindOps::SaveRegister {
                reg: x.to_dwarf(),
                bp_neg_offset: stack_offset,
            }),
            Location::SIMD(x) => self.emit_unwind_op(UnwindOps::SaveRegister {
                reg: x.to_dwarf(),
                bp_neg_offset: stack_offset,
            }),
            _ => (),
        }
        Ok(())
    }

    // List of register to save, depending on the CallingConvention
    fn list_to_save(&self, _calling_convention: CallingConvention) -> Vec<Location> {
        vec![]
    }

    // Get param location, MUST be called in order!
    fn get_param_location(
        &self,
        idx: usize,
        sz: Size,
        stack_args: &mut usize,
        calling_convention: CallingConvention,
    ) -> Location {
        match calling_convention {
            CallingConvention::AppleAarch64 => match idx {
                0 => Location::GPR(GPR::X0),
                1 => Location::GPR(GPR::X1),
                2 => Location::GPR(GPR::X2),
                3 => Location::GPR(GPR::X3),
                4 => Location::GPR(GPR::X4),
                5 => Location::GPR(GPR::X5),
                6 => Location::GPR(GPR::X6),
                7 => Location::GPR(GPR::X7),
                _ => {
                    let sz = 1
                        << match sz {
                            Size::S8 => 0,
                            Size::S16 => 1,
                            Size::S32 => 2,
                            Size::S64 => 3,
                        };
                    // align first
                    if sz > 1 && *stack_args & (sz - 1) != 0 {
                        *stack_args = (*stack_args + (sz - 1)) & !(sz - 1);
                    }
                    let loc = Location::Memory(GPR::XzrSp, *stack_args as i32);
                    *stack_args += sz;
                    loc
                }
            },
            _ => match idx {
                0 => Location::GPR(GPR::X0),
                1 => Location::GPR(GPR::X1),
                2 => Location::GPR(GPR::X2),
                3 => Location::GPR(GPR::X3),
                4 => Location::GPR(GPR::X4),
                5 => Location::GPR(GPR::X5),
                6 => Location::GPR(GPR::X6),
                7 => Location::GPR(GPR::X7),
                _ => {
                    let loc = Location::Memory(GPR::XzrSp, *stack_args as i32);
                    *stack_args += 8;
                    loc
                }
            },
        }
    }
    // Get call param location, MUST be called in order!
    fn get_call_param_location(
        &self,
        idx: usize,
        sz: Size,
        stack_args: &mut usize,
        calling_convention: CallingConvention,
    ) -> Location {
        match calling_convention {
            CallingConvention::AppleAarch64 => match idx {
                0 => Location::GPR(GPR::X0),
                1 => Location::GPR(GPR::X1),
                2 => Location::GPR(GPR::X2),
                3 => Location::GPR(GPR::X3),
                4 => Location::GPR(GPR::X4),
                5 => Location::GPR(GPR::X5),
                6 => Location::GPR(GPR::X6),
                7 => Location::GPR(GPR::X7),
                _ => {
                    let sz = 1
                        << match sz {
                            Size::S8 => 0,
                            Size::S16 => 1,
                            Size::S32 => 2,
                            Size::S64 => 3,
                        };
                    // align first
                    if sz > 1 && *stack_args & (sz - 1) != 0 {
                        *stack_args = (*stack_args + (sz - 1)) & !(sz - 1);
                    }
                    let loc = Location::Memory(GPR::X29, 16 * 2 + *stack_args as i32);
                    *stack_args += sz;
                    loc
                }
            },
            _ => match idx {
                0 => Location::GPR(GPR::X0),
                1 => Location::GPR(GPR::X1),
                2 => Location::GPR(GPR::X2),
                3 => Location::GPR(GPR::X3),
                4 => Location::GPR(GPR::X4),
                5 => Location::GPR(GPR::X5),
                6 => Location::GPR(GPR::X6),
                7 => Location::GPR(GPR::X7),
                _ => {
                    let loc = Location::Memory(GPR::X29, 16 * 2 + *stack_args as i32);
                    *stack_args += 8;
                    loc
                }
            },
        }
    }
    // Get simple param location, Will not be accurate for Apple calling convention on "stack" arguments
    fn get_simple_param_location(
        &self,
        idx: usize,
        calling_convention: CallingConvention,
    ) -> Location {
        #[allow(clippy::match_single_binding)]
        match calling_convention {
            _ => match idx {
                0 => Location::GPR(GPR::X0),
                1 => Location::GPR(GPR::X1),
                2 => Location::GPR(GPR::X2),
                3 => Location::GPR(GPR::X3),
                4 => Location::GPR(GPR::X4),
                5 => Location::GPR(GPR::X5),
                6 => Location::GPR(GPR::X6),
                7 => Location::GPR(GPR::X7),
                _ => Location::Memory(GPR::X29, (16 * 2 + (idx - 8) * 8) as i32),
            },
        }
    }
    // move a location to another
    fn move_location(
        &mut self,
        size: Size,
        source: Location,
        dest: Location,
    ) -> Result<(), CompileError> {
        match source {
            Location::GPR(_) | Location::SIMD(_) => match dest {
                Location::GPR(_) | Location::SIMD(_) => self.assembler.emit_mov(size, source, dest),
                Location::Memory(addr, offs) => {
                    if self.offset_is_ok(size, offs) {
                        self.assembler.emit_str(size, source, dest)
                    } else if self.compatible_imm(offs as i64, ImmType::UnscaledOffset) {
                        self.assembler.emit_stur(size, source, addr, offs)
                    } else {
                        let tmp = GPR::X17;
                        if offs < 0 {
                            self.assembler
                                .emit_mov_imm(Location::GPR(tmp), (-offs) as u64)?;
                            self.assembler.emit_sub(
                                Size::S64,
                                Location::GPR(addr),
                                Location::GPR(tmp),
                                Location::GPR(tmp),
                            )?;
                        } else {
                            self.assembler
                                .emit_mov_imm(Location::GPR(tmp), offs as u64)?;
                            self.assembler.emit_add(
                                Size::S64,
                                Location::GPR(addr),
                                Location::GPR(tmp),
                                Location::GPR(tmp),
                            )?;
                        }
                        self.assembler
                            .emit_str(size, source, Location::Memory(tmp, 0))
                    }
                }
                _ => codegen_error!(
                    "singlepass can't emit move_location {:?} {:?} => {:?}",
                    size,
                    source,
                    dest
                ),
            },
            Location::Imm8(_) => match dest {
                Location::GPR(_) => self.assembler.emit_mov(size, source, dest),
                Location::Memory(_, _) => match size {
                    Size::S64 => self.emit_relaxed_str64(source, dest),
                    Size::S32 => self.emit_relaxed_str32(source, dest),
                    Size::S16 => self.emit_relaxed_str16(source, dest),
                    Size::S8 => self.emit_relaxed_str8(source, dest),
                },
                _ => codegen_error!(
                    "singlepass can't emit move_location {:?} {:?} => {:?}",
                    size,
                    source,
                    dest
                ),
            },
            Location::Imm32(val) => match dest {
                Location::GPR(_) => self.assembler.emit_mov_imm(dest, val as u64),
                Location::Memory(_, _) => match size {
                    Size::S64 => self.emit_relaxed_str64(source, dest),
                    Size::S32 => self.emit_relaxed_str32(source, dest),
                    Size::S16 => self.emit_relaxed_str16(source, dest),
                    Size::S8 => self.emit_relaxed_str8(source, dest),
                },
                _ => codegen_error!(
                    "singlepass can't emit move_location {:?} {:?} => {:?}",
                    size,
                    source,
                    dest
                ),
            },
            Location::Imm64(val) => match dest {
                Location::GPR(_) => self.assembler.emit_mov_imm(dest, val),
                Location::Memory(_, _) => match size {
                    Size::S64 => self.emit_relaxed_str64(source, dest),
                    Size::S32 => self.emit_relaxed_str32(source, dest),
                    Size::S16 => self.emit_relaxed_str16(source, dest),
                    Size::S8 => self.emit_relaxed_str8(source, dest),
                },
                _ => codegen_error!(
                    "singlepass can't emit move_location {:?} {:?} => {:?}",
                    size,
                    source,
                    dest
                ),
            },
            Location::Memory(addr, offs) => match dest {
                Location::GPR(_) | Location::SIMD(_) => {
                    if self.offset_is_ok(size, offs) {
                        self.assembler.emit_ldr(size, dest, source)
                    } else if offs > -256 && offs < 256 {
                        self.assembler.emit_ldur(size, dest, addr, offs)
                    } else {
                        let tmp = GPR::X17;
                        if offs < 0 {
                            self.assembler
                                .emit_mov_imm(Location::GPR(tmp), (-offs) as u64)?;
                            self.assembler.emit_sub(
                                Size::S64,
                                Location::GPR(addr),
                                Location::GPR(tmp),
                                Location::GPR(tmp),
                            )?;
                        } else {
                            self.assembler
                                .emit_mov_imm(Location::GPR(tmp), offs as u64)?;
                            self.assembler.emit_add(
                                Size::S64,
                                Location::GPR(addr),
                                Location::GPR(tmp),
                                Location::GPR(tmp),
                            )?;
                        }
                        self.assembler
                            .emit_ldr(size, dest, Location::Memory(tmp, 0))
                    }
                }
                _ => {
                    let mut temps = vec![];
                    let src =
                        self.location_to_reg(size, source, &mut temps, ImmType::None, true, None)?;
                    self.move_location(size, src, dest)?;
                    for r in temps {
                        self.release_gpr(r);
                    }
                    Ok(())
                }
            },
            _ => codegen_error!(
                "singlepass can't emit move_location {:?} {:?} => {:?}",
                size,
                source,
                dest
            ),
        }
    }
    // move a location to another
    fn move_location_extend(
        &mut self,
        size_val: Size,
        signed: bool,
        source: Location,
        size_op: Size,
        dest: Location,
    ) -> Result<(), CompileError> {
        if size_op != Size::S64 {
            codegen_error!("singlepass move_location_extend unreachable");
        }
        let mut temps = vec![];
        let dst = self.location_to_reg(size_op, dest, &mut temps, ImmType::None, false, None)?;
        let src = match (size_val, signed, source) {
            (Size::S64, _, _) => source,
            (Size::S32, false, Location::GPR(_)) => {
                self.assembler.emit_mov(size_val, source, dst)?;
                dst
            }
            (Size::S8, false, Location::GPR(_)) => {
                self.assembler.emit_uxtb(size_op, source, dst)?;
                dst
            }
            (Size::S16, false, Location::GPR(_)) => {
                self.assembler.emit_uxth(size_op, source, dst)?;
                dst
            }
            (Size::S8, true, Location::GPR(_)) => {
                self.assembler.emit_sxtb(size_op, source, dst)?;
                dst
            }
            (Size::S16, true, Location::GPR(_)) => {
                self.assembler.emit_sxth(size_op, source, dst)?;
                dst
            }
            (Size::S32, true, Location::GPR(_)) => {
                self.assembler.emit_sxtw(size_op, source, dst)?;
                dst
            }
            (Size::S32, false, Location::Memory(_, _)) => {
                self.emit_relaxed_ldr32(size_op, dst, source)?;
                dst
            }
            (Size::S32, true, Location::Memory(_, _)) => {
                self.emit_relaxed_ldr32s(size_op, dst, source)?;
                dst
            }
            (Size::S16, false, Location::Memory(_, _)) => {
                self.emit_relaxed_ldr16(size_op, dst, source)?;
                dst
            }
            (Size::S16, true, Location::Memory(_, _)) => {
                self.emit_relaxed_ldr16s(size_op, dst, source)?;
                dst
            }
            (Size::S8, false, Location::Memory(_, _)) => {
                self.emit_relaxed_ldr8(size_op, dst, source)?;
                dst
            }
            (Size::S8, true, Location::Memory(_, _)) => {
                self.emit_relaxed_ldr8s(size_op, dst, source)?;
                dst
            }
            _ => codegen_error!(
                "singlepass can't emit move_location_extend {:?} {:?} {:?} => {:?} {:?}",
                size_val,
                signed,
                source,
                size_op,
                dest
            ),
        };
        if src != dst {
            self.move_location(size_op, src, dst)?;
        }
        if dst != dest {
            self.move_location(size_op, dst, dest)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn load_address(
        &mut self,
        _size: Size,
        _reg: Location,
        _mem: Location,
    ) -> Result<(), CompileError> {
        codegen_error!("singlepass load_address unimplemented");
    }
    // Init the stack loc counter
    fn init_stack_loc(
        &mut self,
        init_stack_loc_cnt: u64,
        last_stack_loc: Location,
    ) -> Result<(), CompileError> {
        let label = self.assembler.get_label();
        let mut temps = vec![];
        let dest = self.acquire_temp_gpr().ok_or_else(|| {
            CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
        })?;
        temps.push(dest);
        let cnt = self.location_to_reg(
            Size::S64,
            Location::Imm64(init_stack_loc_cnt),
            &mut temps,
            ImmType::None,
            true,
            None,
        )?;
        let dest = match last_stack_loc {
            Location::GPR(_) => codegen_error!("singlepass init_stack_loc unreachable"),
            Location::SIMD(_) => codegen_error!("singlepass init_stack_loc unreachable"),
            Location::Memory(reg, offset) => {
                if offset < 0 {
                    let offset = (-offset) as u32;
                    if self.compatible_imm(offset as i64, ImmType::Bits12) {
                        self.assembler.emit_sub(
                            Size::S64,
                            Location::GPR(reg),
                            Location::Imm32(offset),
                            Location::GPR(dest),
                        )?;
                    } else {
                        let tmp = self.acquire_temp_gpr().ok_or_else(|| {
                            CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                        })?;
                        self.assembler
                            .emit_mov_imm(Location::GPR(tmp), (offset as i64) as u64)?;
                        self.assembler.emit_sub(
                            Size::S64,
                            Location::GPR(reg),
                            Location::GPR(tmp),
                            Location::GPR(dest),
                        )?;
                        temps.push(tmp);
                    }
                    dest
                } else {
                    let offset = offset as u32;
                    if self.compatible_imm(offset as i64, ImmType::Bits12) {
                        self.assembler.emit_add(
                            Size::S64,
                            Location::GPR(reg),
                            Location::Imm32(offset),
                            Location::GPR(dest),
                        )?;
                    } else {
                        let tmp = self.acquire_temp_gpr().ok_or_else(|| {
                            CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                        })?;
                        self.assembler
                            .emit_mov_imm(Location::GPR(tmp), (offset as i64) as u64)?;
                        self.assembler.emit_add(
                            Size::S64,
                            Location::GPR(reg),
                            Location::GPR(tmp),
                            Location::GPR(dest),
                        )?;
                        temps.push(tmp);
                    }
                    dest
                }
            }
            _ => codegen_error!("singlepass can't emit init_stack_loc {:?}", last_stack_loc),
        };
        self.assembler.emit_label(label)?;
        self.assembler
            .emit_stria(Size::S64, Location::GPR(GPR::XzrSp), dest, 8)?;
        self.assembler
            .emit_sub(Size::S64, cnt, Location::Imm8(1), cnt)?;
        self.assembler.emit_cbnz_label(Size::S64, cnt, label)?;
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    // Restore save_area
    fn restore_saved_area(&mut self, saved_area_offset: i32) -> Result<(), CompileError> {
        let real_delta = if saved_area_offset & 15 != 0 {
            self.pushed = true;
            saved_area_offset + 8
        } else {
            self.pushed = false;
            saved_area_offset
        };
        if self.compatible_imm(real_delta as _, ImmType::Bits12) {
            self.assembler.emit_sub(
                Size::S64,
                Location::GPR(GPR::X29),
                Location::Imm32(real_delta as _),
                Location::GPR(GPR::XzrSp),
            )?;
        } else {
            let tmp = self.acquire_temp_gpr().ok_or_else(|| {
                CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
            })?;
            self.assembler
                .emit_mov_imm(Location::GPR(tmp), real_delta as u64)?;
            self.assembler.emit_sub(
                Size::S64,
                Location::GPR(GPR::X29),
                Location::GPR(tmp),
                Location::GPR(GPR::XzrSp),
            )?;
            self.release_gpr(tmp);
        }
        Ok(())
    }
    // Pop a location
    fn pop_location(&mut self, location: Location) -> Result<(), CompileError> {
        self.emit_pop(Size::S64, location)
    }
    // Create a new `MachineState` with default values.
    fn new_machine_state(&self) -> MachineState {
        new_machine_state()
    }

    // assembler finalize
    fn assembler_finalize(self) -> Vec<u8> {
        self.assembler.finalize().unwrap()
    }

    fn get_offset(&self) -> Offset {
        self.assembler.get_offset()
    }

    fn finalize_function(&mut self) -> Result<(), CompileError> {
        self.assembler.finalize_function();
        Ok(())
    }

    fn emit_function_prolog(&mut self) -> Result<(), CompileError> {
        self.emit_double_push(Size::S64, Location::GPR(GPR::X29), Location::GPR(GPR::X30))?; // save LR too
        self.emit_unwind_op(UnwindOps::Push2Regs {
            reg1: GPR::X29.to_dwarf(),
            reg2: GPR::X30.to_dwarf(),
            up_to_sp: 16,
        });
        self.emit_double_push(Size::S64, Location::GPR(GPR::X27), Location::GPR(GPR::X28))?;
        self.emit_unwind_op(UnwindOps::Push2Regs {
            reg1: GPR::X27.to_dwarf(),
            reg2: GPR::X28.to_dwarf(),
            up_to_sp: 32,
        });
        // cannot use mov, because XSP is XZR there. Need to use ADD with #0
        self.assembler.emit_add(
            Size::S64,
            Location::GPR(GPR::XzrSp),
            Location::Imm8(0),
            Location::GPR(GPR::X29),
        )?;
        self.emit_unwind_op(UnwindOps::DefineNewFrame);
        Ok(())
    }

    fn emit_function_epilog(&mut self) -> Result<(), CompileError> {
        // cannot use mov, because XSP is XZR there. Need to use ADD with #0
        self.assembler.emit_add(
            Size::S64,
            Location::GPR(GPR::X29),
            Location::Imm8(0),
            Location::GPR(GPR::XzrSp),
        )?;
        self.pushed = false; // SP is restored, consider it aligned
        self.emit_double_pop(Size::S64, Location::GPR(GPR::X27), Location::GPR(GPR::X28))?;
        self.emit_double_pop(Size::S64, Location::GPR(GPR::X29), Location::GPR(GPR::X30))?;
        Ok(())
    }

    fn emit_function_return_value(
        &mut self,
        ty: WpType,
        canonicalize: bool,
        loc: Location,
    ) -> Result<(), CompileError> {
        if canonicalize {
            self.canonicalize_nan(
                match ty {
                    WpType::F32 => Size::S32,
                    WpType::F64 => Size::S64,
                    _ => unreachable!(),
                },
                loc,
                Location::GPR(GPR::X0),
            )?;
        } else {
            self.emit_relaxed_mov(Size::S64, loc, Location::GPR(GPR::X0))?;
        }
        Ok(())
    }

    fn emit_function_return_float(&mut self) -> Result<(), CompileError> {
        self.assembler
            .emit_mov(Size::S64, Location::GPR(GPR::X0), Location::SIMD(NEON::V0))
    }

    fn arch_supports_canonicalize_nan(&self) -> bool {
        self.assembler.arch_supports_canonicalize_nan()
    }
    fn canonicalize_nan(
        &mut self,
        sz: Size,
        input: Location,
        output: Location,
    ) -> Result<(), CompileError> {
        let mut tempn = vec![];
        let mut temps = vec![];
        let old_fpcr = self.set_default_nan(&mut temps)?;
        // use FMAX (input, intput) => output to automaticaly normalize the NaN
        match (sz, input, output) {
            (Size::S32, Location::SIMD(_), Location::SIMD(_)) => {
                self.assembler.emit_fmax(sz, input, input, output)?;
            }
            (Size::S64, Location::SIMD(_), Location::SIMD(_)) => {
                self.assembler.emit_fmax(sz, input, input, output)?;
            }
            (Size::S32, Location::SIMD(_), _) | (Size::S64, Location::SIMD(_), _) => {
                let tmp = self.location_to_neon(sz, output, &mut tempn, ImmType::None, false)?;
                self.assembler.emit_fmax(sz, input, input, tmp)?;
                self.move_location(sz, tmp, output)?;
            }
            (Size::S32, Location::Memory(_, _), _) | (Size::S64, Location::Memory(_, _), _) => {
                let src = self.location_to_neon(sz, input, &mut tempn, ImmType::None, true)?;
                let tmp = self.location_to_neon(sz, output, &mut tempn, ImmType::None, false)?;
                self.assembler.emit_fmax(sz, src, src, tmp)?;
                if tmp != output {
                    self.move_location(sz, tmp, output)?;
                }
            }
            _ => codegen_error!(
                "singlepass can't emit canonicalize_nan {:?} {:?} {:?}",
                sz,
                input,
                output
            ),
        }

        self.restore_fpcr(old_fpcr)?;
        for r in temps {
            self.release_gpr(r);
        }
        for r in tempn {
            self.release_simd(r);
        }
        Ok(())
    }

    fn emit_illegal_op(&mut self, trap: TrapCode) -> Result<(), CompileError> {
        let offset = self.assembler.get_offset().0;
        self.assembler.emit_udf(0xc0 | (trap as u8) as u16)?;
        self.mark_instruction_address_end(offset);
        Ok(())
    }
    fn get_label(&mut self) -> Label {
        self.assembler.new_dynamic_label()
    }
    fn emit_label(&mut self, label: Label) -> Result<(), CompileError> {
        self.assembler.emit_label(label)
    }
    fn get_grp_for_call(&self) -> GPR {
        GPR::X27
    }
    fn emit_call_register(&mut self, reg: GPR) -> Result<(), CompileError> {
        self.assembler.emit_call_register(reg)
    }
    fn emit_call_label(&mut self, label: Label) -> Result<(), CompileError> {
        self.assembler.emit_call_label(label)
    }
    fn get_gpr_for_ret(&self) -> GPR {
        GPR::X0
    }
    fn get_simd_for_ret(&self) -> NEON {
        NEON::V0
    }

    fn arch_requires_indirect_call_trampoline(&self) -> bool {
        self.assembler.arch_requires_indirect_call_trampoline()
    }

    fn arch_emit_indirect_call_with_trampoline(
        &mut self,
        location: Location,
    ) -> Result<(), CompileError> {
        self.assembler
            .arch_emit_indirect_call_with_trampoline(location)
    }

    fn emit_debug_breakpoint(&mut self) -> Result<(), CompileError> {
        self.assembler.emit_brk()
    }

    fn emit_call_location(&mut self, location: Location) -> Result<(), CompileError> {
        let mut temps = vec![];
        let loc = self.location_to_reg(
            Size::S64,
            location,
            &mut temps,
            ImmType::None,
            true,
            Some(GPR::X27),
        )?;
        match loc {
            Location::GPR(reg) => self.assembler.emit_call_register(reg),
            _ => codegen_error!("singlepass can't emit CALL Location"),
        }?;
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }

    fn location_address(
        &mut self,
        _size: Size,
        _source: Location,
        _dest: Location,
    ) -> Result<(), CompileError> {
        codegen_error!("singlepass location_address not implemented")
    }
    // logic
    fn location_and(
        &mut self,
        _size: Size,
        _source: Location,
        _dest: Location,
        _flags: bool,
    ) -> Result<(), CompileError> {
        codegen_error!("singlepass location_and not implemented")
    }
    fn location_xor(
        &mut self,
        _size: Size,
        _source: Location,
        _dest: Location,
        _flags: bool,
    ) -> Result<(), CompileError> {
        codegen_error!("singlepass location_xor not implemented")
    }
    fn location_or(
        &mut self,
        _size: Size,
        _source: Location,
        _dest: Location,
        _flags: bool,
    ) -> Result<(), CompileError> {
        codegen_error!("singlepass location_or not implemented")
    }
    fn location_test(
        &mut self,
        _size: Size,
        _source: Location,
        _dest: Location,
    ) -> Result<(), CompileError> {
        codegen_error!("singlepass location_test not implemented")
    }
    // math
    fn location_add(
        &mut self,
        size: Size,
        source: Location,
        dest: Location,
        flags: bool,
    ) -> Result<(), CompileError> {
        let mut temps = vec![];
        let src = self.location_to_reg(size, source, &mut temps, ImmType::Bits12, true, None)?;
        let dst = self.location_to_reg(size, dest, &mut temps, ImmType::None, true, None)?;
        if flags {
            self.assembler.emit_adds(size, dst, src, dst)?;
        } else {
            self.assembler.emit_add(size, dst, src, dst)?;
        }
        if dst != dest {
            self.move_location(size, dst, dest)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn location_sub(
        &mut self,
        size: Size,
        source: Location,
        dest: Location,
        flags: bool,
    ) -> Result<(), CompileError> {
        let mut temps = vec![];
        let src = self.location_to_reg(size, source, &mut temps, ImmType::Bits12, true, None)?;
        let dst = self.location_to_reg(size, dest, &mut temps, ImmType::None, true, None)?;
        if flags {
            self.assembler.emit_subs(size, dst, src, dst)?;
        } else {
            self.assembler.emit_sub(size, dst, src, dst)?;
        }
        if dst != dest {
            self.move_location(size, dst, dest)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn location_cmp(
        &mut self,
        size: Size,
        source: Location,
        dest: Location,
    ) -> Result<(), CompileError> {
        self.emit_relaxed_binop(Assembler::emit_cmp, size, source, dest, false)
    }
    fn jmp_unconditionnal(&mut self, label: Label) -> Result<(), CompileError> {
        self.assembler.emit_b_label(label)
    }
    fn jmp_on_equal(&mut self, label: Label) -> Result<(), CompileError> {
        self.assembler.emit_bcond_label_far(Condition::Eq, label)
    }
    fn jmp_on_different(&mut self, label: Label) -> Result<(), CompileError> {
        self.assembler.emit_bcond_label_far(Condition::Ne, label)
    }
    fn jmp_on_above(&mut self, label: Label) -> Result<(), CompileError> {
        self.assembler.emit_bcond_label_far(Condition::Hi, label)
    }
    fn jmp_on_aboveequal(&mut self, label: Label) -> Result<(), CompileError> {
        self.assembler.emit_bcond_label_far(Condition::Cs, label)
    }
    fn jmp_on_belowequal(&mut self, label: Label) -> Result<(), CompileError> {
        self.assembler.emit_bcond_label_far(Condition::Ls, label)
    }
    fn jmp_on_overflow(&mut self, label: Label) -> Result<(), CompileError> {
        self.assembler.emit_bcond_label_far(Condition::Cs, label)
    }

    // jmp table
    fn emit_jmp_to_jumptable(&mut self, label: Label, cond: Location) -> Result<(), CompileError> {
        let tmp1 = self.acquire_temp_gpr().ok_or_else(|| {
            CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
        })?;
        let tmp2 = self.acquire_temp_gpr().ok_or_else(|| {
            CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
        })?;

        self.assembler.emit_load_label(tmp1, label)?;
        self.move_location(Size::S32, cond, Location::GPR(tmp2))?;

        self.assembler.emit_add_lsl(
            Size::S64,
            Location::GPR(tmp1),
            Location::GPR(tmp2),
            2,
            Location::GPR(tmp2),
        )?;
        self.assembler.emit_b_register(tmp2)?;
        self.release_gpr(tmp2);
        self.release_gpr(tmp1);
        Ok(())
    }

    fn align_for_loop(&mut self) -> Result<(), CompileError> {
        // noting to do on ARM64
        Ok(())
    }

    fn emit_ret(&mut self) -> Result<(), CompileError> {
        self.assembler.emit_ret()
    }

    fn emit_push(&mut self, size: Size, loc: Location) -> Result<(), CompileError> {
        self.emit_push(size, loc)
    }
    fn emit_pop(&mut self, size: Size, loc: Location) -> Result<(), CompileError> {
        self.emit_pop(size, loc)
    }

    fn emit_memory_fence(&mut self) -> Result<(), CompileError> {
        self.assembler.emit_dmb()
    }

    fn location_neg(
        &mut self,
        _size_val: Size, // size of src
        _signed: bool,
        _source: Location,
        _size_op: Size,
        _dest: Location,
    ) -> Result<(), CompileError> {
        codegen_error!("singlepass location_neg unimplemented");
    }

    fn emit_imul_imm32(&mut self, size: Size, imm32: u32, gpr: GPR) -> Result<(), CompileError> {
        let tmp = self.acquire_temp_gpr().ok_or_else(|| {
            CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
        })?;
        self.assembler
            .emit_mov_imm(Location::GPR(tmp), imm32 as u64)?;
        self.assembler.emit_mul(
            size,
            Location::GPR(gpr),
            Location::GPR(tmp),
            Location::GPR(gpr),
        )?;
        self.release_gpr(tmp);
        Ok(())
    }

    // relaxed binop based...
    fn emit_relaxed_mov(
        &mut self,
        sz: Size,
        src: Location,
        dst: Location,
    ) -> Result<(), CompileError> {
        self.emit_relaxed_binop(Assembler::emit_mov, sz, src, dst, true)
    }
    fn emit_relaxed_cmp(
        &mut self,
        sz: Size,
        src: Location,
        dst: Location,
    ) -> Result<(), CompileError> {
        self.emit_relaxed_binop(Assembler::emit_cmp, sz, src, dst, false)
    }
    fn emit_relaxed_zero_extension(
        &mut self,
        _sz_src: Size,
        _src: Location,
        _sz_dst: Size,
        _dst: Location,
    ) -> Result<(), CompileError> {
        codegen_error!("singlepass emit_relaxed_zero_extension unimplemented");
    }
    fn emit_relaxed_sign_extension(
        &mut self,
        sz_src: Size,
        src: Location,
        sz_dst: Size,
        dst: Location,
    ) -> Result<(), CompileError> {
        match (src, dst) {
            (Location::Memory(_, _), Location::GPR(_)) => match sz_src {
                Size::S8 => self.emit_relaxed_ldr8s(sz_dst, dst, src),
                Size::S16 => self.emit_relaxed_ldr16s(sz_dst, dst, src),
                Size::S32 => self.emit_relaxed_ldr32s(sz_dst, dst, src),
                _ => codegen_error!("singlepass emit_relaxed_sign_extension unreachable"),
            },
            _ => {
                let mut temps = vec![];
                let src =
                    self.location_to_reg(sz_src, src, &mut temps, ImmType::None, true, None)?;
                let dest =
                    self.location_to_reg(sz_dst, dst, &mut temps, ImmType::None, false, None)?;
                match sz_src {
                    Size::S8 => self.assembler.emit_sxtb(sz_dst, src, dest),
                    Size::S16 => self.assembler.emit_sxth(sz_dst, src, dest),
                    Size::S32 => self.assembler.emit_sxtw(sz_dst, src, dest),
                    _ => codegen_error!("singlepass emit_relaxed_sign_extension unreachable"),
                }?;
                if dst != dest {
                    self.move_location(sz_dst, dest, dst)?;
                }
                for r in temps {
                    self.release_gpr(r);
                }
                Ok(())
            }
        }
    }

    fn emit_binop_add32(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_relaxed_binop3(
            Assembler::emit_add,
            Size::S32,
            loc_a,
            loc_b,
            ret,
            ImmType::Bits12,
        )
    }
    fn emit_binop_sub32(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_relaxed_binop3(
            Assembler::emit_sub,
            Size::S32,
            loc_a,
            loc_b,
            ret,
            ImmType::Bits12,
        )
    }
    fn emit_binop_mul32(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_relaxed_binop3(
            Assembler::emit_mul,
            Size::S32,
            loc_a,
            loc_b,
            ret,
            ImmType::None,
        )
    }
    fn emit_binop_udiv32(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
        integer_division_by_zero: Label,
        _integer_overflow: Label,
    ) -> Result<usize, CompileError> {
        let mut temps = vec![];
        let src1 = self.location_to_reg(Size::S32, loc_a, &mut temps, ImmType::None, true, None)?;
        let src2 = self.location_to_reg(Size::S32, loc_b, &mut temps, ImmType::None, true, None)?;
        let dest = self.location_to_reg(Size::S32, ret, &mut temps, ImmType::None, false, None)?;

        self.assembler
            .emit_cbz_label(Size::S32, src2, integer_division_by_zero)?;
        let offset = self.mark_instruction_with_trap_code(TrapCode::IntegerOverflow);
        self.assembler.emit_udiv(Size::S32, src1, src2, dest)?;
        if ret != dest {
            self.move_location(Size::S32, dest, ret)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(offset)
    }
    fn emit_binop_sdiv32(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
        integer_division_by_zero: Label,
        integer_overflow: Label,
    ) -> Result<usize, CompileError> {
        let mut temps = vec![];
        let src1 = self.location_to_reg(Size::S32, loc_a, &mut temps, ImmType::None, true, None)?;
        let src2 = self.location_to_reg(Size::S32, loc_b, &mut temps, ImmType::None, true, None)?;
        let dest = self.location_to_reg(Size::S32, ret, &mut temps, ImmType::None, false, None)?;

        self.assembler
            .emit_cbz_label(Size::S32, src2, integer_division_by_zero)?;
        let label_nooverflow = self.assembler.get_label();
        let tmp = self.location_to_reg(
            Size::S32,
            Location::Imm32(0x80000000),
            &mut temps,
            ImmType::None,
            true,
            None,
        )?;
        self.assembler.emit_cmp(Size::S32, tmp, src1)?;
        self.assembler
            .emit_bcond_label(Condition::Ne, label_nooverflow)?;
        self.assembler.emit_movn(Size::S32, tmp, 0)?;
        self.assembler.emit_cmp(Size::S32, tmp, src2)?;
        self.assembler
            .emit_bcond_label_far(Condition::Eq, integer_overflow)?;
        let offset = self.mark_instruction_with_trap_code(TrapCode::IntegerOverflow);
        self.assembler.emit_label(label_nooverflow)?;
        self.assembler.emit_sdiv(Size::S32, src1, src2, dest)?;
        if ret != dest {
            self.move_location(Size::S32, dest, ret)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(offset)
    }
    fn emit_binop_urem32(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
        integer_division_by_zero: Label,
        _integer_overflow: Label,
    ) -> Result<usize, CompileError> {
        let mut temps = vec![];
        let src1 = self.location_to_reg(Size::S32, loc_a, &mut temps, ImmType::None, true, None)?;
        let src2 = self.location_to_reg(Size::S32, loc_b, &mut temps, ImmType::None, true, None)?;
        let dest = self.location_to_reg(Size::S32, ret, &mut temps, ImmType::None, false, None)?;
        let dest = if dest == src1 || dest == src2 {
            let tmp = self.acquire_temp_gpr().ok_or_else(|| {
                CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
            })?;
            temps.push(tmp);
            self.assembler
                .emit_mov(Size::S32, dest, Location::GPR(tmp))?;
            Location::GPR(tmp)
        } else {
            dest
        };
        self.assembler
            .emit_cbz_label(Size::S32, src2, integer_division_by_zero)?;
        let offset = self.mark_instruction_with_trap_code(TrapCode::IntegerOverflow);
        self.assembler.emit_udiv(Size::S32, src1, src2, dest)?;
        // unsigned remainder : src1 - (src1/src2)*src2
        self.assembler
            .emit_msub(Size::S32, dest, src2, src1, dest)?;
        if ret != dest {
            self.move_location(Size::S32, dest, ret)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(offset)
    }
    fn emit_binop_srem32(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
        integer_division_by_zero: Label,
        _integer_overflow: Label,
    ) -> Result<usize, CompileError> {
        let mut temps = vec![];
        let src1 = self.location_to_reg(Size::S32, loc_a, &mut temps, ImmType::None, true, None)?;
        let src2 = self.location_to_reg(Size::S32, loc_b, &mut temps, ImmType::None, true, None)?;
        let dest = self.location_to_reg(Size::S32, ret, &mut temps, ImmType::None, false, None)?;
        let dest = if dest == src1 || dest == src2 {
            let tmp = self.acquire_temp_gpr().ok_or_else(|| {
                CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
            })?;
            temps.push(tmp);
            self.assembler
                .emit_mov(Size::S32, dest, Location::GPR(tmp))?;
            Location::GPR(tmp)
        } else {
            dest
        };
        self.assembler
            .emit_cbz_label(Size::S32, src2, integer_division_by_zero)?;
        let offset = self.mark_instruction_with_trap_code(TrapCode::IntegerOverflow);
        self.assembler.emit_sdiv(Size::S32, src1, src2, dest)?;
        // unsigned remainder : src1 - (src1/src2)*src2
        self.assembler
            .emit_msub(Size::S32, dest, src2, src1, dest)?;
        if ret != dest {
            self.move_location(Size::S32, dest, ret)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(offset)
    }
    fn emit_binop_and32(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_relaxed_binop3(
            Assembler::emit_and,
            Size::S32,
            loc_a,
            loc_b,
            ret,
            ImmType::Logical32,
        )
    }
    fn emit_binop_or32(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_relaxed_binop3(
            Assembler::emit_or,
            Size::S32,
            loc_a,
            loc_b,
            ret,
            ImmType::Logical32,
        )
    }
    fn emit_binop_xor32(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_relaxed_binop3(
            Assembler::emit_eor,
            Size::S32,
            loc_a,
            loc_b,
            ret,
            ImmType::Logical32,
        )
    }
    fn i32_cmp_ge_s(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_cmpop_i32_dynamic_b(Condition::Ge, loc_a, loc_b, ret)
    }
    fn i32_cmp_gt_s(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_cmpop_i32_dynamic_b(Condition::Gt, loc_a, loc_b, ret)
    }
    fn i32_cmp_le_s(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_cmpop_i32_dynamic_b(Condition::Le, loc_a, loc_b, ret)
    }
    fn i32_cmp_lt_s(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_cmpop_i32_dynamic_b(Condition::Lt, loc_a, loc_b, ret)
    }
    fn i32_cmp_ge_u(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_cmpop_i32_dynamic_b(Condition::Cs, loc_a, loc_b, ret)
    }
    fn i32_cmp_gt_u(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_cmpop_i32_dynamic_b(Condition::Hi, loc_a, loc_b, ret)
    }
    fn i32_cmp_le_u(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_cmpop_i32_dynamic_b(Condition::Ls, loc_a, loc_b, ret)
    }
    fn i32_cmp_lt_u(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_cmpop_i32_dynamic_b(Condition::Cc, loc_a, loc_b, ret)
    }
    fn i32_cmp_ne(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_cmpop_i32_dynamic_b(Condition::Ne, loc_a, loc_b, ret)
    }
    fn i32_cmp_eq(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_cmpop_i32_dynamic_b(Condition::Eq, loc_a, loc_b, ret)
    }
    fn i32_clz(&mut self, src: Location, dst: Location) -> Result<(), CompileError> {
        self.emit_relaxed_binop(Assembler::emit_clz, Size::S32, src, dst, true)
    }
    fn i32_ctz(&mut self, src: Location, dst: Location) -> Result<(), CompileError> {
        let mut temps = vec![];
        let src = self.location_to_reg(Size::S32, src, &mut temps, ImmType::None, true, None)?;
        let dest = self.location_to_reg(Size::S32, dst, &mut temps, ImmType::None, false, None)?;
        self.assembler.emit_rbit(Size::S32, src, dest)?;
        self.assembler.emit_clz(Size::S32, dest, dest)?;
        if dst != dest {
            self.move_location(Size::S32, dest, dst)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn i32_popcnt(&mut self, loc: Location, ret: Location) -> Result<(), CompileError> {
        // no opcode for that.
        // 2 solutions: using NEON CNT, that count bits per Byte, or using clz with some shift and loop
        let mut temps = vec![];
        let src = self.location_to_reg(Size::S32, loc, &mut temps, ImmType::None, true, None)?;
        let dest = self.location_to_reg(Size::S32, ret, &mut temps, ImmType::None, false, None)?;
        let src = if src == loc {
            let tmp = self.acquire_temp_gpr().ok_or_else(|| {
                CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
            })?;
            temps.push(tmp);
            self.assembler
                .emit_mov(Size::S32, src, Location::GPR(tmp))?;
            Location::GPR(tmp)
        } else {
            src
        };
        let tmp = {
            let tmp = self.acquire_temp_gpr().ok_or_else(|| {
                CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
            })?;
            temps.push(tmp);
            Location::GPR(tmp)
        };
        let label_loop = self.assembler.get_label();
        let label_exit = self.assembler.get_label();
        self.assembler
            .emit_mov(Size::S32, Location::GPR(GPR::XzrSp), dest)?; // 0 => dest
        self.assembler.emit_cbz_label(Size::S32, src, label_exit)?; // src==0, exit
        self.assembler.emit_label(label_loop)?; // loop:
        self.assembler
            .emit_add(Size::S32, dest, Location::Imm8(1), dest)?; // dest += 1
        self.assembler.emit_clz(Size::S32, src, tmp)?; // clz src => tmp
        self.assembler.emit_lsl(Size::S32, src, tmp, src)?; // src << tmp => src
        self.assembler
            .emit_lsl(Size::S32, src, Location::Imm8(1), src)?; // src << 1 => src
        self.assembler.emit_cbnz_label(Size::S32, src, label_loop)?; // if src!=0 goto loop
        self.assembler.emit_label(label_exit)?;
        if ret != dest {
            self.move_location(Size::S32, dest, ret)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn i32_shl(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_relaxed_binop3(
            Assembler::emit_lsl,
            Size::S32,
            loc_a,
            loc_b,
            ret,
            ImmType::Shift32No0,
        )
    }
    fn i32_shr(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_relaxed_binop3(
            Assembler::emit_lsr,
            Size::S32,
            loc_a,
            loc_b,
            ret,
            ImmType::Shift32No0,
        )
    }
    fn i32_sar(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_relaxed_binop3(
            Assembler::emit_asr,
            Size::S32,
            loc_a,
            loc_b,
            ret,
            ImmType::Shift32No0,
        )
    }
    fn i32_rol(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        let mut temps = vec![];
        let src2 = match loc_b {
            Location::Imm8(imm) => Location::Imm8(32 - (imm & 31)),
            Location::Imm32(imm) => Location::Imm8(32 - (imm & 31) as u8),
            Location::Imm64(imm) => Location::Imm8(32 - (imm & 31) as u8),
            _ => {
                let tmp1 = self.location_to_reg(
                    Size::S32,
                    Location::Imm32(32),
                    &mut temps,
                    ImmType::None,
                    true,
                    None,
                )?;
                let tmp2 =
                    self.location_to_reg(Size::S32, loc_b, &mut temps, ImmType::None, true, None)?;
                self.assembler.emit_sub(Size::S32, tmp1, tmp2, tmp1)?;
                tmp1
            }
        };
        self.emit_relaxed_binop3(
            Assembler::emit_ror,
            Size::S32,
            loc_a,
            src2,
            ret,
            ImmType::Shift32No0,
        )?;
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn i32_ror(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_relaxed_binop3(
            Assembler::emit_ror,
            Size::S32,
            loc_a,
            loc_b,
            ret,
            ImmType::Shift32No0,
        )
    }
    fn i32_load(
        &mut self,
        addr: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            addr,
            memarg,
            false,
            4,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_ldr32(Size::S32, ret, Location::Memory(addr, 0)),
        )
    }
    fn i32_load_8u(
        &mut self,
        addr: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            addr,
            memarg,
            false,
            1,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_ldr8(Size::S32, ret, Location::Memory(addr, 0)),
        )
    }
    fn i32_load_8s(
        &mut self,
        addr: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            addr,
            memarg,
            false,
            1,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_ldr8s(Size::S32, ret, Location::Memory(addr, 0)),
        )
    }
    fn i32_load_16u(
        &mut self,
        addr: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            addr,
            memarg,
            false,
            2,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_ldr16(Size::S32, ret, Location::Memory(addr, 0)),
        )
    }
    fn i32_load_16s(
        &mut self,
        addr: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            addr,
            memarg,
            false,
            2,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_ldr16s(Size::S32, ret, Location::Memory(addr, 0)),
        )
    }
    fn i32_atomic_load(
        &mut self,
        addr: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            addr,
            memarg,
            true,
            4,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_ldr32(Size::S32, ret, Location::Memory(addr, 0)),
        )
    }
    fn i32_atomic_load_8u(
        &mut self,
        addr: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            addr,
            memarg,
            true,
            1,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_ldr8(Size::S32, ret, Location::Memory(addr, 0)),
        )
    }
    fn i32_atomic_load_16u(
        &mut self,
        addr: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            addr,
            memarg,
            true,
            2,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_ldr16(Size::S32, ret, Location::Memory(addr, 0)),
        )
    }
    fn i32_save(
        &mut self,
        target_value: Location,
        memarg: &MemArg,
        target_addr: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target_addr,
            memarg,
            false,
            4,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_str32(target_value, Location::Memory(addr, 0)),
        )
    }
    fn i32_save_8(
        &mut self,
        target_value: Location,
        memarg: &MemArg,
        target_addr: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target_addr,
            memarg,
            false,
            4,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_str8(target_value, Location::Memory(addr, 0)),
        )
    }
    fn i32_save_16(
        &mut self,
        target_value: Location,
        memarg: &MemArg,
        target_addr: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target_addr,
            memarg,
            false,
            4,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_str16(target_value, Location::Memory(addr, 0)),
        )
    }
    fn i32_atomic_save(
        &mut self,
        target_value: Location,
        memarg: &MemArg,
        target_addr: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target_addr,
            memarg,
            true,
            4,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_str32(target_value, Location::Memory(addr, 0)),
        )?;
        self.assembler.emit_dmb()
    }
    fn i32_atomic_save_8(
        &mut self,
        target_value: Location,
        memarg: &MemArg,
        target_addr: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target_addr,
            memarg,
            true,
            1,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_str8(target_value, Location::Memory(addr, 0)),
        )?;
        self.assembler.emit_dmb()
    }
    fn i32_atomic_save_16(
        &mut self,
        target_value: Location,
        memarg: &MemArg,
        target_addr: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target_addr,
            memarg,
            true,
            2,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_str16(target_value, Location::Memory(addr, 0)),
        )?;
        self.assembler.emit_dmb()
    }
    // i32 atomic Add with i32
    fn i32_atomic_add(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            4,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S32, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxr(Size::S32, dst, Location::GPR(addr))?;
                this.emit_binop_add32(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxr(
                    Size::S32,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S32, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i32 atomic Add with u8
    fn i32_atomic_add_8u(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            1,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S32, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxrb(Size::S32, dst, Location::GPR(addr))?;
                this.emit_binop_add32(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxrb(
                    Size::S32,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S32, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i32 atomic Add with u16
    fn i32_atomic_add_16u(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            2,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S32, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxrh(Size::S32, dst, Location::GPR(addr))?;
                this.emit_binop_add32(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxrh(
                    Size::S32,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S32, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i32 atomic Sub with i32
    fn i32_atomic_sub(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            4,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S32, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxr(Size::S32, dst, Location::GPR(addr))?;
                this.emit_binop_sub32(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxr(
                    Size::S32,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S32, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i32 atomic Sub with u8
    fn i32_atomic_sub_8u(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            1,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S32, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxrb(Size::S32, dst, Location::GPR(addr))?;
                this.emit_binop_sub32(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxrb(
                    Size::S32,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S32, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i32 atomic Sub with u16
    fn i32_atomic_sub_16u(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            2,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S32, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxrh(Size::S32, dst, Location::GPR(addr))?;
                this.emit_binop_sub32(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxrh(
                    Size::S32,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S32, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i32 atomic And with i32
    fn i32_atomic_and(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            4,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S32, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxr(Size::S32, dst, Location::GPR(addr))?;
                this.emit_binop_and32(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxr(
                    Size::S32,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S32, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i32 atomic And with u8
    fn i32_atomic_and_8u(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            1,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S32, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxrb(Size::S32, dst, Location::GPR(addr))?;
                this.emit_binop_and32(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxrb(
                    Size::S32,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S32, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i32 atomic And with u16
    fn i32_atomic_and_16u(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            2,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S32, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxrh(Size::S32, dst, Location::GPR(addr))?;
                this.emit_binop_and32(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxrh(
                    Size::S32,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S32, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i32 atomic Or with i32
    fn i32_atomic_or(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            4,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S32, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxr(Size::S32, dst, Location::GPR(addr))?;
                this.emit_binop_or32(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxr(
                    Size::S32,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S32, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i32 atomic Or with u8
    fn i32_atomic_or_8u(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            1,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S32, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxrb(Size::S32, dst, Location::GPR(addr))?;
                this.emit_binop_or32(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxrb(
                    Size::S32,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S32, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i32 atomic Or with u16
    fn i32_atomic_or_16u(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            2,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S32, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxrh(Size::S32, dst, Location::GPR(addr))?;
                this.emit_binop_or32(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxrh(
                    Size::S32,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S32, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i32 atomic Xor with i32
    fn i32_atomic_xor(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            4,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S32, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxr(Size::S32, dst, Location::GPR(addr))?;
                this.emit_binop_xor32(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxr(
                    Size::S32,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S32, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i32 atomic Xor with u8
    fn i32_atomic_xor_8u(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            1,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S32, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxrb(Size::S32, dst, Location::GPR(addr))?;
                this.emit_binop_xor32(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxrb(
                    Size::S32,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S32, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i32 atomic Xor with u16
    fn i32_atomic_xor_16u(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            2,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S32, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxrh(Size::S32, dst, Location::GPR(addr))?;
                this.emit_binop_xor32(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxrh(
                    Size::S32,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S32, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i32 atomic Exchange with i32
    fn i32_atomic_xchg(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            4,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S32, ret, &mut temps, ImmType::None, false, None)?;
                let org =
                    this.location_to_reg(Size::S32, loc, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxr(Size::S32, dst, Location::GPR(addr))?;
                this.assembler.emit_stlxr(
                    Size::S32,
                    Location::GPR(tmp),
                    org,
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S32, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i32 atomic Exchange with u8
    fn i32_atomic_xchg_8u(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            1,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S32, ret, &mut temps, ImmType::None, false, None)?;
                let org =
                    this.location_to_reg(Size::S32, loc, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxrb(Size::S32, dst, Location::GPR(addr))?;
                this.assembler.emit_stlxrb(
                    Size::S32,
                    Location::GPR(tmp),
                    org,
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S32, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i32 atomic Exchange with u16
    fn i32_atomic_xchg_16u(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            2,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S32, ret, &mut temps, ImmType::None, false, None)?;
                let org =
                    this.location_to_reg(Size::S32, loc, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxrh(Size::S32, dst, Location::GPR(addr))?;
                this.assembler.emit_stlxrh(
                    Size::S32,
                    Location::GPR(tmp),
                    org,
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S32, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i32 atomic Exchange with i32
    fn i32_atomic_cmpxchg(
        &mut self,
        new: Location,
        cmp: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            4,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S32, ret, &mut temps, ImmType::None, false, None)?;
                let org =
                    this.location_to_reg(Size::S32, new, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();
                let nosame = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxr(Size::S32, dst, Location::GPR(addr))?;
                this.emit_relaxed_cmp(Size::S32, dst, cmp)?;
                this.assembler.emit_bcond_label(Condition::Ne, nosame)?;
                this.assembler.emit_stlxr(
                    Size::S32,
                    Location::GPR(tmp),
                    org,
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp), reread)?;
                this.assembler.emit_dmb()?;

                this.emit_label(nosame)?;
                if dst != ret {
                    this.move_location(Size::S32, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i32 atomic Exchange with u8
    fn i32_atomic_cmpxchg_8u(
        &mut self,
        new: Location,
        cmp: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            1,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S32, ret, &mut temps, ImmType::None, false, None)?;
                let org =
                    this.location_to_reg(Size::S32, new, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();
                let nosame = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxrb(Size::S32, dst, Location::GPR(addr))?;
                this.emit_relaxed_cmp(Size::S32, dst, cmp)?;
                this.assembler.emit_bcond_label(Condition::Ne, nosame)?;
                this.assembler.emit_stlxrb(
                    Size::S32,
                    Location::GPR(tmp),
                    org,
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp), reread)?;
                this.assembler.emit_dmb()?;

                this.emit_label(nosame)?;
                if dst != ret {
                    this.move_location(Size::S32, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i32 atomic Exchange with u16
    fn i32_atomic_cmpxchg_16u(
        &mut self,
        new: Location,
        cmp: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            2,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S32, ret, &mut temps, ImmType::None, false, None)?;
                let org =
                    this.location_to_reg(Size::S32, new, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();
                let nosame = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxrh(Size::S32, dst, Location::GPR(addr))?;
                this.emit_relaxed_cmp(Size::S32, dst, cmp)?;
                this.assembler.emit_bcond_label(Condition::Ne, nosame)?;
                this.assembler.emit_stlxrh(
                    Size::S32,
                    Location::GPR(tmp),
                    org,
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp), reread)?;
                this.assembler.emit_dmb()?;

                this.emit_label(nosame)?;
                if dst != ret {
                    this.move_location(Size::S32, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }

    fn emit_call_with_reloc(
        &mut self,
        _calling_convention: CallingConvention,
        reloc_target: RelocationTarget,
    ) -> Result<Vec<Relocation>, CompileError> {
        let mut relocations = vec![];
        let next = self.get_label();
        let reloc_at = self.assembler.get_offset().0;
        self.emit_label(next)?; // this is to be sure the current imm26 value is 0
        self.assembler.emit_call_label(next)?;
        relocations.push(Relocation {
            kind: RelocationKind::Arm64Call,
            reloc_target,
            offset: reloc_at as u32,
            addend: 0,
        });
        Ok(relocations)
    }

    fn emit_binop_add64(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_relaxed_binop3(
            Assembler::emit_add,
            Size::S64,
            loc_a,
            loc_b,
            ret,
            ImmType::Bits12,
        )
    }
    fn emit_binop_sub64(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_relaxed_binop3(
            Assembler::emit_sub,
            Size::S64,
            loc_a,
            loc_b,
            ret,
            ImmType::Bits12,
        )
    }
    fn emit_binop_mul64(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_relaxed_binop3(
            Assembler::emit_mul,
            Size::S64,
            loc_a,
            loc_b,
            ret,
            ImmType::None,
        )
    }
    fn emit_binop_udiv64(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
        integer_division_by_zero: Label,
        _integer_overflow: Label,
    ) -> Result<usize, CompileError> {
        let mut temps = vec![];
        let src1 = self.location_to_reg(Size::S64, loc_a, &mut temps, ImmType::None, true, None)?;
        let src2 = self.location_to_reg(Size::S64, loc_b, &mut temps, ImmType::None, true, None)?;
        let dest = self.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;

        self.assembler
            .emit_cbz_label(Size::S64, src2, integer_division_by_zero)?;
        let offset = self.mark_instruction_with_trap_code(TrapCode::IntegerOverflow);
        self.assembler.emit_udiv(Size::S64, src1, src2, dest)?;
        if ret != dest {
            self.move_location(Size::S64, dest, ret)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(offset)
    }
    fn emit_binop_sdiv64(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
        integer_division_by_zero: Label,
        integer_overflow: Label,
    ) -> Result<usize, CompileError> {
        let mut temps = vec![];
        let src1 = self.location_to_reg(Size::S64, loc_a, &mut temps, ImmType::None, true, None)?;
        let src2 = self.location_to_reg(Size::S64, loc_b, &mut temps, ImmType::None, true, None)?;
        let dest = self.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;

        self.assembler
            .emit_cbz_label(Size::S64, src2, integer_division_by_zero)?;
        let label_nooverflow = self.assembler.get_label();
        let tmp = self.location_to_reg(
            Size::S64,
            Location::Imm64(0x8000000000000000),
            &mut temps,
            ImmType::None,
            true,
            None,
        )?;
        self.assembler.emit_cmp(Size::S64, tmp, src1)?;
        self.assembler
            .emit_bcond_label(Condition::Ne, label_nooverflow)?;
        self.assembler.emit_movn(Size::S64, tmp, 0)?;
        self.assembler.emit_cmp(Size::S64, tmp, src2)?;
        self.assembler
            .emit_bcond_label_far(Condition::Eq, integer_overflow)?;
        let offset = self.mark_instruction_with_trap_code(TrapCode::IntegerOverflow);
        self.assembler.emit_label(label_nooverflow)?;
        self.assembler.emit_sdiv(Size::S64, src1, src2, dest)?;
        if ret != dest {
            self.move_location(Size::S64, dest, ret)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(offset)
    }
    fn emit_binop_urem64(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
        integer_division_by_zero: Label,
        _integer_overflow: Label,
    ) -> Result<usize, CompileError> {
        let mut temps = vec![];
        let src1 = self.location_to_reg(Size::S64, loc_a, &mut temps, ImmType::None, true, None)?;
        let src2 = self.location_to_reg(Size::S64, loc_b, &mut temps, ImmType::None, true, None)?;
        let dest = self.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
        let dest = if dest == src1 || dest == src2 {
            let tmp = self.acquire_temp_gpr().ok_or_else(|| {
                CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
            })?;
            temps.push(tmp);
            self.assembler
                .emit_mov(Size::S32, dest, Location::GPR(tmp))?;
            Location::GPR(tmp)
        } else {
            dest
        };
        self.assembler
            .emit_cbz_label(Size::S64, src2, integer_division_by_zero)?;
        let offset = self.mark_instruction_with_trap_code(TrapCode::IntegerOverflow);
        self.assembler.emit_udiv(Size::S64, src1, src2, dest)?;
        // unsigned remainder : src1 - (src1/src2)*src2
        self.assembler
            .emit_msub(Size::S64, dest, src2, src1, dest)?;
        if ret != dest {
            self.move_location(Size::S64, dest, ret)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(offset)
    }
    fn emit_binop_srem64(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
        integer_division_by_zero: Label,
        _integer_overflow: Label,
    ) -> Result<usize, CompileError> {
        let mut temps = vec![];
        let src1 = self.location_to_reg(Size::S64, loc_a, &mut temps, ImmType::None, true, None)?;
        let src2 = self.location_to_reg(Size::S64, loc_b, &mut temps, ImmType::None, true, None)?;
        let dest = self.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
        let dest = if dest == src1 || dest == src2 {
            let tmp = self.acquire_temp_gpr().ok_or_else(|| {
                CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
            })?;
            temps.push(tmp);
            self.assembler
                .emit_mov(Size::S64, dest, Location::GPR(tmp))?;
            Location::GPR(tmp)
        } else {
            dest
        };
        self.assembler
            .emit_cbz_label(Size::S64, src2, integer_division_by_zero)?;
        let offset = self.mark_instruction_with_trap_code(TrapCode::IntegerOverflow);
        self.assembler.emit_sdiv(Size::S64, src1, src2, dest)?;
        // unsigned remainder : src1 - (src1/src2)*src2
        self.assembler
            .emit_msub(Size::S64, dest, src2, src1, dest)?;
        if ret != dest {
            self.move_location(Size::S64, dest, ret)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(offset)
    }
    fn emit_binop_and64(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_relaxed_binop3(
            Assembler::emit_and,
            Size::S64,
            loc_a,
            loc_b,
            ret,
            ImmType::Logical64,
        )
    }
    fn emit_binop_or64(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_relaxed_binop3(
            Assembler::emit_or,
            Size::S64,
            loc_a,
            loc_b,
            ret,
            ImmType::Logical64,
        )
    }
    fn emit_binop_xor64(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_relaxed_binop3(
            Assembler::emit_eor,
            Size::S64,
            loc_a,
            loc_b,
            ret,
            ImmType::Logical64,
        )
    }
    fn i64_cmp_ge_s(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_cmpop_i64_dynamic_b(Condition::Ge, loc_a, loc_b, ret)
    }
    fn i64_cmp_gt_s(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_cmpop_i64_dynamic_b(Condition::Gt, loc_a, loc_b, ret)
    }
    fn i64_cmp_le_s(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_cmpop_i64_dynamic_b(Condition::Le, loc_a, loc_b, ret)
    }
    fn i64_cmp_lt_s(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_cmpop_i64_dynamic_b(Condition::Lt, loc_a, loc_b, ret)
    }
    fn i64_cmp_ge_u(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_cmpop_i64_dynamic_b(Condition::Cs, loc_a, loc_b, ret)
    }
    fn i64_cmp_gt_u(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_cmpop_i64_dynamic_b(Condition::Hi, loc_a, loc_b, ret)
    }
    fn i64_cmp_le_u(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_cmpop_i64_dynamic_b(Condition::Ls, loc_a, loc_b, ret)
    }
    fn i64_cmp_lt_u(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_cmpop_i64_dynamic_b(Condition::Cc, loc_a, loc_b, ret)
    }
    fn i64_cmp_ne(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_cmpop_i64_dynamic_b(Condition::Ne, loc_a, loc_b, ret)
    }
    fn i64_cmp_eq(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_cmpop_i64_dynamic_b(Condition::Eq, loc_a, loc_b, ret)
    }
    fn i64_clz(&mut self, src: Location, dst: Location) -> Result<(), CompileError> {
        self.emit_relaxed_binop(Assembler::emit_clz, Size::S64, src, dst, true)
    }
    fn i64_ctz(&mut self, src: Location, dst: Location) -> Result<(), CompileError> {
        let mut temps = vec![];
        let src = self.location_to_reg(Size::S64, src, &mut temps, ImmType::None, true, None)?;
        let dest = self.location_to_reg(Size::S64, dst, &mut temps, ImmType::None, false, None)?;
        self.assembler.emit_rbit(Size::S64, src, dest)?;
        self.assembler.emit_clz(Size::S64, dest, dest)?;
        if dst != dest {
            self.move_location(Size::S64, dest, dst)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn i64_popcnt(&mut self, loc: Location, ret: Location) -> Result<(), CompileError> {
        let mut temps = vec![];
        let src = self.location_to_reg(Size::S64, loc, &mut temps, ImmType::None, true, None)?;
        let dest = self.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
        let src = if src == loc {
            let tmp = self.acquire_temp_gpr().ok_or_else(|| {
                CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
            })?;
            temps.push(tmp);
            self.assembler
                .emit_mov(Size::S64, src, Location::GPR(tmp))?;
            Location::GPR(tmp)
        } else {
            src
        };
        let tmp = {
            let tmp = self.acquire_temp_gpr().ok_or_else(|| {
                CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
            })?;
            temps.push(tmp);
            Location::GPR(tmp)
        };
        let label_loop = self.assembler.get_label();
        let label_exit = self.assembler.get_label();
        self.assembler
            .emit_mov(Size::S32, Location::GPR(GPR::XzrSp), dest)?; // dest <= 0
        self.assembler.emit_cbz_label(Size::S64, src, label_exit)?; // src == 0, then goto label_exit
        self.assembler.emit_label(label_loop)?;
        self.assembler
            .emit_add(Size::S32, dest, Location::Imm8(1), dest)?; // dest += 1
        self.assembler.emit_clz(Size::S64, src, tmp)?; // clz src => tmp
        self.assembler.emit_lsl(Size::S64, src, tmp, src)?; // src << tmp => src
        self.assembler
            .emit_lsl(Size::S64, src, Location::Imm8(1), src)?; // src << 1 => src
        self.assembler.emit_cbnz_label(Size::S64, src, label_loop)?; // src != 0, then goto label_loop
        self.assembler.emit_label(label_exit)?;
        if ret != dest {
            self.move_location(Size::S64, dest, ret)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn i64_shl(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_relaxed_binop3(
            Assembler::emit_lsl,
            Size::S64,
            loc_a,
            loc_b,
            ret,
            ImmType::Shift64No0,
        )
    }
    fn i64_shr(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_relaxed_binop3(
            Assembler::emit_lsr,
            Size::S64,
            loc_a,
            loc_b,
            ret,
            ImmType::Shift64No0,
        )
    }
    fn i64_sar(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_relaxed_binop3(
            Assembler::emit_asr,
            Size::S64,
            loc_a,
            loc_b,
            ret,
            ImmType::Shift64No0,
        )
    }
    fn i64_rol(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        // there is no ROL on ARM64. We use ROR with 64-value instead
        let mut temps = vec![];
        let src2 = match loc_b {
            Location::Imm8(imm) => Location::Imm8(64 - (imm & 63)),
            Location::Imm32(imm) => Location::Imm8(64 - (imm & 63) as u8),
            Location::Imm64(imm) => Location::Imm8(64 - (imm & 63) as u8),
            _ => {
                let tmp1 = self.location_to_reg(
                    Size::S64,
                    Location::Imm32(64),
                    &mut temps,
                    ImmType::None,
                    true,
                    None,
                )?;
                let tmp2 =
                    self.location_to_reg(Size::S64, loc_b, &mut temps, ImmType::None, true, None)?;
                self.assembler.emit_sub(Size::S64, tmp1, tmp2, tmp1)?;
                tmp1
            }
        };
        self.emit_relaxed_binop3(
            Assembler::emit_ror,
            Size::S64,
            loc_a,
            src2,
            ret,
            ImmType::Shift64No0,
        )?;
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn i64_ror(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_relaxed_binop3(
            Assembler::emit_ror,
            Size::S64,
            loc_a,
            loc_b,
            ret,
            ImmType::Shift64No0,
        )
    }
    fn i64_load(
        &mut self,
        addr: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            addr,
            memarg,
            false,
            8,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_ldr64(Size::S64, ret, Location::Memory(addr, 0)),
        )
    }
    fn i64_load_8u(
        &mut self,
        addr: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            addr,
            memarg,
            false,
            1,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_ldr8(Size::S64, ret, Location::Memory(addr, 0)),
        )
    }
    fn i64_load_8s(
        &mut self,
        addr: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            addr,
            memarg,
            false,
            1,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_ldr8s(Size::S64, ret, Location::Memory(addr, 0)),
        )
    }
    fn i64_load_16u(
        &mut self,
        addr: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            addr,
            memarg,
            false,
            2,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_ldr16(Size::S64, ret, Location::Memory(addr, 0)),
        )
    }
    fn i64_load_16s(
        &mut self,
        addr: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            addr,
            memarg,
            false,
            2,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_ldr16s(Size::S64, ret, Location::Memory(addr, 0)),
        )
    }
    fn i64_load_32u(
        &mut self,
        addr: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            addr,
            memarg,
            false,
            4,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_ldr32(Size::S64, ret, Location::Memory(addr, 0)),
        )
    }
    fn i64_load_32s(
        &mut self,
        addr: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            addr,
            memarg,
            false,
            4,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_ldr32s(Size::S64, ret, Location::Memory(addr, 0)),
        )
    }
    fn i64_atomic_load(
        &mut self,
        addr: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            addr,
            memarg,
            true,
            8,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_ldr64(Size::S64, ret, Location::Memory(addr, 0)),
        )
    }
    fn i64_atomic_load_8u(
        &mut self,
        addr: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            addr,
            memarg,
            true,
            1,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_ldr8(Size::S64, ret, Location::Memory(addr, 0)),
        )
    }
    fn i64_atomic_load_16u(
        &mut self,
        addr: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            addr,
            memarg,
            true,
            2,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_ldr16(Size::S64, ret, Location::Memory(addr, 0)),
        )
    }
    fn i64_atomic_load_32u(
        &mut self,
        addr: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            addr,
            memarg,
            true,
            4,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_ldr32(Size::S64, ret, Location::Memory(addr, 0)),
        )
    }
    fn i64_save(
        &mut self,
        target_value: Location,
        memarg: &MemArg,
        target_addr: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target_addr,
            memarg,
            false,
            8,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_str64(target_value, Location::Memory(addr, 0)),
        )
    }
    fn i64_save_8(
        &mut self,
        target_value: Location,
        memarg: &MemArg,
        target_addr: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target_addr,
            memarg,
            false,
            1,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_str8(target_value, Location::Memory(addr, 0)),
        )
    }
    fn i64_save_16(
        &mut self,
        target_value: Location,
        memarg: &MemArg,
        target_addr: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target_addr,
            memarg,
            false,
            2,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_str16(target_value, Location::Memory(addr, 0)),
        )
    }
    fn i64_save_32(
        &mut self,
        target_value: Location,
        memarg: &MemArg,
        target_addr: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target_addr,
            memarg,
            false,
            4,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_str32(target_value, Location::Memory(addr, 0)),
        )
    }
    fn i64_atomic_save(
        &mut self,
        target_value: Location,
        memarg: &MemArg,
        target_addr: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target_addr,
            memarg,
            true,
            8,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_str64(target_value, Location::Memory(addr, 0)),
        )?;
        self.assembler.emit_dmb()
    }
    fn i64_atomic_save_8(
        &mut self,
        target_value: Location,
        memarg: &MemArg,
        target_addr: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target_addr,
            memarg,
            true,
            1,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_str8(target_value, Location::Memory(addr, 0)),
        )?;
        self.assembler.emit_dmb()
    }
    fn i64_atomic_save_16(
        &mut self,
        target_value: Location,
        memarg: &MemArg,
        target_addr: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target_addr,
            memarg,
            true,
            2,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_str16(target_value, Location::Memory(addr, 0)),
        )?;
        self.assembler.emit_dmb()
    }
    fn i64_atomic_save_32(
        &mut self,
        target_value: Location,
        memarg: &MemArg,
        target_addr: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target_addr,
            memarg,
            true,
            4,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_str32(target_value, Location::Memory(addr, 0)),
        )?;
        self.assembler.emit_dmb()
    }
    // i64 atomic Add with i64
    fn i64_atomic_add(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            8,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxr(Size::S64, dst, Location::GPR(addr))?;
                this.emit_binop_add64(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxr(
                    Size::S64,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S64, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i64 atomic Add with u8
    fn i64_atomic_add_8u(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            1,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxrb(Size::S64, dst, Location::GPR(addr))?;
                this.emit_binop_add64(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxrb(
                    Size::S64,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S64, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i64 atomic Add with u16
    fn i64_atomic_add_16u(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            2,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxrh(Size::S64, dst, Location::GPR(addr))?;
                this.emit_binop_add64(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxrh(
                    Size::S64,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S64, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i64 atomic Add with u32
    fn i64_atomic_add_32u(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            4,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxr(Size::S32, dst, Location::GPR(addr))?;
                this.emit_binop_add64(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxr(
                    Size::S32,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S64, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i64 atomic Sub with i64
    fn i64_atomic_sub(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            8,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxr(Size::S64, dst, Location::GPR(addr))?;
                this.emit_binop_sub64(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxr(
                    Size::S64,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S64, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i64 atomic Sub with u8
    fn i64_atomic_sub_8u(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            1,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxrb(Size::S64, dst, Location::GPR(addr))?;
                this.emit_binop_sub64(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxrb(
                    Size::S64,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S64, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i64 atomic Sub with u16
    fn i64_atomic_sub_16u(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            2,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxrh(Size::S64, dst, Location::GPR(addr))?;
                this.emit_binop_sub64(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxrh(
                    Size::S64,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S64, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i64 atomic Sub with u32
    fn i64_atomic_sub_32u(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            4,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxr(Size::S32, dst, Location::GPR(addr))?;
                this.emit_binop_sub64(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxr(
                    Size::S32,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S64, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i64 atomic And with i64
    fn i64_atomic_and(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            8,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxr(Size::S64, dst, Location::GPR(addr))?;
                this.emit_binop_and64(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxr(
                    Size::S64,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S64, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i64 atomic And with u8
    fn i64_atomic_and_8u(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            1,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxrb(Size::S64, dst, Location::GPR(addr))?;
                this.emit_binop_and64(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxrb(
                    Size::S64,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S64, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i64 atomic And with u16
    fn i64_atomic_and_16u(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            2,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxrh(Size::S64, dst, Location::GPR(addr))?;
                this.emit_binop_and64(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxrh(
                    Size::S64,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S64, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i64 atomic And with u32
    fn i64_atomic_and_32u(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            4,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxr(Size::S32, dst, Location::GPR(addr))?;
                this.emit_binop_and64(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxr(
                    Size::S32,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S64, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i64 atomic Or with i64
    fn i64_atomic_or(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            8,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxr(Size::S64, dst, Location::GPR(addr))?;
                this.emit_binop_or64(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxr(
                    Size::S64,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S64, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i64 atomic Or with u8
    fn i64_atomic_or_8u(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            1,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxrb(Size::S64, dst, Location::GPR(addr))?;
                this.emit_binop_or64(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxrb(
                    Size::S64,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S64, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i64 atomic Or with u16
    fn i64_atomic_or_16u(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            2,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxrh(Size::S64, dst, Location::GPR(addr))?;
                this.emit_binop_or64(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxrh(
                    Size::S64,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S64, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i64 atomic Or with u32
    fn i64_atomic_or_32u(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            4,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxr(Size::S32, dst, Location::GPR(addr))?;
                this.emit_binop_or64(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxr(
                    Size::S32,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S64, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i64 atomic Xor with i64
    fn i64_atomic_xor(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            8,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxr(Size::S64, dst, Location::GPR(addr))?;
                this.emit_binop_xor64(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxr(
                    Size::S64,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S64, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i64 atomic Xor with u8
    fn i64_atomic_xor_8u(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            1,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxrb(Size::S64, dst, Location::GPR(addr))?;
                this.emit_binop_xor64(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxrb(
                    Size::S64,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S64, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i64 atomic Xor with u16
    fn i64_atomic_xor_16u(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            2,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxrh(Size::S64, dst, Location::GPR(addr))?;
                this.emit_binop_xor64(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxrh(
                    Size::S64,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S64, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i64 atomic Xor with u32
    fn i64_atomic_xor_32u(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            4,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp1 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let tmp2 = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxr(Size::S32, dst, Location::GPR(addr))?;
                this.emit_binop_xor64(dst, loc, Location::GPR(tmp1))?;
                this.assembler.emit_stlxr(
                    Size::S32,
                    Location::GPR(tmp2),
                    Location::GPR(tmp1),
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp2), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S64, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i64 atomic Exchange with i64
    fn i64_atomic_xchg(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            8,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
                let org =
                    this.location_to_reg(Size::S64, loc, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxr(Size::S64, dst, Location::GPR(addr))?;
                this.assembler.emit_stlxr(
                    Size::S64,
                    Location::GPR(tmp),
                    org,
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S64, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i64 atomic Exchange with u8
    fn i64_atomic_xchg_8u(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            1,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
                let org =
                    this.location_to_reg(Size::S64, loc, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxrb(Size::S64, dst, Location::GPR(addr))?;
                this.assembler.emit_stlxrb(
                    Size::S64,
                    Location::GPR(tmp),
                    org,
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S64, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i64 atomic Exchange with u16
    fn i64_atomic_xchg_16u(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            2,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
                let org =
                    this.location_to_reg(Size::S64, loc, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxrh(Size::S64, dst, Location::GPR(addr))?;
                this.assembler.emit_stlxrh(
                    Size::S64,
                    Location::GPR(tmp),
                    org,
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S64, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i64 atomic Exchange with u32
    fn i64_atomic_xchg_32u(
        &mut self,
        loc: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            4,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
                let org =
                    this.location_to_reg(Size::S64, loc, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxr(Size::S32, dst, Location::GPR(addr))?;
                this.assembler.emit_stlxr(
                    Size::S32,
                    Location::GPR(tmp),
                    org,
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp), reread)?;
                this.assembler.emit_dmb()?;

                if dst != ret {
                    this.move_location(Size::S64, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i64 atomic Exchange with i64
    fn i64_atomic_cmpxchg(
        &mut self,
        new: Location,
        cmp: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            8,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
                let org =
                    this.location_to_reg(Size::S64, new, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();
                let nosame = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxr(Size::S64, dst, Location::GPR(addr))?;
                this.emit_relaxed_cmp(Size::S64, dst, cmp)?;
                this.assembler.emit_bcond_label(Condition::Ne, nosame)?;
                this.assembler.emit_stlxr(
                    Size::S64,
                    Location::GPR(tmp),
                    org,
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp), reread)?;
                this.assembler.emit_dmb()?;

                this.emit_label(nosame)?;
                if dst != ret {
                    this.move_location(Size::S64, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i64 atomic Exchange with u8
    fn i64_atomic_cmpxchg_8u(
        &mut self,
        new: Location,
        cmp: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            1,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
                let org =
                    this.location_to_reg(Size::S64, new, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();
                let nosame = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxrb(Size::S64, dst, Location::GPR(addr))?;
                this.emit_relaxed_cmp(Size::S64, dst, cmp)?;
                this.assembler.emit_bcond_label(Condition::Ne, nosame)?;
                this.assembler.emit_stlxrb(
                    Size::S64,
                    Location::GPR(tmp),
                    org,
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp), reread)?;
                this.assembler.emit_dmb()?;

                this.emit_label(nosame)?;
                if dst != ret {
                    this.move_location(Size::S64, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i64 atomic Exchange with u16
    fn i64_atomic_cmpxchg_16u(
        &mut self,
        new: Location,
        cmp: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            2,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
                let org =
                    this.location_to_reg(Size::S64, new, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();
                let nosame = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxrh(Size::S64, dst, Location::GPR(addr))?;
                this.emit_relaxed_cmp(Size::S64, dst, cmp)?;
                this.assembler.emit_bcond_label(Condition::Ne, nosame)?;
                this.assembler.emit_stlxrh(
                    Size::S64,
                    Location::GPR(tmp),
                    org,
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp), reread)?;
                this.assembler.emit_dmb()?;

                this.emit_label(nosame)?;
                if dst != ret {
                    this.move_location(Size::S64, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }
    // i64 atomic Exchange with u32
    fn i64_atomic_cmpxchg_32u(
        &mut self,
        new: Location,
        cmp: Location,
        target: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            target,
            memarg,
            true,
            4,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                let mut temps = vec![];
                let tmp = this.acquire_temp_gpr().ok_or_else(|| {
                    CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
                })?;
                let dst =
                    this.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
                let org =
                    this.location_to_reg(Size::S64, new, &mut temps, ImmType::None, false, None)?;
                let reread = this.get_label();
                let nosame = this.get_label();

                this.emit_label(reread)?;
                this.assembler
                    .emit_ldaxr(Size::S32, dst, Location::GPR(addr))?;
                this.emit_relaxed_cmp(Size::S64, dst, cmp)?;
                this.assembler.emit_bcond_label(Condition::Ne, nosame)?;
                this.assembler.emit_stlxr(
                    Size::S32,
                    Location::GPR(tmp),
                    org,
                    Location::GPR(addr),
                )?;
                this.assembler
                    .emit_cbnz_label(Size::S32, Location::GPR(tmp), reread)?;
                this.assembler.emit_dmb()?;

                this.emit_label(nosame)?;
                if dst != ret {
                    this.move_location(Size::S64, ret, dst)?;
                }
                for r in temps {
                    this.release_gpr(r);
                }
                Ok(())
            },
        )
    }

    fn f32_load(
        &mut self,
        addr: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            addr,
            memarg,
            false,
            4,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_ldr32(Size::S32, ret, Location::Memory(addr, 0)),
        )
    }
    fn f32_save(
        &mut self,
        target_value: Location,
        memarg: &MemArg,
        target_addr: Location,
        canonicalize: bool,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        let canonicalize = canonicalize && self.arch_supports_canonicalize_nan();
        self.memory_op(
            target_addr,
            memarg,
            false,
            4,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                if !canonicalize {
                    this.emit_relaxed_str32(target_value, Location::Memory(addr, 0))
                } else {
                    this.canonicalize_nan(Size::S32, target_value, Location::Memory(addr, 0))
                }
            },
        )
    }
    fn f64_load(
        &mut self,
        addr: Location,
        memarg: &MemArg,
        ret: Location,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        self.memory_op(
            addr,
            memarg,
            false,
            8,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| this.emit_relaxed_ldr64(Size::S64, ret, Location::Memory(addr, 0)),
        )
    }
    fn f64_save(
        &mut self,
        target_value: Location,
        memarg: &MemArg,
        target_addr: Location,
        canonicalize: bool,
        need_check: bool,
        imported_memories: bool,
        offset: i32,
        heap_access_oob: Label,
        unaligned_atomic: Label,
    ) -> Result<(), CompileError> {
        let canonicalize = canonicalize && self.arch_supports_canonicalize_nan();
        self.memory_op(
            target_addr,
            memarg,
            false,
            8,
            need_check,
            imported_memories,
            offset,
            heap_access_oob,
            unaligned_atomic,
            |this, addr| {
                if !canonicalize {
                    this.emit_relaxed_str64(target_value, Location::Memory(addr, 0))
                } else {
                    this.canonicalize_nan(Size::S64, target_value, Location::Memory(addr, 0))
                }
            },
        )
    }

    fn convert_f64_i64(
        &mut self,
        loc: Location,
        signed: bool,
        ret: Location,
    ) -> Result<(), CompileError> {
        let mut gprs = vec![];
        let mut neons = vec![];
        let src = self.location_to_reg(Size::S64, loc, &mut gprs, ImmType::NoneXzr, true, None)?;
        let dest = self.location_to_neon(Size::S64, ret, &mut neons, ImmType::None, false)?;
        if signed {
            self.assembler.emit_scvtf(Size::S64, src, Size::S64, dest)?;
        } else {
            self.assembler.emit_ucvtf(Size::S64, src, Size::S64, dest)?;
        }
        if ret != dest {
            self.move_location(Size::S64, dest, ret)?;
        }
        for r in gprs {
            self.release_gpr(r);
        }
        for r in neons {
            self.release_simd(r);
        }
        Ok(())
    }
    fn convert_f64_i32(
        &mut self,
        loc: Location,
        signed: bool,
        ret: Location,
    ) -> Result<(), CompileError> {
        let mut gprs = vec![];
        let mut neons = vec![];
        let src = self.location_to_reg(Size::S32, loc, &mut gprs, ImmType::NoneXzr, true, None)?;
        let dest = self.location_to_neon(Size::S64, ret, &mut neons, ImmType::None, false)?;
        if signed {
            self.assembler.emit_scvtf(Size::S32, src, Size::S64, dest)?;
        } else {
            self.assembler.emit_ucvtf(Size::S32, src, Size::S64, dest)?;
        }
        if ret != dest {
            self.move_location(Size::S64, dest, ret)?;
        }
        for r in gprs {
            self.release_gpr(r);
        }
        for r in neons {
            self.release_simd(r);
        }
        Ok(())
    }
    fn convert_f32_i64(
        &mut self,
        loc: Location,
        signed: bool,
        ret: Location,
    ) -> Result<(), CompileError> {
        let mut gprs = vec![];
        let mut neons = vec![];
        let src = self.location_to_reg(Size::S64, loc, &mut gprs, ImmType::NoneXzr, true, None)?;
        let dest = self.location_to_neon(Size::S32, ret, &mut neons, ImmType::None, false)?;
        if signed {
            self.assembler.emit_scvtf(Size::S64, src, Size::S32, dest)?;
        } else {
            self.assembler.emit_ucvtf(Size::S64, src, Size::S32, dest)?;
        }
        if ret != dest {
            self.move_location(Size::S32, dest, ret)?;
        }
        for r in gprs {
            self.release_gpr(r);
        }
        for r in neons {
            self.release_simd(r);
        }
        Ok(())
    }
    fn convert_f32_i32(
        &mut self,
        loc: Location,
        signed: bool,
        ret: Location,
    ) -> Result<(), CompileError> {
        let mut gprs = vec![];
        let mut neons = vec![];
        let src = self.location_to_reg(Size::S32, loc, &mut gprs, ImmType::NoneXzr, true, None)?;
        let dest = self.location_to_neon(Size::S32, ret, &mut neons, ImmType::None, false)?;
        if signed {
            self.assembler.emit_scvtf(Size::S32, src, Size::S32, dest)?;
        } else {
            self.assembler.emit_ucvtf(Size::S32, src, Size::S32, dest)?;
        }
        if ret != dest {
            self.move_location(Size::S32, dest, ret)?;
        }
        for r in gprs {
            self.release_gpr(r);
        }
        for r in neons {
            self.release_simd(r);
        }
        Ok(())
    }
    fn convert_i64_f64(
        &mut self,
        loc: Location,
        ret: Location,
        signed: bool,
        sat: bool,
    ) -> Result<(), CompileError> {
        let mut gprs = vec![];
        let mut neons = vec![];
        let src = self.location_to_neon(Size::S64, loc, &mut neons, ImmType::None, true)?;
        let dest = self.location_to_reg(Size::S64, ret, &mut gprs, ImmType::None, false, None)?;
        let old_fpcr = if !sat {
            self.reset_exception_fpsr()?;
            self.set_trap_enabled(&mut gprs)?
        } else {
            GPR::XzrSp
        };
        if signed {
            self.assembler
                .emit_fcvtzs(Size::S64, src, Size::S64, dest)?;
        } else {
            self.assembler
                .emit_fcvtzu(Size::S64, src, Size::S64, dest)?;
        }
        if !sat {
            self.trap_float_convertion_errors(old_fpcr, Size::S64, src, &mut gprs)?;
        }
        if ret != dest {
            self.move_location(Size::S64, dest, ret)?;
        }
        for r in gprs {
            self.release_gpr(r);
        }
        for r in neons {
            self.release_simd(r);
        }
        Ok(())
    }
    fn convert_i32_f64(
        &mut self,
        loc: Location,
        ret: Location,
        signed: bool,
        sat: bool,
    ) -> Result<(), CompileError> {
        let mut gprs = vec![];
        let mut neons = vec![];
        let src = self.location_to_neon(Size::S64, loc, &mut neons, ImmType::None, true)?;
        let dest = self.location_to_reg(Size::S32, ret, &mut gprs, ImmType::None, false, None)?;
        let old_fpcr = if !sat {
            self.reset_exception_fpsr()?;
            self.set_trap_enabled(&mut gprs)?
        } else {
            GPR::XzrSp
        };
        if signed {
            self.assembler
                .emit_fcvtzs(Size::S64, src, Size::S32, dest)?;
        } else {
            self.assembler
                .emit_fcvtzu(Size::S64, src, Size::S32, dest)?;
        }
        if !sat {
            self.trap_float_convertion_errors(old_fpcr, Size::S64, src, &mut gprs)?;
        }
        if ret != dest {
            self.move_location(Size::S32, dest, ret)?;
        }
        for r in gprs {
            self.release_gpr(r);
        }
        for r in neons {
            self.release_simd(r);
        }
        Ok(())
    }
    fn convert_i64_f32(
        &mut self,
        loc: Location,
        ret: Location,
        signed: bool,
        sat: bool,
    ) -> Result<(), CompileError> {
        let mut gprs = vec![];
        let mut neons = vec![];
        let src = self.location_to_neon(Size::S32, loc, &mut neons, ImmType::None, true)?;
        let dest = self.location_to_reg(Size::S64, ret, &mut gprs, ImmType::None, false, None)?;
        let old_fpcr = if !sat {
            self.reset_exception_fpsr()?;
            self.set_trap_enabled(&mut gprs)?
        } else {
            GPR::XzrSp
        };
        if signed {
            self.assembler
                .emit_fcvtzs(Size::S32, src, Size::S64, dest)?;
        } else {
            self.assembler
                .emit_fcvtzu(Size::S32, src, Size::S64, dest)?;
        }
        if !sat {
            self.trap_float_convertion_errors(old_fpcr, Size::S32, src, &mut gprs)?;
        }
        if ret != dest {
            self.move_location(Size::S64, dest, ret)?;
        }
        for r in gprs {
            self.release_gpr(r);
        }
        for r in neons {
            self.release_simd(r);
        }
        Ok(())
    }
    fn convert_i32_f32(
        &mut self,
        loc: Location,
        ret: Location,
        signed: bool,
        sat: bool,
    ) -> Result<(), CompileError> {
        let mut gprs = vec![];
        let mut neons = vec![];
        let src = self.location_to_neon(Size::S32, loc, &mut neons, ImmType::None, true)?;
        let dest = self.location_to_reg(Size::S32, ret, &mut gprs, ImmType::None, false, None)?;
        let old_fpcr = if !sat {
            self.reset_exception_fpsr()?;
            self.set_trap_enabled(&mut gprs)?
        } else {
            GPR::XzrSp
        };
        if signed {
            self.assembler
                .emit_fcvtzs(Size::S32, src, Size::S32, dest)?;
        } else {
            self.assembler
                .emit_fcvtzu(Size::S32, src, Size::S32, dest)?;
        }
        if !sat {
            self.trap_float_convertion_errors(old_fpcr, Size::S32, src, &mut gprs)?;
        }
        if ret != dest {
            self.move_location(Size::S32, dest, ret)?;
        }
        for r in gprs {
            self.release_gpr(r);
        }
        for r in neons {
            self.release_simd(r);
        }
        Ok(())
    }
    fn convert_f64_f32(&mut self, loc: Location, ret: Location) -> Result<(), CompileError> {
        self.emit_relaxed_binop_neon(Assembler::emit_fcvt, Size::S32, loc, ret, true)
    }
    fn convert_f32_f64(&mut self, loc: Location, ret: Location) -> Result<(), CompileError> {
        self.emit_relaxed_binop_neon(Assembler::emit_fcvt, Size::S64, loc, ret, true)
    }
    fn f64_neg(&mut self, loc: Location, ret: Location) -> Result<(), CompileError> {
        self.emit_relaxed_binop_neon(Assembler::emit_fneg, Size::S64, loc, ret, true)
    }
    fn f64_abs(&mut self, loc: Location, ret: Location) -> Result<(), CompileError> {
        let tmp = self.acquire_temp_gpr().ok_or_else(|| {
            CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
        })?;

        self.move_location(Size::S64, loc, Location::GPR(tmp))?;
        self.assembler.emit_and(
            Size::S64,
            Location::GPR(tmp),
            Location::Imm64(0x7fffffffffffffffu64),
            Location::GPR(tmp),
        )?;
        self.move_location(Size::S64, Location::GPR(tmp), ret)?;

        self.release_gpr(tmp);
        Ok(())
    }
    fn emit_i64_copysign(&mut self, tmp1: GPR, tmp2: GPR) -> Result<(), CompileError> {
        self.assembler.emit_and(
            Size::S64,
            Location::GPR(tmp1),
            Location::Imm64(0x7fffffffffffffffu64),
            Location::GPR(tmp1),
        )?;

        self.assembler.emit_and(
            Size::S64,
            Location::GPR(tmp2),
            Location::Imm64(0x8000000000000000u64),
            Location::GPR(tmp2),
        )?;

        self.assembler.emit_or(
            Size::S64,
            Location::GPR(tmp1),
            Location::GPR(tmp2),
            Location::GPR(tmp1),
        )
    }
    fn f64_sqrt(&mut self, loc: Location, ret: Location) -> Result<(), CompileError> {
        self.emit_relaxed_binop_neon(Assembler::emit_fsqrt, Size::S64, loc, ret, true)
    }
    fn f64_trunc(&mut self, loc: Location, ret: Location) -> Result<(), CompileError> {
        self.emit_relaxed_binop_neon(Assembler::emit_frintz, Size::S64, loc, ret, true)
    }
    fn f64_ceil(&mut self, loc: Location, ret: Location) -> Result<(), CompileError> {
        self.emit_relaxed_binop_neon(Assembler::emit_frintp, Size::S64, loc, ret, true)
    }
    fn f64_floor(&mut self, loc: Location, ret: Location) -> Result<(), CompileError> {
        self.emit_relaxed_binop_neon(Assembler::emit_frintm, Size::S64, loc, ret, true)
    }
    fn f64_nearest(&mut self, loc: Location, ret: Location) -> Result<(), CompileError> {
        self.emit_relaxed_binop_neon(Assembler::emit_frintn, Size::S64, loc, ret, true)
    }
    fn f64_cmp_ge(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        let mut temps = vec![];
        let dest = self.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
        self.emit_relaxed_binop_neon(Assembler::emit_fcmp, Size::S64, loc_b, loc_a, false)?;
        self.assembler.emit_cset(Size::S32, dest, Condition::Ls)?;
        if ret != dest {
            self.move_location(Size::S32, dest, ret)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn f64_cmp_gt(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        let mut temps = vec![];
        let dest = self.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
        self.emit_relaxed_binop_neon(Assembler::emit_fcmp, Size::S64, loc_b, loc_a, false)?;
        self.assembler.emit_cset(Size::S32, dest, Condition::Cc)?;
        if ret != dest {
            self.move_location(Size::S32, dest, ret)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn f64_cmp_le(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        let mut temps = vec![];
        let dest = self.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
        self.emit_relaxed_binop_neon(Assembler::emit_fcmp, Size::S64, loc_a, loc_b, false)?;
        self.assembler.emit_cset(Size::S32, dest, Condition::Ls)?;
        if ret != dest {
            self.move_location(Size::S32, dest, ret)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn f64_cmp_lt(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        let mut temps = vec![];
        let dest = self.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
        self.emit_relaxed_binop_neon(Assembler::emit_fcmp, Size::S64, loc_a, loc_b, false)?;
        self.assembler.emit_cset(Size::S32, dest, Condition::Cc)?;
        if ret != dest {
            self.move_location(Size::S32, dest, ret)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn f64_cmp_ne(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        let mut temps = vec![];
        let dest = self.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
        self.emit_relaxed_binop_neon(Assembler::emit_fcmp, Size::S64, loc_a, loc_b, false)?;
        self.assembler.emit_cset(Size::S32, dest, Condition::Ne)?;
        if ret != dest {
            self.move_location(Size::S32, dest, ret)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn f64_cmp_eq(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        let mut temps = vec![];
        let dest = self.location_to_reg(Size::S64, ret, &mut temps, ImmType::None, false, None)?;
        self.emit_relaxed_binop_neon(Assembler::emit_fcmp, Size::S64, loc_a, loc_b, false)?;
        self.assembler.emit_cset(Size::S32, dest, Condition::Eq)?;
        if ret != dest {
            self.move_location(Size::S32, dest, ret)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn f64_min(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        let mut temps = vec![];
        let old_fpcr = self.set_default_nan(&mut temps)?;
        self.emit_relaxed_binop3_neon(
            Assembler::emit_fmin,
            Size::S64,
            loc_a,
            loc_b,
            ret,
            ImmType::None,
        )?;
        self.restore_fpcr(old_fpcr)?;
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn f64_max(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        let mut temps = vec![];
        let old_fpcr = self.set_default_nan(&mut temps)?;
        self.emit_relaxed_binop3_neon(
            Assembler::emit_fmax,
            Size::S64,
            loc_a,
            loc_b,
            ret,
            ImmType::None,
        )?;
        self.restore_fpcr(old_fpcr)?;
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn f64_add(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_relaxed_binop3_neon(
            Assembler::emit_fadd,
            Size::S64,
            loc_a,
            loc_b,
            ret,
            ImmType::None,
        )
    }
    fn f64_sub(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_relaxed_binop3_neon(
            Assembler::emit_fsub,
            Size::S64,
            loc_a,
            loc_b,
            ret,
            ImmType::None,
        )
    }
    fn f64_mul(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_relaxed_binop3_neon(
            Assembler::emit_fmul,
            Size::S64,
            loc_a,
            loc_b,
            ret,
            ImmType::None,
        )
    }
    fn f64_div(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_relaxed_binop3_neon(
            Assembler::emit_fdiv,
            Size::S64,
            loc_a,
            loc_b,
            ret,
            ImmType::None,
        )
    }
    fn f32_neg(&mut self, loc: Location, ret: Location) -> Result<(), CompileError> {
        self.emit_relaxed_binop_neon(Assembler::emit_fneg, Size::S32, loc, ret, true)
    }
    fn f32_abs(&mut self, loc: Location, ret: Location) -> Result<(), CompileError> {
        let tmp = self.acquire_temp_gpr().ok_or_else(|| {
            CompileError::Codegen("singlepass cannot acquire temp gpr".to_owned())
        })?;
        self.move_location(Size::S32, loc, Location::GPR(tmp))?;
        self.assembler.emit_and(
            Size::S32,
            Location::GPR(tmp),
            Location::Imm32(0x7fffffffu32),
            Location::GPR(tmp),
        )?;
        self.move_location(Size::S32, Location::GPR(tmp), ret)?;
        self.release_gpr(tmp);
        Ok(())
    }
    fn emit_i32_copysign(&mut self, tmp1: GPR, tmp2: GPR) -> Result<(), CompileError> {
        self.assembler.emit_and(
            Size::S32,
            Location::GPR(tmp1),
            Location::Imm32(0x7fffffffu32),
            Location::GPR(tmp1),
        )?;
        self.assembler.emit_and(
            Size::S32,
            Location::GPR(tmp2),
            Location::Imm32(0x80000000u32),
            Location::GPR(tmp2),
        )?;
        self.assembler.emit_or(
            Size::S32,
            Location::GPR(tmp1),
            Location::GPR(tmp2),
            Location::GPR(tmp1),
        )
    }
    fn f32_sqrt(&mut self, loc: Location, ret: Location) -> Result<(), CompileError> {
        self.emit_relaxed_binop_neon(Assembler::emit_fsqrt, Size::S32, loc, ret, true)
    }
    fn f32_trunc(&mut self, loc: Location, ret: Location) -> Result<(), CompileError> {
        self.emit_relaxed_binop_neon(Assembler::emit_frintz, Size::S32, loc, ret, true)
    }
    fn f32_ceil(&mut self, loc: Location, ret: Location) -> Result<(), CompileError> {
        self.emit_relaxed_binop_neon(Assembler::emit_frintp, Size::S32, loc, ret, true)
    }
    fn f32_floor(&mut self, loc: Location, ret: Location) -> Result<(), CompileError> {
        self.emit_relaxed_binop_neon(Assembler::emit_frintm, Size::S32, loc, ret, true)
    }
    fn f32_nearest(&mut self, loc: Location, ret: Location) -> Result<(), CompileError> {
        self.emit_relaxed_binop_neon(Assembler::emit_frintn, Size::S32, loc, ret, true)
    }
    fn f32_cmp_ge(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        let mut temps = vec![];
        let dest = self.location_to_reg(Size::S32, ret, &mut temps, ImmType::None, false, None)?;
        self.emit_relaxed_binop_neon(Assembler::emit_fcmp, Size::S32, loc_b, loc_a, false)?;
        self.assembler.emit_cset(Size::S32, dest, Condition::Ls)?;
        if ret != dest {
            self.move_location(Size::S32, dest, ret)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn f32_cmp_gt(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        let mut temps = vec![];
        let dest = self.location_to_reg(Size::S32, ret, &mut temps, ImmType::None, false, None)?;
        self.emit_relaxed_binop_neon(Assembler::emit_fcmp, Size::S32, loc_b, loc_a, false)?;
        self.assembler.emit_cset(Size::S32, dest, Condition::Cc)?;
        if ret != dest {
            self.move_location(Size::S32, dest, ret)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn f32_cmp_le(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        let mut temps = vec![];
        let dest = self.location_to_reg(Size::S32, ret, &mut temps, ImmType::None, false, None)?;
        self.emit_relaxed_binop_neon(Assembler::emit_fcmp, Size::S32, loc_a, loc_b, false)?;
        self.assembler.emit_cset(Size::S32, dest, Condition::Ls)?;
        if ret != dest {
            self.move_location(Size::S32, dest, ret)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn f32_cmp_lt(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        let mut temps = vec![];
        let dest = self.location_to_reg(Size::S32, ret, &mut temps, ImmType::None, false, None)?;
        self.emit_relaxed_binop_neon(Assembler::emit_fcmp, Size::S32, loc_a, loc_b, false)?;
        self.assembler.emit_cset(Size::S32, dest, Condition::Cc)?;
        if ret != dest {
            self.move_location(Size::S32, dest, ret)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn f32_cmp_ne(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        let mut temps = vec![];
        let dest = self.location_to_reg(Size::S32, ret, &mut temps, ImmType::None, false, None)?;
        self.emit_relaxed_binop_neon(Assembler::emit_fcmp, Size::S32, loc_a, loc_b, false)?;
        self.assembler.emit_cset(Size::S32, dest, Condition::Ne)?;
        if ret != dest {
            self.move_location(Size::S32, dest, ret)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn f32_cmp_eq(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        let mut temps = vec![];
        let dest = self.location_to_reg(Size::S32, ret, &mut temps, ImmType::None, false, None)?;
        self.emit_relaxed_binop_neon(Assembler::emit_fcmp, Size::S32, loc_a, loc_b, false)?;
        self.assembler.emit_cset(Size::S32, dest, Condition::Eq)?;
        if ret != dest {
            self.move_location(Size::S32, dest, ret)?;
        }
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn f32_min(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        let mut temps = vec![];
        let old_fpcr = self.set_default_nan(&mut temps)?;
        self.emit_relaxed_binop3_neon(
            Assembler::emit_fmin,
            Size::S32,
            loc_a,
            loc_b,
            ret,
            ImmType::None,
        )?;
        self.restore_fpcr(old_fpcr)?;
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn f32_max(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        let mut temps = vec![];
        let old_fpcr = self.set_default_nan(&mut temps)?;
        self.emit_relaxed_binop3_neon(
            Assembler::emit_fmax,
            Size::S32,
            loc_a,
            loc_b,
            ret,
            ImmType::None,
        )?;
        self.restore_fpcr(old_fpcr)?;
        for r in temps {
            self.release_gpr(r);
        }
        Ok(())
    }
    fn f32_add(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_relaxed_binop3_neon(
            Assembler::emit_fadd,
            Size::S32,
            loc_a,
            loc_b,
            ret,
            ImmType::None,
        )
    }
    fn f32_sub(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_relaxed_binop3_neon(
            Assembler::emit_fsub,
            Size::S32,
            loc_a,
            loc_b,
            ret,
            ImmType::None,
        )
    }
    fn f32_mul(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_relaxed_binop3_neon(
            Assembler::emit_fmul,
            Size::S32,
            loc_a,
            loc_b,
            ret,
            ImmType::None,
        )
    }
    fn f32_div(
        &mut self,
        loc_a: Location,
        loc_b: Location,
        ret: Location,
    ) -> Result<(), CompileError> {
        self.emit_relaxed_binop3_neon(
            Assembler::emit_fdiv,
            Size::S32,
            loc_a,
            loc_b,
            ret,
            ImmType::None,
        )
    }

    fn gen_std_trampoline(
        &self,
        sig: &FunctionType,
        calling_convention: CallingConvention,
    ) -> Result<FunctionBody, CompileError> {
        gen_std_trampoline_arm64(sig, calling_convention)
    }
    // Generates dynamic import function call trampoline for a function type.
    fn gen_std_dynamic_import_trampoline(
        &self,
        vmoffsets: &VMOffsets,
        sig: &FunctionType,
        calling_convention: CallingConvention,
    ) -> Result<FunctionBody, CompileError> {
        gen_std_dynamic_import_trampoline_arm64(vmoffsets, sig, calling_convention)
    }
    // Singlepass calls import functions through a trampoline.
    fn gen_import_call_trampoline(
        &self,
        vmoffsets: &VMOffsets,
        index: FunctionIndex,
        sig: &FunctionType,
        calling_convention: CallingConvention,
    ) -> Result<CustomSection, CompileError> {
        gen_import_call_trampoline_arm64(vmoffsets, index, sig, calling_convention)
    }
    #[cfg(feature = "unwind")]
    fn gen_dwarf_unwind_info(&mut self, code_len: usize) -> Option<UnwindInstructions> {
        let mut instructions = vec![];
        for &(instruction_offset, ref inst) in &self.unwind_ops {
            let instruction_offset = instruction_offset as u32;
            match *inst {
                UnwindOps::PushFP { up_to_sp } => {
                    instructions.push((
                        instruction_offset,
                        CallFrameInstruction::CfaOffset(up_to_sp as i32),
                    ));
                    instructions.push((
                        instruction_offset,
                        CallFrameInstruction::Offset(AArch64::X29, -(up_to_sp as i32)),
                    ));
                }
                UnwindOps::Push2Regs {
                    reg1,
                    reg2,
                    up_to_sp,
                } => {
                    instructions.push((
                        instruction_offset,
                        CallFrameInstruction::CfaOffset(up_to_sp as i32),
                    ));
                    instructions.push((
                        instruction_offset,
                        CallFrameInstruction::Offset(dwarf_index(reg2), -(up_to_sp as i32) + 8),
                    ));
                    instructions.push((
                        instruction_offset,
                        CallFrameInstruction::Offset(dwarf_index(reg1), -(up_to_sp as i32)),
                    ));
                }
                UnwindOps::DefineNewFrame => {
                    instructions.push((
                        instruction_offset,
                        CallFrameInstruction::CfaRegister(AArch64::X29),
                    ));
                }
                UnwindOps::SaveRegister { reg, bp_neg_offset } => instructions.push((
                    instruction_offset,
                    CallFrameInstruction::Offset(dwarf_index(reg), -bp_neg_offset),
                )),
            }
        }
        Some(UnwindInstructions {
            instructions,
            len: code_len as u32,
        })
    }
    #[cfg(not(feature = "unwind"))]
    fn gen_dwarf_unwind_info(&mut self, _code_len: usize) -> Option<UnwindInstructions> {
        None
    }

    fn gen_windows_unwind_info(&mut self, _code_len: usize) -> Option<Vec<u8>> {
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_move_location(machine: &mut MachineARM64, size: Size) -> Result<(), CompileError> {
        machine.move_location(size, Location::GPR(GPR::X1), Location::GPR(GPR::X2))?;
        machine.move_location(size, Location::GPR(GPR::X1), Location::Memory(GPR::X2, 10))?;
        machine.move_location(size, Location::GPR(GPR::X1), Location::Memory(GPR::X2, -10))?;
        machine.move_location(
            size,
            Location::GPR(GPR::X1),
            Location::Memory(GPR::X2, 1024),
        )?;
        machine.move_location(
            size,
            Location::GPR(GPR::X1),
            Location::Memory(GPR::X2, -1024),
        )?;
        machine.move_location(size, Location::Memory(GPR::X2, 10), Location::GPR(GPR::X1))?;
        machine.move_location(size, Location::Memory(GPR::X2, -10), Location::GPR(GPR::X1))?;
        machine.move_location(
            size,
            Location::Memory(GPR::X2, 1024),
            Location::GPR(GPR::X1),
        )?;
        machine.move_location(
            size,
            Location::Memory(GPR::X2, -1024),
            Location::GPR(GPR::X1),
        )?;
        machine.move_location(size, Location::GPR(GPR::X1), Location::SIMD(NEON::V0))?;
        machine.move_location(size, Location::SIMD(NEON::V0), Location::GPR(GPR::X1))?;
        machine.move_location(
            size,
            Location::SIMD(NEON::V0),
            Location::Memory(GPR::X2, 10),
        )?;
        machine.move_location(
            size,
            Location::SIMD(NEON::V0),
            Location::Memory(GPR::X2, -10),
        )?;
        machine.move_location(
            size,
            Location::SIMD(NEON::V0),
            Location::Memory(GPR::X2, 1024),
        )?;
        machine.move_location(
            size,
            Location::SIMD(NEON::V0),
            Location::Memory(GPR::X2, -1024),
        )?;
        machine.move_location(
            size,
            Location::Memory(GPR::X2, 10),
            Location::SIMD(NEON::V0),
        )?;
        machine.move_location(
            size,
            Location::Memory(GPR::X2, -10),
            Location::SIMD(NEON::V0),
        )?;
        machine.move_location(
            size,
            Location::Memory(GPR::X2, 1024),
            Location::SIMD(NEON::V0),
        )?;
        machine.move_location(
            size,
            Location::Memory(GPR::X2, -1024),
            Location::SIMD(NEON::V0),
        )?;

        Ok(())
    }

    fn test_move_location_extended(
        machine: &mut MachineARM64,
        signed: bool,
        sized: Size,
    ) -> Result<(), CompileError> {
        machine.move_location_extend(
            sized,
            signed,
            Location::GPR(GPR::X0),
            Size::S64,
            Location::GPR(GPR::X1),
        )?;
        machine.move_location_extend(
            sized,
            signed,
            Location::GPR(GPR::X0),
            Size::S64,
            Location::Memory(GPR::X1, 10),
        )?;
        machine.move_location_extend(
            sized,
            signed,
            Location::GPR(GPR::X0),
            Size::S64,
            Location::Memory(GPR::X1, 16),
        )?;
        machine.move_location_extend(
            sized,
            signed,
            Location::GPR(GPR::X0),
            Size::S64,
            Location::Memory(GPR::X1, -16),
        )?;
        machine.move_location_extend(
            sized,
            signed,
            Location::GPR(GPR::X0),
            Size::S64,
            Location::Memory(GPR::X1, 1024),
        )?;
        machine.move_location_extend(
            sized,
            signed,
            Location::GPR(GPR::X0),
            Size::S64,
            Location::Memory(GPR::X1, -1024),
        )?;
        machine.move_location_extend(
            sized,
            signed,
            Location::Memory(GPR::X0, 10),
            Size::S64,
            Location::GPR(GPR::X1),
        )?;

        Ok(())
    }

    fn test_binop_op(
        machine: &mut MachineARM64,
        op: fn(&mut MachineARM64, Location, Location, Location) -> Result<(), CompileError>,
    ) -> Result<(), CompileError> {
        op(
            machine,
            Location::GPR(GPR::X2),
            Location::GPR(GPR::X2),
            Location::GPR(GPR::X0),
        )?;
        op(
            machine,
            Location::GPR(GPR::X2),
            Location::Imm32(10),
            Location::GPR(GPR::X0),
        )?;
        op(
            machine,
            Location::GPR(GPR::X0),
            Location::GPR(GPR::X0),
            Location::GPR(GPR::X0),
        )?;
        op(
            machine,
            Location::Imm32(10),
            Location::GPR(GPR::X2),
            Location::GPR(GPR::X0),
        )?;
        op(
            machine,
            Location::GPR(GPR::X0),
            Location::GPR(GPR::X2),
            Location::Memory(GPR::X0, 10),
        )?;
        op(
            machine,
            Location::GPR(GPR::X0),
            Location::Memory(GPR::X2, 16),
            Location::Memory(GPR::X0, 10),
        )?;
        op(
            machine,
            Location::Memory(GPR::X0, 0),
            Location::Memory(GPR::X2, 16),
            Location::Memory(GPR::X0, 10),
        )?;

        Ok(())
    }

    fn test_float_binop_op(
        machine: &mut MachineARM64,
        op: fn(&mut MachineARM64, Location, Location, Location) -> Result<(), CompileError>,
    ) -> Result<(), CompileError> {
        op(
            machine,
            Location::SIMD(NEON::V3),
            Location::SIMD(NEON::V2),
            Location::SIMD(NEON::V0),
        )?;
        op(
            machine,
            Location::SIMD(NEON::V0),
            Location::SIMD(NEON::V2),
            Location::SIMD(NEON::V0),
        )?;
        op(
            machine,
            Location::SIMD(NEON::V0),
            Location::SIMD(NEON::V0),
            Location::SIMD(NEON::V0),
        )?;
        op(
            machine,
            Location::Memory(GPR::X0, 0),
            Location::SIMD(NEON::V2),
            Location::SIMD(NEON::V0),
        )?;
        op(
            machine,
            Location::Memory(GPR::X0, 0),
            Location::Memory(GPR::X1, 10),
            Location::SIMD(NEON::V0),
        )?;
        op(
            machine,
            Location::Memory(GPR::X0, 0),
            Location::Memory(GPR::X1, 16),
            Location::Memory(GPR::X2, 32),
        )?;
        op(
            machine,
            Location::SIMD(NEON::V0),
            Location::Memory(GPR::X1, 16),
            Location::Memory(GPR::X2, 32),
        )?;
        op(
            machine,
            Location::SIMD(NEON::V0),
            Location::SIMD(NEON::V1),
            Location::Memory(GPR::X2, 32),
        )?;

        Ok(())
    }

    fn test_float_cmp_op(
        machine: &mut MachineARM64,
        op: fn(&mut MachineARM64, Location, Location, Location) -> Result<(), CompileError>,
    ) -> Result<(), CompileError> {
        op(
            machine,
            Location::SIMD(NEON::V3),
            Location::SIMD(NEON::V2),
            Location::GPR(GPR::X0),
        )?;
        op(
            machine,
            Location::SIMD(NEON::V0),
            Location::SIMD(NEON::V0),
            Location::GPR(GPR::X0),
        )?;
        op(
            machine,
            Location::Memory(GPR::X1, 0),
            Location::SIMD(NEON::V2),
            Location::GPR(GPR::X0),
        )?;
        op(
            machine,
            Location::Memory(GPR::X1, 0),
            Location::Memory(GPR::X2, 10),
            Location::GPR(GPR::X0),
        )?;
        op(
            machine,
            Location::Memory(GPR::X1, 0),
            Location::Memory(GPR::X2, 16),
            Location::Memory(GPR::X0, 32),
        )?;
        op(
            machine,
            Location::SIMD(NEON::V0),
            Location::Memory(GPR::X2, 16),
            Location::Memory(GPR::X0, 32),
        )?;
        op(
            machine,
            Location::SIMD(NEON::V0),
            Location::SIMD(NEON::V1),
            Location::Memory(GPR::X0, 32),
        )?;

        Ok(())
    }

    #[test]
    fn tests_arm64() -> Result<(), CompileError> {
        let mut machine = MachineARM64::new();

        test_move_location(&mut machine, Size::S32)?;
        test_move_location(&mut machine, Size::S64)?;
        test_move_location_extended(&mut machine, false, Size::S8)?;
        test_move_location_extended(&mut machine, false, Size::S16)?;
        test_move_location_extended(&mut machine, false, Size::S32)?;
        test_move_location_extended(&mut machine, true, Size::S8)?;
        test_move_location_extended(&mut machine, true, Size::S16)?;
        test_move_location_extended(&mut machine, true, Size::S32)?;
        test_binop_op(&mut machine, MachineARM64::emit_binop_add32)?;
        test_binop_op(&mut machine, MachineARM64::emit_binop_add64)?;
        test_binop_op(&mut machine, MachineARM64::emit_binop_sub32)?;
        test_binop_op(&mut machine, MachineARM64::emit_binop_sub64)?;
        test_binop_op(&mut machine, MachineARM64::emit_binop_and32)?;
        test_binop_op(&mut machine, MachineARM64::emit_binop_and64)?;
        test_binop_op(&mut machine, MachineARM64::emit_binop_xor32)?;
        test_binop_op(&mut machine, MachineARM64::emit_binop_xor64)?;
        test_binop_op(&mut machine, MachineARM64::emit_binop_or32)?;
        test_binop_op(&mut machine, MachineARM64::emit_binop_or64)?;
        test_binop_op(&mut machine, MachineARM64::emit_binop_mul32)?;
        test_binop_op(&mut machine, MachineARM64::emit_binop_mul64)?;
        test_float_binop_op(&mut machine, MachineARM64::f32_add)?;
        test_float_binop_op(&mut machine, MachineARM64::f32_sub)?;
        test_float_binop_op(&mut machine, MachineARM64::f32_mul)?;
        test_float_binop_op(&mut machine, MachineARM64::f32_div)?;
        test_float_cmp_op(&mut machine, MachineARM64::f32_cmp_eq)?;
        test_float_cmp_op(&mut machine, MachineARM64::f32_cmp_lt)?;
        test_float_cmp_op(&mut machine, MachineARM64::f32_cmp_le)?;

        Ok(())
    }
}
