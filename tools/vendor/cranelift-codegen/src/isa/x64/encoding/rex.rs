//! Encodes instructions in the standard x86 encoding mode. This is called IA-32E mode in the Intel
//! manuals but corresponds to the addition of the REX-prefix format (hence the name of this module)
//! that allowed encoding instructions in both compatibility mode (32-bit instructions running on a
//! 64-bit OS) and in 64-bit mode (using the full 64-bit address space).
//!
//! For all of the routines that take both a memory-or-reg operand (sometimes called "E" in the
//! Intel documentation, see the Intel Developer's manual, vol. 2, section A.2) and a reg-only
//! operand ("G" in Intelese), the order is always G first, then E. The term "enc" in the following
//! means "hardware register encoding number".

use crate::{
    ir::TrapCode,
    isa::x64::inst::{
        args::{Amode, OperandSize},
        regs, EmitInfo, EmitState, Inst, LabelUse,
    },
    machinst::MachBuffer,
};
use regalloc::{Reg, RegClass};

pub(crate) fn low8_will_sign_extend_to_64(x: u32) -> bool {
    let xs = (x as i32) as i64;
    xs == ((xs << 56) >> 56)
}

pub(crate) fn low8_will_sign_extend_to_32(x: u32) -> bool {
    let xs = x as i32;
    xs == ((xs << 24) >> 24)
}

/// Encode the ModR/M byte.
#[inline(always)]
pub fn encode_modrm(m0d: u8, enc_reg_g: u8, rm_e: u8) -> u8 {
    debug_assert!(m0d < 4);
    debug_assert!(enc_reg_g < 8);
    debug_assert!(rm_e < 8);
    ((m0d & 3) << 6) | ((enc_reg_g & 7) << 3) | (rm_e & 7)
}

#[inline(always)]
pub(crate) fn encode_sib(shift: u8, enc_index: u8, enc_base: u8) -> u8 {
    debug_assert!(shift < 4);
    debug_assert!(enc_index < 8);
    debug_assert!(enc_base < 8);
    ((shift & 3) << 6) | ((enc_index & 7) << 3) | (enc_base & 7)
}

/// Get the encoding number of a GPR.
#[inline(always)]
pub(crate) fn int_reg_enc(reg: impl Into<Reg>) -> u8 {
    let reg = reg.into();
    debug_assert!(reg.is_real());
    debug_assert_eq!(reg.get_class(), RegClass::I64);
    reg.get_hw_encoding()
}

/// Get the encoding number of any register.
#[inline(always)]
pub(crate) fn reg_enc(reg: impl Into<Reg>) -> u8 {
    let reg = reg.into();
    debug_assert!(reg.is_real());
    reg.get_hw_encoding()
}

/// A small bit field to record a REX prefix specification:
/// - bit 0 set to 1 indicates REX.W must be 0 (cleared).
/// - bit 1 set to 1 indicates the REX prefix must always be emitted.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub(crate) struct RexFlags(u8);

impl RexFlags {
    /// By default, set the W field, and don't always emit.
    #[inline(always)]
    pub(crate) fn set_w() -> Self {
        Self(0)
    }
    /// Creates a new RexPrefix for which the REX.W bit will be cleared.
    #[inline(always)]
    pub(crate) fn clear_w() -> Self {
        Self(1)
    }

    #[inline(always)]
    pub(crate) fn always_emit(&mut self) -> &mut Self {
        self.0 = self.0 | 2;
        self
    }

    #[inline(always)]
    pub(crate) fn always_emit_if_8bit_needed(&mut self, reg: Reg) -> &mut Self {
        let enc_reg = int_reg_enc(reg);
        if enc_reg >= 4 && enc_reg <= 7 {
            self.always_emit();
        }
        self
    }

    #[inline(always)]
    pub(crate) fn must_clear_w(&self) -> bool {
        (self.0 & 1) != 0
    }
    #[inline(always)]
    pub(crate) fn must_always_emit(&self) -> bool {
        (self.0 & 2) != 0
    }

    #[inline(always)]
    pub(crate) fn emit_two_op(&self, sink: &mut MachBuffer<Inst>, enc_g: u8, enc_e: u8) {
        let w = if self.must_clear_w() { 0 } else { 1 };
        let r = (enc_g >> 3) & 1;
        let x = 0;
        let b = (enc_e >> 3) & 1;
        let rex = 0x40 | (w << 3) | (r << 2) | (x << 1) | b;
        if rex != 0x40 || self.must_always_emit() {
            sink.put1(rex);
        }
    }

