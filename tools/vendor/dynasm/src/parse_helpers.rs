//! This file contains parsing helpers used by multiple parsing backends
use syn::parse;
use std::convert::TryInto;

/**
 * Jump types
 */

pub trait ParseOpt: Sized {
    fn parse(input: parse::ParseStream) -> parse::Result<Option<Self>>;
}

pub trait ParseOptExt {
    /// Parses a syntax tree node of type `T`, advancing the position of our
    /// parse stream past it if it was found.
    fn parse_opt<T: ParseOpt>(&self) -> parse::Result<Option<T>>;
}

impl<'a> ParseOptExt for parse::ParseBuffer<'a> {
    fn parse_opt<T: ParseOpt>(&self) -> parse::Result<Option<T>> {
        T::parse(self)
    }
}

/// Tries to parse an ident that has a specific name as a keyword. Returns true if it worked.
pub fn eat_pseudo_keyword(input: parse::ParseStream, kw: &str) -> bool {
    input.step(|cursor| {
        if let Some((ident, rest)) = cursor.ident() {
            if ident == kw {
                return Ok(((), rest));
            }
        }
        Err(cursor.error("expected identifier"))
    }).is_ok()
}

/// parses an ident, but instead of syn's Parse impl it does also parse keywords as idents
pub fn parse_ident_or_rust_keyword(input: parse::ParseStream) -> parse::Result<syn::Ident> {
    input.step(|cursor| {
        if let Some((ident, rest)) = cursor.ident() {
            return Ok((ident, rest));
        }
        Err(cursor.error("expected identifier"))
    })
}

/// checks if an expression is simply an ident, and if so, returns a clone of it.
pub fn as_ident(expr: &syn::Expr) -> Option<&syn::Ident> {
    let path = match *expr {
        syn::Expr::Path(syn::ExprPath {ref path, qself: None, ..}) => path,
        _ => return None
    };

    if path.leading_colon.is_some() || path.segments.len() != 1 {
        return None;
    }

    let segment = &path.segments[0];
    if segment.arguments != syn::PathArguments::None {
        return None;
    }

    Some(&segment.ident)
}

/// checks if an expression is a simple literal, allowing us to perform compile-time analysis of an expression
pub fn as_lit(expr: &syn::Expr) -> Option<&syn::Lit> {
    // strip any wrapping Group nodes due to delimiting
    let mut inner = expr;
    while let syn::Expr::Group(syn::ExprGroup { expr, .. }) = inner {
        inner = expr;
    }

    match inner {
        syn::Expr::Lit(syn::ExprLit { ref lit, .. } ) => Some(lit),
        _ => None
    }
}

/// checks if an expression is a literal with possible negation
pub fn as_lit_with_negation(expr: &syn::Expr) -> Option<(&syn::Lit, bool)> {
    // strip any wrapping Group nodes due to delimiting
    let mut inner = expr;
    while let syn::Expr::Group(syn::ExprGroup { expr, .. }) = inner {
        inner = expr;
    }

    match inner {
        syn::Expr::Lit(syn::ExprLit { ref lit, .. } ) => Some((lit, false)),
        syn::Expr::Unary(syn::ExprUnary { op: syn::UnOp::Neg(_), ref expr, .. } ) => {
            match &**expr {
                syn::Expr::Lit(syn::ExprLit { ref lit, .. } ) => Some((lit, true)),
                _ => None
            }
        }
        _ => None
    }
}

/// checks if an expression is a constant number literal
pub fn as_number(expr: &syn::Expr) -> Option<u64> {
    match as_lit(expr)?  {
        syn::Lit::Int(i) => i.base10_parse().ok(),
        _ => None
    }
}

/// checks if an expression is a signed number literal
pub fn as_signed_number(expr: &syn::Expr) -> Option<i64> {
    let (expr, negated) = as_lit_with_negation(expr)?;
    match expr {
        syn::Lit::Int(i) => if let Ok(value) = i.base10_parse::<u64>() {
            let value: i64 = value.try_into().ok()?;
            Some (if negated {-value} else {value})
        } else {
            None
        },
        _ => None
    }
}

/// checks if an expression is a constant float literal
pub fn as_float(expr: &syn::Expr) -> Option<f64> {
    let (expr, negated) = as_lit_with_negation(expr)?;
    match expr {
        syn::Lit::Float(i) => i.base10_parse::<f64>().ok().map(|i| if negated { -i } else { i } ),
        _ => None
    }
}
