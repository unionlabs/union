//! This module contains various infrastructure that is common across all assembler backends
use proc_macro2::{Span, TokenTree};
use quote::ToTokens;
use quote::quote;
use syn::spanned::Spanned;
use syn::parse;
use syn::Token;

use crate::parse_helpers::{ParseOpt, eat_pseudo_keyword};
use crate::serialize;

/// Enum representing the result size of a value/expression/register/etc in bytes.
/// Uses the NASM syntax for sizes (a word is 16 bits)
#[derive(Debug, PartialOrd, PartialEq, Ord, Eq, Hash, Clone, Copy)]
pub enum Size {
    BYTE  = 1,
    WORD  = 2,
    DWORD = 4,
    FWORD = 6,
    QWORD = 8,
    PWORD = 10,
    OWORD = 16,
    HWORD = 32,
}

impl Size {
    pub fn in_bytes(self) -> u8 {
        self as u8
    }

    pub fn as_literal(self) -> syn::Ident {
        syn::Ident::new(match self {
            Size::BYTE  => "i8",
            Size::WORD  => "i16",
            Size::DWORD => "i32",
            Size::FWORD => "i48",
            Size::QWORD => "i64",
            Size::PWORD => "i80",
            Size::OWORD => "i128",
            Size::HWORD => "i256"
        }, Span::mixed_site())
    }
}


/**
 * Jump types
 */
#[derive(Debug, Clone)]
pub struct Jump {
    pub kind: JumpKind,
    pub offset: Option<syn::Expr>
}

#[derive(Debug, Clone)]
pub enum JumpKind {
    // note: these symbol choices try to avoid stuff that is a valid starting symbol for parse_expr
    // in order to allow the full range of expressions to be used. the only currently existing ambiguity is
    // with the symbol <, as this symbol is also the starting symbol for the universal calling syntax <Type as Trait>.method(args)
    Global(syn::Ident),   // -> label (["+" "-"] offset)?
    Backward(syn::Ident), //  > label (["+" "-"] offset)?
    Forward(syn::Ident),  //  < label (["+" "-"] offset)?
    Dynamic(syn::Expr),   // =>expr | => (expr) (["+" "-"] offset)?
    Bare(syn::Expr)       // jump to this address
}

impl ParseOpt for Jump {
    fn parse(input: parse::ParseStream) -> parse::Result<Option<Jump>> {
        // extern label
        if eat_pseudo_keyword(input, "extern") {
            let expr: syn::Expr = input.parse()?;

            return Ok(Some(Jump { kind: JumpKind::Bare(expr), offset: None }));
        }

        // -> global_label
        let kind = if input.peek(Token![->]) {
            let _: Token![->] = input.parse()?;
            let name: syn::Ident = input.parse()?;

            JumpKind::Global(name)

        // > forward_label
        } else if input.peek(Token![>]) {
            let _: Token![>] = input.parse()?;
            let name: syn::Ident = input.parse()?;

            JumpKind::Forward(name)

        // < backwards_label
        } else if input.peek(Token![<]) {
            let _: Token![<] = input.parse()?;
            let name: syn::Ident = input.parse()?;

            JumpKind::Backward(name)

        // => dynamic_label
        } else if input.peek(Token![=>]) {
            let _: Token![=>] = input.parse()?;

            let expr: syn::Expr = if input.peek(syn::token::Paren) {
                let inner;
                let _ = syn::parenthesized!(inner in input);
                let inner = &inner;

                inner.parse()?
            } else {
                input.parse()?
            };

            JumpKind::Dynamic(expr)

        // nothing
        } else {
            return Ok(None);
        };

        // parse optional offset
        let offset = if input.peek(Token![-]) || input.peek(Token![+]) {
            if input.peek(Token![+]) {
                let _: Token![+] = input.parse()?;
            }

            let expr: syn::Expr = input.parse()?;
            Some(expr)

        } else {
            None
        };

        Ok(Some(Jump::new(kind, offset)))
    }
}

impl Jump {
    pub fn new(kind: JumpKind, offset: Option<syn::Expr>) -> Jump {
        Jump {
            kind,
            offset
        }
    }