    #[inline(always)]
    pub fn emit_three_op(
        &self,
        sink: &mut MachBuffer<Inst>,
        enc_g: u8,
        enc_index: u8,
        enc_base: u8,
    ) {
        let w = if self.must_clear_w() { 0 } else { 1 };
        let r = (enc_g >> 3) & 1;
        let x = (enc_index >> 3) & 1;
        let b = (enc_base >> 3) & 1;
        let rex = 0x40 | (w << 3) | (r << 2) | (x << 1) | b;
        if rex != 0x40 || self.must_always_emit() {
            sink.put1(rex);
        }
    }
}

/// Generate the proper Rex flags for the given operand size.
impl From<OperandSize> for RexFlags {
    fn from(size: OperandSize) -> Self {
        match size {
            OperandSize::Size64 => RexFlags::set_w(),
            _ => RexFlags::clear_w(),
        }
    }
}
/// Generate Rex flags for an OperandSize/register tuple.
impl From<(OperandSize, Reg)> for RexFlags {
    fn from((size, reg): (OperandSize, Reg)) -> Self {
        let mut rex = RexFlags::from(size);
        if size == OperandSize::Size8 {
            rex.always_emit_if_8bit_needed(reg);
        }
        rex
    }
}

/// Allows using the same opcode byte in different "opcode maps" to allow for more instruction
/// encodings. See appendix A in the Intel Software Developer's Manual, volume 2A, for more details.
#[allow(missing_docs)]
pub enum OpcodeMap {
    None,
    _0F,
    _0F38,
    _0F3A,
}

impl OpcodeMap {
    /// Normally the opcode map is specified as bytes in the instruction, but some x64 encoding
    /// formats pack this information as bits in a prefix (e.g. EVEX).
    pub(crate) fn bits(&self) -> u8 {
        match self {
            OpcodeMap::None => 0b00,
            OpcodeMap::_0F => 0b01,
            OpcodeMap::_0F38 => 0b10,
            OpcodeMap::_0F3A => 0b11,
        }
    }
}

impl Default for OpcodeMap {
    fn default() -> Self {
        Self::None
    }
}

/// We may need to include one or more legacy prefix bytes before the REX prefix.  This enum
/// covers only the small set of possibilities that we actually need.
pub enum LegacyPrefixes {
    /// No prefix bytes.
    None,
    /// Operand Size Override -- here, denoting "16-bit operation".
    _66,
    /// The Lock prefix.
    _F0,
    /// Operand size override and Lock.
    _66F0,
    /// REPNE, but no specific meaning here -- is just an opcode extension.
    _F2,
    /// REP/REPE, but no specific meaning here -- is just an opcode extension.
    _F3,
    /// Operand size override and same effect as F3.
    _66F3,
}

impl LegacyPrefixes {
    /// Emit the legacy prefix as bytes (e.g. in REX instructions).
    #[inline(always)]
    pub(crate) fn emit(&self, sink: &mut MachBuffer<Inst>) {
        match self {
            Self::_66 => sink.put1(0x66),
            Self::_F0 => sink.put1(0xF0),
            Self::_66F0 => {
                // I don't think the order matters, but in any case, this is the same order that
                // the GNU assembler uses.
                sink.put1(0x66);
                sink.put1(0xF0);
            }
            Self::_F2 => sink.put1(0xF2),
            Self::_F3 => sink.put1(0xF3),
            Self::_66F3 => {
                sink.put1(0x66);
                sink.put1(0xF3);
            }
            Self::None => (),
        }
    }

    /// Emit the legacy prefix as bits (e.g. for EVEX instructions).
    #[inline(always)]
    pub(crate) fn bits(&self) -> u8 {
        match self {
            Self::None => 0b00,
            Self::_66 => 0b01,
            Self::_F3 => 0b10,
            Self::_F2 => 0b11,
            _ => panic!(
                "VEX and EVEX bits can only be extracted from single prefixes: None, 66, F3, F2"
            ),
        }
    }
}

impl Default for LegacyPrefixes {
    fn default() -> Self {
        Self::None
    }
}

