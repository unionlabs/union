use syn::{parse, Token};
use syn::spanned::Spanned;
use proc_macro2::Span;
use proc_macro_error::emit_error;

use lazy_static::lazy_static;

use crate::common::Size;
use crate::parse_helpers::{eat_pseudo_keyword, parse_ident_or_rust_keyword, as_ident, ParseOptExt};

use super::{Context, X86Mode};
use super::ast::{Instruction, RawArg, Register, RegId, RegFamily, MemoryRefItem};

use std::collections::HashMap;

/*
 * Code
 */

// parses a full instruction
// syntax for a single op: PREFIX* ident (SIZE? expr ("," SIZE? expr)*)? ";"
pub(super) fn parse_instruction(ctx: &mut Context, input: parse::ParseStream) -> parse::Result<(Instruction, Vec<RawArg>)> {
    let span = input.cursor().span();

    let mut ops = Vec::new();

    // read prefixes + op
    let mut op = parse_ident_or_rust_keyword(input)?;
    while is_prefix(&op) {
        ops.push(op);
        op = parse_ident_or_rust_keyword(input)?;
    }
    ops.push(op);

    // parse (sizehint? expr),*
    let mut args = Vec::new();

    if !(input.is_empty() || input.peek(Token![;])) {
        args.push(parse_arg(ctx, input)?);

        while input.peek(Token![,]) {
            let _: Token![,] = input.parse()?;

            args.push(parse_arg(ctx, input)?);
        }
    }

    // let span = span.join(input.cursor().span()); // FIXME can't join spans ATM

    Ok((
        Instruction {
            idents: ops,
            span
        },
        args
    ))
}

/// checks if the given ident is a valid x86 prefix
fn is_prefix(ident: &syn::Ident) -> bool {
    const PREFIXES: [&str; 12] = [
        "lock",
        "rep", "repe", "repz",
        "repne", "repnz",
        "ss", "cs", "ds", "es", "fs", "gs"
    ];

    PREFIXES.contains(&ident.to_string().as_str())
}

/// if a size hint is present in the parse stream, returning the indicated size
fn eat_size_hint(ctx: &Context, input: parse::ParseStream) -> Option<Size> {
    const X86_SIZES: [(&str, Size); 9] = [
        ("BYTE", Size::BYTE),
        ("WORD", Size::WORD),
        ("DWORD", Size::DWORD),
        ("AWORD", Size::DWORD),
        ("FWORD", Size::FWORD),
        ("QWORD", Size::QWORD),
        ("TWORD", Size::PWORD),
        ("OWORD", Size::OWORD),
        ("YWORD", Size::HWORD)
    ];
    const X64_SIZES: [(&str, Size); 9] = [
        ("BYTE", Size::BYTE),
        ("WORD", Size::WORD),
        ("DWORD", Size::DWORD),
        ("FWORD", Size::FWORD),
        ("AWORD", Size::QWORD),
        ("QWORD", Size::QWORD),
        ("TWORD", Size::PWORD),
        ("OWORD", Size::OWORD),
        ("YWORD", Size::HWORD)
    ];

    let sizes = match ctx.mode {
        X86Mode::Protected => &X86_SIZES,
        X86Mode::Long      => &X64_SIZES
    };
    for &(kw, size) in sizes {
        if eat_pseudo_keyword(input, kw) {
            return Some(size);
        }
    }
    None
}

