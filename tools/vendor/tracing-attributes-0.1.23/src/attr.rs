use std::collections::HashSet;
use syn::{punctuated::Punctuated, Expr, Ident, LitInt, LitStr, Path, Token};

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::ext::IdentExt as _;
use syn::parse::{Parse, ParseStream};

#[derive(Clone, Default, Debug)]
pub(crate) struct InstrumentArgs {
    level: Option<Level>,
    pub(crate) name: Option<LitStr>,
    target: Option<LitStr>,
    pub(crate) parent: Option<Expr>,
    pub(crate) follows_from: Option<Expr>,
    pub(crate) skips: HashSet<Ident>,
    pub(crate) skip_all: bool,
    pub(crate) fields: Option<Fields>,
    pub(crate) err_mode: Option<FormatMode>,
    pub(crate) ret_mode: Option<FormatMode>,
    /// Errors describing any unrecognized parse inputs that we skipped.
    parse_warnings: Vec<syn::Error>,
}

impl InstrumentArgs {
    pub(crate) fn level(&self) -> impl ToTokens {
        fn is_level(lit: &LitInt, expected: u64) -> bool {
            match lit.base10_parse::<u64>() {
                Ok(value) => value == expected,
                Err(_) => false,
            }
        }

        match &self.level {
            Some(Level::Str(ref lit)) if lit.value().eq_ignore_ascii_case("trace") => {
                quote!(tracing::Level::TRACE)
            }
            Some(Level::Str(ref lit)) if lit.value().eq_ignore_ascii_case("debug") => {
                quote!(tracing::Level::DEBUG)
            }
            Some(Level::Str(ref lit)) if lit.value().eq_ignore_ascii_case("info") => {
                quote!(tracing::Level::INFO)
            }
            Some(Level::Str(ref lit)) if lit.value().eq_ignore_ascii_case("warn") => {
                quote!(tracing::Level::WARN)
            }
            Some(Level::Str(ref lit)) if lit.value().eq_ignore_ascii_case("error") => {
                quote!(tracing::Level::ERROR)
            }
            Some(Level::Int(ref lit)) if is_level(lit, 1) => quote!(tracing::Level::TRACE),
            Some(Level::Int(ref lit)) if is_level(lit, 2) => quote!(tracing::Level::DEBUG),
            Some(Level::Int(ref lit)) if is_level(lit, 3) => quote!(tracing::Level::INFO),
            Some(Level::Int(ref lit)) if is_level(lit, 4) => quote!(tracing::Level::WARN),
            Some(Level::Int(ref lit)) if is_level(lit, 5) => quote!(tracing::Level::ERROR),
            Some(Level::Path(ref pat)) => quote!(#pat),
            Some(_) => quote! {
                compile_error!(
                    "unknown verbosity level, expected one of \"trace\", \
                     \"debug\", \"info\", \"warn\", or \"error\", or a number 1-5"
                )
            },
            None => quote!(tracing::Level::INFO),
        }
    }

    pub(crate) fn target(&self) -> impl ToTokens {
        if let Some(ref target) = self.target {
            quote!(#target)
        } else {
            quote!(module_path!())
        }
    }

    /// Generate "deprecation" warnings for any unrecognized attribute inputs
    /// that we skipped.
    ///
    /// For backwards compatibility, we need to emit compiler warnings rather
    /// than errors for unrecognized inputs. Generating a fake deprecation is
    /// the only way to do this on stable Rust right now.
    pub(crate) fn warnings(&self) -> impl ToTokens {
        let warnings = self.parse_warnings.iter().map(|err| {
            let msg = format!("found unrecognized input, {}", err);
            let msg = LitStr::new(&msg, err.span());
            // TODO(eliza): This is a bit of a hack, but it's just about the
            // only way to emit warnings from a proc macro on stable Rust.
            // Eventually, when the `proc_macro::Diagnostic` API stabilizes, we
            // should definitely use that instead.
            quote_spanned! {err.span()=>
                #[warn(deprecated)]
                {
                    #[deprecated(since = "not actually deprecated", note = #msg)]
                    const TRACING_INSTRUMENT_WARNING: () = ();
                    let _ = TRACING_INSTRUMENT_WARNING;
                }
            }
        });
        quote! {
            { #(#warnings)* }
        }
    }
}

impl Parse for InstrumentArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut args = Self::default();
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::name) {
                if args.name.is_some() {
                    return Err(input.error("expected only a single `name` argument"));
                }
                let name = input.parse::<StrArg<kw::name>>()?.value;
                args.name = Some(name);
            } else if lookahead.peek(LitStr) {
                // XXX: apparently we support names as either named args with an
                // sign, _or_ as unnamed string literals. That's weird, but
                // changing it is apparently breaking.
                if args.name.is_some() {
                    return Err(input.error("expected only a single `name` argument"));
                }
                args.name = Some(input.parse()?);
            } else if lookahead.peek(kw::target) {
                if args.target.is_some() {
                    return Err(input.error("expected only a single `target` argument"));
                }
                let target = input.parse::<StrArg<kw::target>>()?.value;
                args.target = Some(target);
            } else if lookahead.peek(kw::parent) {
                if args.target.is_some() {
                    return Err(input.error("expected only a single `parent` argument"));
                }
                let parent = input.parse::<ExprArg<kw::parent>>()?;
                args.parent = Some(parent.value);
            } else if lookahead.peek(kw::follows_from) {
                if args.target.is_some() {
                    return Err(input.error("expected only a single `follows_from` argument"));
                }
                let follows_from = input.parse::<ExprArg<kw::follows_from>>()?;
                args.follows_from = Some(follows_from.value);
            } else if lookahead.peek(kw::level) {
                if args.level.is_some() {
                    return Err(input.error("expected only a single `level` argument"));
                }
                args.level = Some(input.parse()?);
            } else if lookahead.peek(kw::skip) {
                if !args.skips.is_empty() {
                    return Err(input.error("expected only a single `skip` argument"));
                }
                if args.skip_all {
                    return Err(input.error("expected either `skip` or `skip_all` argument"));
                }
                let Skips(skips) = input.parse()?;
                args.skips = skips;
            } else if lookahead.peek(kw::skip_all) {
                if args.skip_all {
                    return Err(input.error("expected only a single `skip_all` argument"));
                }
                if !args.skips.is_empty() {
                    return Err(input.error("expected either `skip` or `skip_all` argument"));
                }
                let _ = input.parse::<kw::skip_all>()?;
                args.skip_all = true;
            } else if lookahead.peek(kw::fields) {
                if args.fields.is_some() {
                    return Err(input.error("expected only a single `fields` argument"));
                }
                args.fields = Some(input.parse()?);
            } else if lookahead.peek(kw::err) {
                let _ = input.parse::<kw::err>();
                let mode = FormatMode::parse(input)?;
                args.err_mode = Some(mode);
            } else if lookahead.peek(kw::ret) {
                let _ = input.parse::<kw::ret>()?;
                let mode = FormatMode::parse(input)?;
                args.ret_mode = Some(mode);
            } else if lookahead.peek(Token![,]) {
                let _ = input.parse::<Token![,]>()?;
            } else {
                // We found a token that we didn't expect!
                // We want to emit warnings for these, rather than errors, so
                // we'll add it to the list of unrecognized inputs we've seen so
                // far and keep going.
                args.parse_warnings.push(lookahead.error());
                // Parse the unrecognized token tree to advance the parse
                // stream, and throw it away so we can keep parsing.
                let _ = input.parse::<proc_macro2::TokenTree>();
            }
        }
        Ok(args)
    }
}

