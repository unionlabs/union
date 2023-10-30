use syn;
use syn::parse;
use syn::spanned::Spanned;
use proc_macro2::{Span, TokenStream, TokenTree, Literal};
use quote::{quote, quote_spanned, ToTokens};

use byteorder::{ByteOrder, LittleEndian};

use crate::common::{Size, Stmt, delimited, Relocation};

use std::convert::TryInto;


/// Converts a sequence of abstract Statements to actual tokens
pub fn serialize(name: &TokenTree, stmts: Vec<Stmt>) -> TokenStream {
    // first, try to fold constants into a byte stream
    let mut folded_stmts = Vec::new();
    let mut const_buffer = Vec::new();
    for stmt in stmts {
        match stmt {
            Stmt::Const(value, size) => {
                match size {
                    Size::BYTE => const_buffer.push(value as u8),
                    Size::WORD => {
                        let mut buffer = [0u8; 2];
                        LittleEndian::write_u16(&mut buffer, value as u16);
                        const_buffer.extend(&buffer);
                    },
                    Size::DWORD => {
                        let mut buffer = [0u8; 4];
                        LittleEndian::write_u32(&mut buffer, value as u32);
                        const_buffer.extend(&buffer);
                    },
                    Size::QWORD => {
                        let mut buffer = [0u8; 8];
                        LittleEndian::write_u64(&mut buffer, value as u64);
                        const_buffer.extend(&buffer);
                    },
                    _ => unimplemented!()
                }
            },
            Stmt::Extend(data) => {
                const_buffer.extend(data);
            },
            s => {
                // empty the const buffer
                if !const_buffer.is_empty() {
                    folded_stmts.push(Stmt::Extend(const_buffer));
                    const_buffer = Vec::new();
                }
                folded_stmts.push(s);
            }
        }
        while const_buffer.len() > 32 {
            let new_buffer = const_buffer.split_off(32);
            folded_stmts.push(Stmt::Extend(const_buffer));
            const_buffer = new_buffer;
        }
    }
    if !const_buffer.is_empty() {
        folded_stmts.push(Stmt::Extend(const_buffer));
    }

    // and now do the final output pass in one go
    let mut output = TokenStream::new();

    for stmt in folded_stmts {
        let (method, args) = match stmt {
            Stmt::Const(_, _) => unreachable!(),
            Stmt::ExprUnsigned(expr, Size::BYTE)  => ("push",     vec![expr]),
            Stmt::ExprUnsigned(expr, Size::WORD)  => ("push_u16", vec![expr]),
            Stmt::ExprUnsigned(expr, Size::DWORD) => ("push_u32", vec![expr]),
            Stmt::ExprUnsigned(expr, Size::QWORD) => ("push_u64", vec![expr]),
            Stmt::ExprUnsigned(_, _) => unimplemented!(),
            Stmt::ExprSigned(  expr, Size::BYTE)  => ("push_i8",  vec![expr]),
            Stmt::ExprSigned(  expr, Size::WORD)  => ("push_i16", vec![expr]),
            Stmt::ExprSigned(  expr, Size::DWORD) => ("push_i32", vec![expr]),
            Stmt::ExprSigned(  expr, Size::QWORD) => ("push_i64", vec![expr]),
            Stmt::ExprSigned(_, _) => unimplemented!(),
            Stmt::Extend(data)     => ("extend", vec![Literal::byte_string(&data).into()]),
            Stmt::ExprExtend(expr) => ("extend", vec![expr]),
            Stmt::Align(expr, with)      => ("align", vec![expr, with]),
            Stmt::GlobalLabel(n) => ("global_label", vec![expr_string_from_ident(&n)]),
            Stmt::LocalLabel(n)  => ("local_label", vec![expr_string_from_ident(&n)]),
            Stmt::DynamicLabel(expr) => ("dynamic_label", vec![expr]),
            Stmt::GlobalJumpTarget(n, Relocation { target_offset, field_offset, ref_offset, kind }) => 
                ("global_reloc"  , vec![expr_string_from_ident(&n), target_offset, Literal::u8_suffixed(field_offset).into(), Literal::u8_suffixed(ref_offset).into(), kind]),
            Stmt::ForwardJumpTarget(n, Relocation { target_offset, field_offset, ref_offset, kind }) =>
                ("forward_reloc" , vec![expr_string_from_ident(&n), target_offset, Literal::u8_suffixed(field_offset).into(), Literal::u8_suffixed(ref_offset).into(), kind]),
            Stmt::BackwardJumpTarget(n, Relocation { target_offset, field_offset, ref_offset, kind }) =>
                ("backward_reloc", vec![expr_string_from_ident(&n), target_offset, Literal::u8_suffixed(field_offset).into(), Literal::u8_suffixed(ref_offset).into(), kind]),
            Stmt::DynamicJumpTarget(expr, Relocation { target_offset, field_offset, ref_offset, kind }) =>
                ("dynamic_reloc" , vec![expr, target_offset, Literal::u8_suffixed(field_offset).into(), Literal::u8_suffixed(ref_offset).into(), kind]),
            Stmt::BareJumpTarget(expr, Relocation { field_offset, ref_offset, kind, .. })    =>
                ("bare_reloc"    , vec![expr, Literal::u8_suffixed(field_offset).into(), Literal::u8_suffixed(ref_offset).into(), kind]),
            Stmt::PrefixStmt(s)
            | Stmt::Stmt(s) => {
                output.extend(quote! {
                    #s ;
                });
                continue;
            }
        };

        // and construct the appropriate method call
        let method = syn::Ident::new(method, Span::mixed_site());
        output.extend(quote! {
            #name . #method ( #( #args ),* ) ;
        })
    }

    // if we have nothing to emit, expand to nothing. Else, wrap it into a block.
    if output.is_empty() {
        output
    } else {
        quote!{
            {
                #output
            }
        }
    }
}

