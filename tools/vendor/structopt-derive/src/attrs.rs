// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::doc_comments::process_doc_comment;
use crate::{parse::*, spanned::Sp, ty::Ty};

use std::env;

use heck::{CamelCase, KebabCase, MixedCase, ShoutySnakeCase, SnakeCase};
use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort;
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    self, ext::IdentExt, spanned::Spanned, Attribute, Expr, Ident, LitStr, MetaNameValue, Type,
};

#[derive(Clone)]
pub enum Kind {
    Arg(Sp<Ty>),
    Subcommand(Sp<Ty>),
    ExternalSubcommand,
    Flatten,
    Skip(Option<Expr>),
}

#[derive(Clone)]
pub struct Method {
    name: Ident,
    args: TokenStream,
}

#[derive(Clone)]
pub struct Parser {
    pub kind: Sp<ParserKind>,
    pub func: TokenStream,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParserKind {
    FromStr,
    TryFromStr,
    FromOsStr,
    TryFromOsStr,
    FromOccurrences,
    FromFlag,
}

/// Defines the casing for the attributes long representation.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CasingStyle {
    /// Indicate word boundaries with uppercase letter, excluding the first word.
    Camel,
    /// Keep all letters lowercase and indicate word boundaries with hyphens.
    Kebab,
    /// Indicate word boundaries with uppercase letter, including the first word.
    Pascal,
    /// Keep all letters uppercase and indicate word boundaries with underscores.
    ScreamingSnake,
    /// Keep all letters lowercase and indicate word boundaries with underscores.
    Snake,
    /// Use the original attribute name defined in the code.
    Verbatim,
    /// Keep all letters lowercase and remove word boundaries.
    Lower,
    /// Keep all letters uppercase and remove word boundaries.
    Upper,
}

#[derive(Clone)]
pub enum Name {
    Derived(Ident),
    Assigned(TokenStream),
}

#[derive(Clone)]
pub struct Attrs {
    name: Name,
    casing: Sp<CasingStyle>,
    env_casing: Sp<CasingStyle>,
    ty: Option<Type>,
    doc_comment: Vec<Method>,
    methods: Vec<Method>,
    parser: Sp<Parser>,
    author: Option<Method>,
    about: Option<Method>,
    version: Option<Method>,
    no_version: Option<Ident>,
    verbatim_doc_comment: Option<Ident>,
    has_custom_parser: bool,
    kind: Sp<Kind>,
}

impl Method {
    pub fn new(name: Ident, args: TokenStream) -> Self {
        Method { name, args }
    }

