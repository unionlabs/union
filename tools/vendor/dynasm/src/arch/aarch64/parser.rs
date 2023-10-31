use syn::{parse, Token};

use lazy_static::lazy_static;

use crate::parse_helpers::{parse_ident_or_rust_keyword, ParseOpt, ParseOptExt};
use crate::common::Size;

use super::Context;
use super::ast::{Instruction, RawArg, Register, RegId, RegKind, RegScalar, RegVector, RegFamily, RefItem, Modifier, ModifyExpr};

use std::collections::HashMap;

// parses a full instruction
// syntax for a single op: ident ("." expr)* (arg ("," arg)*)? ";"
pub(super) fn parse_instruction(ctx: &mut Context, input: parse::ParseStream) -> parse::Result<(Instruction, Vec<RawArg>)> {
    let span = input.cursor().span();

    // read the full dot-separated op
    let op = parse_ident_or_rust_keyword(input)?;

    let mut args = Vec::new();

    // parse any dot args
    while input.peek(Token![.]) {
        let span = input.cursor().span();
        let _: Token![.] = input.parse()?;
        let arg: syn::Ident = input.parse()?;

        args.push(RawArg::Dot { span } );
        args.push(RawArg::Lit { ident: arg });
    }

    // parse 0 or more comma-separated args
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
            ident: op,
            span
        },
        args
    ))
}