/// Inverts the order of a sequence of Statements, reordering relocations as required
pub fn invert(stmts: Vec<Stmt>) -> Vec<Stmt> {
    // create the output buffer, and iterate through the stmts buffer in reverse
    let mut reversed = Vec::new();

    // vector to store relocation stmts in while we deal with them
    let mut relocation_buf = Vec::new();
    let mut counter = 0usize;

    let mut iter = stmts.into_iter().rev().peekable();

    while let Some(stmt) = iter.next() {
        // if we find a relocation, note it down together with the current counter value and the value at which it can be safely emitted
        match stmt {
            Stmt::GlobalJumpTarget(_, Relocation { field_offset, ref_offset, .. } )
            | Stmt::ForwardJumpTarget(_, Relocation { field_offset, ref_offset, .. } )
            | Stmt::BackwardJumpTarget(_, Relocation { field_offset, ref_offset, .. } )
            | Stmt::DynamicJumpTarget(_, Relocation { field_offset, ref_offset, .. } )
            | Stmt::BareJumpTarget(_, Relocation { field_offset, ref_offset, .. } ) => {
                let trigger = counter + std::cmp::max(field_offset, ref_offset) as usize;
                relocation_buf.push((trigger, counter, stmt));
                continue;
            },
            _ => ()
        };

        while let Some(Stmt::PrefixStmt(_)) = iter.peek() {
            // ensure prefix statements still end up before their following emitting statement
            let s = iter.next().unwrap();
            let i = reversed.len() - 1;
            reversed.insert(i, s);
        }

        // otherwise, calculate the size of the current statement and add that to the counter
        let size = match &stmt {
            Stmt::Const(_, size)
            | Stmt::ExprUnsigned(_, size)
            | Stmt::ExprSigned(_, size) => size.in_bytes() as usize,
            Stmt::Extend(buf) => buf.len(),
            Stmt::ExprExtend(_)
            | Stmt::Align(_, _) => {
                assert!(relocation_buf.is_empty(), "Tried to hoist relocation over unknown size");
                0
            },
            Stmt::GlobalLabel(_)
            | Stmt::LocalLabel(_)
            | Stmt::DynamicLabel(_)
            | Stmt::GlobalJumpTarget(_, _)
            | Stmt::ForwardJumpTarget(_, _)
            | Stmt::BackwardJumpTarget(_, _)
            | Stmt::DynamicJumpTarget(_, _)
            | Stmt::BareJumpTarget(_, _)
            | Stmt::PrefixStmt(_)
            | Stmt::Stmt(_) => 0,
        };

        counter += size;
        reversed.push(stmt);

        // check if we can emit any collected relocations safely. Slightly overcomplicated as drain_filter ain't stable yet.
        let mut new_relocation_buf = Vec::new();
        for (trigger, orig_counter, mut stmt) in relocation_buf {
            if counter < trigger {
                new_relocation_buf.push((trigger, orig_counter, stmt));
                continue;
            }

            // apply the fixups and emit
            let change: u8 = (counter - orig_counter).try_into().expect("Tried to hoist a relocation by over 255 bytes");
            match &mut stmt {
                Stmt::GlobalJumpTarget(_, Relocation { field_offset, ref_offset, .. } )
                | Stmt::ForwardJumpTarget(_, Relocation { field_offset, ref_offset, .. } )
                | Stmt::BackwardJumpTarget(_, Relocation { field_offset, ref_offset, .. } )
                | Stmt::DynamicJumpTarget(_, Relocation { field_offset, ref_offset, .. } )
                | Stmt::BareJumpTarget(_, Relocation { field_offset, ref_offset, .. } ) => {
                    *field_offset = change - *field_offset;
                    *ref_offset = change - *ref_offset;
                },
                _ => unreachable!()
            }
            reversed.push(stmt);
        }
        relocation_buf = new_relocation_buf;
    }

    reversed
}

// below here are all kinds of utility functions to quickly generate TokenTree constructs
// this collection is arbitrary and purely based on what special things are needed for assembler
// codegen implementations