/// tries to parse a full arg definition
fn parse_arg(ctx: &mut Context, input: parse::ParseStream) -> parse::Result<RawArg> {
    // sizehint
    let size = eat_size_hint(ctx, input);

    let _start = input.cursor().span(); // FIXME can't join spans yet

    // bare label
    if let Some(jump) = input.parse_opt()? {
        return Ok(RawArg::JumpTarget {
            jump,
            size
        })
    }

    // indirect
    if input.peek(syn::token::Bracket) {
        let span = input.cursor().span();
        let inner;
        let _ = syn::bracketed!(inner in input);
        let inner = &inner;

        // label
        if let Some(jump) = inner.parse_opt()? {
            return Ok(RawArg::IndirectJumpTarget {
                jump,
                size
            })
        }

        // memory reference
        let nosplit = eat_pseudo_keyword(inner, "NOSPLIT");
        let disp_size = eat_size_hint(ctx, inner);
        let expr: syn::Expr = inner.parse()?;

        // split the expression into the different (displacement, register, scaled register) components
        let items = parse_adds(ctx, expr);

        return Ok(RawArg::IndirectRaw {
            span,
            nosplit,
            value_size: size,
            disp_size,
            items
        })

    }

    // it's a normal (register/immediate/typemapped) operand
    let arg: syn::Expr = input.parse()?;

    // typemapped: expr => type [expr] . ident
    if input.peek(Token![=>]) {
        let _: Token![=>] = input.parse()?;

        let base = if let Some((_, base)) = parse_reg(ctx, &arg) {
            base
        } else {
            emit_error!(arg, "Expected register");
            return Ok(RawArg::Invalid);
        };

        let ty: syn::Path = input.parse()?;

        // any attribute, register as index and immediate in index
        let mut nosplit = false;
        let mut disp_size = None;

        let items = if input.peek(syn::token::Bracket) {
            let inner;
            let _brackets = syn::bracketed!(inner in input);
            let inner = &inner;

            nosplit = eat_pseudo_keyword(inner, "NOSPLIT");
            disp_size = eat_size_hint(ctx, inner);
            let index_expr: syn::Expr = inner.parse()?;

            parse_adds(ctx, index_expr)
        } else {
            Vec::new()
        };

        let attr: Option<syn::Ident> = if input.peek(Token![.]) {
            let _: Token![.] = input.parse()?;
            Some(input.parse()?)
        } else {
            None
        };

        return Ok(RawArg::TypeMappedRaw {
            span: arg.span(), // FIXME can't join spans yet
            nosplit,
            value_size: size,
            disp_size,
            base_reg: base,
            scale: ty,
            scaled_items: items,
            attribute: attr,
        });
    }

    // direct register
    if let Some((span, reg)) = parse_reg(ctx, &arg) {
        if size.is_some() {
            emit_error!(span, "size hint with direct register");
        }
        return Ok(RawArg::Direct {
            reg,
            span
        })
    }

    // immediate
    Ok(RawArg::Immediate {
        value: arg,
        size
    })
}

/// checks if an expression is interpretable as a register reference.
fn parse_reg(ctx: &Context, expr: &syn::Expr) -> Option<(Span, Register)> {
    if let Some(path) = as_ident(expr) {
        // static register names

        let name = path.to_string();
        let mut name = name.as_str();
        if let Some(x) = ctx.state.invocation_context.aliases.get(name) {
            name = x;
        }

        let (reg, size) = match ctx.mode {
            X86Mode::Long      => X64_REGISTERS.get(&name).cloned(),
            X86Mode::Protected => X86_REGISTERS.get(&name).cloned()
        }?;

        Some((
            path.span(),
            Register::new_static(size, reg)
        ))

    } else if let syn::Expr::Call(syn::ExprCall {ref func, ref args, ..}) = expr {
        // dynamically chosen registers ( ident(expr) )
        if args.len() != 1 {
            return None;
        }

        let called = if let Some(called) = as_ident(&&*func) {
            called
        } else {
            return None;
        };

        let name = called.to_string();
        let name = name.as_str();
        let (size, family) = match ctx.mode {
            X86Mode::Long      => X64_FAMILIES.get(&name).cloned(),
            X86Mode::Protected => X86_FAMILIES.get(&name).cloned()
        }?;

        Some((
            expr.span(), // FIXME:can't join spans atm
            Register::new_dynamic(size, family, args[0].clone())
        ))
    } else {
        None
    }
}

/// splits an expression into different components of a memory reference.
fn parse_adds(ctx: &Context, expr: syn::Expr) -> Vec<MemoryRefItem> {
    let mut adds = Vec::new();
    collect_adds(expr, &mut adds);

    let mut items = Vec::new();

    // detect what kind of equation we're dealing with
    for node in adds {
        // simple reg
        if let Some((_, reg)) = parse_reg(ctx, &node) {
            items.push(MemoryRefItem::Register(reg));
            continue;
        }
        if let syn::Expr::Binary(syn::ExprBinary { op: syn::BinOp::Mul(_), ref left, ref right, .. } ) = node {
            // reg * const
            if let Some((_, reg)) = parse_reg(ctx, left) {
                if let syn::Expr::Lit(syn::ExprLit {ref lit, ..}) = **right {
                    if let syn::Lit::Int(lit) = lit {
                        if let Ok(value) = lit.base10_parse::<isize>() {
                            items.push(MemoryRefItem::ScaledRegister(reg, value));
                            continue;
                        }
                    }
                }
            } // const * reg
            if let Some((_, reg)) = parse_reg(ctx, right) {
                if let syn::Expr::Lit(syn::ExprLit {ref lit, ..}) = **left {
                    if let syn::Lit::Int(lit) = lit {
                        if let Ok(value) = lit.base10_parse::<isize>() {
                            items.push(MemoryRefItem::ScaledRegister(reg, value));
                            continue;
                        }
                    }
                }
            }
        }
        items.push(MemoryRefItem::Displacement(node));
    }

    items
}

