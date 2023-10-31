use std::collections::hash_map::Entry;

use syn::parse;
use syn::Token;
use quote::quote;
use proc_macro_error::emit_error;

use crate::common::{Stmt, Size, delimited};
use crate::arch;
use crate::DynasmContext;
use crate::parse_helpers::ParseOptExt;

pub(crate) fn evaluate_directive(invocation_context: &mut DynasmContext, stmts: &mut Vec<Stmt>, input: parse::ParseStream) -> parse::Result<()> {
    let directive: syn::Ident = input.parse()?;

    match directive.to_string().as_str() {
        // TODO: oword, qword, float, double, long double

        "arch" => {
            // ; .arch ident
            let arch: syn::Ident = input.parse()?;
            if let Some(a) = arch::from_str(arch.to_string().as_str()) {
                invocation_context.current_arch = a;
            } else {
                emit_error!(arch, "Unknown architecture '{}'", arch);
            }
        },
        "feature" => {
            // ; .feature ident ("," ident) *
            let mut features = Vec::new();
            let ident: syn::Ident = input.parse()?;
            features.push(ident);

            while input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
                let ident: syn::Ident = input.parse()?;
                features.push(ident);
            }

            // ;.feature none  cancels all features
            if features.len() == 1 && features[0] == "none" {
                features.pop();
            }
            invocation_context.current_arch.set_features(&features);
        },
        // ; .byte (expr ("," expr)*)?
        "byte"  => directive_const(invocation_context, stmts, input, Size::BYTE)?,
        "word"  => directive_const(invocation_context, stmts, input, Size::WORD)?,
        "dword" => directive_const(invocation_context, stmts, input, Size::DWORD)?,
        "qword" => directive_const(invocation_context, stmts, input, Size::QWORD)?,
        "bytes" => {
            // ; .bytes expr
            let iterator: syn::Expr = input.parse()?;
            stmts.push(Stmt::ExprExtend(delimited(iterator)));
        },
        "align" => {
            // ; .align expr ("," expr)
            // this might need to be architecture dependent
            let value: syn::Expr = input.parse()?;

            let with = if input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
                let with: syn::Expr = input.parse()?;
                delimited(with)
            } else {
                let with = invocation_context.current_arch.default_align();
                delimited(quote!(#with))
            };

            stmts.push(Stmt::Align(delimited(value), with));
        },
        "alias" => {
            // ; .alias ident, ident
            // consider changing this to ; .alias ident = ident next breaking change
            let alias = input.parse::<syn::Ident>()?;
            let _: Token![,] = input.parse()?;
            let reg = input.parse::<syn::Ident>()?;

            let alias_name = alias.to_string();

            match invocation_context.aliases.entry(alias_name) {
                Entry::Occupied(_) => {
                    emit_error!(alias, "Duplicate alias definition, alias '{}' was already defined", alias);
                },
                Entry::Vacant(v) => {
                    v.insert(reg.to_string());
                }
            }
        },
        d => {
            // unknown directive. skip ahead until we hit a ; so the parser can recover
            emit_error!(directive, "unknown directive '{}'", d);
            skip_until_semicolon(input);
        }
    }

    Ok(())
}

fn directive_const(invocation_context: &mut DynasmContext, stmts: &mut Vec<Stmt>, input: parse::ParseStream, size: Size) -> parse::Result<()> {
    // FIXME: this could be replaced by a Punctuated parser?
    // parse (expr (, expr)*)?

    if input.is_empty() || input.peek(Token![;]) {
        return Ok(())
    }

    if let Some(jump) = input.parse_opt()? {
        invocation_context.current_arch.handle_static_reloc(stmts, jump, size);
    } else {
        let expr: syn::Expr = input.parse()?;
        stmts.push(Stmt::ExprSigned(delimited(expr), size));
    }


    while input.peek(Token![,]) {
        let _: Token![,] = input.parse()?;

        if let Some(jump) = input.parse_opt()? {
            invocation_context.current_arch.handle_static_reloc(stmts, jump, size);
        } else {
            let expr: syn::Expr = input.parse()?;
            stmts.push(Stmt::ExprSigned(delimited(expr), size));
        }
    }

    Ok(())
}

/// In case a directive is unknown, try to skip up to the next ; and resume parsing.
fn skip_until_semicolon(input: parse::ParseStream) {
    let _ = input.step(|cursor| {
        let mut rest = *cursor;
        while let Some((tt, next)) = rest.token_tree() {
            match tt {
                ::proc_macro2::TokenTree::Punct(ref punct) if punct.as_char() == ';' => {
                    return Ok(((), rest));
                }
                _ => rest = next,
            }
        }
        Ok(((), rest))
    });
}
