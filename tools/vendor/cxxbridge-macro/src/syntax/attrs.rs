use crate::syntax::cfg::CfgExpr;
use crate::syntax::namespace::Namespace;
use crate::syntax::report::Errors;
use crate::syntax::Atom::{self, *};
use crate::syntax::{cfg, Derive, Doc, ForeignName};
use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use syn::parse::{Nothing, Parse, ParseStream, Parser as _};
use syn::{parenthesized, token, Attribute, Error, LitStr, Path, Result, Token};

// Intended usage:
//
//     let mut doc = Doc::new();
//     let mut cxx_name = None;
//     let mut rust_name = None;
//     /* ... */
//     let attrs = attrs::parse(
//         cx,
//         item.attrs,
//         attrs::Parser {
//             doc: Some(&mut doc),
//             cxx_name: Some(&mut cxx_name),
//             rust_name: Some(&mut rust_name),
//             /* ... */
//             ..Default::default()
//         },
//     );
//
#[derive(Default)]
pub struct Parser<'a> {
    pub cfg: Option<&'a mut CfgExpr>,
    pub doc: Option<&'a mut Doc>,
    pub derives: Option<&'a mut Vec<Derive>>,
    pub repr: Option<&'a mut Option<Atom>>,
    pub namespace: Option<&'a mut Namespace>,
    pub cxx_name: Option<&'a mut Option<ForeignName>>,
    pub rust_name: Option<&'a mut Option<Ident>>,
    pub variants_from_header: Option<&'a mut Option<Attribute>>,
    pub ignore_unrecognized: bool,

    // Suppress clippy needless_update lint ("struct update has no effect, all
    // the fields in the struct have already been specified") when preemptively
    // writing `..Default::default()`.
    pub(crate) _more: (),
}

pub fn parse(cx: &mut Errors, attrs: Vec<Attribute>, mut parser: Parser) -> OtherAttrs {
    let mut passthrough_attrs = Vec::new();
    for attr in attrs {
        if attr.path.is_ident("doc") {
            match parse_doc_attribute.parse2(attr.tokens.clone()) {
                Ok(attr) => {
                    if let Some(doc) = &mut parser.doc {
                        match attr {
                            DocAttribute::Doc(lit) => doc.push(lit),
                            DocAttribute::Hidden => doc.hidden = true,
                        }
                        continue;
                    }
                }
                Err(err) => {
                    cx.push(err);
                    break;
                }
            }
        } else if attr.path.is_ident("derive") {
            match attr.parse_args_with(|attr: ParseStream| parse_derive_attribute(cx, attr)) {
                Ok(attr) => {
                    if let Some(derives) = &mut parser.derives {
                        derives.extend(attr);
                        continue;
                    }
                }
                Err(err) => {
                    cx.push(err);
                    break;
                }
            }
        } else if attr.path.is_ident("repr") {
            match attr.parse_args_with(parse_repr_attribute) {
                Ok(attr) => {
                    if let Some(repr) = &mut parser.repr {
                        **repr = Some(attr);
                        continue;
                    }
                }
                Err(err) => {
                    cx.push(err);
                    break;
                }
            }
        } else if attr.path.is_ident("namespace") {
            match parse_namespace_attribute.parse2(attr.tokens.clone()) {
                Ok(attr) => {
                    if let Some(namespace) = &mut parser.namespace {
                        **namespace = attr;
                        continue;
                    }
                }
                Err(err) => {
                    cx.push(err);
                    break;
                }
            }
        } else if attr.path.is_ident("cxx_name") {
            match parse_cxx_name_attribute.parse2(attr.tokens.clone()) {
                Ok(attr) => {
                    if let Some(cxx_name) = &mut parser.cxx_name {
                        **cxx_name = Some(attr);
                        continue;
                    }
                }
                Err(err) => {
                    cx.push(err);
                    break;
                }
            }
        } else if attr.path.is_ident("rust_name") {
            match parse_rust_name_attribute.parse2(attr.tokens.clone()) {
                Ok(attr) => {
                    if let Some(rust_name) = &mut parser.rust_name {
                        **rust_name = Some(attr);
                        continue;
                    }
                }
                Err(err) => {
                    cx.push(err);
                    break;
                }
            }
        } else if attr.path.is_ident("cfg") {
            match cfg::parse_attribute.parse2(attr.tokens.clone()) {
                Ok(cfg_expr) => {
                    if let Some(cfg) = &mut parser.cfg {
                        cfg.merge(cfg_expr);
                        passthrough_attrs.push(attr);
                        continue;
                    }
                }
                Err(err) => {
                    cx.push(err);
                    break;
                }
            }
        } else if attr.path.is_ident("variants_from_header")
            && cfg!(feature = "experimental-enum-variants-from-header")
        {
            if let Err(err) = Nothing::parse.parse2(attr.tokens.clone()) {
                cx.push(err);
            }
            if let Some(variants_from_header) = &mut parser.variants_from_header {
                **variants_from_header = Some(attr);
                continue;
            }
        } else if attr.path.is_ident("allow")
            || attr.path.is_ident("warn")
            || attr.path.is_ident("deny")
            || attr.path.is_ident("forbid")
            || attr.path.is_ident("deprecated")
            || attr.path.is_ident("must_use")
        {
            // https://doc.rust-lang.org/reference/attributes/diagnostics.html
            passthrough_attrs.push(attr);
            continue;
        } else if attr.path.is_ident("serde") {
            passthrough_attrs.push(attr);
            continue;
        } else if attr.path.segments.len() > 1 {
            let tool = &attr.path.segments.first().unwrap().ident;
            if tool == "rustfmt" {
                // Skip, rustfmt only needs to find it in the pre-expansion source file.
                continue;
            } else if tool == "clippy" {
                passthrough_attrs.push(attr);
                continue;
            }
        }
        if !parser.ignore_unrecognized {
            cx.error(attr, "unsupported attribute");
            break;
        }
    }
    OtherAttrs(passthrough_attrs)
}

