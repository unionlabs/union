use syn::spanned::Spanned;
use proc_macro2::{Span, TokenTree};
use quote::{quote_spanned};
use proc_macro_error::emit_error;

use crate::common::{Stmt, Size, Jump, JumpKind, delimited};
use crate::serialize;

use super::{Context, X86Mode};
use super::ast::{RawArg, CleanArg, SizedArg, Instruction, MemoryRefItem, Register, RegKind, RegFamily, RegId};
use super::x64data::get_mnemnonic_data;
use super::x64data::Flags;
use super::x64data::Features;
use super::debug::format_opdata_list;

use std::mem::swap;
use std::slice;
use std::iter;


/*
 * Instruction encoding data formats
 */

#[derive(Debug)]
pub struct Opdata {
    pub args:  &'static [u8],  // format string of arg format
    pub ops:   &'static [u8],
    pub reg:   u8,
    pub flags: Flags,
    pub features: Features
}

pub struct FormatStringIterator<'a> {
    inner: iter::Cloned<slice::Iter<'a, u8>>
}

impl<'a> FormatStringIterator<'a> {
    pub fn new(buf: &'a [u8]) -> FormatStringIterator<'a> {
        FormatStringIterator { inner: buf.iter().cloned() }
    }
}

impl<'a> Iterator for FormatStringIterator<'a> {
    type Item = (u8, u8);

