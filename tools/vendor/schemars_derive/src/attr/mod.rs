mod doc;
mod schemars_to_serde;
mod validation;

pub use schemars_to_serde::process_serde_attrs;
pub use validation::ValidationAttrs;

use crate::metadata::SchemaMetadata;
use proc_macro2::{Group, Span, TokenStream, TokenTree};
use quote::ToTokens;
use serde_derive_internals::Ctxt;
use syn::parse::{self, Parse};
use syn::Meta::{List, NameValue};
use syn::MetaNameValue;
use syn::NestedMeta::{Lit, Meta};

// FIXME using the same struct for containers+variants+fields means that
//  with/schema_with are accepted (but ignored) on containers, and
//  repr/crate_name are accepted (but ignored) on variants and fields etc.

#[derive(Debug, Default)]
pub struct Attrs {
    pub with: Option<WithAttr>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub deprecated: bool,
    pub examples: Vec<syn::Path>,
    pub repr: Option<syn::Type>,
    pub crate_name: Option<syn::Path>,
    pub is_renamed: bool,
}

#[derive(Debug)]
pub enum WithAttr {
    Type(syn::Type),
    Function(syn::Path),
}

impl Attrs {
    pub fn new(attrs: &[syn::Attribute], errors: &Ctxt) -> Self {
        let mut result = Attrs::default()
            .populate(attrs, "schemars", false, errors)
            .populate(attrs, "serde", true, errors);

        result.deprecated = attrs.iter().any(|a| a.path.is_ident("deprecated"));
        result.repr = attrs
            .iter()
            .find(|a| a.path.is_ident("repr"))
            .and_then(|a| a.parse_args().ok());

        let (doc_title, doc_description) = doc::get_title_and_desc_from_doc(attrs);
        result.title = result.title.or(doc_title);
        result.description = result.description.or(doc_description);

        result
    }

    pub fn as_metadata(&self) -> SchemaMetadata<'_> {
        #[allow(clippy::ptr_arg)]
        fn none_if_empty(s: &String) -> Option<&str> {
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        }

        SchemaMetadata {
            title: self.title.as_ref().and_then(none_if_empty),
            description: self.description.as_ref().and_then(none_if_empty),
            deprecated: self.deprecated,
            examples: &self.examples,
            read_only: false,
            write_only: false,
            default: None,
        }
    }

    fn populate(
        mut self,
        attrs: &[syn::Attribute],
        attr_type: &'static str,
        ignore_errors: bool,
        errors: &Ctxt,
    ) -> Self {
        let duplicate_error = |meta: &MetaNameValue| {
            if !ignore_errors {
                let msg = format!(
                    "duplicate schemars attribute `{}`",
                    meta.path.get_ident().unwrap()
                );
                errors.error_spanned_by(meta, msg)
            }
        };
        let mutual_exclusive_error = |meta: &MetaNameValue, other: &str| {
            if !ignore_errors {
                let msg = format!(
                    "schemars attribute cannot contain both `{}` and `{}`",
                    meta.path.get_ident().unwrap(),
                    other,
                );
                errors.error_spanned_by(meta, msg)
            }
        };

        for meta_item in attrs
            .iter()
            .flat_map(|attr| get_meta_items(attr, attr_type, errors, ignore_errors))
            .flatten()
        {
            match &meta_item {
                Meta(NameValue(m)) if m.path.is_ident("with") => {
                    if let Ok(ty) = parse_lit_into_ty(errors, attr_type, "with", &m.lit) {
                        match self.with {
                            Some(WithAttr::Type(_)) => duplicate_error(m),
                            Some(WithAttr::Function(_)) => mutual_exclusive_error(m, "schema_with"),
                            None => self.with = Some(WithAttr::Type(ty)),
                        }
                    }
                }

                Meta(NameValue(m)) if m.path.is_ident("schema_with") => {
                    if let Ok(fun) = parse_lit_into_path(errors, attr_type, "schema_with", &m.lit) {
                        match self.with {
                            Some(WithAttr::Function(_)) => duplicate_error(m),
                            Some(WithAttr::Type(_)) => mutual_exclusive_error(m, "with"),
                            None => self.with = Some(WithAttr::Function(fun)),
                        }
                    }
                }

                Meta(NameValue(m)) if m.path.is_ident("title") => {
                    if let Ok(title) = get_lit_str(errors, attr_type, "title", &m.lit) {
                        match self.title {
                            Some(_) => duplicate_error(m),
                            None => self.title = Some(title.value()),
                        }
                    }
                }

                Meta(NameValue(m)) if m.path.is_ident("description") => {
                    if let Ok(description) = get_lit_str(errors, attr_type, "description", &m.lit) {
                        match self.description {
                            Some(_) => duplicate_error(m),
                            None => self.description = Some(description.value()),
                        }
                    }
                }

                Meta(NameValue(m)) if m.path.is_ident("example") => {
                    if let Ok(fun) = parse_lit_into_path(errors, attr_type, "example", &m.lit) {
                        self.examples.push(fun)
                    }
                }

                Meta(NameValue(m)) if m.path.is_ident("rename") => self.is_renamed = true,

                Meta(NameValue(m)) if m.path.is_ident("crate") && attr_type == "schemars" => {
                    if let Ok(p) = parse_lit_into_path(errors, attr_type, "crate", &m.lit) {
                        if self.crate_name.is_some() {
                            duplicate_error(m)
                        } else {
                            self.crate_name = Some(p)
                        }
                    }
                }

                _ if ignore_errors => {}

                Meta(meta_item) => {
                    if !is_known_serde_or_validation_keyword(meta_item) {
                        let path = meta_item
                            .path()
                            .into_token_stream()
                            .to_string()
                            .replace(' ', "");
                        errors.error_spanned_by(
                            meta_item.path(),
                            format!("unknown schemars attribute `{}`", path),
                        );
                    }
                }

                Lit(lit) => {
                    errors.error_spanned_by(lit, "unexpected literal in schemars attribute");
                }
            }
        }
        self
    }

    pub fn is_default(&self) -> bool {
        match self {
            Self {
                with: None,
                title: None,
                description: None,
                deprecated: false,
                examples,
                repr: None,
                crate_name: None,
                is_renamed: _,
            } if examples.is_empty() => true,
            _ => false,
        }
    }
}