    fn from_lit_or_env(ident: Ident, lit: Option<LitStr>, env_var: &str) -> Self {
        let mut lit = match lit {
            Some(lit) => lit,

            None => match env::var(env_var) {
                Ok(val) => LitStr::new(&val, ident.span()),
                Err(_) => {
                    abort!(ident,
                        "cannot derive `{}` from Cargo.toml", ident;
                        note = "`{}` environment variable is not set", env_var;
                        help = "use `{} = \"...\"` to set {} manually", ident, ident;
                    );
                }
            },
        };

        if ident == "author" {
            let edited = process_author_str(&lit.value());
            lit = LitStr::new(&edited, lit.span());
        }

        Method::new(ident, quote!(#lit))
    }
}

impl ToTokens for Method {
    fn to_tokens(&self, ts: &mut TokenStream) {
        let Method { ref name, ref args } = self;
        quote!(.#name(#args)).to_tokens(ts);
    }
}

impl Parser {
    fn default_spanned(span: Span) -> Sp<Self> {
        let kind = Sp::new(ParserKind::TryFromStr, span);
        let func = quote_spanned!(span=> ::std::str::FromStr::from_str);
        Sp::new(Parser { kind, func }, span)
    }

    fn from_spec(parse_ident: Ident, spec: ParserSpec) -> Sp<Self> {
        use ParserKind::*;

        let kind = match &*spec.kind.to_string() {
            "from_str" => FromStr,
            "try_from_str" => TryFromStr,
            "from_os_str" => FromOsStr,
            "try_from_os_str" => TryFromOsStr,
            "from_occurrences" => FromOccurrences,
            "from_flag" => FromFlag,
            s => abort!(spec.kind, "unsupported parser `{}`", s),
        };

        let func = match spec.parse_func {
            None => match kind {
                FromStr | FromOsStr => {
                    quote_spanned!(spec.kind.span()=> ::std::convert::From::from)
                }
                TryFromStr => quote_spanned!(spec.kind.span()=> ::std::str::FromStr::from_str),
                TryFromOsStr => abort!(
                    spec.kind,
                    "you must set parser for `try_from_os_str` explicitly"
                ),
                FromOccurrences => quote_spanned!(spec.kind.span()=> { |v| v as _ }),
                FromFlag => quote_spanned!(spec.kind.span()=> ::std::convert::From::from),
            },

            Some(func) => match func {
                syn::Expr::Path(_) => quote!(#func),
                _ => abort!(func, "`parse` argument must be a function path"),
            },
        };

        let kind = Sp::new(kind, spec.kind.span());
        let parser = Parser { kind, func };
        Sp::new(parser, parse_ident.span())
    }
}

impl CasingStyle {
    fn from_lit(name: LitStr) -> Sp<Self> {
        use CasingStyle::*;

        let normalized = name.value().to_camel_case().to_lowercase();
        let cs = |kind| Sp::new(kind, name.span());

        match normalized.as_ref() {
            "camel" | "camelcase" => cs(Camel),
            "kebab" | "kebabcase" => cs(Kebab),
            "pascal" | "pascalcase" => cs(Pascal),
            "screamingsnake" | "screamingsnakecase" => cs(ScreamingSnake),
            "snake" | "snakecase" => cs(Snake),
            "verbatim" | "verbatimcase" => cs(Verbatim),
            "lower" | "lowercase" => cs(Lower),
            "upper" | "uppercase" => cs(Upper),
            s => abort!(name, "unsupported casing: `{}`", s),
        }
    }
}

impl Name {
    pub fn translate(self, style: CasingStyle) -> TokenStream {
        use CasingStyle::*;

        match self {
            Name::Assigned(tokens) => tokens,
            Name::Derived(ident) => {
                let s = ident.unraw().to_string();
                let s = match style {
                    Pascal => s.to_camel_case(),
                    Kebab => s.to_kebab_case(),
                    Camel => s.to_mixed_case(),
                    ScreamingSnake => s.to_shouty_snake_case(),
                    Snake => s.to_snake_case(),
                    Verbatim => s,
                    Lower => s.to_snake_case().replace("_", ""),
                    Upper => s.to_shouty_snake_case().replace("_", ""),
                };
                quote_spanned!(ident.span()=> #s)
            }
        }
    }
}

impl Attrs {
    fn new(
        default_span: Span,
        name: Name,
        parent_attrs: Option<&Attrs>,
        ty: Option<Type>,
        casing: Sp<CasingStyle>,
        env_casing: Sp<CasingStyle>,
    ) -> Self {
        let no_version = parent_attrs
            .as_ref()
            .map(|attrs| attrs.no_version.clone())
            .unwrap_or(None);

        Self {
            name,
            ty,
            casing,
            env_casing,
            doc_comment: vec![],
            methods: vec![],
            parser: Parser::default_spanned(default_span),
            about: None,
            author: None,
            version: None,
            no_version,
            verbatim_doc_comment: None,

            has_custom_parser: false,
            kind: Sp::new(Kind::Arg(Sp::new(Ty::Other, default_span)), default_span),
        }
    }

    fn push_method(&mut self, name: Ident, arg: impl ToTokens) {
        if name == "name" {
            self.name = Name::Assigned(quote!(#arg));
        } else if name == "version" {
            self.version = Some(Method::new(name, quote!(#arg)));
        } else {
            self.methods.push(Method::new(name, quote!(#arg)))
        }
    }

    fn push_attrs(&mut self, attrs: &[Attribute]) {
        use crate::parse::StructOptAttr::*;

        for attr in parse_structopt_attributes(attrs) {
            match attr {
                Short(ident) | Long(ident) => {
                    self.push_method(ident, self.name.clone().translate(*self.casing));
                }

                Env(ident) => {
                    self.push_method(ident, self.name.clone().translate(*self.env_casing));
                }

                Subcommand(ident) => {
                    let ty = Sp::call_site(Ty::Other);
                    let kind = Sp::new(Kind::Subcommand(ty), ident.span());
                    self.set_kind(kind);
                }

                ExternalSubcommand(ident) => {
                    self.kind = Sp::new(Kind::ExternalSubcommand, ident.span());
                }

                Flatten(ident) => {
                    let kind = Sp::new(Kind::Flatten, ident.span());
                    self.set_kind(kind);
                }

                Skip(ident, expr) => {
                    let kind = Sp::new(Kind::Skip(expr), ident.span());
                    self.set_kind(kind);
                }

                NoVersion(ident) => self.no_version = Some(ident),

                VerbatimDocComment(ident) => self.verbatim_doc_comment = Some(ident),

                DefaultValue(ident, lit) => {
                    let val = if let Some(lit) = lit {
                        quote!(#lit)
                    } else {
                        let ty = if let Some(ty) = self.ty.as_ref() {
                            ty
                        } else {
                            abort!(
                                ident,
                                "#[structopt(default_value)] (without an argument) can be used \
                                only on field level";

                                note = "see \
                                    https://docs.rs/structopt/0.3.5/structopt/#magical-methods")
                        };

                        quote_spanned!(ident.span()=> {
                            ::structopt::lazy_static::lazy_static! {
                                static ref DEFAULT_VALUE: &'static str = {
                                    let val = <#ty as ::std::default::Default>::default();
                                    let s = ::std::string::ToString::to_string(&val);
                                    ::std::boxed::Box::leak(s.into_boxed_str())
                                };
                            }
                            *DEFAULT_VALUE
                        })
                    };

                    self.methods.push(Method::new(ident, val));
                }

                About(ident, about) => {
                    self.about = Some(Method::from_lit_or_env(
                        ident,
                        about,
                        "CARGO_PKG_DESCRIPTION",
                    ));
                }

                Author(ident, author) => {
                    self.author = Some(Method::from_lit_or_env(ident, author, "CARGO_PKG_AUTHORS"));
                }

                Version(ident, version) => {
                    self.push_method(ident, version);
                }

                NameLitStr(name, lit) => {
                    self.push_method(name, lit);
                }

                NameExpr(name, expr) => {
                    self.push_method(name, expr);
                }

                MethodCall(name, args) => self.push_method(name, quote!(#(#args),*)),

                RenameAll(_, casing_lit) => {
                    self.casing = CasingStyle::from_lit(casing_lit);
                }

                RenameAllEnv(_, casing_lit) => {
                    self.env_casing = CasingStyle::from_lit(casing_lit);
                }

                Parse(ident, spec) => {
                    self.has_custom_parser = true;
                    self.parser = Parser::from_spec(ident, spec);
                }
            }
        }
    }

    fn push_doc_comment(&mut self, attrs: &[Attribute], name: &str) {
        use crate::Lit::*;
        use crate::Meta::*;

        let comment_parts: Vec<_> = attrs
            .iter()
            .filter(|attr| attr.path.is_ident("doc"))
            .filter_map(|attr| {
                if let Ok(NameValue(MetaNameValue { lit: Str(s), .. })) = attr.parse_meta() {
                    Some(s.value())
                } else {
                    // non #[doc = "..."] attributes are not our concern
                    // we leave them for rustc to handle
                    None
                }
            })
            .collect();

        self.doc_comment =
            process_doc_comment(comment_parts, name, self.verbatim_doc_comment.is_none());
    }

    pub fn from_struct(
        span: Span,
        attrs: &[Attribute],
        name: Name,
        parent_attrs: Option<&Attrs>,
        argument_casing: Sp<CasingStyle>,
        env_casing: Sp<CasingStyle>,
        allow_skip: bool,
    ) -> Self {
        let mut res = Self::new(span, name, parent_attrs, None, argument_casing, env_casing);
        res.push_attrs(attrs);
        res.push_doc_comment(attrs, "about");

        if res.has_custom_parser {
            abort!(
                res.parser.span(),
                "`parse` attribute is only allowed on fields"
            );
        }
        match &*res.kind {
            Kind::Subcommand(_) => abort!(res.kind.span(), "subcommand is only allowed on fields"),
            Kind::Skip(_) if !allow_skip => {
                abort!(res.kind.span(), "skip is only allowed on fields")
            }
            Kind::Arg(_) | Kind::ExternalSubcommand | Kind::Flatten | Kind::Skip(_) => res,
        }
    }

    pub fn from_field(
        field: &syn::Field,
        parent_attrs: Option<&Attrs>,
        struct_casing: Sp<CasingStyle>,
        env_casing: Sp<CasingStyle>,
    ) -> Self {
        let name = field.ident.clone().unwrap();
        let mut res = Self::new(
            field.span(),
            Name::Derived(name),
            parent_attrs,
            Some(field.ty.clone()),
            struct_casing,
            env_casing,
        );
        res.push_attrs(&field.attrs);
        res.push_doc_comment(&field.attrs, "help");

        match &*res.kind {
            Kind::Flatten => {
                if res.has_custom_parser {
                    abort!(
                        res.parser.span(),
                        "parse attribute is not allowed for flattened entry"
                    );
                }
                if res.has_explicit_methods() {
                    abort!(
                        res.kind.span(),
                        "methods are not allowed for flattened entry"
                    );
                }

                if res.has_doc_methods() {
                    res.doc_comment = vec![];
                }
            }

            Kind::ExternalSubcommand => {}

            Kind::Subcommand(_) => {
                if res.has_custom_parser {
                    abort!(
                        res.parser.span(),
                        "parse attribute is not allowed for subcommand"
                    );
                }
                if res.has_explicit_methods() {
                    abort!(
                        res.kind.span(),
                        "methods in attributes are not allowed for subcommand"
                    );
                }

                let ty = Ty::from_syn_ty(&field.ty);
                match *ty {
                    Ty::OptionOption => {
                        abort!(
                            field.ty,
                            "Option<Option<T>> type is not allowed for subcommand"
                        );
                    }
                    Ty::OptionVec => {
                        abort!(
                            field.ty,
                            "Option<Vec<T>> type is not allowed for subcommand"
                        );
                    }
                    _ => (),
                }

                res.kind = Sp::new(Kind::Subcommand(ty), res.kind.span());
            }
            Kind::Skip(_) => {
                if res.has_explicit_methods() {
                    abort!(
                        res.kind.span(),
                        "methods are not allowed for skipped fields"
                    );
                }
            }
            Kind::Arg(orig_ty) => {
                let mut ty = Ty::from_syn_ty(&field.ty);
                if res.has_custom_parser {
                    match *ty {
                        Ty::Option | Ty::Vec | Ty::OptionVec => (),
                        _ => ty = Sp::new(Ty::Other, ty.span()),
                    }
                }

                match *ty {
                    Ty::Bool => {
                        if res.is_positional() && !res.has_custom_parser {
                            abort!(field.ty,
                                "`bool` cannot be used as positional parameter with default parser";
                                help = "if you want to create a flag add `long` or `short`";
                                help = "If you really want a boolean parameter \
                                    add an explicit parser, for example `parse(try_from_str)`";
                                note = "see also https://github.com/TeXitoi/structopt/tree/master/examples/true_or_false.rs";
                            )
                        }
                        if let Some(m) = res.find_method("default_value") {
                            abort!(m.name, "default_value is meaningless for bool")
                        }
                        if let Some(m) = res.find_method("required") {
                            abort!(m.name, "required is meaningless for bool")
                        }
                    }
                    Ty::Option => {
                        if let Some(m) = res.find_method("default_value") {
                            abort!(m.name, "default_value is meaningless for Option")
                        }
                        if let Some(m) = res.find_method("required") {
                            abort!(m.name, "required is meaningless for Option")
                        }
                    }
                    Ty::OptionOption => {
                        if res.is_positional() {
                            abort!(
                                field.ty,
                                "Option<Option<T>> type is meaningless for positional argument"
                            )
                        }
                    }
                    Ty::OptionVec => {
                        if res.is_positional() {
                            abort!(
                                field.ty,
                                "Option<Vec<T>> type is meaningless for positional argument"
                            )
                        }
                    }

                    _ => (),
                }
                res.kind = Sp::new(Kind::Arg(ty), orig_ty.span());
            }
        }

        res
    }

    fn set_kind(&mut self, kind: Sp<Kind>) {
        if let Kind::Arg(_) = *self.kind {
            self.kind = kind;
        } else {
            abort!(
                kind.span(),
                "subcommand, flatten and skip cannot be used together"
            );
        }
    }

    pub fn has_method(&self, name: &str) -> bool {
        self.find_method(name).is_some()
    }

    pub fn find_method(&self, name: &str) -> Option<&Method> {
        self.methods.iter().find(|m| m.name == name)
    }

    /// generate methods from attributes on top of struct or enum
    pub fn top_level_methods(&self) -> TokenStream {
        let author = &self.author;
        let about = &self.about;
        let methods = &self.methods;
        let doc_comment = &self.doc_comment;

        quote!( #(#doc_comment)* #author #about #(#methods)*  )
    }

    /// generate methods on top of a field
    pub fn field_methods(&self) -> TokenStream {
        let methods = &self.methods;
        let doc_comment = &self.doc_comment;
        quote!( #(#doc_comment)* #(#methods)* )
    }

    pub fn version(&self) -> TokenStream {
        match (&self.no_version, &self.version) {
            (None, Some(m)) => m.to_token_stream(),

            (None, None) => std::env::var("CARGO_PKG_VERSION")
                .map(|version| quote!( .version(#version) ))
                .unwrap_or_default(),

            _ => quote!(),
        }
    }

    pub fn cased_name(&self) -> TokenStream {
        self.name.clone().translate(*self.casing)
    }

    pub fn parser(&self) -> &Sp<Parser> {
        &self.parser
    }

    pub fn kind(&self) -> Sp<Kind> {
        self.kind.clone()
    }

    pub fn casing(&self) -> Sp<CasingStyle> {
        self.casing.clone()
    }

    pub fn env_casing(&self) -> Sp<CasingStyle> {
        self.env_casing.clone()
    }

    pub fn is_positional(&self) -> bool {
        self.methods
            .iter()
            .all(|m| m.name != "long" && m.name != "short")
    }

    pub fn has_explicit_methods(&self) -> bool {
        self.methods
            .iter()
            .any(|m| m.name != "help" && m.name != "long_help")
    }

    pub fn has_doc_methods(&self) -> bool {
        !self.doc_comment.is_empty()
            || self.methods.iter().any(|m| {
                m.name == "help"
                    || m.name == "long_help"
                    || m.name == "about"
                    || m.name == "long_about"
            })
    }
}

/// replace all `:` with `, ` when not inside the `<>`
///
/// `"author1:author2:author3" => "author1, author2, author3"`
/// `"author1 <http://website1.com>:author2" => "author1 <http://website1.com>, author2"
fn process_author_str(author: &str) -> String {
    let mut res = String::with_capacity(author.len());
    let mut inside_angle_braces = 0usize;

    for ch in author.chars() {
        if inside_angle_braces > 0 && ch == '>' {
            inside_angle_braces -= 1;
            res.push(ch);
        } else if ch == '<' {
            inside_angle_braces += 1;
            res.push(ch);
        } else if inside_angle_braces == 0 && ch == ':' {
            res.push_str(", ");
        } else {
            res.push(ch);
        }
    }

    res
}