    fn next(&mut self) -> Option<(u8, u8)> {
        if let Some(ty) = self.inner.next() {
            let size = self.inner.next().expect("Invalid format string data");
            Some((ty, size))
        } else {
            None
        }
    }
}

/*
 * Instruction encoding constants
 */

const MOD_DIRECT: u8 = 0b11;
const MOD_NODISP: u8 = 0b00; // normal addressing
const MOD_NOBASE: u8 = 0b00; // VSIB addressing
const MOD_DISP8:  u8 = 0b01;
const MOD_DISP32: u8 = 0b10;


#[derive(Debug, Clone, Copy)]
enum RelocationKind {
    /// A rip-relative relocation. No need to keep track of.
    Relative,
    // An absolute offset to a rip-relative location.
    Absolute,
    // A relative offset to an absolute location,
    Extern,
}

impl RelocationKind {
    fn to_id(self) -> u8 {
        match self {
            RelocationKind::Relative => 0,
            RelocationKind::Absolute => 1,
            RelocationKind::Extern   => 2
        }
    }
}

/*
 * Implementation
 */

pub(super) fn compile_instruction(ctx: &mut Context, instruction: Instruction, args: Vec<RawArg>) -> Result<(), Option<String>> {
    let mut ops = instruction.idents;
    let op = ops.pop().unwrap();
    let prefixes = ops;

    // Fold RawArgs into CleanArgs
    let mut args = args.into_iter().map(clean_memoryref).collect::<Result<Vec<CleanArg>, _>>()?;

    // sanitize memory references, determine address size, and size immediates/displacements if possible
    let addr_size = sanitize_indirects_and_sizes(&ctx, &mut args)?;
    let addr_size = addr_size.unwrap_or(match ctx.mode {
        X86Mode::Long => Size::QWORD,
        X86Mode::Protected => Size::DWORD
    });

    // determine if we need an address size override prefix
    let pref_addr = match (ctx.mode, addr_size) {
        (X86Mode::Long, Size::QWORD) => false,
        (X86Mode::Long, Size::DWORD) => true,
        (X86Mode::Protected, Size::DWORD) => false,
        (X86Mode::Protected, Size::WORD) => true,
        _ => return Err(Some("Impossible address size".into()))
    };

    // find a matching op
    let data = match_op_format(ctx, &op, &args)?;

    // determine if the features required for this op are fulfilled
    if !ctx.features.contains(data.features) {
        return Err(Some(format!(
            "This instruction uses features that are not indicated to be available: {}",
            data.features - ctx.features
        )));
    }

    // determine legacy prefixes
    let (mut pref_mod, pref_seg) = get_legacy_prefixes(data, prefixes)?;

    // fill in size info from the format string to create the final SizedArg vec
    let (op_size, args) = size_operands(data, args)?;

    let mut pref_size = false;
    let mut rex_w = false;
    let mut vex_l = false;

    // determine if size prefixes are necessary
    if data.flags.intersects(Flags::AUTO_SIZE | Flags::AUTO_NO32 | Flags::AUTO_REXW | Flags::AUTO_VEXL) {
        // if any of these flags are true an operand size should've been calculated
        let op_size = op_size.expect("Bad formatting data? No wildcard sizes");

        match ctx.mode {
            X86Mode::Protected => if op_size == Size::QWORD {
                return Err(Some(format!("'{}': Does not support 64 bit operands in 32-bit mode", op.to_string())));
            },
            X86Mode::Long => ()
        }

        if data.flags.contains(Flags::AUTO_NO32) {
            match (op_size, ctx.mode) {
                (Size::WORD, _) => pref_size = true,
                (Size::QWORD, X86Mode::Long) => (),
                (Size::DWORD, X86Mode::Protected) => (),
                (Size::DWORD, X86Mode::Long) => return Err(Some(format!("'{}': Does not support 32 bit operands in 64-bit mode", op.to_string()))),
                (_, _) => panic!("bad formatting data"),
            }
        } else if data.flags.contains(Flags::AUTO_REXW) {
            if op_size == Size::QWORD {
                rex_w = true;
            } else if op_size != Size::DWORD {
                return Err(Some(format!("'{}': Does not support 16-bit operands", op.to_string())));
            }
        } else if data.flags.contains(Flags::AUTO_VEXL) {
            if op_size == Size::HWORD {
                vex_l = true;
            } else if op_size != Size::OWORD {
                panic!("bad formatting data");
            }
        } else if op_size == Size::WORD {
            pref_size = true;
        } else if op_size == Size::QWORD {
            rex_w = true;
        } else if op_size != Size::DWORD {
            panic!("bad formatting data");
        }
    }

    // mandatory prefixes
    let pref_size = pref_size || data.flags.contains(Flags::WORD_SIZE);
    let rex_w     = rex_w     || data.flags.contains(Flags::WITH_REXW);
    let vex_l     = vex_l     || data.flags.contains(Flags::WITH_VEXL);
    let pref_addr = pref_addr || data.flags.contains(Flags::PREF_67);

    if        data.flags.contains(Flags::PREF_F0) { pref_mod = Some(0xF0);
    } else if data.flags.contains(Flags::PREF_F2) { pref_mod = Some(0xF2);
    } else if data.flags.contains(Flags::PREF_F3) { pref_mod = Some(0xF3);
    }

    // check if this combination of args can actually be encoded and whether a rex prefix is necessary
    let need_rex = check_rex(ctx, data, &args, rex_w)?;

    // split args
    let (mut rm, reg, vvvv, ireg, mut args) = extract_args(data, args);

    // we'll need this to keep track of where relocations need to be made
    // (target, offset, size, kind)
    let mut relocations = Vec::new();

    let mut ops = data.ops;

    // deal with ops that encode the final byte in an immediate
    let immediate_opcode = if data.flags.intersects(Flags::IMM_OP) {
        let (&imm, rest) = ops.split_last().expect("bad formatting data");
        ops = rest;
        Some(imm)
    } else {
        None
    };

    // shorthand
    let buffer = &mut ctx.state.stmts;

    // legacy-only prefixes
    if let Some(pref) = pref_seg {
        buffer.push(Stmt::u8(pref));
    }
    if pref_addr {
        buffer.push(Stmt::u8(0x67));
    }

    // VEX/XOP prefixes embed the operand size prefix / modification prefixes in them.
    if data.flags.intersects(Flags::VEX_OP | Flags::XOP_OP) {
        let prefix = if pref_size        { 0b01
        } else if pref_mod == Some(0xF3) { 0b10
        } else if pref_mod == Some(0xF2) { 0b11
        } else                           { 0
        };
        // map_sel is stored in the first byte of the opcode
        let (&map_sel, tail) = ops.split_first().expect("bad formatting data");
        ops = tail;
        compile_vex_xop(ctx.mode, buffer, data, &reg, &rm, map_sel, rex_w, &vvvv, vex_l, prefix);
    // otherwise, the size/mod prefixes have to be pushed and check if a rex prefix has to be generated.
    } else {
        if let Some(pref) = pref_mod {
            buffer.push(Stmt::u8(pref));
        }
        if pref_size {
            buffer.push(Stmt::u8(0x66));
        }
        if need_rex {
            // Certain SSE/AVX legacy encoded operations are not available in 32-bit mode
            // as they require a REX.W prefix to be encoded, which is impossible. We catch those cases here
            if ctx.mode == X86Mode::Protected {
                return Err(Some(format!("'{}': Does not support 64 bit operand size in 32-bit mode", op.to_string())))
            }
            compile_rex(buffer, rex_w, &reg, &rm);
        }
    }

    // if rm is embedded in the last opcode byte, push it here
    if data.flags.contains(Flags::SHORT_ARG) {
        let (last, head) = ops.split_last().expect("bad formatting data");
        ops = head;
        buffer.push(Stmt::Extend(Vec::from(ops)));

        let rm_k = if let Some(SizedArg::Direct {reg, ..}) = rm.take() {
            reg.kind
        } else {
            panic!("bad formatting data")
        };

        if let RegKind::Dynamic(_, expr) = rm_k {
            let last: TokenTree = proc_macro2::Literal::u8_suffixed(*last).into();
            buffer.push(Stmt::ExprUnsigned(serialize::expr_mask_shift_or(&last, &delimited(expr), 7, 0), Size::BYTE));
        } else {
            buffer.push(Stmt::u8(last + (rm_k.encode() & 7)));
        }
    // just push the opcode
    } else {
        buffer.push(Stmt::Extend(Vec::from(ops)));
    }

    // Direct ModRM addressing
    if let Some(SizedArg::Direct {reg: rm, ..}) = rm {
        let reg_k = if let Some(SizedArg::Direct {reg, ..}) = reg {
            reg.kind
        } else {
            RegKind::from_number(data.reg)
        };

        compile_modrm_sib(buffer, MOD_DIRECT, reg_k, rm.kind);
    // Indirect ModRM (+SIB) addressing
    } else if let Some(SizedArg::Indirect {disp_size, base, index, disp, ..}) = rm {
        let reg_k = if let Some(SizedArg::Direct {reg, ..}) = reg {
            reg.kind
        } else {
            RegKind::from_number(data.reg)
        };

        // check addressing mode special cases
        let mode_vsib = index.as_ref().map_or(false, |&(ref i, _, _)| i.kind.family() == RegFamily::XMM);
        let mode_16bit = addr_size == Size::WORD;
        let mode_rip_relative = base.as_ref().map_or(false, |b| b.kind.family() == RegFamily::RIP);
        let mode_rbp_base = base.as_ref().map_or(false, |b| b == &RegId::RBP || b == &RegId::R13 || b.kind.is_dynamic());

        if mode_vsib {
            let (index, scale, scale_expr) = index.unwrap();
            let index = index.kind;

            // VSIB addressing has simplified rules.
            let (base, mode) = if let Some(base) = base {
                (base.kind, match (&disp, disp_size) {
                    (&Some(_), Some(Size::BYTE)) => MOD_DISP8,
                    (&Some(_), _) => MOD_DISP32,
                    (&None, _) => MOD_DISP8
                })
            } else {
                (RegKind::Static(RegId::RBP), MOD_NOBASE)
            };

            // always need a SIB byte for VSIB addressing
            compile_modrm_sib(buffer, mode, reg_k, RegKind::Static(RegId::RSP));

            if let Some(expr) = scale_expr {
                compile_sib_dynscale(buffer, scale as u8, expr, index, base);
            } else {
                compile_modrm_sib(buffer, encode_scale(scale).unwrap(), index, base);
            }

            if let Some(disp) = disp {
                buffer.push(Stmt::ExprSigned(delimited(disp), if mode == MOD_DISP8 {Size::BYTE} else {Size::DWORD}));
            } else if mode == MOD_DISP8 {
                // no displacement was asked for, but we have to encode one as there's a base
                buffer.push(Stmt::u8(0));
            } else {
                // MODE_NOBASE requires a dword displacement, and if we got here no displacement was asked for.
                buffer.push(Stmt::u32(0));
            }

        } else if mode_16bit {
            // 16-bit mode: the index/base combination has been encoded in the base register.
            // this register is guaranteed to be present.
            let base_k = base.unwrap().kind;
            let mode = match (&disp, disp_size) {
                (&Some(_), Some(Size::BYTE)) => MOD_DISP8,
                (&Some(_), _) => MOD_DISP32, // well, technically 16-bit.
                (&None, _) => if mode_rbp_base {MOD_DISP8} else {MOD_NODISP}
            };

            // only need a mod.r/m byte for 16-bit addressing
            compile_modrm_sib(buffer, mode, reg_k, base_k);

            if let Some(disp) = disp {
                buffer.push(Stmt::ExprSigned(delimited(disp), if mode == MOD_DISP8 {Size::BYTE} else {Size::WORD}));
            } else if mode == MOD_DISP8 {
                buffer.push(Stmt::u8(0));
            }

        } else if mode_rip_relative {
            // encode the RIP + disp32 or disp32 form
            compile_modrm_sib(buffer, MOD_NODISP, reg_k, RegKind::Static(RegId::RBP));

            match ctx.mode {
                X86Mode::Long => if let Some(disp) = disp {
                    buffer.push(Stmt::ExprSigned(delimited(disp), Size::DWORD));
                } else {
                    buffer.push(Stmt::u32(0))
                },
                X86Mode::Protected => {
                    // x86 doesn't actually allow RIP-relative addressing
                    // but we can work around it with relocations
                    buffer.push(Stmt::u32(0));
                    let disp = disp.unwrap_or_else(|| serialize::reparse(&serialize::expr_zero()).expect("Invalid expression generated"));
                    relocations.push((Jump::new(JumpKind::Bare(disp), None), 0, Size::DWORD, RelocationKind::Absolute));
                },
            }

        } else {
            // normal addressing
            let no_base = base.is_none();

            // RBP can only be encoded as base if a displacement is present.
            let mode = if mode_rbp_base && disp.is_none() {
                MOD_DISP8
            // mode_nodisp if no base is to be encoded. note that in these scenarions a 32-bit disp has to be emitted
            } else if disp.is_none() || no_base {
                MOD_NODISP
            } else if let Some(Size::BYTE) = disp_size {
                MOD_DISP8
            } else {
                MOD_DISP32
            };

            // if there's an index we need to escape into the SIB byte
            if let Some((index, scale, scale_expr)) = index {
                // to encode the lack of a base we encode RBP
                let base = if let Some(base) = base {
                    base.kind
                } else {
                    RegKind::Static(RegId::RBP)
                };

                // escape into the SIB byte
                compile_modrm_sib(buffer, mode, reg_k, RegKind::Static(RegId::RSP));

                if let Some(expr) = scale_expr {
                    compile_sib_dynscale(buffer, scale as u8, expr, index.kind, base);
                } else {
                    compile_modrm_sib(buffer, encode_scale(scale).unwrap(), index.kind, base);
                }

            // no index, only a base. RBP at MOD_NODISP is used to encode RIP, but this is already handled
            } else if let Some(base) = base {
                compile_modrm_sib(buffer, mode, reg_k, base.kind);

            // no base, no index. only disp. Easy in x86, but in x64 escape, use RBP as base and RSP as index
            } else {
                match ctx.mode {
                    X86Mode::Protected => {
                        compile_modrm_sib(buffer, mode, reg_k, RegKind::Static(RegId::RBP));
                    },
                    X86Mode::Long => {
                        compile_modrm_sib(buffer, mode, reg_k, RegKind::Static(RegId::RSP));
                        compile_modrm_sib(buffer, 0, RegKind::Static(RegId::RSP), RegKind::Static(RegId::RBP));
                    }
                }
            }

            // Disp
            if let Some(disp) = disp {
                buffer.push(Stmt::ExprSigned(delimited(disp), if mode == MOD_DISP8 {Size::BYTE} else {Size::DWORD}));
            } else if no_base {
                buffer.push(Stmt::u32(0));
            } else if mode == MOD_DISP8 {
                buffer.push(Stmt::u8(0));
            }
        }

    // jump-target relative addressing
    } else if let Some(SizedArg::IndirectJumpTarget {jump, ..}) = rm {
        let reg_k = if let Some(SizedArg::Direct {reg, ..}) = reg {
            reg.kind
        } else {
            RegKind::from_number(data.reg)
        };
        compile_modrm_sib(buffer, MOD_NODISP, reg_k, RegKind::Static(RegId::RBP));

        buffer.push(Stmt::u32(0));
        match ctx.mode {
            X86Mode::Long      => relocations.push((jump, 0, Size::DWORD, RelocationKind::Relative)),
            X86Mode::Protected => relocations.push((jump, 0, Size::DWORD, RelocationKind::Absolute))
        }
    }

    // opcode encoded after the displacement
    if let Some(code) = immediate_opcode {
        buffer.push(Stmt::u8(code));

        // bump relocations
        relocations.iter_mut().for_each(|r| r.1 += 1);
    }

    // register in immediate argument
    if let Some(SizedArg::Direct {reg: ireg, ..}) = ireg {
        let ireg = ireg.kind;
        let byte = ireg.encode() << 4;

        let mut byte: TokenTree = proc_macro2::Literal::u8_suffixed(byte).into();
        if let RegKind::Dynamic(_, expr) = ireg {
            byte = serialize::expr_mask_shift_or(&byte, &delimited(expr), 0xF, 4);
        }
        // if immediates are present, the register argument will be merged into the
        // first immediate byte.
        if !args.is_empty() {
            if let SizedArg::Immediate {value, size: Size::BYTE} = args.remove(0) {
                byte = serialize::expr_mask_shift_or(&byte, &delimited(value), 0xF, 0);
            } else {
                panic!("bad formatting data")
            }
        }
        buffer.push(Stmt::ExprUnsigned(byte, Size::BYTE));

        // bump relocations
        relocations.iter_mut().for_each(|r| r.1 += 1);
    }

    // immediates
    for arg in args {
        match arg {
            SizedArg::Immediate {value, size} => {
                buffer.push(Stmt::ExprSigned(delimited(value), size));

                // bump relocations
                relocations.iter_mut().for_each(|r| r.1 += size.in_bytes());
            },
            SizedArg::JumpTarget {jump, size} => {
                // placeholder
                buffer.push(Stmt::Const(0, size));

                // bump relocations
                relocations.iter_mut().for_each(|r| r.1 += size.in_bytes());

                // add the new relocation
                if let JumpKind::Bare(_) = &jump.kind {
                    match ctx.mode {
                        X86Mode::Protected => relocations.push((jump, 0, size, RelocationKind::Extern)),
                        X86Mode::Long => return Err(Some("Extern relocations are not supported in x64 mode".to_string()))
                    }
                } else {
                    relocations.push((jump, 0, size, RelocationKind::Relative));
                }
            },
            _ => panic!("bad immediate data")
        };
    }

    // push relocations
    for (target, offset, size, kind) in relocations {
        let data = [size.in_bytes(), kind.to_id()];
        let data = match ctx.mode {
            X86Mode::Protected => &data,
            X86Mode::Long      => &data[..1],
        };

        // field offset has been tracked, and ref_offset is 0 as x86 offsets are relative to the end of the instruction
        buffer.push(target.encode(offset + size.in_bytes(), 0, data));
    }

    Ok(())
}

// Folds RawArgs into CleanArgs by analyzing the different raw memoryref variants
fn clean_memoryref(arg: RawArg) -> Result<CleanArg, Option<String>> {
    Ok(match arg {
        RawArg::Direct {span, reg} => CleanArg::Direct {span, reg},
        RawArg::JumpTarget {jump, size} => CleanArg::JumpTarget {jump, size},
        RawArg::IndirectJumpTarget {jump, size} => {
            if let JumpKind::Bare(_) = jump.kind {
                return Err(Some("Extern indirect jumps are not supported. Use a displacement".to_string()))
            }
            CleanArg::IndirectJumpTarget {jump, size}
        },
        RawArg::Immediate {value, size} => CleanArg::Immediate {value, size},
        RawArg::Invalid => return Err(None),
        RawArg::IndirectRaw {span, value_size, nosplit, disp_size, items} => {
            // split the ast on the memoryrefitem types
            let mut scaled = Vec::new();
            let mut regs = Vec::new();
            let mut disps = Vec::new();
            for item in items {
                match item {
                    MemoryRefItem::Register(reg) => regs.push(reg),
                    MemoryRefItem::ScaledRegister(reg, value) => scaled.push((reg, value)),
                    MemoryRefItem::Displacement(expr) => disps.push(expr)
                }
            }

            // figure out the base register if possible
            let mut base_reg_index = None;
            for (i, reg) in regs.iter().enumerate() {
                if !(regs.iter().enumerate().any(|(j, other)| i != j && reg == other) ||
                     scaled.iter().any(|&(ref other, _)| reg == other)) {
                    base_reg_index = Some(i);
                    break;
                }
            }
            let mut base = base_reg_index.map(|i| regs.remove(i));

            // join all registers
            scaled.extend(regs.into_iter().map(|r| (r, 1)));
            let mut joined_regs = Vec::new();
            for (reg, s) in scaled {
                // does this register already have a spot?
                if let Some(i) = joined_regs.iter().position(|&(ref other, _)| &reg == other) {
                    joined_regs[i].1 += s;
                } else {
                    joined_regs.push((reg, s));
                }
            }

            // identify an index candidate (and a base if one wasn't found yet)
            let index = if base.is_none() {
                // we have to look for a base candidate first as not all index candidates
                // are viable base candidates
                base_reg_index = joined_regs.iter().position(|&(_, s)| s == 1);
                base = base_reg_index.map(|i| joined_regs.remove(i).0);
                // get the index
                let index = joined_regs.pop();
                // if nosplit was used, a scaled base was found but not an index, swap them
                // as there was only one register and this register was scaled.
                if nosplit && index.is_none() && base.is_some() {
                    base.take().map(|reg| (reg, 1))
                } else {
                    index
                }
            } else {
                joined_regs.pop()
            };


            if !joined_regs.is_empty() {
                emit_error!(span, "Impossible memory argument");
                return Err(None);
            }

            // merge disps
            let disp = serialize::expr_add_many(span, disps.into_iter().map(delimited));
            let disp = disp.map(|d| serialize::reparse(&d).expect("Invalid expression generated internally"));

            // finalize the memoryref
            CleanArg::Indirect {
                span,
                nosplit,
                size: value_size,
                disp_size,
                base,
                index: index.map(|(r, s)| (r, s, None)),
                disp,
            }
        },
        RawArg::TypeMappedRaw {span, base_reg, scale, value_size, nosplit, disp_size, scaled_items, attribute} => {
            let base = base_reg;

            // collect registers / displacements
            let mut scaled = Vec::new();
            let mut disps = Vec::new();
            for item in scaled_items {
                match item {
                    MemoryRefItem::Register(reg) => scaled.push((reg, 1)),
                    MemoryRefItem::ScaledRegister(reg, scale) => scaled.push((reg, scale)),
                    MemoryRefItem::Displacement(expr) => disps.push(expr)
                }
            }

            // join all registers
            let mut joined_regs = Vec::new();
            for (reg, s) in scaled {
                // does this register already have a spot?
                if let Some(i) = joined_regs.iter().position(|&(ref other, _)| &reg == other) {
                    joined_regs[i].1 += s;
                } else {
                    joined_regs.push((reg, s));
                }
            }

            // identify an index register
            let index = joined_regs.pop();

            if !joined_regs.is_empty() {
                emit_error!(span, "Impossible memory argument");
                return Err(None);
            }

            // index = scale * index_scale * index
            // disp = scale * disps + attribute

            // Displacement size calculation intermezzo:
            // in typemaps, the following equations are the relations for the scale and displacement.
            // Now as we can't figure these out at compile time (this'd be nice), by default we'll
            // always generate a 32-bit displacement if disp_size isn't set. This means we know
            // the size of disp at this point already.
            // as for the index calculation: that doesn't change size.
            let true_disp_size = disp_size.unwrap_or(Size::DWORD);

            // merge disps [a, b, c] into (a + b + c)
            let scaled_disp = serialize::expr_add_many(span, disps.into_iter().map(delimited));

            // scale disps (a + b + c) * size_of<scale> as disp_size
            let scaled_disp = scaled_disp.map(|disp| serialize::expr_size_of_scale(&scale, &disp, true_disp_size));

            // attribute displacement offset_of(scale, attr) as disp_size
            let attr_disp = attribute.map(|attr| serialize::expr_offset_of(&scale, &attr, true_disp_size));

            // add displacement sources together
            let disp = if let Some(scaled_disp) = scaled_disp {
                if let Some(attr_disp) = attr_disp {
                    serialize::expr_add_many(span, vec![attr_disp, scaled_disp].into_iter())
                } else {
                    Some(scaled_disp)
                }
            } else {
                attr_disp
            };

            let disp = disp.map(|d| serialize::reparse(&d).expect("Invalid expression generated internally"));

            let index = index.map(|(r, s)| {
                let scale_expr = serialize::reparse(&serialize::expr_size_of(&scale)).expect("Invalid expression generated internally");
                (r, s, Some(scale_expr))
            });

            // finalize the memoryref
            CleanArg::Indirect {
                span,
                nosplit,
                size: value_size,
                disp_size,
                base: Some(base),
                index,
                disp,
            }
        },
    })
}

// Go through the CleanArgs, check for impossible to encode indirect arguments, fill in immediate/displacement size information
// and return the effective address size
fn sanitize_indirects_and_sizes(ctx: &Context, args: &mut [CleanArg]) -> Result<Option<Size>, Option<String>> {
    // determine if an address size prefix is necessary, and sanitize the register choice for memoryrefs
    let mut addr_size = None;
    let mut encountered_indirect = false;

    for arg in args.iter_mut() {
        match *arg {
            CleanArg::Indirect {span, nosplit, ref mut disp_size, ref mut base, ref mut index, ref disp, ..} => {

                if encountered_indirect {
                    emit_error!(span, "Multiple memory references in a single instruction")
                }
                encountered_indirect = true;

                // figure out the effective address size and error on impossible combinations
                addr_size = sanitize_indirect(ctx, span, nosplit, base, index)?;

                if let Some((_, scale, _)) = *index {
                    if encode_scale(scale).is_none() {
                        emit_error!(span, "Impossible scale");
                    }
                }

                // if specified, sanitize the displacement size. Else, derive one
                if let Some(size) = *disp_size {
                    if disp.is_none() {
                        emit_error!(span, "Displacement size without displacement");
                    }

                    // 16-bit addressing has smaller displacements
                    if addr_size == Some(Size::WORD) {
                        if size != Size::BYTE && size != Size::WORD {
                            emit_error!(span, "Invalid displacement size, only BYTE or WORD are possible");
                        }
                    } else if size != Size::BYTE && size != Size::DWORD {
                        emit_error!(span, "Invalid displacement size, only BYTE or DWORD are possible");
                    }
                } else if let Some(ref disp) = *disp {
                    match derive_size(disp) {
                        Some(Size::BYTE)                         => *disp_size = Some(Size::BYTE),
                        Some(_) if addr_size == Some(Size::WORD) => *disp_size = Some(Size::WORD),
                        Some(_)                                  => *disp_size = Some(Size::DWORD),
                        None => ()
                    }
                }
            },
            CleanArg::Immediate {ref value, size: ref mut size @ None} => {
                *size = derive_size(value);
            },
            _ => ()
        }
    }

    Ok(addr_size)
}

// Tries to find the maximum size necessary to hold the result of an expression.
fn derive_size(expr: &syn::Expr) -> Option<Size> {
    // strip any wrapping Group nodes due to delimiting
    let mut inner = expr;
    while let syn::Expr::Group(syn::ExprGroup { expr, .. }) = inner {
        inner = expr;
    }

    match inner {
        syn::Expr::Lit(syn::ExprLit { ref lit, .. } ) => match lit {
            syn::Lit::Byte(_) => Some(Size::BYTE),
            syn::Lit::Int(i) => match i.base10_parse::<u32>() {
                Ok(x) if x < 0x80 => Some(Size::BYTE),
                Ok(x) if x < 0x8000 => Some(Size::WORD),
                Ok(x) if x < 0x8000_0000 => Some(Size::DWORD),
                _ => Some(Size::QWORD),
            },
            _ => None
        },
        syn::Expr::Unary(syn::ExprUnary { op: syn::UnOp::Neg(_), ref expr, .. } ) => match &**expr {
            syn::Expr::Lit(syn::ExprLit { ref lit, .. } ) => match lit {
                syn::Lit::Byte(_) => Some(Size::BYTE),
                syn::Lit::Int(i) => match i.base10_parse::<u32>() {
                    Ok(x) if x <= 0x80 => Some(Size::BYTE),
                    Ok(x) if x <= 0x8000 => Some(Size::WORD),
                    Ok(x) if x <= 0x8000_0000 => Some(Size::DWORD),
                    _ => Some(Size::QWORD),
                },
                _ => None
            },
            _ => None
        },
        _ => None
    }
}

/// Validates that the base/index combination can actually be encoded and returns the effective address size.
/// If the address size can't be determined (purely displacement, or VSIB without base), the result is None.
fn sanitize_indirect(ctx: &Context, span: Span, nosplit: bool, base: &mut Option<Register>,
                     index: &mut Option<(Register, isize, Option<syn::Expr>)>) -> Result<Option<Size>, Option<String>>
{

    // figure out the addressing size/mode used.
    // size can be 16, 32, or 64-bit.
    // mode can be legacy, rip-relative, or vsib
    // note that rip-relative and vsib only support 32 and 64-bit
    let b = base.as_ref().map(|b| (b.kind.family(), b.size()));
    let i = index.as_ref().map(|i| (i.0.kind.family(), i.0.size()));

    let size;
    let family;
    let mut vsib_mode = false;

    // figure out the addressing mode and size
    match (&b, &i) {
        (&None, &None) => return Ok(None),
        (&Some((f, s)), &None) |
        (&None, &Some((f, s))) => {
            size = s;
            family = f;
        },
        (&Some((f1, s1)), &Some((f2, s2))) => if f1 == f2 {
            if s1 != s2 {
                emit_error!(span, "Registers of differing sizes");
                return Err(None);
            }
            size = s1;
            family = f1;

        // allow only vsib addressing
        } else if f1 == RegFamily::XMM {
            vsib_mode = true;
            size = s2;
            family = f2;
        } else if f2 == RegFamily::XMM {
            vsib_mode = true;
            size = s1;
            family = f1;
        } else {
            emit_error!(span, "Register type combination not supported");
            return Err(None);
        }
    }

    // filter out combinations that are impossible to encode
    match family {
        RegFamily::RIP => if b.is_some() && i.is_some() {
            emit_error!(span, "Register type combination not supported");
            return Err(None);
        },
        RegFamily::LEGACY => match size {
            Size::DWORD => (),
            Size::QWORD => (), // only valid in long mode, but should only be possible in long mode
            Size::WORD  => if ctx.mode == X86Mode::Protected || vsib_mode {
                emit_error!(span, "16-bit addressing is not supported in this mode");
                return Err(None);
            },
            _ => {
                emit_error!(span, "Register type not supported");
                return Err(None);
            }
        },
        RegFamily::XMM => if b.is_some() && i.is_some() {
            emit_error!(span, "Register type combination not supported");
        },
        _ => {
            emit_error!(span, "Register type not supported");
            return Err(None);
        }
    }

    // RIP-relative encoding
    if family == RegFamily::RIP {
        // we're guaranteed that RIP is only present as one register.
        match index.take() {
            Some((index, 1, None)) => *base = Some(index),
            Some(_) => {
                emit_error!(span, "RIP cannot be scaled");
                return Err(None);
            },
            None => ()
        }
        return Ok(Some(size));
    }

    // VSIB without base
    // TODO validate index sizes?
    if family == RegFamily::XMM {
        if let Some(reg) = base.take() {
            *index = Some((reg, 1, None));
        }
        return Ok(None);
    }

    // VSIB with base
    if vsib_mode {
        // we're guaranteed that the other register is a legacy register, either DWORD or QWORD size
        // so we just have to check if an index/base swap is necessary
        if base.as_ref().unwrap().kind.family() == RegFamily::XMM {
            // try to swap if possible
            // TODO: honour nosplit?
            if let (ref mut i, 1, None) = index.as_mut().unwrap() {
                swap(i, base.as_mut().unwrap())
            } else {
                emit_error!(span, "vsib addressing requires a general purpose register as base");
                return Err(None);
            }
        }

        return Ok(Some(size));
    }

    // 16-bit legacy addressing
    if size == Size::WORD {
        // 16-bit addressing has no concept of index.
        let mut first_reg = base.take();
        let mut second_reg = match index.take() {
            Some((i, 1, None)) => Some(i),
            None => None,
            Some(_) => {
                emit_error!(span, "16-bit addressing with scaled index");
                return Err(None);
            },
        };

        if first_reg.is_none() {
            first_reg = second_reg.take();
        }

        let encoded_base = match (&first_reg, &second_reg) {
            (r1, r2) if (r1 == &RegId::RBX && r2 == &RegId::RSI) ||
                        (r1 == &RegId::RSI && r2 == &RegId::RBX) => RegId::from_number(0),
            (r1, r2) if (r1 == &RegId::RBX && r2 == &RegId::RDI) ||
                        (r1 == &RegId::RDI && r2 == &RegId::RBX) => RegId::from_number(1),
            (r1, r2) if (r1 == &RegId::RBP && r2 == &RegId::RSI) ||
                        (r1 == &RegId::RSI && r2 == &RegId::RBP) => RegId::from_number(2),
            (r1, r2) if (r1 == &RegId::RBP && r2 == &RegId::RDI) ||
                        (r1 == &RegId::RDI && r2 == &RegId::RBP) => RegId::from_number(3),
            (r, None) if r == &RegId::RSI => RegId::from_number(4),
            (r, None) if r == &RegId::RDI => RegId::from_number(5),
            (r, None) if r == &RegId::RBP => RegId::from_number(6),
            (r, None) if r == &RegId::RBX => RegId::from_number(7),
            _ => {
                emit_error!(span, "Impossible register combination");
                return Err(None);
            }
        };

        *base = Some(Register::new_static(Size::WORD, encoded_base));
        return Ok(Some(size));
    }

    // normal addressing

    // optimize indexes if a base is not present
    if !nosplit && base.is_none() {
        if let Some((ref reg, ref mut scale, None)) = *index {
            match *scale {
                2 | 3 | 5 | 9 => {
                    *base = Some(reg.clone());
                    *scale -= 1
                },
                _ => ()
            }
        }
    }

    // RSP as index field can not be represented. Check if we can swap it with base
    if let Some((i, scale, scale_expr)) = index.take() {
        if i == RegId::RSP {
            if *base != RegId::RSP && scale == 1 && scale_expr.is_none() {
                *index = base.take().map(|reg| (reg, 1, None));
                *base = Some(i);
            } else {
                emit_error!(span, "'rsp' cannot be used as index field");
                return Err(None);
            }
        } else {
            *index = Some((i, scale, scale_expr))
        }
    }

    // RSP, R12 or a dynamic register as base without index (add an index so we escape into SIB)
    if index.is_none() && (*base == RegId::RSP || *base == RegId::R12 || base.as_ref().map_or(false, |r| r.kind.is_dynamic())) {
        *index = Some((Register::new_static(size, RegId::RSP), 1, None));
    }

    // RBP as base field just requires a mandatory MOD_DISP8, so we only process that at encoding time
    Ok(Some(size))
}

fn match_op_format(ctx: &Context, ident: &syn::Ident, args: &[CleanArg]) -> Result<&'static Opdata, Option<String>> {
    let name = ident.to_string();
    let name = name.as_str();