/// This is the core 'emit' function for instructions that reference memory.
///
/// For an instruction that has as operands a reg encoding `enc_g` and a memory address `mem_e`,
/// create and emit:
/// - first the legacy prefixes, if any
/// - then the REX prefix, if needed
/// - then caller-supplied opcode byte(s) (`opcodes` and `num_opcodes`),
/// - then the MOD/RM byte,
/// - then optionally, a SIB byte,
/// - and finally optionally an immediate that will be derived from the `mem_e` operand.
///
/// For most instructions up to and including SSE4.2, that will be the whole instruction: this is
/// what we call "standard" instructions, and abbreviate "std" in the name here. VEX-prefixed
/// instructions will require their own emitter functions.
///
/// This will also work for 32-bits x86 instructions, assuming no REX prefix is provided.
///
/// The opcodes are written bigendianly for the convenience of callers.  For example, if the opcode
/// bytes to be emitted are, in this order, F3 0F 27, then the caller should pass `opcodes` ==
/// 0xF3_0F_27 and `num_opcodes` == 3.
///
/// The register operand is represented here not as a `Reg` but as its hardware encoding, `enc_g`.
/// `rex` can specify special handling for the REX prefix.  By default, the REX prefix will
/// indicate a 64-bit operation and will be deleted if it is redundant (0x40).  Note that for a
/// 64-bit operation, the REX prefix will normally never be redundant, since REX.W must be 1 to
/// indicate a 64-bit operation.
pub(crate) fn emit_std_enc_mem(
    sink: &mut MachBuffer<Inst>,
    state: &EmitState,
    info: &EmitInfo,
    prefixes: LegacyPrefixes,
    opcodes: u32,
    mut num_opcodes: usize,
    enc_g: u8,
    mem_e: &Amode,
    rex: RexFlags,
) {
    // General comment for this function: the registers in `mem_e` must be
    // 64-bit integer registers, because they are part of an address
    // expression.  But `enc_g` can be derived from a register of any class.

    let srcloc = state.cur_srcloc();
    let can_trap = mem_e.can_trap();
    if can_trap {
        sink.add_trap(srcloc, TrapCode::HeapOutOfBounds);
    }

    prefixes.emit(sink);

    match mem_e {
        Amode::ImmReg { simm32, base, .. } => {
            // If this is an access based off of RSP, it may trap with a stack overflow if it's the
            // first touch of a new stack page.
            if *base == regs::rsp() && !can_trap && info.flags.enable_probestack() {
                sink.add_trap(srcloc, TrapCode::StackOverflow);
            }

            // First, the REX byte.
            let enc_e = int_reg_enc(*base);
            rex.emit_two_op(sink, enc_g, enc_e);

            // Now the opcode(s).  These include any other prefixes the caller
            // hands to us.
            while num_opcodes > 0 {
                num_opcodes -= 1;
                sink.put1(((opcodes >> (num_opcodes << 3)) & 0xFF) as u8);
            }

            // Now the mod/rm and associated immediates.  This is
            // significantly complicated due to the multiple special cases.
            if *simm32 == 0
                && enc_e != regs::ENC_RSP
                && enc_e != regs::ENC_RBP
                && enc_e != regs::ENC_R12
                && enc_e != regs::ENC_R13
            {
                // FIXME JRS 2020Feb11: those four tests can surely be
                // replaced by a single mask-and-compare check.  We should do
                // that because this routine is likely to be hot.
                sink.put1(encode_modrm(0, enc_g & 7, enc_e & 7));
            } else if *simm32 == 0 && (enc_e == regs::ENC_RSP || enc_e == regs::ENC_R12) {
                sink.put1(encode_modrm(0, enc_g & 7, 4));
                sink.put1(0x24);
            } else if low8_will_sign_extend_to_32(*simm32)
                && enc_e != regs::ENC_RSP
                && enc_e != regs::ENC_R12
            {
                sink.put1(encode_modrm(1, enc_g & 7, enc_e & 7));
                sink.put1((simm32 & 0xFF) as u8);
            } else if enc_e != regs::ENC_RSP && enc_e != regs::ENC_R12 {
                sink.put1(encode_modrm(2, enc_g & 7, enc_e & 7));
                sink.put4(*simm32);
            } else if (enc_e == regs::ENC_RSP || enc_e == regs::ENC_R12)
                && low8_will_sign_extend_to_32(*simm32)
            {
                // REX.B distinguishes RSP from R12
                sink.put1(encode_modrm(1, enc_g & 7, 4));
                sink.put1(0x24);
                sink.put1((simm32 & 0xFF) as u8);
            } else if enc_e == regs::ENC_R12 || enc_e == regs::ENC_RSP {
                //.. wait for test case for RSP case
                // REX.B distinguishes RSP from R12
                sink.put1(encode_modrm(2, enc_g & 7, 4));
                sink.put1(0x24);
                sink.put4(*simm32);
            } else {
                unreachable!("ImmReg");
            }
        }

        Amode::ImmRegRegShift {
            simm32,
            base: reg_base,
            index: reg_index,
            shift,
            ..
        } => {
            // If this is an access based off of RSP, it may trap with a stack overflow if it's the
            // first touch of a new stack page.
            if *reg_base == regs::rsp() && !can_trap && info.flags.enable_probestack() {
                sink.add_trap(srcloc, TrapCode::StackOverflow);
            }

            let enc_base = int_reg_enc(*reg_base);
            let enc_index = int_reg_enc(*reg_index);

            // The rex byte.
            rex.emit_three_op(sink, enc_g, enc_index, enc_base);

            // All other prefixes and opcodes.
            while num_opcodes > 0 {
                num_opcodes -= 1;
                sink.put1(((opcodes >> (num_opcodes << 3)) & 0xFF) as u8);
            }

            // modrm, SIB, immediates.
            if low8_will_sign_extend_to_32(*simm32) && enc_index != regs::ENC_RSP {
                sink.put1(encode_modrm(1, enc_g & 7, 4));
                sink.put1(encode_sib(*shift, enc_index & 7, enc_base & 7));
                sink.put1(*simm32 as u8);
            } else if enc_index != regs::ENC_RSP {
                sink.put1(encode_modrm(2, enc_g & 7, 4));
                sink.put1(encode_sib(*shift, enc_index & 7, enc_base & 7));
                sink.put4(*simm32);
            } else {
                panic!("ImmRegRegShift");
            }
        }

        Amode::RipRelative { ref target } => {
            // First, the REX byte, with REX.B = 0.
            rex.emit_two_op(sink, enc_g, 0);

            // Now the opcode(s).  These include any other prefixes the caller
            // hands to us.
            while num_opcodes > 0 {
                num_opcodes -= 1;
                sink.put1(((opcodes >> (num_opcodes << 3)) & 0xFF) as u8);
            }

            // RIP-relative is mod=00, rm=101.
            sink.put1(encode_modrm(0, enc_g & 7, 0b101));

            let offset = sink.cur_offset();
            sink.use_label_at_offset(offset, *target, LabelUse::JmpRel32);
            sink.put4(0);
        }
    }
}

