//! Encodes VEX instructions. These instructions are those added by the Advanced Vector Extensions
//! (AVX).

use super::evex::Register;
use super::rex::{LegacyPrefixes, OpcodeMap};
use super::ByteSink;
use crate::isa::x64::encoding::rex::encode_modrm;

/// Constructs a VEX-encoded instruction using a builder pattern. This approach makes it visually
/// easier to transform something the manual's syntax, `VEX.128.66.0F 73 /7 ib` to code:
/// `VexInstruction::new().length(...).prefix(...).map(...).w(true).opcode(0x1F).reg(...).rm(...)`.
pub struct VexInstruction {
    length: VexVectorLength,
    prefix: LegacyPrefixes,
    map: OpcodeMap,
    opcode: u8,
    w: bool,
    reg: u8,
    rm: Register,
    vvvv: Option<Register>,
    imm: Option<u8>,
}

impl Default for VexInstruction {
    fn default() -> Self {
        Self {
            length: VexVectorLength::default(),
            prefix: LegacyPrefixes::None,
            map: OpcodeMap::None,
            opcode: 0x00,
            w: false,
            reg: 0x00,
            rm: Register::default(),
            vvvv: None,
            imm: None,
        }
    }
}

impl VexInstruction {
    /// Construct a default VEX instruction.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the length of the instruction.
    #[inline(always)]
    pub fn length(mut self, length: VexVectorLength) -> Self {
        self.length = length;
        self
    }

    /// Set the legacy prefix byte of the instruction: None | 66 | F2 | F3. VEX instructions
    /// pack these into the prefix, not as separate bytes.
    #[inline(always)]
    pub fn prefix(mut self, prefix: LegacyPrefixes) -> Self {
        debug_assert!(
            prefix == LegacyPrefixes::None
                || prefix == LegacyPrefixes::_66
                || prefix == LegacyPrefixes::_F2
                || prefix == LegacyPrefixes::_F3
        );

        self.prefix = prefix;
        self
    }

    /// Set the opcode map byte of the instruction: None | 0F | 0F38 | 0F3A. VEX instructions pack
    /// these into the prefix, not as separate bytes.
    #[inline(always)]
    pub fn map(mut self, map: OpcodeMap) -> Self {
        self.map = map;
        self
    }

    /// Set the W bit, denoted by `.W1` or `.W0` in the instruction string.
    /// Typically used to indicate an instruction using 64 bits of an operand (e.g.
    /// 64 bit lanes). EVEX packs this bit in the EVEX prefix; previous encodings used the REX
    /// prefix.
    #[inline(always)]
    pub fn w(mut self, w: bool) -> Self {
        self.w = w;
        self
    }

    /// Set the instruction opcode byte.
    #[inline(always)]
    pub fn opcode(mut self, opcode: u8) -> Self {
        self.opcode = opcode;
        self
    }

    /// Set the register to use for the `reg` bits; many instructions use this as the write operand.
    #[inline(always)]
    pub fn reg(mut self, reg: impl Into<Register>) -> Self {
        self.reg = reg.into().into();
        self
    }

    /// Some instructions use the ModRM.reg field as an opcode extension. This is usually denoted by
    /// a `/n` field in the manual.
    #[inline(always)]
    pub fn opcode_ext(mut self, n: u8) -> Self {
        self.reg = n;
        self
    }

    /// Set the register to use for the `rm` bits; many instructions use this as the "read from
    /// register/memory" operand. Currently this does not support memory addressing (TODO).Setting
    /// this affects both the ModRM byte (`rm` section) and the VEX prefix (the extension bits for
    /// register encodings > 8).
    #[inline(always)]
    pub fn rm(mut self, reg: impl Into<Register>) -> Self {
        self.rm = reg.into();
        self
    }

    /// Set the `vvvv` register; some instructions allow using this as a second, non-destructive
    /// source register in 3-operand instructions (e.g. 2 read, 1 write).
    #[allow(dead_code)]
    #[inline(always)]
    pub fn vvvv(mut self, reg: impl Into<Register>) -> Self {
        self.vvvv = Some(reg.into());
        self
    }