struct StrArg<T> {
    value: LitStr,
    _p: std::marker::PhantomData<T>,
}

impl<T: Parse> Parse for StrArg<T> {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let _ = input.parse::<T>()?;
        let _ = input.parse::<Token![=]>()?;
        let value = input.parse()?;
        Ok(Self {
            value,
            _p: std::marker::PhantomData,
        })
    }
}

struct ExprArg<T> {
    value: Expr,
    _p: std::marker::PhantomData<T>,
}

impl<T: Parse> Parse for ExprArg<T> {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let _ = input.parse::<T>()?;
        let _ = input.parse::<Token![=]>()?;
        let value = input.parse()?;
        Ok(Self {
            value,
            _p: std::marker::PhantomData,
        })
    }
}

struct Skips(HashSet<Ident>);

impl Parse for Skips {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let _ = input.parse::<kw::skip>();
        let content;
        let _ = syn::parenthesized!(content in input);
        let names: Punctuated<Ident, Token![,]> = content.parse_terminated(Ident::parse_any)?;
        let mut skips = HashSet::new();
        for name in names {
            if skips.contains(&name) {
                return Err(syn::Error::new(
                    name.span(),
                    "tried to skip the same field twice",
                ));
            } else {
                skips.insert(name);
            }
        }
        Ok(Self(skips))
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub(crate) enum FormatMode {
    Default,
    Display,
    Debug,
}

impl Default for FormatMode {
    fn default() -> Self {
        FormatMode::Default
    }
}

impl Parse for FormatMode {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if !input.peek(syn::token::Paren) {
            return Ok(FormatMode::default());
        }
        let content;
        let _ = syn::parenthesized!(content in input);
        let maybe_mode: Option<Ident> = content.parse()?;
        maybe_mode.map_or(Ok(FormatMode::default()), |ident| {
            match ident.to_string().as_str() {
                "Debug" => Ok(FormatMode::Debug),
                "Display" => Ok(FormatMode::Display),
                _ => Err(syn::Error::new(
                    ident.span(),
                    "unknown error mode, must be Debug or Display",
                )),
            }
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Fields(pub(crate) Punctuated<Field, Token![,]>);

#[derive(Clone, Debug)]
pub(crate) struct Field {
    pub(crate) name: Punctuated<Ident, Token![.]>,
    pub(crate) value: Option<Expr>,
    pub(crate) kind: FieldKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FieldKind {
    Debug,
    Display,
    Value,
}

impl Parse for Fields {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let _ = input.parse::<kw::fields>();
        let content;
        let _ = syn::parenthesized!(content in input);
        let fields: Punctuated<_, Token![,]> = content.parse_terminated(Field::parse)?;
        Ok(Self(fields))
    }
}

impl ToTokens for Fields {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}

impl Parse for Field {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut kind = FieldKind::Value;
        if input.peek(Token![%]) {
            input.parse::<Token![%]>()?;
            kind = FieldKind::Display;
        } else if input.peek(Token![?]) {
            input.parse::<Token![?]>()?;
            kind = FieldKind::Debug;
        };
        let name = Punctuated::parse_separated_nonempty_with(input, Ident::parse_any)?;
        let value = if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            if input.peek(Token![%]) {
                input.parse::<Token![%]>()?;
                kind = FieldKind::Display;
            } else if input.peek(Token![?]) {
                input.parse::<Token![?]>()?;
                kind = FieldKind::Debug;
            };
            Some(input.parse()?)
        } else {
            None
        };
        Ok(Self { name, value, kind })
    }
}

impl ToTokens for Field {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(ref value) = self.value {
            let name = &self.name;
            let kind = &self.kind;
            tokens.extend(quote! {
                #name = #kind#value
            })
        } else if self.kind == FieldKind::Value {
            // XXX(eliza): I don't like that fields without values produce
            // empty fields rather than local variable shorthand...but,
            // we've released a version where field names without values in
            // `instrument` produce empty field values, so changing it now
            // is a breaking change. agh.
            let name = &self.name;
            tokens.extend(quote!(#name = tracing::field::Empty))
        } else {
            self.kind.to_tokens(tokens);
            self.name.to_tokens(tokens);
        }
    }
}

impl ToTokens for FieldKind {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            FieldKind::Debug => tokens.extend(quote! { ? }),
            FieldKind::Display => tokens.extend(quote! { % }),
            _ => {}
        }
    }
}

#[derive(Clone, Debug)]
enum Level {
    Str(LitStr),
    Int(LitInt),
    Path(Path),
}

impl Parse for Level {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let _ = input.parse::<kw::level>()?;
        let _ = input.parse::<Token![=]>()?;
        let lookahead = input.lookahead1();
        if lookahead.peek(LitStr) {
            Ok(Self::Str(input.parse()?))
        } else if lookahead.peek(LitInt) {
            Ok(Self::Int(input.parse()?))
        } else if lookahead.peek(Ident) {
            Ok(Self::Path(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

mod kw {
    syn::custom_keyword!(fields);
    syn::custom_keyword!(skip);
    syn::custom_keyword!(skip_all);
    syn::custom_keyword!(level);
    syn::custom_keyword!(target);
    syn::custom_keyword!(parent);
    syn::custom_keyword!(follows_from);
    syn::custom_keyword!(name);
    syn::custom_keyword!(err);
    syn::custom_keyword!(ret);
}