/// This is the core 'emit' function for instructions that do not reference memory.
///
/// This is conceptually the same as emit_modrm_sib_enc_ge, except it is for the case where the E
/// operand is a register rather than memory.  Hence it is much simpler.
pub(crate) fn emit_std_enc_enc(
    sink: &mut MachBuffer<Inst>,
    prefixes: LegacyPrefixes,
    opcodes: u32,
    mut num_opcodes: usize,
    enc_g: u8,
    enc_e: u8,
    rex: RexFlags,
) {
    // EncG and EncE can be derived from registers of any class, and they
    // don't even have to be from the same class.  For example, for an
    // integer-to-FP conversion insn, one might be RegClass::I64 and the other
    // RegClass::V128.

    // The legacy prefixes.
    prefixes.emit(sink);

    // The rex byte.
    rex.emit_two_op(sink, enc_g, enc_e);

    // All other prefixes and opcodes.
    while num_opcodes > 0 {
        num_opcodes -= 1;
        sink.put1(((opcodes >> (num_opcodes << 3)) & 0xFF) as u8);
    }

    // Now the mod/rm byte.  The instruction we're generating doesn't access
    // memory, so there is no SIB byte or immediate -- we're done.
    sink.put1(encode_modrm(3, enc_g & 7, enc_e & 7));
}

// These are merely wrappers for the above two functions that facilitate passing
// actual `Reg`s rather than their encodings.

pub(crate) fn emit_std_reg_mem(
    sink: &mut MachBuffer<Inst>,
    state: &EmitState,
    info: &EmitInfo,
    prefixes: LegacyPrefixes,
    opcodes: u32,
    num_opcodes: usize,
    reg_g: Reg,
    mem_e: &Amode,
    rex: RexFlags,
) {
    let enc_g = reg_enc(reg_g);
    emit_std_enc_mem(
        sink,
        state,
        info,
        prefixes,
        opcodes,
        num_opcodes,
        enc_g,
        mem_e,
        rex,
    );
}

pub(crate) fn emit_std_reg_reg(
    sink: &mut MachBuffer<Inst>,
    prefixes: LegacyPrefixes,
    opcodes: u32,
    num_opcodes: usize,
    reg_g: Reg,
    reg_e: Reg,
    rex: RexFlags,
) {
    let enc_g = reg_enc(reg_g);
    let enc_e = reg_enc(reg_e);
    emit_std_enc_enc(sink, prefixes, opcodes, num_opcodes, enc_g, enc_e, rex);
}

/// Write a suitable number of bits from an imm64 to the sink.
pub(crate) fn emit_simm(sink: &mut MachBuffer<Inst>, size: u8, simm32: u32) {
    match size {
        8 | 4 => sink.put4(simm32),
        2 => sink.put2(simm32 as u16),
        1 => sink.put1(simm32 as u8),
        _ => unreachable!(),
    }
}