    /// Set the imm byte when used for a register. The reg bits are stored in `imm8[7:4]` with
    /// the lower bits unused. Overrides a previously set [Self::imm] field.
    #[inline(always)]
    pub fn imm_reg(mut self, reg: impl Into<Register>) -> Self {
        let reg: u8 = reg.into().into();
        self.imm = Some((reg & 0xf) << 4);
        self
    }

    /// Set the imm byte.
    /// Overrides a previously set [Self::imm_reg] field.
    #[inline(always)]
    pub fn imm(mut self, imm: u8) -> Self {
        self.imm = Some(imm);
        self
    }

    /// The R bit in encoded format (inverted).
    #[inline(always)]
    fn r_bit(&self) -> u8 {
        (!(self.reg >> 3)) & 1
    }

    /// The X bit in encoded format (inverted).
    #[inline(always)]
    fn x_bit(&self) -> u8 {
        // TODO
        (!0) & 1
    }

    /// The B bit in encoded format (inverted).
    #[inline(always)]
    fn b_bit(&self) -> u8 {
        let rm: u8 = self.rm.into();
        (!(rm >> 3)) & 1
    }

    /// Is the 2 byte prefix available for this instruction?
    /// We essentially just check if we need any of the bits that are only available
    /// in the 3 byte instruction
    #[inline(always)]
    fn use_2byte_prefix(&self) -> bool {
        // These bits are only represented on the 3 byte prefix, so their presence
        // implies the use of the 3 byte prefix
        self.b_bit() == 1 && self.x_bit() == 1 &&
        // The presence of W1 in the opcode column implies the opcode must be encoded using the
        // 3-byte form of the VEX prefix.
        self.w == false &&
        // The presence of 0F3A and 0F38 in the opcode column implies that opcode can only be
        // encoded by the three-byte form of VEX
        !(self.map == OpcodeMap::_0F3A || self.map == OpcodeMap::_0F38)
    }
    /// The last byte of the 2byte and 3byte prefixes is mostly the same, share the common
    /// encoding logic here.
    #[inline(always)]
    fn prefix_last_byte(&self) -> u8 {
        let vvvv = self.vvvv.map(|r| r.into()).unwrap_or(0x00);

        let mut byte = 0x00;
        byte |= self.prefix.bits();
        byte |= self.length.bits() << 2;
        byte |= ((!vvvv) & 0xF) << 3;
        byte
    }

    /// Encode the 2 byte prefix
    #[inline(always)]
    fn encode_2byte_prefix<CS: ByteSink + ?Sized>(&self, sink: &mut CS) {
        //  2 bytes:
        //    +-----+ +-------------------+
        //    | C5h | | R | vvvv | L | pp |
        //    +-----+ +-------------------+

        let last_byte = self.prefix_last_byte() | (self.r_bit() << 7);

        sink.put1(0xC5);
        sink.put1(last_byte);
    }

    /// Encode the 3 byte prefix
    #[inline(always)]
    fn encode_3byte_prefix<CS: ByteSink + ?Sized>(&self, sink: &mut CS) {
        //  3 bytes:
        //    +-----+ +--------------+ +-------------------+
        //    | C4h | | RXB | m-mmmm | | W | vvvv | L | pp |
        //    +-----+ +--------------+ +-------------------+

        let mut second_byte = 0x00;
        second_byte |= self.map.bits(); // m-mmmm field
        second_byte |= self.b_bit() << 5;
        second_byte |= self.x_bit() << 6;
        second_byte |= self.r_bit() << 7;

        let w_bit = self.w as u8;
        let last_byte = self.prefix_last_byte() | (w_bit << 7);

        sink.put1(0xC4);
        sink.put1(second_byte);
        sink.put1(last_byte);
    }