    let data = if let Some(data) = get_mnemnonic_data(name) {
        data
    } else {
        emit_error!(ident, "'{}' is not a valid instruction", name);
        return Err(None);
    };

    for format in data {
        if let Ok(()) = match_format_string(ctx, format, args) {
            return Ok(format);
        }
    }

    Err(Some(
        format!("'{}': argument type/size mismatch, expected one of the following forms:\n{}", name, format_opdata_list(name, data))
    ))
}

fn match_format_string(ctx: &Context, fmt: &Opdata, args: &[CleanArg]) -> Result<(), &'static str> {
    let fmtstr = &fmt.args;

    if ctx.mode != X86Mode::Protected && fmt.flags.intersects(Flags::X86_ONLY) {
        return Err("Not available in 32-bit mode");
    }

    if fmtstr.len() != args.len() * 2 {
        return Err("argument length mismatch");
    }
    // i : immediate
    // o : instruction offset

    // m : memory
    // k : vsib addressing, 32 bit result, size determines xmm or ymm
    // l : vsib addressing, 64 bit result, size determines xmm or ymm

    // r : legacy reg
    // f : fp reg
    // x : mmx reg
    // y : xmm/ymm reg
    // s : segment reg
    // c : control reg
    // d : debug reg
    // b : bound reg

    // v : r and m
    // u : x and m
    // w : y and m

    // A ..= P: match rax - r15
    // Q ..= V: match es, cs, ss, ds, fs, gs
    // W: matches CR8
    // X: matches st0

    // b, w, d, q, o, h match a byte, word, doubleword, quadword, octword and hexadecword
    // p matches a PWORD (10 bytes)
    // f matches an FWORD (6 bytes)
    // * matches all possible sizes for this operand (w/d for i, w/d/q for r/v, o/h for y/w and everything for m)
    // ! matches a lack of size, only useful in combination with m
    // ? matches any size and doesn't participate in the operand size calculation
    let mut args = args.iter();
    for (code, fsize) in FormatStringIterator::new(fmtstr) {
        let arg = args.next().unwrap();

        let size = match (code, arg) {
            // immediates
            (b'i', &CleanArg::Immediate{size, ..})  |
            (b'o', &CleanArg::Immediate{size, ..})  |
            (b'o', &CleanArg::JumpTarget{size, ..}) => size,

            // specific legacy regs
            (x @ b'A' ..= b'P', &CleanArg::Direct{ref reg, ..}) if
                reg.kind.family() == RegFamily::LEGACY &&
                reg.kind.code() == Some(x - b'A') => Some(reg.size()),

            // specific segment regs
            (x @ b'Q' ..= b'V', &CleanArg::Direct{ref reg, ..}) if
                reg.kind.family() == RegFamily::SEGMENT &&
                reg.kind.code() == Some(x - b'Q') => Some(reg.size()),

            // CR8 can be specially referenced
            (b'W', &CleanArg::Direct{ref reg, ..}) if
                reg.kind == RegId::CR8 => Some(reg.size()),

            // top of the fp stack is also often used
            (b'X', &CleanArg::Direct{ref reg, ..}) if
                reg.kind == RegId::ST0 => Some(reg.size()),

            // generic legacy regs
            (b'r', &CleanArg::Direct{ref reg, ..}) |
            (b'v', &CleanArg::Direct{ref reg, ..}) if
                reg.kind.family() == RegFamily::LEGACY ||
                reg.kind.family() == RegFamily::HIGHBYTE => Some(reg.size()),

            // other reg types often mixed with memory refs
            (b'x', &CleanArg::Direct{ref reg, ..}) |
            (b'u', &CleanArg::Direct{ref reg, ..}) if
                reg.kind.family() == RegFamily::MMX => Some(reg.size()),
            (b'y', &CleanArg::Direct{ref reg, ..}) |
            (b'w', &CleanArg::Direct{ref reg, ..}) if
                reg.kind.family() == RegFamily::XMM => Some(reg.size()),

            // other reg types
            (b'f', &CleanArg::Direct{ref reg, ..}) if
                reg.kind.family() == RegFamily::FP => Some(reg.size()),
            (b's', &CleanArg::Direct{ref reg, ..}) if
                reg.kind.family() == RegFamily::SEGMENT => Some(reg.size()),
            (b'c', &CleanArg::Direct{ref reg, ..}) if
                reg.kind.family() == RegFamily::CONTROL => Some(reg.size()),
            (b'd', &CleanArg::Direct{ref reg, ..}) if
                reg.kind.family() == RegFamily::DEBUG => Some(reg.size()),
            (b'b', &CleanArg::Direct{ref reg, ..}) if
                reg.kind.family() == RegFamily::BOUND => Some(reg.size()),

            // memory offsets
            (b'm',          &CleanArg::Indirect {size, ref index, ..}) |
            (b'u' ..= b'w', &CleanArg::Indirect {size, ref index, ..}) if
                index.is_none() || index.as_ref().unwrap().0.kind.family() != RegFamily::XMM => size,

            (b'm',          &CleanArg::IndirectJumpTarget {size, ..}) |
            (b'u' ..= b'w', &CleanArg::IndirectJumpTarget {size, ..}) => size,

            // vsib addressing. as they have two sizes that must be checked they check one of the sizes here
            (b'k', &CleanArg::Indirect {size, index: Some((ref index, _, _)), ..}) if
                (size.is_none() || size == Some(Size::DWORD)) &&
                index.kind.family() == RegFamily::XMM => Some(index.size()),
            (b'l', &CleanArg::Indirect {size, index: Some((ref index, _, _)), ..}) if
                (size.is_none() ||  size == Some(Size::QWORD)) &&
                index.kind.family() == RegFamily::XMM => Some(index.size()),
            _ => return Err("argument type mismatch")
        };

        // if size is none it always matches (and will later be coerced to a more specific type if the match is successful)
        if let Some(size) = size {
            if !match (fsize, code) {
                // immediates can always fit in larger slots
                (b'w', b'i') => size <= Size::WORD,
                (b'd', b'i') => size <= Size::DWORD,
                (b'q', b'i') => size <= Size::QWORD,
                (b'*', b'i') => size <= Size::DWORD,
                // normal size matches
                (b'b', _)    => size == Size::BYTE,
                (b'w', _)    => size == Size::WORD,
                (b'd', _)    => size == Size::DWORD,
                (b'q', _)    => size == Size::QWORD,
                (b'f', _)    => size == Size::FWORD,
                (b'p', _)    => size == Size::PWORD,
                (b'o', _)    => size == Size::OWORD,
                (b'h', _)    => size == Size::HWORD,
                // what is allowed for wildcards
                (b'*', b'k') |
                (b'*', b'l') |
                (b'*', b'y') |
                (b'*', b'w') => size == Size::OWORD || size == Size::HWORD,
                (b'*', b'r') |
                (b'*', b'A' ..= b'P') |
                (b'*', b'v') => size == Size::WORD || size == Size::DWORD || size == Size::QWORD,
                (b'*', b'm') => true,
                (b'*', _)    => panic!("Invalid size wildcard"),
                (b'?', _)    => true,
                (b'!', _)    => false,
                _ => panic!("invalid format string")
            } {
                return Err("argument size mismatch");
            }
        } else if fsize != b'*' && fmt.flags.contains(Flags::EXACT_SIZE) {
            // Basically, this format is a more specific version of an instruction
            // that also has more general versions. This should only be picked
            // if the size constraints are met, not if the size is unspecified
            return Err("alternate variant exists");
        }
    }

    // match found
    Ok(())
}

