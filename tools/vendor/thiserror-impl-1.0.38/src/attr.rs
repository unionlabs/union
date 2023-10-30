use proc_macro2::{Delimiter, Group, Span, TokenStream, TokenTree};
use quote::{format_ident, quote, ToTokens};
use std::collections::BTreeSet as Set;
use std::iter::FromIterator;
use syn::parse::{Nothing, ParseStream};
use syn::{
    braced, bracketed, parenthesized, token, Attribute, Error, Ident, Index, LitInt, LitStr,
    Result, Token,
};

pub struct Attrs<'a> {
    pub display: Option<Display<'a>>,
    pub source: Option<&'a Attribute>,
    pub backtrace: Option<&'a Attribute>,
    pub from: Option<&'a Attribute>,
    pub transparent: Option<Transparent<'a>>,
}

#[derive(Clone)]
pub struct Display<'a> {
    pub original: &'a Attribute,
    pub fmt: LitStr,
    pub args: TokenStream,
    pub has_bonus_display: bool,
    pub implied_bounds: Set<(usize, Trait)>,
}

#[derive(Copy, Clone)]
pub struct Transparent<'a> {
    pub original: &'a Attribute,
    pub span: Span,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Trait {
    Debug,
    Display,
    Octal,
    LowerHex,
    UpperHex,
    Pointer,
    Binary,
    LowerExp,
    UpperExp,
}

pub fn get(input: &[Attribute]) -> Result<Attrs> {
    let mut attrs = Attrs {
        display: None,
        source: None,
        backtrace: None,
        from: None,
        transparent: None,
    };

    for attr in input {
        if attr.path.is_ident("error") {
            parse_error_attribute(&mut attrs, attr)?;
        } else if attr.path.is_ident("source") {
            require_empty_attribute(attr)?;
            if attrs.source.is_some() {
                return Err(Error::new_spanned(attr, "duplicate #[source] attribute"));
            }
            attrs.source = Some(attr);
        } else if attr.path.is_ident("backtrace") {
            require_empty_attribute(attr)?;
            if attrs.backtrace.is_some() {
                return Err(Error::new_spanned(attr, "duplicate #[backtrace] attribute"));
            }
            attrs.backtrace = Some(attr);
        } else if attr.path.is_ident("from") {
            if !attr.tokens.is_empty() {
                // Assume this is meant for derive_more crate or something.
                continue;
            }
            if attrs.from.is_some() {
                return Err(Error::new_spanned(attr, "duplicate #[from] attribute"));
            }
            attrs.from = Some(attr);
        }
    }

    Ok(attrs)
}

fn parse_error_attribute<'a>(attrs: &mut Attrs<'a>, attr: &'a Attribute) -> Result<()> {
    syn::custom_keyword!(transparent);

    attr.parse_args_with(|input: ParseStream| {
        if let Some(kw) = input.parse::<Option<transparent>>()? {
            if attrs.transparent.is_some() {
                return Err(Error::new_spanned(
                    attr,
                    "duplicate #[error(transparent)] attribute",
                ));
            }
            attrs.transparent = Some(Transparent {
                original: attr,
                span: kw.span,
            });
            return Ok(());
        }

        let display = Display {
            original: attr,
            fmt: input.parse()?,
            args: parse_token_expr(input, false)?,
            has_bonus_display: false,
            implied_bounds: Set::new(),
        };
        if attrs.display.is_some() {
            return Err(Error::new_spanned(
                attr,
                "only one #[error(...)] attribute is allowed",
            ));
        }
        attrs.display = Some(display);
        Ok(())
    })
}

fn parse_token_expr(input: ParseStream, mut begin_expr: bool) -> Result<TokenStream> {
    let mut tokens = Vec::new();
    while !input.is_empty() {
        if begin_expr && input.peek(Token![.]) {
            if input.peek2(Ident) {
                input.parse::<Token![.]>()?;
                begin_expr = false;
                continue;
            }
            if input.peek2(LitInt) {
                input.parse::<Token![.]>()?;
                let int: Index = input.parse()?;
                let ident = format_ident!("_{}", int.index, span = int.span);
                tokens.push(TokenTree::Ident(ident));
                begin_expr = false;
                continue;
            }
        }

        begin_expr = input.peek(Token![break])
            || input.peek(Token![continue])
            || input.peek(Token![if])
            || input.peek(Token![in])
            || input.peek(Token![match])
            || input.peek(Token![mut])
            || input.peek(Token![return])
            || input.peek(Token![while])
            || input.peek(Token![+])
            || input.peek(Token![&])
            || input.peek(Token![!])
            || input.peek(Token![^])
            || input.peek(Token![,])
            || input.peek(Token![/])
            || input.peek(Token![=])
            || input.peek(Token![>])
            || input.peek(Token![<])
            || input.peek(Token![|])
            || input.peek(Token![%])
            || input.peek(Token![;])
            || input.peek(Token![*])
            || input.peek(Token![-]);

        let token: TokenTree = if input.peek(token::Paren) {
            let content;
            let delimiter = parenthesized!(content in input);
            let nested = parse_token_expr(&content, true)?;
            let mut group = Group::new(Delimiter::Parenthesis, nested);
            group.set_span(delimiter.span);
            TokenTree::Group(group)
        } else if input.peek(token::Brace) {
            let content;
            let delimiter = braced!(content in input);
            let nested = parse_token_expr(&content, true)?;
            let mut group = Group::new(Delimiter::Brace, nested);
            group.set_span(delimiter.span);
            TokenTree::Group(group)
        } else if input.peek(token::Bracket) {
            let content;
            let delimiter = bracketed!(content in input);
            let nested = parse_token_expr(&content, true)?;
            let mut group = Group::new(Delimiter::Bracket, nested);
            group.set_span(delimiter.span);
            TokenTree::Group(group)
        } else {
            input.parse()?
        };
        tokens.push(token);
    }
    Ok(TokenStream::from_iter(tokens))
}

fn require_empty_attribute(attr: &Attribute) -> Result<()> {
    syn::parse2::<Nothing>(attr.tokens.clone())?;
    Ok(())
}

impl ToTokens for Display<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let fmt = &self.fmt;
        let args = &self.args;
        tokens.extend(quote! {
            write!(__formatter, #fmt #args)
        });
    }
}

impl ToTokens for Trait {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let trait_name = format_ident!("{}", format!("{:?}", self));
        tokens.extend(quote!(std::fmt::#trait_name));
    }
}