// expression of value 0. sometimes needed.
pub fn expr_zero() -> TokenTree {
    proc_macro2::Literal::u8_unsuffixed(0).into()
}

// given an ident, makes it into a "string"
pub fn expr_string_from_ident(i: &syn::Ident) -> TokenTree {
    let name = i.to_string();
    proc_macro2::Literal::string(&name).into()
}

// Makes a dynamic scale expression. Useful for x64 generic addressing mode
pub fn expr_dynscale(scale: &TokenTree, rest: &TokenTree) -> (TokenTree, TokenTree) {
    let tempval = expr_encode_x64_sib_scale(&scale);
    (delimited(quote_spanned! { Span::mixed_site()=>
        let temp = #tempval
    }), delimited(quote_spanned! { Span::mixed_site()=>
         #rest | ((temp & 3) << 6)
    }))
}

// makes (a, b)
pub fn expr_tuple_of_u8s(span: Span, data: &[u8]) -> TokenTree {
    delimited(if data.len() == 1 {
        let data = data[0];
        quote_spanned! {span=>
            (#data,)
        }
    } else {
        quote_spanned! {span=>
            (#(#data),*)
        }
    })
}

// makes sum(exprs)
pub fn expr_add_many<T: Iterator<Item=TokenTree>>(span: Span, mut exprs: T) -> Option<TokenTree> {
    let first_expr = exprs.next()?;

    let tokens = quote_spanned!{ span=>
        #first_expr #( + #exprs )*
    };

    Some(delimited(tokens))
}

// makes (size_of<ty>() * value)
pub fn expr_size_of_scale(ty: &syn::Path, value: &TokenTree, size: Size) -> TokenTree {
    let span = value.span();
    let size = size.as_literal();

    delimited(quote_spanned! { span=>
        (::std::mem::size_of::<#ty>() as #size) * #value
    })
}

/// returns orig | ((expr & mask) << shift)
pub fn expr_mask_shift_or(orig: &TokenTree, expr: &TokenTree, mask: u64, shift: i8) -> TokenTree {
    let span = expr.span();

    let mask: TokenTree = proc_macro2::Literal::u64_unsuffixed(mask).into();

    delimited(if shift >= 0 {
        let shift: TokenTree = proc_macro2::Literal::i8_unsuffixed(shift).into();
        quote_spanned! { span=>
            #orig | ((#expr & #mask) << #shift)
        }
    } else {
        let shift: TokenTree = proc_macro2::Literal::i8_unsuffixed(-shift).into();
        quote_spanned! { span=>
            #orig | ((#expr & #mask) >> #shift)
        }
    })
}


/// returns orig & !((expr & mask) << shift)
pub fn expr_mask_shift_inverted_and(orig: &TokenTree, expr: &TokenTree, mask: u64, shift: i8) -> TokenTree {
    let span = expr.span();

    let mask: TokenTree = proc_macro2::Literal::u64_unsuffixed(mask).into();

    delimited(if shift >= 0 {
        let shift: TokenTree = proc_macro2::Literal::i8_unsuffixed(shift).into();
        quote_spanned! { span=>
            #orig & !((#expr & #mask) << #shift)
        }
    } else {
        let shift: TokenTree = proc_macro2::Literal::i8_unsuffixed(-shift).into();
        quote_spanned! { span=>
            #orig & !((#expr & #mask) >> #shift)
        }
    })
}

/// returns (offset_of!(path, attr) as size)
pub fn expr_offset_of(path: &syn::Path, attr: &syn::Ident, size: Size) -> TokenTree {
    // generate a P<Expr> that resolves into the offset of an attribute to a type.
    // this is somewhat ridiculously complex because we can't expand macros here

    let span = path.span();
    let size = size.as_literal();

    delimited(quote_spanned! { span=>
        unsafe {
            let #path {#attr: _, ..};
            let temp = ::std::mem::MaybeUninit::<#path>::uninit();
            let rv = &(*temp.as_ptr()).#attr as *const _ as usize - temp.as_ptr() as usize;
            rv as #size
        }
    })
}

// returns std::mem::size_of<path>()
pub fn expr_size_of(path: &syn::Path) -> TokenTree {
    // generate a P<Expr> that returns the size of type at path
    let span = path.span();

    delimited(quote_spanned! { span=>
        ::std::mem::size_of::<#path>()
    })
}

// makes the following
// match size {
//    8 => 3,
//    4 => 2,
//    2 => 1,
//    1 => 0,
//    _ => panic!r("Type size not representable as scale")
//}
pub fn expr_encode_x64_sib_scale(size: &TokenTree) -> TokenTree {
    let span = size.span();

    delimited(quote_spanned! { span=>
        match #size {
            8 => 3,
            4 => 2,
            2 => 1,
            1 => 0,
            _ => panic!("Type size not representable as scale")
        }
    })
}

// Reparses a tokentree into an expression
pub fn reparse(tt: &TokenTree) -> parse::Result<syn::Expr> {
    syn::parse2(tt.into_token_stream())
}