/// Takes an expression and splits all added (and subtracted) components
fn collect_adds(node: syn::Expr, collection: &mut Vec<syn::Expr>) {
    if let syn::Expr::Binary(syn::ExprBinary { op: syn::BinOp::Add(_), left, right, .. } ) = node {
        collect_adds(*left, collection);
        collect_adds(*right, collection);
    } else if let syn::Expr::Binary(syn::ExprBinary { op: syn::BinOp::Sub(sub), left, right, .. } ) = node {
        collect_adds(*left, collection);
        collection.push(syn::Expr::Unary(syn::ExprUnary { op: syn::UnOp::Neg(sub), expr: right, attrs: Vec::new() } ));
    } else {
        collection.push(node);
    }
}

// why
lazy_static!{
    static ref X64_REGISTERS: HashMap<&'static str, (RegId, Size)> = {
        use self::RegId::*;
        use crate::common::Size::*;

        static MAP: &[(&str, (RegId, Size))] = &[
            ("rax", (RAX, QWORD)),
            ("r0" , (RAX, QWORD)),
            ("rcx", (RCX, QWORD)),
            ("r1" , (RCX, QWORD)),
            ("rdx", (RDX, QWORD)),
            ("r2" , (RDX, QWORD)),
            ("rbx", (RBX, QWORD)),
            ("r3" , (RBX, QWORD)),
            ("rsp", (RSP, QWORD)),
            ("r4" , (RSP, QWORD)),
            ("rbp", (RBP, QWORD)),
            ("r5" , (RBP, QWORD)),
            ("rsi", (RSI, QWORD)),
            ("r6" , (RSI, QWORD)),
            ("rdi", (RDI, QWORD)),
            ("r7" , (RDI, QWORD)),
            ("r8" , (R8,  QWORD)),
            ("r9" , (R9,  QWORD)),
            ("r10", (R10, QWORD)),
            ("r11", (R11, QWORD)),
            ("r12", (R12, QWORD)),
            ("r13", (R13, QWORD)),
            ("r14", (R14, QWORD)),
            ("r15", (R15, QWORD)),

            ("eax" , (RAX, DWORD)),
            ("r0d" , (RAX, DWORD)),
            ("ecx" , (RCX, DWORD)),
            ("r1d" , (RCX, DWORD)),
            ("edx" , (RDX, DWORD)),
            ("r2d" , (RDX, DWORD)),
            ("ebx" , (RBX, DWORD)),
            ("r3d" , (RBX, DWORD)),
            ("esp" , (RSP, DWORD)),
            ("r4d" , (RSP, DWORD)),
            ("ebp" , (RBP, DWORD)),
            ("r5d" , (RBP, DWORD)),
            ("esi" , (RSI, DWORD)),
            ("r6d" , (RSI, DWORD)),
            ("edi" , (RDI, DWORD)),
            ("r7d" , (RDI, DWORD)),
            ("r8d" , (R8,  DWORD)),
            ("r9d" , (R9,  DWORD)),
            ("r10d", (R10, DWORD)),
            ("r11d", (R11, DWORD)),
            ("r12d", (R12, DWORD)),
            ("r13d", (R13, DWORD)),
            ("r14d", (R14, DWORD)),
            ("r15d", (R15, DWORD)),

            ("ax"  , (RAX, WORD)),
            ("r0w" , (RAX, WORD)),
            ("cx"  , (RCX, WORD)),
            ("r1w" , (RCX, WORD)),
            ("dx"  , (RDX, WORD)),
            ("r2w" , (RDX, WORD)),
            ("bx"  , (RBX, WORD)),
            ("r3w" , (RBX, WORD)),
            ("sp"  , (RSP, WORD)),
            ("r4w" , (RSP, WORD)),
            ("bp"  , (RBP, WORD)),
            ("r5w" , (RBP, WORD)),
            ("si"  , (RSI, WORD)),
            ("r6w" , (RSI, WORD)),
            ("di"  , (RDI, WORD)),
            ("r7w" , (RDI, WORD)),
            ("r8w" , (R8,  WORD)),
            ("r9w" , (R9,  WORD)),
            ("r10w", (R10, WORD)),
            ("r11w", (R11, WORD)),
            ("r12w", (R12, WORD)),
            ("r13w", (R13, WORD)),
            ("r14w", (R14, WORD)),
            ("r15w", (R15, WORD)),

            ("al"  , (RAX, BYTE)),
            ("r0b" , (RAX, BYTE)),
            ("cl"  , (RCX, BYTE)),
            ("r1b" , (RCX, BYTE)),
            ("dl"  , (RDX, BYTE)),
            ("r2b" , (RDX, BYTE)),
            ("bl"  , (RBX, BYTE)),
            ("r3b" , (RBX, BYTE)),
            ("spl" , (RSP, BYTE)),
            ("r4b" , (RSP, BYTE)),
            ("bpl" , (RBP, BYTE)),
            ("r5b" , (RBP, BYTE)),
            ("sil" , (RSI, BYTE)),
            ("r6b" , (RSI, BYTE)),
            ("dil" , (RDI, BYTE)),
            ("r7b" , (RDI, BYTE)),
            ("r8b" , (R8,  BYTE)),
            ("r9b" , (R9,  BYTE)),
            ("r10b", (R10, BYTE)),
            ("r11b", (R11, BYTE)),
            ("r12b", (R12, BYTE)),
            ("r13b", (R13, BYTE)),
            ("r14b", (R14, BYTE)),
            ("r15b", (R15, BYTE)),

            ("rip", (RIP, QWORD)),
            ("eip", (RIP, DWORD)),

            ("ah", (AH, BYTE)),
            ("ch", (CH, BYTE)),
            ("dh", (DH, BYTE)),
            ("bh", (BH, BYTE)),

            ("st0", (ST0, PWORD)),
            ("st1", (ST1, PWORD)),
            ("st2", (ST2, PWORD)),
            ("st3", (ST3, PWORD)),
            ("st4", (ST4, PWORD)),
            ("st5", (ST5, PWORD)),
            ("st6", (ST6, PWORD)),
            ("st7", (ST7, PWORD)),

            ("mm0", (MMX0, QWORD)),
            ("mm1", (MMX1, QWORD)),
            ("mm2", (MMX2, QWORD)),
            ("mm3", (MMX3, QWORD)),
            ("mm4", (MMX4, QWORD)),
            ("mm5", (MMX5, QWORD)),
            ("mm6", (MMX6, QWORD)),
            ("mm7", (MMX7, QWORD)),

            ("xmm0" , (XMM0 , OWORD)),
            ("xmm1" , (XMM1 , OWORD)),
            ("xmm2" , (XMM2 , OWORD)),
            ("xmm3" , (XMM3 , OWORD)),
            ("xmm4" , (XMM4 , OWORD)),
            ("xmm5" , (XMM5 , OWORD)),
            ("xmm6" , (XMM6 , OWORD)),
            ("xmm7" , (XMM7 , OWORD)),
            ("xmm8" , (XMM8 , OWORD)),
            ("xmm9" , (XMM9 , OWORD)),
            ("xmm10", (XMM10, OWORD)),
            ("xmm11", (XMM11, OWORD)),
            ("xmm12", (XMM12, OWORD)),
            ("xmm13", (XMM13, OWORD)),
            ("xmm14", (XMM14, OWORD)),
            ("xmm15", (XMM15, OWORD)),

            ("ymm0" , (XMM0 , HWORD)),
            ("ymm1" , (XMM1 , HWORD)),
            ("ymm2" , (XMM2 , HWORD)),
            ("ymm3" , (XMM3 , HWORD)),
            ("ymm4" , (XMM4 , HWORD)),
            ("ymm5" , (XMM5 , HWORD)),
            ("ymm6" , (XMM6 , HWORD)),
            ("ymm7" , (XMM7 , HWORD)),
            ("ymm8" , (XMM8 , HWORD)),
            ("ymm9" , (XMM9 , HWORD)),
            ("ymm10", (XMM10, HWORD)),
            ("ymm11", (XMM11, HWORD)),
            ("ymm12", (XMM12, HWORD)),
            ("ymm13", (XMM13, HWORD)),
            ("ymm14", (XMM14, HWORD)),
            ("ymm15", (XMM15, HWORD)),

            ("es", (ES, WORD)),
            ("cs", (CS, WORD)),
            ("ss", (SS, WORD)),
            ("ds", (DS, WORD)),
            ("fs", (FS, WORD)),
            ("gs", (GS, WORD)),

            ("cr0" , (CR0 , QWORD)),
            ("cr1" , (CR1 , QWORD)),
            ("cr2" , (CR2 , QWORD)),
            ("cr3" , (CR3 , QWORD)),
            ("cr4" , (CR4 , QWORD)),
            ("cr5" , (CR5 , QWORD)),
            ("cr6" , (CR6 , QWORD)),
            ("cr7" , (CR7 , QWORD)),
            ("cr8" , (CR8 , QWORD)),
            ("cr9" , (CR9 , QWORD)),
            ("cr10", (CR10, QWORD)),
            ("cr11", (CR11, QWORD)),
            ("cr12", (CR12, QWORD)),
            ("cr13", (CR13, QWORD)),
            ("cr14", (CR14, QWORD)),
            ("cr15", (CR15, QWORD)),

            ("dr0" , (DR0 , QWORD)),
            ("dr1" , (DR1 , QWORD)),
            ("dr2" , (DR2 , QWORD)),
            ("dr3" , (DR3 , QWORD)),
            ("dr4" , (DR4 , QWORD)),
            ("dr5" , (DR5 , QWORD)),
            ("dr6" , (DR6 , QWORD)),
            ("dr7" , (DR7 , QWORD)),
            ("dr8" , (DR8 , QWORD)),
            ("dr9" , (DR9 , QWORD)),
            ("dr10", (DR10, QWORD)),
            ("dr11", (DR11, QWORD)),
            ("dr12", (DR12, QWORD)),
            ("dr13", (DR13, QWORD)),
            ("dr14", (DR14, QWORD)),
            ("dr15", (DR15, QWORD)),

            ("bnd0", (BND0, OWORD)),
            ("bnd1", (BND1, OWORD)),
            ("bnd2", (BND2, OWORD)),
            ("bnd3", (BND3, OWORD)),
        ];
        MAP.iter().cloned().collect()
    };
    static ref X86_REGISTERS: HashMap<&'static str, (RegId, Size)> = {
        use self::RegId::*;
        use crate::common::Size::*;

        static MAP: &[(&str, (RegId, Size))] = &[
            ("eax", (RAX, DWORD)),
            ("ecx", (RCX, DWORD)),
            ("edx", (RDX, DWORD)),
            ("ebx", (RBX, DWORD)),
            ("esp", (RSP, DWORD)),
            ("ebp", (RBP, DWORD)),
            ("esi", (RSI, DWORD)),
            ("edi", (RDI, DWORD)),

            ("ax", (RAX, WORD)),
            ("cx", (RCX, WORD)),
            ("dx", (RDX, WORD)),
            ("bx", (RBX, WORD)),
            ("sp", (RSP, WORD)),
            ("bp", (RBP, WORD)),
            ("si", (RSI, WORD)),
            ("di", (RDI, WORD)),

            ("al", (RAX, BYTE)),
            ("cl", (RCX, BYTE)),
            ("dl", (RDX, BYTE)),
            ("bl", (RBX, BYTE)),

            ("eip", (RIP, DWORD)),

            ("ah", (AH, BYTE)),
            ("ch", (CH, BYTE)),
            ("dh", (DH, BYTE)),
            ("bh", (BH, BYTE)),

            ("st0", (ST0, PWORD)),
            ("st1", (ST1, PWORD)),
            ("st2", (ST2, PWORD)),
            ("st3", (ST3, PWORD)),
            ("st4", (ST4, PWORD)),
            ("st5", (ST5, PWORD)),
            ("st6", (ST6, PWORD)),
            ("st7", (ST7, PWORD)),

            ("mm0", (MMX0, QWORD)),
            ("mm1", (MMX1, QWORD)),
            ("mm2", (MMX2, QWORD)),
            ("mm3", (MMX3, QWORD)),
            ("mm4", (MMX4, QWORD)),
            ("mm5", (MMX5, QWORD)),
            ("mm6", (MMX6, QWORD)),
            ("mm7", (MMX7, QWORD)),

            ("xmm0", (XMM0, OWORD)),
            ("xmm1", (XMM1, OWORD)),
            ("xmm2", (XMM2, OWORD)),
            ("xmm3", (XMM3, OWORD)),
            ("xmm4", (XMM4, OWORD)),
            ("xmm5", (XMM5, OWORD)),
            ("xmm6", (XMM6, OWORD)),
            ("xmm7", (XMM7, OWORD)),

            ("ymm0", (XMM0, HWORD)),
            ("ymm1", (XMM1, HWORD)),
            ("ymm2", (XMM2, HWORD)),
            ("ymm3", (XMM3, HWORD)),
            ("ymm4", (XMM4, HWORD)),
            ("ymm5", (XMM5, HWORD)),
            ("ymm6", (XMM6, HWORD)),
            ("ymm7", (XMM7, HWORD)),

            ("es", (ES, WORD)),
            ("cs", (CS, WORD)),
            ("ss", (SS, WORD)),
            ("ds", (DS, WORD)),
            ("fs", (FS, WORD)),
            ("gs", (GS, WORD)),

            ("cr0", (CR0, DWORD)),
            ("cr1", (CR1, DWORD)),
            ("cr2", (CR2, DWORD)),
            ("cr3", (CR3, DWORD)),
            ("cr4", (CR4, DWORD)),
            ("cr5", (CR5, DWORD)),
            ("cr6", (CR6, DWORD)),
            ("cr7", (CR7, DWORD)),

            ("dr0", (DR0, DWORD)),
            ("dr1", (DR1, DWORD)),
            ("dr2", (DR2, DWORD)),
            ("dr3", (DR3, DWORD)),
            ("dr4", (DR4, DWORD)),
            ("dr5", (DR5, DWORD)),
            ("dr6", (DR6, DWORD)),
            ("dr7", (DR7, DWORD)),

            ("bnd0", (BND0, OWORD)),
            ("bnd1", (BND1, OWORD)),
            ("bnd2", (BND2, OWORD)),
            ("bnd3", (BND3, OWORD)),
        ];
        MAP.iter().cloned().collect()
    };
    static ref X64_FAMILIES:  HashMap<&'static str, (Size, RegFamily)> = {
        static MAP: &[(&str, (Size, RegFamily))] = &[
            ("Rb", (Size::BYTE,  RegFamily::LEGACY)),
            ("Rh", (Size::BYTE,  RegFamily::HIGHBYTE)),
            ("Rw", (Size::WORD,  RegFamily::LEGACY)),
            ("Rd", (Size::DWORD, RegFamily::LEGACY)),
            ("Ra", (Size::QWORD, RegFamily::LEGACY)),
            ("Rq", (Size::QWORD, RegFamily::LEGACY)),
            ("Rf", (Size::PWORD, RegFamily::FP)),
            ("Rm", (Size::QWORD, RegFamily::MMX)),
            ("Rx", (Size::OWORD, RegFamily::XMM)),
            ("Ry", (Size::HWORD, RegFamily::XMM)),
            ("Rs", (Size::WORD,  RegFamily::SEGMENT)),
            ("RC", (Size::QWORD, RegFamily::CONTROL)),
            ("RD", (Size::QWORD, RegFamily::DEBUG)),
            ("RB", (Size::OWORD, RegFamily::BOUND)),
        ];
        MAP.iter().cloned().collect()
    };
    static ref X86_FAMILIES:  HashMap<&'static str, (Size, RegFamily)> = {
        static MAP: &[(&str, (Size, RegFamily))] = &[
            ("Rb",(Size::BYTE,  RegFamily::LEGACY)),
            ("Rh",(Size::BYTE,  RegFamily::HIGHBYTE)),
            ("Rw",(Size::WORD,  RegFamily::LEGACY)),
            ("Ra",(Size::DWORD, RegFamily::LEGACY)),
            ("Rd",(Size::DWORD, RegFamily::LEGACY)),
            ("Rf",(Size::PWORD, RegFamily::FP)),
            ("Rm",(Size::QWORD, RegFamily::MMX)),
            ("Rx",(Size::OWORD, RegFamily::XMM)),
            ("Ry",(Size::HWORD, RegFamily::XMM)),
            ("Rs",(Size::WORD,  RegFamily::SEGMENT)),
            ("RC",(Size::DWORD, RegFamily::CONTROL)),
            ("RD",(Size::DWORD, RegFamily::DEBUG)),
            ("RB",(Size::OWORD, RegFamily::BOUND)),
        ];
        MAP.iter().cloned().collect()
    };
}