fn size_operands(fmt: &Opdata, args: Vec<CleanArg>) -> Result<(Option<Size>, Vec<SizedArg>), Option<String>> {
    // sizing operands requires two passes.
    // In the first one, we determine the effective operand size if necessary (if *'s are present)
    // In the second one, we create the final sized AST

    let mut has_arg = false;
    let mut op_size = None;
    let mut im_size = None;

    // operand size determination loop
    for (arg, (_, fsize)) in args.iter().zip(FormatStringIterator::new(&fmt.args)) {
        if fsize != b'*' {
            continue;
        }

        match *arg {
            CleanArg::Direct {ref reg, ..} => {
                has_arg = true;
                let size = reg.size();
                if op_size.map_or(false, |s| s != size,) {
                    return Err(Some("Conflicting operand sizes".to_string()));
                }
                op_size = Some(size);
            },
            CleanArg::IndirectJumpTarget {size, ..} => {
                has_arg = true;
                if let Some(size) = size {
                    if op_size.map_or(false, |s| s != size) {
                        return Err(Some("Conflicting operand sizes".to_string()));
                    }
                    op_size = Some(size);
                }
            }
            CleanArg::Indirect {mut size, ref index, ..} => {
                has_arg = true;
                // VSIB addressing
                if let Some((ref reg, _, _)) = *index {
                    if reg.kind.family() == RegFamily::XMM {
                        size = Some(reg.size());
                    }
                }

                if let Some(size) = size {
                    if op_size.map_or(false, |s| s != size) {
                        return Err(Some("Conflicting operand sizes".to_string()));
                    }
                    op_size = Some(size);
                }
            },
            CleanArg::Immediate {size, ..} |
            CleanArg::JumpTarget {size, ..} => {
                if im_size.is_some() {
                    panic!("Bad formatting data? multiple immediates with wildcard size");
                }
                im_size = size;
            }
        }
    }

    if let Some(o) = op_size {
        let ref_im_size = if o > Size::DWORD {Size::DWORD} else {o};
        if let Some(i) = im_size {
            if i > ref_im_size {
                return Err(Some("Immediate size mismatch".to_string()));
            }
        }
        im_size = Some(ref_im_size);
    } else if has_arg {
        return Err(Some("Unknown operand size".to_string()));
    }

    // fill-in loop. default should never be used.
    let mut new_args = Vec::new();
    for (arg, (code, fsize)) in args.into_iter().zip(FormatStringIterator::new(&fmt.args)) {

        //get the specified operand size from the format string
        let size = match (fsize, code) {
            (b'b', _) => Size::BYTE,
            (b'w', _) => Size::WORD,
            (_, b'k') |
            (b'd', _) => Size::DWORD,
            (_, b'l') |
            (b'q', _) => Size::QWORD,
            (b'f', _) => Size::FWORD,
            (b'p', _) => Size::PWORD,
            (b'o', _) => Size::OWORD,
            (b'h', _) => Size::HWORD,
            (b'*', b'i') => im_size.unwrap(),
            (b'*', _) => op_size.unwrap(),
            (b'!', _) => Size::BYTE, // will never be used, placeholder
            _ => unreachable!()
        };

        new_args.push(match arg {
            CleanArg::Direct {span, reg} =>
                SizedArg::Direct {span, reg},
            CleanArg::JumpTarget {jump, ..} =>
                SizedArg::JumpTarget {jump, size},
            CleanArg::IndirectJumpTarget {jump, ..} =>
                SizedArg::IndirectJumpTarget {jump},
            CleanArg::Immediate {value, ..} =>
                SizedArg::Immediate {value, size},
            CleanArg::Indirect {span, disp_size, base, index, disp, ..} =>
                SizedArg::Indirect {span, disp_size, base, index, disp},
        });
    }

    Ok((op_size, new_args))
}