fn is_known_serde_or_validation_keyword(meta: &syn::Meta) -> bool {
    let mut known_keywords = schemars_to_serde::SERDE_KEYWORDS
        .iter()
        .chain(validation::VALIDATION_KEYWORDS);
    meta.path()
        .get_ident()
        .map(|i| known_keywords.any(|k| i == k))
        .unwrap_or(false)
}

fn get_meta_items(
    attr: &syn::Attribute,
    attr_type: &'static str,
    errors: &Ctxt,
    ignore_errors: bool,
) -> Result<Vec<syn::NestedMeta>, ()> {
    if !attr.path.is_ident(attr_type) {
        return Ok(Vec::new());
    }

    match attr.parse_meta() {
        Ok(List(meta)) => Ok(meta.nested.into_iter().collect()),
        Ok(other) => {
            if !ignore_errors {
                errors.error_spanned_by(other, format!("expected #[{}(...)]", attr_type))
            }
            Err(())
        }
        Err(err) => {
            if !ignore_errors {
                errors.error_spanned_by(attr, err)
            }
            Err(())
        }
    }
}

fn get_lit_str<'a>(
    cx: &Ctxt,
    attr_type: &'static str,
    meta_item_name: &'static str,
    lit: &'a syn::Lit,
) -> Result<&'a syn::LitStr, ()> {
    if let syn::Lit::Str(lit) = lit {
        Ok(lit)
    } else {
        cx.error_spanned_by(
            lit,
            format!(
                "expected {} {} attribute to be a string: `{} = \"...\"`",
                attr_type, meta_item_name, meta_item_name
            ),
        );
        Err(())
    }
}

fn parse_lit_into_ty(
    cx: &Ctxt,
    attr_type: &'static str,
    meta_item_name: &'static str,
    lit: &syn::Lit,
) -> Result<syn::Type, ()> {
    let string = get_lit_str(cx, attr_type, meta_item_name, lit)?;

    parse_lit_str(string).map_err(|_| {
        cx.error_spanned_by(
            lit,
            format!(
                "failed to parse type: `{} = {:?}`",
                meta_item_name,
                string.value()
            ),
        )
    })
}

fn parse_lit_into_path(
    cx: &Ctxt,
    attr_type: &'static str,
    meta_item_name: &'static str,
    lit: &syn::Lit,
) -> Result<syn::Path, ()> {
    let string = get_lit_str(cx, attr_type, meta_item_name, lit)?;

    parse_lit_str(string).map_err(|_| {
        cx.error_spanned_by(
            lit,
            format!(
                "failed to parse path: `{} = {:?}`",
                meta_item_name,
                string.value()
            ),
        )
    })
}

fn parse_lit_str<T>(s: &syn::LitStr) -> parse::Result<T>
where
    T: Parse,
{
    let tokens = spanned_tokens(s)?;
    syn::parse2(tokens)
}

fn spanned_tokens(s: &syn::LitStr) -> parse::Result<TokenStream> {
    let stream = syn::parse_str(&s.value())?;
    Ok(respan_token_stream(stream, s.span()))
}

fn respan_token_stream(stream: TokenStream, span: Span) -> TokenStream {
    stream
        .into_iter()
        .map(|token| respan_token_tree(token, span))
        .collect()
}

fn respan_token_tree(mut token: TokenTree, span: Span) -> TokenTree {
    if let TokenTree::Group(g) = &mut token {
        *g = Group::new(g.delimiter(), respan_token_stream(g.stream(), span));
    }
    token.set_span(span);
    token
}