enum DocAttribute {
    Doc(LitStr),
    Hidden,
}

mod kw {
    syn::custom_keyword!(hidden);
}

fn parse_doc_attribute(input: ParseStream) -> Result<DocAttribute> {
    let lookahead = input.lookahead1();
    if lookahead.peek(Token![=]) {
        input.parse::<Token![=]>()?;
        let lit: LitStr = input.parse()?;
        Ok(DocAttribute::Doc(lit))
    } else if lookahead.peek(token::Paren) {
        let content;
        parenthesized!(content in input);
        content.parse::<kw::hidden>()?;
        Ok(DocAttribute::Hidden)
    } else {
        Err(lookahead.error())
    }
}

fn parse_derive_attribute(cx: &mut Errors, input: ParseStream) -> Result<Vec<Derive>> {
    let paths = input.parse_terminated::<Path, Token![,]>(Path::parse_mod_style)?;

    let mut derives = Vec::new();
    for path in paths {
        if let Some(ident) = path.get_ident() {
            if let Some(derive) = Derive::from(ident) {
                derives.push(derive);
                continue;
            }
        }
        cx.error(path, "unsupported derive");
    }
    Ok(derives)
}

fn parse_repr_attribute(input: ParseStream) -> Result<Atom> {
    let begin = input.cursor();
    let ident: Ident = input.parse()?;
    if let Some(atom) = Atom::from(&ident) {
        match atom {
            U8 | U16 | U32 | U64 | Usize | I8 | I16 | I32 | I64 | Isize if input.is_empty() => {
                return Ok(atom);
            }
            _ => {}
        }
    }
    Err(Error::new_spanned(
        begin.token_stream(),
        "unrecognized repr",
    ))
}

fn parse_namespace_attribute(input: ParseStream) -> Result<Namespace> {
    input.parse::<Token![=]>()?;
    let namespace = input.parse::<Namespace>()?;
    Ok(namespace)
}

fn parse_cxx_name_attribute(input: ParseStream) -> Result<ForeignName> {
    input.parse::<Token![=]>()?;
    if input.peek(LitStr) {
        let lit: LitStr = input.parse()?;
        ForeignName::parse(&lit.value(), lit.span())
    } else {
        let ident: Ident = input.parse()?;
        ForeignName::parse(&ident.to_string(), ident.span())
    }
}

fn parse_rust_name_attribute(input: ParseStream) -> Result<Ident> {
    input.parse::<Token![=]>()?;
    if input.peek(LitStr) {
        let lit: LitStr = input.parse()?;
        lit.parse()
    } else {
        input.parse()
    }
}

#[derive(Clone)]
pub struct OtherAttrs(Vec<Attribute>);

impl OtherAttrs {
    pub fn none() -> Self {
        OtherAttrs(Vec::new())
    }

    pub fn extend(&mut self, other: Self) {
        self.0.extend(other.0);
    }
}

impl ToTokens for OtherAttrs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for attr in &self.0 {
            let Attribute {
                pound_token,
                style,
                bracket_token,
                path,
                tokens: attr_tokens,
            } = attr;
            pound_token.to_tokens(tokens);
            let _ = style; // ignore; render outer and inner attrs both as outer
            bracket_token.surround(tokens, |tokens| {
                path.to_tokens(tokens);
                attr_tokens.to_tokens(tokens);
            });
        }
    }
}