fn get_legacy_prefixes(fmt: &'static Opdata, idents: Vec<syn::Ident>) -> Result<(Option<u8>, Option<u8>), Option<String>> {
    let mut group1 = None;
    let mut group2 = None;

    for prefix in idents {
        let name = prefix.to_string();
        let (group, value) = match name.as_str() {
            "rep"   => if fmt.flags.contains(Flags::REP) {
                (&mut group1, 0xF3)
            } else {
                emit_error!(prefix, "Cannot use prefix {} on this instruction", name);
                return Err(None);
            },
            "repe"  |
            "repz"  => if fmt.flags.contains(Flags::REPE) {
                (&mut group1, 0xF3)
            } else {
                emit_error!(prefix, "Cannot use prefix {} on this instruction", name);
                return Err(None);
            },
            "repnz" |
            "repne" => if fmt.flags.contains(Flags::REPE) {
                (&mut group1, 0xF2)
            } else {
                emit_error!(prefix, "Cannot use prefix {} on this instruction", name);
                return Err(None);
            },
            "lock"  => if fmt.flags.contains(Flags::LOCK) {
                (&mut group1, 0xF0)
            } else {
                emit_error!(prefix, "Cannot use prefix {} on this instruction", name);
                return Err(None);
            },
            "ss"    => (&mut group2, 0x36),
            "cs"    => (&mut group2, 0x2E),
            "ds"    => (&mut group2, 0x3E),
            "es"    => (&mut group2, 0x26),
            "fs"    => (&mut group2, 0x64),
            "gs"    => (&mut group2, 0x65),
            _       => panic!("unimplemented prefix")
        };
        if group.is_some() {
            emit_error!(prefix, "Duplicate prefix group");
            return Err(None);
        }
        *group = Some(value);
    }

    Ok((group1, group2))
}