    /// Takes a jump and encodes it as a relocation starting `start_offset` bytes ago, relative to `ref_offset`.
    /// Any data detailing the type of relocation emitted should be contained in `data`, which is emitted as a tuple of u8's.
    pub fn encode(self, field_offset: u8, ref_offset: u8, data: &[u8]) -> Stmt {
        let span = self.span();

        let target_offset = delimited(if let Some(offset) = self.offset {
            quote!(#offset)
        } else {
            quote!(0isize)
        });

        // Create a relocation descriptor, containing all information about the actual jump except for the target itself.
        let relocation = Relocation {
            target_offset,
            field_offset,
            ref_offset,
            kind: serialize::expr_tuple_of_u8s(span, data)
        };
        match self.kind {
            JumpKind::Global(ident) => Stmt::GlobalJumpTarget(ident, relocation),
            JumpKind::Backward(ident) => Stmt::BackwardJumpTarget(ident, relocation),
            JumpKind::Forward(ident) => Stmt::ForwardJumpTarget(ident, relocation),
            JumpKind::Dynamic(expr) => Stmt::DynamicJumpTarget(delimited(expr), relocation),
            JumpKind::Bare(expr) => Stmt::BareJumpTarget(delimited(expr), relocation),
        }
    }

    pub fn span(&self) -> Span {
        match &self.kind {
            JumpKind::Global(ident) => ident.span(),
            JumpKind::Backward(ident) => ident.span(),
            JumpKind::Forward(ident) => ident.span(),
            JumpKind::Dynamic(expr) => expr.span(),
            JumpKind::Bare(expr) => expr.span(),
        }
    }
}


/// A relocation entry description
#[derive(Debug, Clone)]
pub struct Relocation {
    pub target_offset: TokenTree,
    pub field_offset: u8,
    pub ref_offset: u8,
    pub kind: TokenTree,
}


/// An abstract representation of a dynasm runtime statement to be emitted
#[derive(Debug, Clone)]
pub enum Stmt {
    // simply push data into the instruction stream. unsigned
    Const(u64, Size),
    // push data that is stored inside of an expression. unsigned
    ExprUnsigned(TokenTree, Size),
    // push signed data into the instruction stream. signed
    ExprSigned(TokenTree, Size),

    // extend the instruction stream with unsigned bytes
    Extend(Vec<u8>),
    // extend the instruction stream with unsigned bytes
    ExprExtend(TokenTree),
    // align the instruction stream to some alignment
    Align(TokenTree, TokenTree),

    // label declarations
    GlobalLabel(syn::Ident),
    LocalLabel(syn::Ident),
    DynamicLabel(TokenTree),

    // and their respective relocations (as expressions as they differ per assembler).
    GlobalJumpTarget(syn::Ident, Relocation),
    ForwardJumpTarget(syn::Ident, Relocation),
    BackwardJumpTarget(syn::Ident, Relocation),
    DynamicJumpTarget(TokenTree, Relocation),
    BareJumpTarget(TokenTree, Relocation),

    // a statement that provides some information for the next statement,
    // and should therefore not be reordered with it
    PrefixStmt(TokenTree),

    // a random statement that has to be inserted between assembly hunks
    Stmt(TokenTree)
}

// convenience methods
impl Stmt {
    #![allow(dead_code)]

    pub fn u8(value: u8) -> Stmt {
        Stmt::Const(u64::from(value), Size::BYTE)
    }

    pub fn u16(value: u16) -> Stmt {
        Stmt::Const(u64::from(value), Size::WORD)
    }

    pub fn u32(value: u32) -> Stmt {
        Stmt::Const(u64::from(value), Size::DWORD)
    }

    pub fn u64(value: u64) -> Stmt {
        Stmt::Const(value, Size::QWORD)
    }
}


// Makes a None-delimited TokenTree item out of anything that can be converted to tokens.
// This is a useful shortcut to escape issues around not-properly delimited tokenstreams
// because it is guaranteed to be parsed back properly to its source ast at type-level.
pub fn delimited<T: ToTokens>(expr: T) -> TokenTree {
    let span = expr.span();
    let mut group = proc_macro2::Group::new(
        proc_macro2::Delimiter::None, expr.into_token_stream()
    );
    group.set_span(span);
    proc_macro2::TokenTree::Group(group)
}


/// Create a bitmask with `scale` bits set
pub fn bitmask(scale: u8) -> u32 {
    1u32.checked_shl(u32::from(scale)).unwrap_or(0).wrapping_sub(1)
}


/// Create a bitmask with `scale` bits set
pub fn bitmask64(scale: u8) -> u64 {
    1u64.checked_shl(u32::from(scale)).unwrap_or(0).wrapping_sub(1)
}