/// tries to parse a full arg definition
fn parse_arg(ctx: &mut Context, input: parse::ParseStream) -> parse::Result<RawArg> {
    let _start = input.cursor().span(); // FIXME can't join spans yet

    // a label
    if let Some(jump) = input.parse_opt()? {
        return Ok(RawArg::JumpTarget {
            jump
        });
    }

    // reference
    if input.peek(syn::token::Bracket) {
        let span = input.cursor().span();
        let inner;
        let _ = syn::bracketed!(inner in input);
        let inner = &inner;

        // parse comma-separated inner items
        let mut items = Vec::new();
        items.push(parse_refitem(ctx, inner)?);

        while inner.peek(Token![,]) {
            let _: Token![,] = inner.parse()?;

            items.push(parse_refitem(ctx, inner)?);
        }

        // TODO: confirm we parsed everything?

        let bang = if input.peek(Token![!]) {
            let _: Token![!] = input.parse()?;
            true
        } else {
            false
        };

        return Ok(RawArg::Reference {
            span,
            items,
            bang
        });
    }

    // registerlist
    if input.peek(syn::token::Brace) {
        let span = input.cursor().span();
        let inner;
        let _ = syn::braced!(inner in input);
        let inner = &inner;

        let first = parse_reg(ctx, inner)?.ok_or_else(|| inner.error("Expected register"))?;

        // parses {reg - reg}
        let mut ast = if inner.peek(Token![-]) {
            let _: Token![-] = inner.parse()?;

            let last = parse_reg(ctx, inner)?.ok_or_else(|| inner.error("Expected register"))?;

            RawArg::DashList {
                span,
                first,
                last,
                element: None
            }

        // parses {reg, reg .. , reg}
        } else if inner.peek(Token![*]) {
            let _: Token![*] = inner.parse()?;

            let amount: syn::Expr = inner.parse()?;

            RawArg::AmountList {
                span,
                first,
                amount,
                element: None
            }

        } else {
            let mut items = Vec::new();
            items.push(first);

            while inner.peek(Token![,]) {
                let _: Token![,] = inner.parse()?;

                items.push(parse_reg(ctx, inner)?.ok_or_else(|| inner.error("Expected register"))?);
            }

            RawArg::CommaList {
                span,
                items,
                element: None
            }
        };

        // parse a trailing [element] declaration
        if input.peek(syn::token::Bracket) {
            let inner;
            let _ = syn::bracketed!(inner in input);
            let inner = &inner;

            let expr: syn::Expr = inner.parse()?;
            match ast {
                RawArg::DashList {ref mut element, ..} |
                RawArg::CommaList {ref mut element, ..} |
                RawArg::AmountList {ref mut element, ..} => *element = Some(expr),
                _ => ()
            }
        }

        return Ok(ast)
    }

    // modifier
    if let Some(modifier) = input.parse_opt()? {
        return Ok(RawArg::Modifier {
            span: _start,
            modifier
        });
    }

    // immediate (arm notation)
    if input.peek(Token![#]) {
        let _: Token![#] = input.parse()?;
        let arg: syn::Expr = input.parse()?;
        return Ok(RawArg::Immediate {
            prefixed: true,
            value: arg
        });
    }

    // register
    if let Some(reg) = parse_reg(ctx, input)? {
        return Ok(RawArg::Direct {
            reg,
            span: _start
        })
    }

    // immediate (relaxed notation)
    let arg: syn::Expr = input.parse()?;
    Ok(RawArg::Immediate {
        prefixed: false,
        value: arg
    })
}

fn parse_refitem(ctx: &mut Context, input: parse::ParseStream) -> parse::Result<RefItem> {
    let _start = input.cursor().span(); // FIXME can't join spans yet

    // modifier
    if let Some(modifier) = input.parse_opt()? {
        return Ok(RefItem::Modifier {
            span: _start,
            modifier
        });
    }

    // immediate (arm notation)
    if input.peek(Token![#]) {
        let _: Token![#] = input.parse()?;
        let arg: syn::Expr = input.parse()?;
        return Ok(RefItem::Immediate {
            value: arg
        });
    }

    // register
    if let Some(reg) = parse_reg(ctx, input)? {
        return Ok(RefItem::Direct {
            reg,
            span: _start
        })
    }

    // immediate (relaxed notation)
    let arg: syn::Expr = input.parse()?;
    Ok(RefItem::Immediate {
        value: arg
    })
}

fn parse_reg(ctx: &mut Context, input: parse::ParseStream) -> parse::Result<Option<Register>> {
    let name = match input.step(|cursor| {
        if let Some((ident, rest)) = cursor.ident() {
            let mut ident = ident.to_string();

            if AARCH64_FAMILIES.contains_key(&*ident) {
                return Ok((ident, rest));
            }

            if let Some(repl) = ctx.state.invocation_context.aliases.get(&ident) {
                ident = repl.clone();
            }

            if AARCH64_REGISTERS.contains_key(&*ident) {
                return Ok((ident, rest));
            }
        }
        Err(cursor.error("expected identifier"))
    }) {
        Ok(name) => name,
        Err(_) => return Ok(None)
    };

    let kind;
    let size;

    if let Some(&(id, s)) = AARCH64_REGISTERS.get(&*name) {
        size = s;
        kind = RegKind::Static(id);

    } else if let Some(&(family, s)) = AARCH64_FAMILIES.get(&*name) {

        // parse the dynamic register expression
        let inner;
        let _ = syn::parenthesized!(inner in input);
        let inner = &inner;

        let expr: syn::Expr = inner.parse()?;

        size = s;
        kind = RegKind::Dynamic(family, expr);
    } else {
        unreachable!();
    };

    if let Some(size) = size {
        Ok(Some(Register::Scalar(RegScalar {
            kind,
            size
        })))
    } else {
        // parse possible vector trailers

        // parse the elementsize/lanes specifier ("." [BHSDQ][124816])
        // note: actual ARM register width specifiers put the lane count before
        // the size, but this cannot be parsed by the rust tokenizer
        let _: Token![.] = input.parse()?;
        let (element_size, lanes) = input.step(|cursor| {
            if let Some((ident, rest)) = cursor.ident() {
                let ident = ident.to_string();

                if !ident.is_char_boundary(1) {
                    return Err(cursor.error("Invalid width specifier"));
                }

                let (first, trailer) = ident.split_at(1);
                let element_size = match first {
                    "B" | "b" => Size::BYTE,
                    "H" | "h" => Size::WORD,
                    "S" | "s" => Size::DWORD,
                    "D" | "d" => Size::QWORD,
                    "Q" | "q" => Size::OWORD,
                    _ => return Err(cursor.error("Invalid width specifier"))
                };

                let lanes = if trailer.is_empty() {
                    None
                } else {
                    Some(match trailer.parse::<u8>() {
                        Ok(1) => 1,
                        Ok(2) => 2,
                        Ok(4) => 4,
                        Ok(8) => 8,
                        Ok(16)=> 16,
                        _ => return Err(cursor.error("Invalid width specifier"))
                    })
                };

                return Ok(((element_size, lanes), rest));
            }
            Err(cursor.error("Expected a width/lane specifier"))
        })?;

        // parse the element specifier
        let element = if input.peek(syn::token::Bracket) {
            let inner;
            let _ = syn::bracketed!(inner in input);
            let inner = &inner;

            let expr: syn::Expr = inner.parse()?;
            Some(expr)
        } else {
            None
        };

        Ok(Some(Register::Vector(RegVector {
            kind,
            element_size,
            lanes,
            element
        })))
    }
}

impl ParseOpt for ModifyExpr {
    fn parse(input: parse::ParseStream) -> parse::Result<Option<Self>> {
        let modifier: Modifier = match input.parse() {
            Ok(m) => m,
            Err(_) => return Ok(None)
        };

        // valid terminating symbols
        if input.is_empty() || input.peek(Token![;]) || input.peek(Token![,]) {
            return Ok(Some(ModifyExpr::new(modifier, None)));
        }

        // parse an expr with possible leading # sign
        if input.peek(Token![#]) {
            let _: Token![#] = input.parse()?;
        }
        let expr: syn::Expr = input.parse()?;

        Ok(Some(ModifyExpr::new(modifier, Some(expr))))
    }
}

impl parse::Parse for Modifier {
    fn parse(input: parse::ParseStream) -> parse::Result<Modifier> {
        input.step(|cursor| {
            if let Some((ident, rest)) = cursor.ident() {
                let modifier = match &*ident.to_string() {
                    "LSL"  | "lsl"  => Modifier::LSL,
                    "LSR"  | "lsr"  => Modifier::LSR,
                    "ASR"  | "asr"  => Modifier::ASR,
                    "ROR"  | "ror"  => Modifier::ROR,
                    "SXTX" | "sxtx" => Modifier::SXTX,
                    "SXTW" | "sxtw" => Modifier::SXTW,
                    "SXTH" | "sxth" => Modifier::SXTH,
                    "SXTB" | "sxtb" => Modifier::SXTB,
                    "UXTX" | "uxtx" => Modifier::UXTX,
                    "UXTW" | "uxtw" => Modifier::UXTW,
                    "UXTH" | "uxth" => Modifier::UXTH,
                    "UXTB" | "uxtb" => Modifier::UXTB,
                    "MSL"  | "msl"  => Modifier::MSL,
                    _ => return Err(cursor.error("Unknown modifier"))
                };

                Ok((modifier, rest))
            } else {
                Err(cursor.error("Expected modifier"))
            }
        })
    }
}

lazy_static!{
    static ref AARCH64_REGISTERS: HashMap<&'static str, (RegId, Option<Size>)> = {
        use self::RegId::*;
        use crate::common::Size::*;

        static MAP: &[(&str, (RegId, Option<Size>))] = &[
            ("x0" , (X0 , Some(QWORD))),
            ("x1" , (X1 , Some(QWORD))),
            ("x2" , (X2 , Some(QWORD))),
            ("x3" , (X3 , Some(QWORD))),
            ("x4" , (X4 , Some(QWORD))),
            ("x5" , (X5 , Some(QWORD))),
            ("x6" , (X6 , Some(QWORD))),
            ("x7" , (X7 , Some(QWORD))),
            ("x8" , (X8 , Some(QWORD))),
            ("x9" , (X9 , Some(QWORD))),
            ("x10", (X10, Some(QWORD))),
            ("x11", (X11, Some(QWORD))),
            ("x12", (X12, Some(QWORD))),
            ("x13", (X13, Some(QWORD))),
            ("x14", (X14, Some(QWORD))),
            ("x15", (X15, Some(QWORD))),
            ("x16", (X16, Some(QWORD))),
            ("x17", (X17, Some(QWORD))),
            ("x18", (X18, Some(QWORD))),
            ("x19", (X19, Some(QWORD))),
            ("x20", (X20, Some(QWORD))),
            ("x21", (X21, Some(QWORD))),
            ("x22", (X22, Some(QWORD))),
            ("x23", (X23, Some(QWORD))),
            ("x24", (X24, Some(QWORD))),
            ("x25", (X25, Some(QWORD))),
            ("x26", (X26, Some(QWORD))),
            ("x27", (X27, Some(QWORD))),
            ("x28", (X28, Some(QWORD))),
            ("x29", (X29, Some(QWORD))),
            ("x30", (X30, Some(QWORD))),

            ("w0" , (X0 , Some(DWORD))),
            ("w1" , (X1 , Some(DWORD))),
            ("w2" , (X2 , Some(DWORD))),
            ("w3" , (X3 , Some(DWORD))),
            ("w4" , (X4 , Some(DWORD))),
            ("w5" , (X5 , Some(DWORD))),
            ("w6" , (X6 , Some(DWORD))),
            ("w7" , (X7 , Some(DWORD))),
            ("w8" , (X8 , Some(DWORD))),
            ("w9" , (X9 , Some(DWORD))),
            ("w10", (X10, Some(DWORD))),
            ("w11", (X11, Some(DWORD))),
            ("w12", (X12, Some(DWORD))),
            ("w13", (X13, Some(DWORD))),
            ("w14", (X14, Some(DWORD))),
            ("w15", (X15, Some(DWORD))),
            ("w16", (X16, Some(DWORD))),
            ("w17", (X17, Some(DWORD))),
            ("w18", (X18, Some(DWORD))),
            ("w19", (X19, Some(DWORD))),
            ("w20", (X20, Some(DWORD))),
            ("w21", (X21, Some(DWORD))),
            ("w22", (X22, Some(DWORD))),
            ("w23", (X23, Some(DWORD))),
            ("w24", (X24, Some(DWORD))),
            ("w25", (X25, Some(DWORD))),
            ("w26", (X26, Some(DWORD))),
            ("w27", (X27, Some(DWORD))),
            ("w28", (X28, Some(DWORD))),
            ("w29", (X29, Some(DWORD))),
            ("w30", (X30, Some(DWORD))),

            ("sp",  (SP,  Some(QWORD))),
            ("wsp", (SP,  Some(DWORD))),

            ("xzr", (XZR, Some(QWORD))),
            ("wzr", (XZR, Some(DWORD))),

            ("b0" , (V0 , Some(BYTE))),
            ("b1" , (V1 , Some(BYTE))),
            ("b2" , (V2 , Some(BYTE))),
            ("b3" , (V3 , Some(BYTE))),
            ("b4" , (V4 , Some(BYTE))),
            ("b5" , (V5 , Some(BYTE))),
            ("b6" , (V6 , Some(BYTE))),
            ("b7" , (V7 , Some(BYTE))),
            ("b8" , (V8 , Some(BYTE))),
            ("b9" , (V9 , Some(BYTE))),
            ("b10", (V10, Some(BYTE))),
            ("b11", (V11, Some(BYTE))),
            ("b12", (V12, Some(BYTE))),
            ("b13", (V13, Some(BYTE))),
            ("b14", (V14, Some(BYTE))),
            ("b15", (V15, Some(BYTE))),
            ("b16", (V16, Some(BYTE))),
            ("b17", (V17, Some(BYTE))),
            ("b18", (V18, Some(BYTE))),
            ("b19", (V19, Some(BYTE))),
            ("b20", (V20, Some(BYTE))),
            ("b21", (V21, Some(BYTE))),
            ("b22", (V22, Some(BYTE))),
            ("b23", (V23, Some(BYTE))),
            ("b24", (V24, Some(BYTE))),
            ("b25", (V25, Some(BYTE))),
            ("b26", (V26, Some(BYTE))),
            ("b27", (V27, Some(BYTE))),
            ("b28", (V28, Some(BYTE))),
            ("b29", (V29, Some(BYTE))),
            ("b30", (V30, Some(BYTE))),
            ("b31", (V31, Some(BYTE))),

            ("h0" , (V0 , Some(WORD))),
            ("h1" , (V1 , Some(WORD))),
            ("h2" , (V2 , Some(WORD))),
            ("h3" , (V3 , Some(WORD))),
            ("h4" , (V4 , Some(WORD))),
            ("h5" , (V5 , Some(WORD))),
            ("h6" , (V6 , Some(WORD))),
            ("h7" , (V7 , Some(WORD))),
            ("h8" , (V8 , Some(WORD))),
            ("h9" , (V9 , Some(WORD))),
            ("h10", (V10, Some(WORD))),
            ("h11", (V11, Some(WORD))),
            ("h12", (V12, Some(WORD))),
            ("h13", (V13, Some(WORD))),
            ("h14", (V14, Some(WORD))),
            ("h15", (V15, Some(WORD))),
            ("h16", (V16, Some(WORD))),
            ("h17", (V17, Some(WORD))),
            ("h18", (V18, Some(WORD))),
            ("h19", (V19, Some(WORD))),
            ("h20", (V20, Some(WORD))),
            ("h21", (V21, Some(WORD))),
            ("h22", (V22, Some(WORD))),
            ("h23", (V23, Some(WORD))),
            ("h24", (V24, Some(WORD))),
            ("h25", (V25, Some(WORD))),
            ("h26", (V26, Some(WORD))),
            ("h27", (V27, Some(WORD))),
            ("h28", (V28, Some(WORD))),
            ("h29", (V29, Some(WORD))),
            ("h30", (V30, Some(WORD))),
            ("h31", (V31, Some(WORD))),

            ("s0" , (V0 , Some(DWORD))),
            ("s1" , (V1 , Some(DWORD))),
            ("s2" , (V2 , Some(DWORD))),
            ("s3" , (V3 , Some(DWORD))),
            ("s4" , (V4 , Some(DWORD))),
            ("s5" , (V5 , Some(DWORD))),
            ("s6" , (V6 , Some(DWORD))),
            ("s7" , (V7 , Some(DWORD))),
            ("s8" , (V8 , Some(DWORD))),
            ("s9" , (V9 , Some(DWORD))),
            ("s10", (V10, Some(DWORD))),
            ("s11", (V11, Some(DWORD))),
            ("s12", (V12, Some(DWORD))),
            ("s13", (V13, Some(DWORD))),
            ("s14", (V14, Some(DWORD))),
            ("s15", (V15, Some(DWORD))),
            ("s16", (V16, Some(DWORD))),
            ("s17", (V17, Some(DWORD))),
            ("s18", (V18, Some(DWORD))),
            ("s19", (V19, Some(DWORD))),
            ("s20", (V20, Some(DWORD))),
            ("s21", (V21, Some(DWORD))),
            ("s22", (V22, Some(DWORD))),
            ("s23", (V23, Some(DWORD))),
            ("s24", (V24, Some(DWORD))),
            ("s25", (V25, Some(DWORD))),
            ("s26", (V26, Some(DWORD))),
            ("s27", (V27, Some(DWORD))),
            ("s28", (V28, Some(DWORD))),
            ("s29", (V29, Some(DWORD))),
            ("s30", (V30, Some(DWORD))),
            ("s31", (V31, Some(DWORD))),

            ("d0" , (V0 , Some(QWORD))),
            ("d1" , (V1 , Some(QWORD))),
            ("d2" , (V2 , Some(QWORD))),
            ("d3" , (V3 , Some(QWORD))),
            ("d4" , (V4 , Some(QWORD))),
            ("d5" , (V5 , Some(QWORD))),
            ("d6" , (V6 , Some(QWORD))),
            ("d7" , (V7 , Some(QWORD))),
            ("d8" , (V8 , Some(QWORD))),
            ("d9" , (V9 , Some(QWORD))),
            ("d10", (V10, Some(QWORD))),
            ("d11", (V11, Some(QWORD))),
            ("d12", (V12, Some(QWORD))),
            ("d13", (V13, Some(QWORD))),
            ("d14", (V14, Some(QWORD))),
            ("d15", (V15, Some(QWORD))),
            ("d16", (V16, Some(QWORD))),
            ("d17", (V17, Some(QWORD))),
            ("d18", (V18, Some(QWORD))),
            ("d19", (V19, Some(QWORD))),
            ("d20", (V20, Some(QWORD))),
            ("d21", (V21, Some(QWORD))),
            ("d22", (V22, Some(QWORD))),
            ("d23", (V23, Some(QWORD))),
            ("d24", (V24, Some(QWORD))),
            ("d25", (V25, Some(QWORD))),
            ("d26", (V26, Some(QWORD))),
            ("d27", (V27, Some(QWORD))),
            ("d28", (V28, Some(QWORD))),
            ("d29", (V29, Some(QWORD))),
            ("d30", (V30, Some(QWORD))),
            ("d31", (V31, Some(QWORD))),

            ("q0" , (V0 , Some(OWORD))),
            ("q1" , (V1 , Some(OWORD))),
            ("q2" , (V2 , Some(OWORD))),
            ("q3" , (V3 , Some(OWORD))),
            ("q4" , (V4 , Some(OWORD))),
            ("q5" , (V5 , Some(OWORD))),
            ("q6" , (V6 , Some(OWORD))),
            ("q7" , (V7 , Some(OWORD))),
            ("q8" , (V8 , Some(OWORD))),
            ("q9" , (V9 , Some(OWORD))),
            ("q10", (V10, Some(OWORD))),
            ("q11", (V11, Some(OWORD))),
            ("q12", (V12, Some(OWORD))),
            ("q13", (V13, Some(OWORD))),
            ("q14", (V14, Some(OWORD))),
            ("q15", (V15, Some(OWORD))),
            ("q16", (V16, Some(OWORD))),
            ("q17", (V17, Some(OWORD))),
            ("q18", (V18, Some(OWORD))),
            ("q19", (V19, Some(OWORD))),
            ("q20", (V20, Some(OWORD))),
            ("q21", (V21, Some(OWORD))),
            ("q22", (V22, Some(OWORD))),
            ("q23", (V23, Some(OWORD))),
            ("q24", (V24, Some(OWORD))),
            ("q25", (V25, Some(OWORD))),
            ("q26", (V26, Some(OWORD))),
            ("q27", (V27, Some(OWORD))),
            ("q28", (V28, Some(OWORD))),
            ("q29", (V29, Some(OWORD))),
            ("q30", (V30, Some(OWORD))),
            ("q31", (V31, Some(OWORD))),

            ("v0" , (V0 , None)),
            ("v1" , (V1 , None)),
            ("v2" , (V2 , None)),
            ("v3" , (V3 , None)),
            ("v4" , (V4 , None)),
            ("v5" , (V5 , None)),
            ("v6" , (V6 , None)),
            ("v7" , (V7 , None)),
            ("v8" , (V8 , None)),
            ("v9" , (V9 , None)),
            ("v10", (V10, None)),
            ("v11", (V11, None)),
            ("v12", (V12, None)),
            ("v13", (V13, None)),
            ("v14", (V14, None)),
            ("v15", (V15, None)),
            ("v16", (V16, None)),
            ("v17", (V17, None)),
            ("v18", (V18, None)),
            ("v19", (V19, None)),
            ("v20", (V20, None)),
            ("v21", (V21, None)),
            ("v22", (V22, None)),
            ("v23", (V23, None)),
            ("v24", (V24, None)),
            ("v25", (V25, None)),
            ("v26", (V26, None)),
            ("v27", (V27, None)),
            ("v28", (V28, None)),
            ("v29", (V29, None)),
            ("v30", (V30, None)),
            ("v31", (V31, None)),
        ];
        MAP.iter().cloned().collect()
    };

    static ref AARCH64_FAMILIES: HashMap<&'static str, (RegFamily, Option<Size>)> = {
        static MAP: &[(&str, (RegFamily, Option<Size>))] = &[
            ("X",   (RegFamily::INTEGER,   Some(Size::QWORD))),
            ("W",   (RegFamily::INTEGER,   Some(Size::DWORD))),
            ("XSP", (RegFamily::INTEGERSP, Some(Size::QWORD))),
            ("WSP", (RegFamily::INTEGERSP, Some(Size::DWORD))),

            ("B", (RegFamily::SIMD, Some(Size::BYTE))),
            ("H", (RegFamily::SIMD, Some(Size::WORD))),
            ("S", (RegFamily::SIMD, Some(Size::DWORD))),
            ("D", (RegFamily::SIMD, Some(Size::QWORD))),
            ("Q", (RegFamily::SIMD, Some(Size::OWORD))),

            ("V", (RegFamily::SIMD, None)),
        ];
        MAP.iter().cloned().collect()
    };
}