fn check_rex(ctx: &Context, fmt: &'static Opdata, args: &[SizedArg], rex_w: bool) -> Result<bool, Option<String>> {
    // performs checks for not encodable arg combinations
    // output arg indicates if a rex prefix can be encoded
    if ctx.mode == X86Mode::Protected {
        if rex_w {
            return Err(Some("Does not support 64 bit operand size in 32-bit mode".to_string()));
        } else {
            return Ok(false);
        }
    }

    let mut requires_rex    = rex_w;
    let mut requires_no_rex = false;

    for (arg, (c, _)) in args.iter().zip(FormatStringIterator::new(fmt.args)) {
        // only scan args that are actually encoded
        if let b'a' ..= b'z' = c {
            match *arg {
                SizedArg::Direct {ref reg, ..} => {
                    if reg.kind.family() == RegFamily::HIGHBYTE {
                        requires_no_rex = true;

                    } else if reg.kind.is_extended() || (reg.size() == Size::BYTE &&
                        (reg.kind == RegId::RSP || reg.kind == RegId::RBP || reg.kind == RegId::RSI || reg.kind == RegId::RDI)) {
                        requires_rex = true;
                    }
                },
                SizedArg::Indirect {ref base, ref index, ..} => {
                    if let Some(ref reg) = *base {
                        requires_rex = requires_rex || reg.kind.is_extended();
                    }
                    if let Some((ref reg, _, _)) = *index {
                        requires_rex = requires_rex || reg.kind.is_extended();
                    }
                },
                _ => (),
            }
        }
    }

    if requires_rex && requires_no_rex {
        Err(Some("High byte register combined with extended registers or 64-bit operand size".to_string()))
    } else {
        Ok(requires_rex)
    }
}