    /// Emit the VEX-encoded instruction to the code sink:
    pub fn encode<CS: ByteSink + ?Sized>(&self, sink: &mut CS) {
        // 2/3 byte prefix
        if self.use_2byte_prefix() {
            self.encode_2byte_prefix(sink);
        } else {
            self.encode_3byte_prefix(sink);
        }

        // 1 Byte Opcode
        sink.put1(self.opcode);

        // 1 ModRM Byte
        // Not all instructions use Reg as a reg, some use it as an extension of the opcode.
        let rm: u8 = self.rm.into();
        sink.put1(encode_modrm(3, self.reg & 7, rm & 7));

        // TODO: 0/1 byte SIB
        // TODO: 0/1/2/4 bytes DISP

        // Optional 1 Byte imm
        if let Some(imm) = self.imm {
            sink.put1(imm);
        }
    }
}

/// The VEX format allows choosing a vector length in the `L` bit.
#[allow(dead_code, missing_docs)] // Wider-length vectors are not yet used.
pub enum VexVectorLength {
    V128,
    V256,
}

impl VexVectorLength {
    /// Encode the `L` bit.
    fn bits(&self) -> u8 {
        match self {
            Self::V128 => 0b0,
            Self::V256 => 0b1,
        }
    }
}

impl Default for VexVectorLength {
    fn default() -> Self {
        Self::V128
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::isa::x64::inst::regs;
    use std::vec::Vec;

    #[test]
    fn vpslldq() {
        // VEX.128.66.0F 73 /7 ib
        // VPSLLDQ xmm1, xmm2, imm8

        let dst = regs::xmm1().to_real_reg().unwrap().hw_enc();
        let src = regs::xmm2().to_real_reg().unwrap().hw_enc();
        let mut sink0 = Vec::new();

        VexInstruction::new()
            .length(VexVectorLength::V128)
            .prefix(LegacyPrefixes::_66)
            .map(OpcodeMap::_0F)
            .opcode(0x73)
            .opcode_ext(7)
            .vvvv(dst)
            .rm(src)
            .imm(0x17)
            .encode(&mut sink0);

        assert_eq!(sink0, vec![0xc5, 0xf1, 0x73, 0xfa, 0x17]);
    }

    #[test]
    fn vblendvpd() {
        // A four operand instruction
        // VEX.128.66.0F3A.W0 4B /r /is4
        // VBLENDVPD xmm1, xmm2, xmm3, xmm4

        let dst = regs::xmm1().to_real_reg().unwrap().hw_enc();
        let a = regs::xmm2().to_real_reg().unwrap().hw_enc();
        let b = regs::xmm3().to_real_reg().unwrap().hw_enc();
        let c = regs::xmm4().to_real_reg().unwrap().hw_enc();
        let mut sink0 = Vec::new();

        VexInstruction::new()
            .length(VexVectorLength::V128)
            .prefix(LegacyPrefixes::_66)
            .map(OpcodeMap::_0F3A)
            .w(false)
            .opcode(0x4B)
            .reg(dst)
            .vvvv(a)
            .rm(b)
            .imm_reg(c)
            .encode(&mut sink0);

        assert_eq!(sink0, vec![0xc4, 0xe3, 0x69, 0x4b, 0xcb, 0x40]);
    }

    #[test]
    fn vcmpps() {
        // VEX.128.0F.WIG C2 /r ib
        // VCMPPS ymm10, ymm11, ymm12, 4 // neq

        let dst = regs::xmm10().to_real_reg().unwrap().hw_enc();
        let a = regs::xmm11().to_real_reg().unwrap().hw_enc();
        let b = regs::xmm12().to_real_reg().unwrap().hw_enc();
        let mut sink0 = Vec::new();

        VexInstruction::new()
            .length(VexVectorLength::V256)
            .prefix(LegacyPrefixes::None)
            .map(OpcodeMap::_0F)
            .opcode(0xC2)
            .reg(dst)
            .vvvv(a)
            .rm(b)
            .imm(4)
            .encode(&mut sink0);

        assert_eq!(sink0, vec![0xc4, 0x41, 0x24, 0xc2, 0xd4, 0x04]);
    }
}