fn extract_args(fmt: &'static Opdata, args: Vec<SizedArg>) -> (Option<SizedArg>, Option<SizedArg>, Option<SizedArg>, Option<SizedArg>, Vec<SizedArg>) {
    // way operand order works:

    // if there's a memory/reg operand, this operand goes into modrm.r/m
    // if there's a segment/control/debug register, it goes into reg.

    // default argument encoding order is as follows:
    // no encoding flag: m, rm, rvm, rvim
    // ENC_MR:              mr, rmv, rvmi
    // ENC_VM:              vm, mvr
    // these can also be chosen based on the location of a memory argument (except for vm)

    let mut memarg = None;
    let mut regarg = None;
    let mut regs = Vec::new();
    let mut immediates = Vec::new();

    for (arg, (c, _)) in args.into_iter().zip(FormatStringIterator::new(fmt.args)) {
        match c {
            b'm' | b'u' | b'v' | b'w' | b'k' | b'l'  => if memarg.is_some() {
                panic!("multiple memory arguments in format string");
            } else {
                memarg = Some(regs.len());
                regs.push(arg)
            },
            b'f' | b'x' | b'r' | b'y' | b'b' => regs.push(arg),
            b'c' | b'd' | b's'        => if regarg.is_some() {
                panic!("multiple segment, debug or control registers in format string");
            } else {
                regarg = Some(regs.len());
                regs.push(arg)
            },
            b'i' | b'o' => immediates.push(arg),
            _ => () // hardcoded regs don't have to be encoded
        }
    }

    let len = regs.len();
    if len > 4 {
        panic!("too many arguments");
    }
    let mut regs = regs.drain(..).fuse();

    let mut m = None;
    let mut r = None;
    let mut v = None;
    let mut i = None;

    if let Some(i) = regarg {
        if i == 0 {
            r = regs.next();
            m = regs.next();
        } else {
            m = regs.next();
            r = regs.next();
        }
    } else if len == 1 {
        m = regs.next();
    } else if len == 2 {
        if fmt.flags.contains(Flags::ENC_MR) || memarg == Some(0) {
            m = regs.next();
            r = regs.next();
        } else if fmt.flags.contains(Flags::ENC_VM) {
            v = regs.next();
            m = regs.next();
        } else {
            r = regs.next();
            m = regs.next();
        }
    } else if len == 3 {
        if fmt.flags.contains(Flags::ENC_MR) || memarg == Some(1) {
            r = regs.next();
            m = regs.next();
            v = regs.next();
        } else if fmt.flags.contains(Flags::ENC_VM) || memarg == Some(0) {
            m = regs.next();
            v = regs.next();
            r = regs.next();
        } else {
            r = regs.next();
            v = regs.next();
            m = regs.next();
        }
    } else if len == 4 {
        if fmt.flags.contains(Flags::ENC_MR) || memarg == Some(2) {
            r = regs.next();
            v = regs.next();
            m = regs.next();
            i = regs.next();
        } else {
            r = regs.next();
            v = regs.next();
            i = regs.next();
            m = regs.next();
        }
    }

    (m, r, v, i, immediates)
}

fn encode_scale(scale: isize) -> Option<u8> {
    match scale {
        1 => Some(0),
        2 => Some(1),
        4 => Some(2),
        8 => Some(3),
        _ => None
    }
}

fn compile_rex(buffer: &mut Vec<Stmt>, rex_w: bool, reg: &Option<SizedArg>, rm: &Option<SizedArg>) {
    let mut reg_k   = RegKind::from_number(0);
    let mut index_k = RegKind::from_number(0);
    let mut base_k  = RegKind::from_number(0);

    if let Some(SizedArg::Direct {ref reg, ..}) = *reg {
        reg_k = reg.kind.clone();
    }
    if let Some(SizedArg::Direct {ref reg, ..}) = *rm {
        base_k = reg.kind.clone();
    }
    if let Some(SizedArg::Indirect {ref base, ref index, ..} ) = *rm {
        if let Some(ref base) = *base {
            base_k = base.kind.clone();
        }
        if let Some((ref index, _, _)) = *index {
            index_k = index.kind.clone();
        }
    }

    let rex = 0x40 | (rex_w          as u8) << 3 |
                     (reg_k.encode()   & 8) >> 1 |
                     (index_k.encode() & 8) >> 2 |
                     (base_k.encode()  & 8) >> 3 ;
    if !reg_k.is_dynamic() && !index_k.is_dynamic() && !base_k.is_dynamic() {
        buffer.push(Stmt::u8(rex));
        return;
    }

    let mut rex: TokenTree = proc_macro2::Literal::u8_suffixed(rex).into();

    if let RegKind::Dynamic(_, expr) = reg_k {
        rex = serialize::expr_mask_shift_or(&rex, &delimited(expr), 8, -1);
    }
    if let RegKind::Dynamic(_, expr) = index_k {
        rex = serialize::expr_mask_shift_or(&rex, &delimited(expr), 8, -2);
    }
    if let RegKind::Dynamic(_, expr) = base_k {
        rex = serialize::expr_mask_shift_or(&rex, &delimited(expr), 8, -3);
    }
    buffer.push(Stmt::ExprUnsigned(rex, Size::BYTE));
}

fn compile_vex_xop(mode: X86Mode, buffer: &mut Vec<Stmt>, data: &'static Opdata, reg: &Option<SizedArg>,
rm: &Option<SizedArg>, map_sel: u8, rex_w: bool, vvvv: &Option<SizedArg>, vex_l: bool, prefix: u8) {
    let mut reg_k   = RegKind::from_number(0);
    let mut index_k = RegKind::from_number(0);
    let mut base_k  = RegKind::from_number(0);
    let mut vvvv_k  = RegKind::from_number(0);

    let byte1 = match mode {
        X86Mode::Long => {
            if let Some(SizedArg::Direct {ref reg, ..}) = *reg {
                reg_k = reg.kind.clone();
            }
            if let Some(SizedArg::Direct {ref reg, ..}) = *rm {
                base_k = reg.kind.clone();
            }
            if let Some(SizedArg::Indirect {ref base, ref index, ..}) = *rm {
                if let Some(ref base) = *base {
                    base_k = base.kind.clone();
                }
                if let Some((ref index, _, _)) = *index {
                    index_k = index.kind.clone();
                }
            }

            (map_sel        & 0x1F)      |
            (!reg_k.encode()   & 8) << 4 |
            (!index_k.encode() & 8) << 3 |
            (!base_k.encode()  & 8) << 2
        },
        X86Mode::Protected => {
            (map_sel & 0x1f) | 0xE0
        }
    };

    if let Some(SizedArg::Direct {ref reg, ..}) = *vvvv {
        vvvv_k = reg.kind.clone();
    }

    let byte2 = (prefix           & 0x3)      |
                (rex_w            as u8) << 7 |
                (!vvvv_k.encode() & 0xF) << 3 |
                (vex_l            as u8) << 2 ;

    if data.flags.contains(Flags::VEX_OP) && (byte1 & 0x7F) == 0x61 && (byte2 & 0x80) == 0 &&
    ((!index_k.is_dynamic() && !base_k.is_dynamic()) || mode == X86Mode::Protected) {
        // 2-byte vex
        buffer.push(Stmt::u8(0xC5));

        let byte1 = (byte1 & 0x80) | (byte2 & 0x7F);
        if !reg_k.is_dynamic() && !vvvv_k.is_dynamic() {
            buffer.push(Stmt::u8(byte1));
            return;
        }

        let mut byte1: TokenTree = proc_macro2::Literal::u8_suffixed(byte1).into();
        if let RegKind::Dynamic(_, expr) = reg_k {
            byte1 = serialize::expr_mask_shift_inverted_and(&byte1, &delimited(expr), 8, 4)
        }
        if let RegKind::Dynamic(_, expr) = vvvv_k {
            byte1 = serialize::expr_mask_shift_inverted_and(&byte1, &delimited(expr), 0xF, 3)
        }
        buffer.push(Stmt::ExprUnsigned(byte1, Size::BYTE));
        return;
    }

    buffer.push(Stmt::u8(if data.flags.contains(Flags::VEX_OP) {0xC4} else {0x8F}));

    if mode == X86Mode::Long && (reg_k.is_dynamic() || index_k.is_dynamic() || base_k.is_dynamic()) {
        let mut byte1: TokenTree = proc_macro2::Literal::u8_suffixed(byte1).into();

        if let RegKind::Dynamic(_, expr) = reg_k {
            byte1 = serialize::expr_mask_shift_inverted_and(&byte1, &delimited(expr), 8, 4);
        }
        if let RegKind::Dynamic(_, expr) = index_k {
            byte1 = serialize::expr_mask_shift_inverted_and(&byte1, &delimited(expr), 8, 3);
        }
        if let RegKind::Dynamic(_, expr) = base_k {
            byte1 = serialize::expr_mask_shift_inverted_and(&byte1, &delimited(expr), 8, 2);
        }
        buffer.push(Stmt::ExprUnsigned(byte1, Size::BYTE));
    } else {
        buffer.push(Stmt::u8(byte1));
    }

    if vvvv_k.is_dynamic() {
        let mut byte2: TokenTree = proc_macro2::Literal::u8_suffixed(byte2).into();

        if let RegKind::Dynamic(_, expr) = vvvv_k {
            byte2 = serialize::expr_mask_shift_inverted_and(&byte2, &delimited(expr), 0xF, 3)
        }
        buffer.push(Stmt::ExprUnsigned(byte2, Size::BYTE));
    } else {
        buffer.push(Stmt::u8(byte2));
    }
}

fn compile_modrm_sib(buffer: &mut Vec<Stmt>, mode: u8, reg1: RegKind, reg2: RegKind) {
    let byte = mode                << 6 |
              (reg1.encode()  & 7) << 3 |
              (reg2.encode()  & 7)      ;

    if !reg1.is_dynamic() && !reg2.is_dynamic() {
        buffer.push(Stmt::u8(byte));
        return;
    }

    let mut byte: TokenTree = proc_macro2::Literal::u8_suffixed(byte).into();

    if let RegKind::Dynamic(_, expr) = reg1 {
        byte = serialize::expr_mask_shift_or(&byte, &delimited(expr), 7, 3);
    }
    if let RegKind::Dynamic(_, expr) = reg2 {
        byte = serialize::expr_mask_shift_or(&byte, &delimited(expr), 7, 0);
    }
    buffer.push(Stmt::ExprUnsigned(byte, Size::BYTE));
}

fn compile_sib_dynscale(buffer: &mut Vec<Stmt>, scale: u8, scale_expr: syn::Expr, reg1: RegKind, reg2: RegKind) {
    let byte = (reg1.encode()  & 7) << 3 |
               (reg2.encode()  & 7)      ;

    let mut byte: TokenTree = proc_macro2::Literal::u8_suffixed(byte).into();
    let scale: TokenTree = proc_macro2::Literal::u8_unsuffixed(scale).into();

    if let RegKind::Dynamic(_, expr) = reg1 {
        byte = serialize::expr_mask_shift_or(&byte, &delimited(expr), 7, 3);
    }
    if let RegKind::Dynamic(_, expr) = reg2 {
        byte = serialize::expr_mask_shift_or(&byte, &delimited(expr), 7, 0);
    }

    let span = scale_expr.span();
    let scale_expr = delimited(scale_expr);

    let (expr1, expr2) = serialize::expr_dynscale(
        &delimited(quote_spanned!{ span=>
            #scale_expr * #scale
        }),
        &byte
    );
    buffer.push(Stmt::PrefixStmt(expr1));
    buffer.push(Stmt::ExprUnsigned(expr2, Size::BYTE));
}
