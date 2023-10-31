// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>,
// Kevin Knapp (@kbknapp) <kbknapp@gmail.com>, and
// Ana Hobden (@hoverbear) <operator@hoverbear.org>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// This work was derived from Structopt (https://github.com/TeXitoi/structopt)
// commit#ea76fa1b1b273e65e3b0b1046643715b49bec51f which is licensed under the
// MIT/Apache 2.0 license.

use std::env;

use heck::{ToKebabCase, ToLowerCamelCase, ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};
use proc_macro2::{self, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::DeriveInput;
use syn::{self, ext::IdentExt, spanned::Spanned, Attribute, Field, Ident, LitStr, Type, Variant};

use crate::attr::*;
use crate::utils::{extract_doc_comment, format_doc_comment, inner_type, is_simple_ty, Sp, Ty};

/// Default casing style for generated arguments.
pub const DEFAULT_CASING: CasingStyle = CasingStyle::Kebab;

/// Default casing style for environment variables
pub const DEFAULT_ENV_CASING: CasingStyle = CasingStyle::ScreamingSnake;

#[derive(Clone)]
pub struct Item {
    name: Name,
    casing: Sp<CasingStyle>,
    env_casing: Sp<CasingStyle>,
    ty: Option<Type>,
    doc_comment: Vec<Method>,
    methods: Vec<Method>,
    deprecations: Vec<Deprecation>,
    value_parser: Option<ValueParser>,
    action: Option<Action>,
    verbatim_doc_comment: bool,
    force_long_help: bool,
    next_display_order: Option<Method>,
    next_help_heading: Option<Method>,
    is_enum: bool,
    is_positional: bool,
    skip_group: bool,
    group_id: Name,
    group_methods: Vec<Method>,
    kind: Sp<Kind>,
}

impl Item {
    pub fn from_args_struct(input: &DeriveInput, name: Name) -> Result<Self, syn::Error> {
        let ident = input.ident.clone();
        let span = input.ident.span();
        let attrs = &input.attrs;
        let argument_casing = Sp::new(DEFAULT_CASING, span);
        let env_casing = Sp::new(DEFAULT_ENV_CASING, span);
        let kind = Sp::new(Kind::Command(Sp::new(Ty::Other, span)), span);

        let mut res = Self::new(name, ident, None, argument_casing, env_casing, kind);
        let parsed_attrs = ClapAttr::parse_all(attrs)?;
        res.infer_kind(&parsed_attrs)?;
        res.push_attrs(&parsed_attrs)?;
        res.push_doc_comment(attrs, "about", Some("long_about"));

        Ok(res)
    }

    pub fn from_subcommand_enum(input: &DeriveInput, name: Name) -> Result<Self, syn::Error> {
        let ident = input.ident.clone();
        let span = input.ident.span();
        let attrs = &input.attrs;
        let argument_casing = Sp::new(DEFAULT_CASING, span);
        let env_casing = Sp::new(DEFAULT_ENV_CASING, span);
        let kind = Sp::new(Kind::Command(Sp::new(Ty::Other, span)), span);

        let mut res = Self::new(name, ident, None, argument_casing, env_casing, kind);
        let parsed_attrs = ClapAttr::parse_all(attrs)?;
        res.infer_kind(&parsed_attrs)?;
        res.push_attrs(&parsed_attrs)?;
        res.push_doc_comment(attrs, "about", Some("long_about"));

        Ok(res)
    }

    pub fn from_value_enum(input: &DeriveInput, name: Name) -> Result<Self, syn::Error> {
        let ident = input.ident.clone();
        let span = input.ident.span();
        let attrs = &input.attrs;
        let argument_casing = Sp::new(DEFAULT_CASING, span);
        let env_casing = Sp::new(DEFAULT_ENV_CASING, span);
        let kind = Sp::new(Kind::Value, span);

        let mut res = Self::new(name, ident, None, argument_casing, env_casing, kind);
        let parsed_attrs = ClapAttr::parse_all(attrs)?;
        res.infer_kind(&parsed_attrs)?;
        res.push_attrs(&parsed_attrs)?;
        // Ignoring `push_doc_comment` as there is no top-level clap builder to add documentation
        // to

        if res.has_explicit_methods() {
            abort!(
                res.methods[0].name.span(),
                "{} doesn't exist for `ValueEnum` enums",
                res.methods[0].name
            );
        }

        Ok(res)
    }

    pub fn from_subcommand_variant(
        variant: &Variant,
        struct_casing: Sp<CasingStyle>,
        env_casing: Sp<CasingStyle>,
    ) -> Result<Self, syn::Error> {
        let name = variant.ident.clone();
        let ident = variant.ident.clone();
        let span = variant.span();
        let ty = match variant.fields {
            syn::Fields::Unnamed(syn::FieldsUnnamed { ref unnamed, .. }) if unnamed.len() == 1 => {
                Ty::from_syn_ty(&unnamed[0].ty)
            }
            syn::Fields::Named(_) | syn::Fields::Unnamed(..) | syn::Fields::Unit => {
                Sp::new(Ty::Other, span)
            }
        };
        let kind = Sp::new(Kind::Command(ty), span);
        let mut res = Self::new(
            Name::Derived(name),
            ident,
            None,
            struct_casing,
            env_casing,
            kind,
        );
        let parsed_attrs = ClapAttr::parse_all(&variant.attrs)?;
        res.infer_kind(&parsed_attrs)?;
        res.push_attrs(&parsed_attrs)?;
        if matches!(&*res.kind, Kind::Command(_) | Kind::Subcommand(_)) {
            res.push_doc_comment(&variant.attrs, "about", Some("long_about"));
        }

        match &*res.kind {
            Kind::Flatten(_) => {
                if res.has_explicit_methods() {
                    abort!(
                        res.kind.span(),
                        "methods are not allowed for flattened entry"
                    );
                }
            }

            Kind::Subcommand(_)
            | Kind::ExternalSubcommand
            | Kind::FromGlobal(_)
            | Kind::Skip(_, _)
            | Kind::Command(_)
            | Kind::Value
            | Kind::Arg(_) => (),
        }

        Ok(res)
    }

    pub fn from_value_enum_variant(
        variant: &Variant,
        argument_casing: Sp<CasingStyle>,
        env_casing: Sp<CasingStyle>,
    ) -> Result<Self, syn::Error> {
        let ident = variant.ident.clone();
        let span = variant.span();
        let kind = Sp::new(Kind::Value, span);
        let mut res = Self::new(
            Name::Derived(variant.ident.clone()),
            ident,
            None,
            argument_casing,
            env_casing,
            kind,
        );
        let parsed_attrs = ClapAttr::parse_all(&variant.attrs)?;
        res.infer_kind(&parsed_attrs)?;
        res.push_attrs(&parsed_attrs)?;
        if matches!(&*res.kind, Kind::Value) {
            res.push_doc_comment(&variant.attrs, "help", None);
        }

        Ok(res)
    }

    pub fn from_args_field(
        field: &Field,
        struct_casing: Sp<CasingStyle>,
        env_casing: Sp<CasingStyle>,
    ) -> Result<Self, syn::Error> {
        let name = field.ident.clone().unwrap();
        let ident = field.ident.clone().unwrap();
        let span = field.span();
        let ty = Ty::from_syn_ty(&field.ty);
        let kind = Sp::new(Kind::Arg(ty), span);
        let mut res = Self::new(
            Name::Derived(name),
            ident,
            Some(field.ty.clone()),
            struct_casing,
            env_casing,
            kind,
        );
        let parsed_attrs = ClapAttr::parse_all(&field.attrs)?;
        res.infer_kind(&parsed_attrs)?;
        res.push_attrs(&parsed_attrs)?;
        if matches!(&*res.kind, Kind::Arg(_)) {
            res.push_doc_comment(&field.attrs, "help", Some("long_help"));
        }

        match &*res.kind {
            Kind::Flatten(_) => {
                if res.has_explicit_methods() {
                    abort!(
                        res.kind.span(),
                        "methods are not allowed for flattened entry"
                    );
                }
            }

            Kind::Subcommand(_) => {
                if res.has_explicit_methods() {
                    abort!(
                        res.kind.span(),
                        "methods in attributes are not allowed for subcommand"
                    );
                }
            }
            Kind::Skip(_, _)
            | Kind::FromGlobal(_)
            | Kind::Arg(_)
            | Kind::Command(_)
            | Kind::Value
            | Kind::ExternalSubcommand => {}
        }

        Ok(res)
    }

    fn new(
        name: Name,
        ident: Ident,
        ty: Option<Type>,
        casing: Sp<CasingStyle>,
        env_casing: Sp<CasingStyle>,
        kind: Sp<Kind>,
    ) -> Self {
        let group_id = Name::Derived(ident);
        Self {
            name,
            ty,
            casing,
            env_casing,
            doc_comment: vec![],
            methods: vec![],
            deprecations: vec![],
            value_parser: None,
            action: None,
            verbatim_doc_comment: false,
            force_long_help: false,
            next_display_order: None,
            next_help_heading: None,
            is_enum: false,
            is_positional: true,
            skip_group: false,
            group_id,
            group_methods: vec![],
            kind,
        }
    }

    fn push_method(&mut self, kind: AttrKind, name: Ident, arg: impl ToTokens) {
        self.push_method_(kind, name, arg.to_token_stream());
    }

    fn push_method_(&mut self, kind: AttrKind, name: Ident, arg: TokenStream) {
        if name == "id" {
            match kind {
                AttrKind::Command | AttrKind::Value => {
                    self.deprecations.push(Deprecation {
                        span: name.span(),
                        id: "id_is_only_for_arg",
                        version: "4.0.0",
                        description: format!(
                            "`#[{}(id)] was allowed by mistake, instead use `#[{}(name)]`",
                            kind.as_str(),
                            kind.as_str()
                        ),
                    });
                    self.name = Name::Assigned(arg);
                }
                AttrKind::Group => {
                    self.group_id = Name::Assigned(arg);
                }
                AttrKind::Arg | AttrKind::Clap | AttrKind::StructOpt => {
                    self.name = Name::Assigned(arg);
                }
            }
        } else if name == "name" {
            match kind {
                AttrKind::Arg => {
                    self.deprecations.push(Deprecation {
                        span: name.span(),
                        id: "id_is_only_for_arg",
                        version: "4.0.0",
                        description: format!(
                            "`#[{}(name)] was allowed by mistake, instead use `#[{}(id)]` or `#[{}(value_name)]`",
                            kind.as_str(),
                            kind.as_str(),
                            kind.as_str()
                        ),
                    });
                    self.name = Name::Assigned(arg);
                }
                AttrKind::Group => self.group_methods.push(Method::new(name, arg)),
                AttrKind::Command | AttrKind::Value | AttrKind::Clap | AttrKind::StructOpt => {
                    self.name = Name::Assigned(arg);
                }
            }
        } else if name == "value_parser" {
            self.value_parser = Some(ValueParser::Explicit(Method::new(name, arg)));
        } else if name == "action" {
            self.action = Some(Action::Explicit(Method::new(name, arg)));
        } else {
            if name == "short" || name == "long" {
                self.is_positional = false;
            }
            match kind {
                AttrKind::Group => self.group_methods.push(Method::new(name, arg)),
                _ => self.methods.push(Method::new(name, arg)),
            };
        }
    }

    fn infer_kind(&mut self, attrs: &[ClapAttr]) -> Result<(), syn::Error> {
        for attr in attrs {
            if let Some(AttrValue::Call(_)) = &attr.value {
                continue;
            }

            let actual_attr_kind = *attr.kind.get();
            let kind = match &attr.magic {
                Some(MagicAttrName::FromGlobal) => {
                    if attr.value.is_some() {
                        let expr = attr.value_or_abort()?;
                        abort!(expr, "attribute `{}` does not accept a value", attr.name);
                    }
                    let ty = self
                        .kind()
                        .ty()
                        .cloned()
                        .unwrap_or_else(|| Sp::new(Ty::Other, self.kind.span()));
                    let kind = Sp::new(Kind::FromGlobal(ty), attr.name.clone().span());
                    Some(kind)
                }
                Some(MagicAttrName::Subcommand) if attr.value.is_none() => {
                    if attr.value.is_some() {
                        let expr = attr.value_or_abort()?;
                        abort!(expr, "attribute `{}` does not accept a value", attr.name);
                    }
                    let ty = self
                        .kind()
                        .ty()
                        .cloned()
                        .unwrap_or_else(|| Sp::new(Ty::Other, self.kind.span()));
                    let kind = Sp::new(Kind::Subcommand(ty), attr.name.clone().span());
                    Some(kind)
                }
                Some(MagicAttrName::ExternalSubcommand) if attr.value.is_none() => {
                    if attr.value.is_some() {
                        let expr = attr.value_or_abort()?;
                        abort!(expr, "attribute `{}` does not accept a value", attr.name);
                    }
                    let kind = Sp::new(Kind::ExternalSubcommand, attr.name.clone().span());
                    Some(kind)
                }
                Some(MagicAttrName::Flatten) if attr.value.is_none() => {
                    if attr.value.is_some() {
                        let expr = attr.value_or_abort()?;
                        abort!(expr, "attribute `{}` does not accept a value", attr.name);
                    }
                    let ty = self
                        .kind()
                        .ty()
                        .cloned()
                        .unwrap_or_else(|| Sp::new(Ty::Other, self.kind.span()));
                    let kind = Sp::new(Kind::Flatten(ty), attr.name.clone().span());
                    Some(kind)
                }
                Some(MagicAttrName::Skip) if actual_attr_kind != AttrKind::Group => {
                    let expr = attr.value.clone();
                    let kind = Sp::new(
                        Kind::Skip(expr, self.kind.attr_kind()),
                        attr.name.clone().span(),
                    );
                    Some(kind)
                }
                _ => None,
            };

            if let Some(kind) = kind {
                self.set_kind(kind)?;
            }
        }

        Ok(())
    }

    fn push_attrs(&mut self, attrs: &[ClapAttr]) -> Result<(), syn::Error> {
        for attr in attrs {
            let actual_attr_kind = *attr.kind.get();
            let expected_attr_kind = self.kind.attr_kind();
            match (actual_attr_kind, expected_attr_kind) {
                (AttrKind::Clap, _) | (AttrKind::StructOpt, _) => {
                    self.deprecations.push(Deprecation::attribute(
                        "4.0.0",
                        actual_attr_kind,
                        expected_attr_kind,
                        attr.kind.span(),
                    ));
                }

                (AttrKind::Group, AttrKind::Command) => {}

                _ if attr.kind != expected_attr_kind => {
                    abort!(
                        attr.kind.span(),
                        "Expected `{}` attribute instead of `{}`",
                        expected_attr_kind.as_str(),
                        actual_attr_kind.as_str()
                    );
                }

                _ => {}
            }

            if let Some(AttrValue::Call(tokens)) = &attr.value {
                // Force raw mode with method call syntax
                self.push_method(*attr.kind.get(), attr.name.clone(), quote!(#(#tokens),*));
                continue;
            }

            match &attr.magic {
                Some(MagicAttrName::Short) if attr.value.is_none() => {
                    assert_attr_kind(attr, &[AttrKind::Arg])?;

                    self.push_method(
                        *attr.kind.get(),
                        attr.name.clone(),
                        self.name.clone().translate_char(*self.casing),
                    );
                }

                Some(MagicAttrName::Long) if attr.value.is_none() => {
                    assert_attr_kind(attr, &[AttrKind::Arg])?;

                    self.push_method(*attr.kind.get(), attr.name.clone(), self.name.clone().translate(*self.casing));
                }

                Some(MagicAttrName::ValueParser) if attr.value.is_none() => {
                    assert_attr_kind(attr, &[AttrKind::Arg])?;

                    self.deprecations.push(Deprecation {
                        span: attr.name.span(),
                        id: "bare_value_parser",
                        version: "4.0.0",
                        description: "`#[arg(value_parser)]` is now the default and is no longer needed`".to_owned(),
                    });
                    self.value_parser = Some(ValueParser::Implicit(attr.name.clone()));
                }

                Some(MagicAttrName::Action) if attr.value.is_none() => {
                    assert_attr_kind(attr, &[AttrKind::Arg])?;

                    self.deprecations.push(Deprecation {
                        span: attr.name.span(),
                        id: "bare_action",
                        version: "4.0.0",
                        description: "`#[arg(action)]` is now the default and is no longer needed`".to_owned(),
                    });
                    self.action = Some(Action::Implicit(attr.name.clone()));
                }

                Some(MagicAttrName::Env) if attr.value.is_none() => {
                    assert_attr_kind(attr, &[AttrKind::Arg])?;

                    self.push_method(
                        *attr.kind.get(),
                        attr.name.clone(),
                        self.name.clone().translate(*self.env_casing),
                    );
                }

                Some(MagicAttrName::ValueEnum) if attr.value.is_none() => {
                    assert_attr_kind(attr, &[AttrKind::Arg])?;

                    self.is_enum = true
                }

                Some(MagicAttrName::VerbatimDocComment) if attr.value.is_none() => {
                    self.verbatim_doc_comment = true
                }

                Some(MagicAttrName::About) if attr.value.is_none() => {
                    assert_attr_kind(attr, &[AttrKind::Command])?;

                    if let Some(method) =
                        Method::from_env(attr.name.clone(), "CARGO_PKG_DESCRIPTION")?
                    {
                        self.methods.push(method);
                    }
                }

                Some(MagicAttrName::LongAbout) if attr.value.is_none() => {
                    assert_attr_kind(attr, &[AttrKind::Command])?;

                    self.force_long_help = true;
                }

                Some(MagicAttrName::LongHelp) if attr.value.is_none() => {
                    assert_attr_kind(attr, &[AttrKind::Arg])?;

                    self.force_long_help = true;
                }

                Some(MagicAttrName::Author) if attr.value.is_none() => {
                    assert_attr_kind(attr, &[AttrKind::Command])?;

                    if let Some(method) = Method::from_env(attr.name.clone(), "CARGO_PKG_AUTHORS")? {
                        self.methods.push(method);
                    }
                }

                Some(MagicAttrName::Version) if attr.value.is_none() => {
                    assert_attr_kind(attr, &[AttrKind::Command])?;

                    if let Some(method) = Method::from_env(attr.name.clone(), "CARGO_PKG_VERSION")? {
                        self.methods.push(method);
                    }
                }

                Some(MagicAttrName::DefaultValueT) => {
                    assert_attr_kind(attr, &[AttrKind::Arg])?;

                    let ty = if let Some(ty) = self.ty.as_ref() {
                        ty
                    } else {
                        abort!(
                            attr.name.clone(),
                            "#[arg(default_value_t)] (without an argument) can be used \
                            only on field level\n\n= note: {note}\n\n",

                            note = "see \
                                https://github.com/clap-rs/clap/blob/master/examples/derive_ref/README.md#magic-attributes")
                    };

                    let val = if let Some(expr) = &attr.value {
                        quote!(#expr)
                    } else {
                        quote!(<#ty as ::std::default::Default>::default())
                    };

                    let val = if attrs
                        .iter()
                        .any(|a| a.magic == Some(MagicAttrName::ValueEnum))
                    {
                        quote_spanned!(attr.name.clone().span()=> {
                            static DEFAULT_VALUE: clap::__derive_refs::once_cell::sync::Lazy<String> = clap::__derive_refs::once_cell::sync::Lazy::new(|| {
                                let val: #ty = #val;
                                clap::ValueEnum::to_possible_value(&val).unwrap().get_name().to_owned()
                            });
                            let s: &'static str = &*DEFAULT_VALUE;
                            s
                        })
                    } else {
                        quote_spanned!(attr.name.clone().span()=> {
                            static DEFAULT_VALUE: clap::__derive_refs::once_cell::sync::Lazy<String> = clap::__derive_refs::once_cell::sync::Lazy::new(|| {
                                let val: #ty = #val;
                                ::std::string::ToString::to_string(&val)
                            });
                            let s: &'static str = &*DEFAULT_VALUE;
                            s
                        })
                    };

                    let raw_ident = Ident::new("default_value", attr.name.clone().span());
                    self.methods.push(Method::new(raw_ident, val));
                }

                Some(MagicAttrName::DefaultValuesT) => {
                    assert_attr_kind(attr, &[AttrKind::Arg])?;

                    let ty = if let Some(ty) = self.ty.as_ref() {
                        ty
                    } else {
                        abort!(
                            attr.name.clone(),
                            "#[arg(default_values_t)] (without an argument) can be used \
                            only on field level\n\n= note: {note}\n\n",

                            note = "see \
                                https://github.com/clap-rs/clap/blob/master/examples/derive_ref/README.md#magic-attributes")
                    };
                    let expr = attr.value_or_abort()?;

                    let container_type = Ty::from_syn_ty(ty);
                    if *container_type != Ty::Vec {
                        abort!(
                            attr.name.clone(),
                            "#[arg(default_values_t)] can be used only on Vec types\n\n= note: {note}\n\n",

                            note = "see \
                                https://github.com/clap-rs/clap/blob/master/examples/derive_ref/README.md#magic-attributes")
                    }
                    let inner_type = inner_type(ty);

                    // Use `Borrow<#inner_type>` so we accept `&Vec<#inner_type>` and
                    // `Vec<#inner_type>`.
                    let val = if attrs
                        .iter()
                        .any(|a| a.magic == Some(MagicAttrName::ValueEnum))
                    {
                        quote_spanned!(attr.name.clone().span()=> {
                            {
                                fn iter_to_vals<T>(iterable: impl IntoIterator<Item = T>) -> impl Iterator<Item=String>
                                where
                                    T: ::std::borrow::Borrow<#inner_type>
                                {
                                    iterable
                                        .into_iter()
                                        .map(|val| {
                                            clap::ValueEnum::to_possible_value(val.borrow()).unwrap().get_name().to_owned()
                                        })
                                }

                                static DEFAULT_STRINGS: clap::__derive_refs::once_cell::sync::Lazy<Vec<::std::string::String>> = clap::__derive_refs::once_cell::sync::Lazy::new(|| {
                                    iter_to_vals(#expr).collect()
                                });

                                static DEFAULT_VALUES: clap::__derive_refs::once_cell::sync::Lazy<Vec<&str>> = clap::__derive_refs::once_cell::sync::Lazy::new(|| {
                                    DEFAULT_STRINGS.iter().map(::std::string::String::as_str).collect()
                                });
                                DEFAULT_VALUES.iter().copied()
                            }
                        })
                    } else {
                        quote_spanned!(attr.name.clone().span()=> {
                            {
                                fn iter_to_vals<T>(iterable: impl IntoIterator<Item = T>) -> impl Iterator<Item=String>
                                where
                                    T: ::std::borrow::Borrow<#inner_type>
                                {
                                    iterable.into_iter().map(|val| val.borrow().to_string())
                                }

                                static DEFAULT_STRINGS: clap::__derive_refs::once_cell::sync::Lazy<Vec<::std::string::String>> = clap::__derive_refs::once_cell::sync::Lazy::new(|| {
                                    iter_to_vals(#expr).collect()
                                });

                                static DEFAULT_VALUES: clap::__derive_refs::once_cell::sync::Lazy<Vec<&str>> = clap::__derive_refs::once_cell::sync::Lazy::new(|| {
                                    DEFAULT_STRINGS.iter().map(::std::string::String::as_str).collect()
                                });
                                DEFAULT_VALUES.iter().copied()
                            }
                        })
                    };

                    self.methods.push(Method::new(
                        Ident::new("default_values", attr.name.clone().span()),
                        val,
                    ));
                }

                Some(MagicAttrName::DefaultValueOsT) => {
                    assert_attr_kind(attr, &[AttrKind::Arg])?;

                    let ty = if let Some(ty) = self.ty.as_ref() {
                        ty
                    } else {
                        abort!(
                            attr.name.clone(),
                            "#[arg(default_value_os_t)] (without an argument) can be used \
                            only on field level\n\n= note: {note}\n\n",

                            note = "see \
                                https://github.com/clap-rs/clap/blob/master/examples/derive_ref/README.md#magic-attributes")
                    };

                    let val = if let Some(expr) = &attr.value {
                        quote!(#expr)
                    } else {
                        quote!(<#ty as ::std::default::Default>::default())
                    };

                    let val = if attrs
                        .iter()
                        .any(|a| a.magic == Some(MagicAttrName::ValueEnum))
                    {
                        quote_spanned!(attr.name.clone().span()=> {
                            static DEFAULT_VALUE: clap::__derive_refs::once_cell::sync::Lazy<::std::ffi::OsString> = clap::__derive_refs::once_cell::sync::Lazy::new(|| {
                                let val: #ty = #val;
                                clap::ValueEnum::to_possible_value(&val).unwrap().get_name().to_owned()
                            });
                            let s: &'static ::std::ffi::OsStr = &*DEFAULT_VALUE;
                            s
                        })
                    } else {
                        quote_spanned!(attr.name.clone().span()=> {
                            static DEFAULT_VALUE: clap::__derive_refs::once_cell::sync::Lazy<::std::ffi::OsString> = clap::__derive_refs::once_cell::sync::Lazy::new(|| {
                                let val: #ty = #val;
                                ::std::ffi::OsString::from(val)
                            });
                            let s: &'static ::std::ffi::OsStr = &*DEFAULT_VALUE;
                            s
                        })
                    };

                    let raw_ident = Ident::new("default_value", attr.name.clone().span());
                    self.methods.push(Method::new(raw_ident, val));
                }

                Some(MagicAttrName::DefaultValuesOsT) => {
                    assert_attr_kind(attr, &[AttrKind::Arg])?;

                    let ty = if let Some(ty) = self.ty.as_ref() {
                        ty
                    } else {
                        abort!(
                            attr.name.clone(),
                            "#[arg(default_values_os_t)] (without an argument) can be used \
                            only on field level\n\n= note: {note}\n\n",

                            note = "see \
                                https://github.com/clap-rs/clap/blob/master/examples/derive_ref/README.md#magic-attributes")
                    };
                    let expr = attr.value_or_abort()?;

                    let container_type = Ty::from_syn_ty(ty);
                    if *container_type != Ty::Vec {
                        abort!(
                            attr.name.clone(),
                            "#[arg(default_values_os_t)] can be used only on Vec types\n\n= note: {note}\n\n",

                            note = "see \
                                https://github.com/clap-rs/clap/blob/master/examples/derive_ref/README.md#magic-attributes")
                    }
                    let inner_type = inner_type(ty);

                    // Use `Borrow<#inner_type>` so we accept `&Vec<#inner_type>` and
                    // `Vec<#inner_type>`.
                    let val = if attrs
                        .iter()
                        .any(|a| a.magic == Some(MagicAttrName::ValueEnum))
                    {
                        quote_spanned!(attr.name.clone().span()=> {
                            {
                                fn iter_to_vals<T>(iterable: impl IntoIterator<Item = T>) -> impl Iterator<Item=::std::ffi::OsString>
                                where
                                    T: ::std::borrow::Borrow<#inner_type>
                                {
                                    iterable
                                        .into_iter()
                                        .map(|val| {
                                            clap::ValueEnum::to_possible_value(val.borrow()).unwrap().get_name().to_owned().into()
                                        })
                                }

                                static DEFAULT_OS_STRINGS: clap::__derive_refs::once_cell::sync::Lazy<Vec<::std::ffi::OsString>> = clap::__derive_refs::once_cell::sync::Lazy::new(|| {
                                    iter_to_vals(#expr).collect()
                                });

                                static DEFAULT_VALUES: clap::__derive_refs::once_cell::sync::Lazy<Vec<&::std::ffi::OsStr>> = clap::__derive_refs::once_cell::sync::Lazy::new(|| {
                                    DEFAULT_OS_STRINGS.iter().map(::std::ffi::OsString::as_os_str).collect()
                                });
                                DEFAULT_VALUES.iter().copied()
                            }
                        })
                    } else {
                        quote_spanned!(attr.name.clone().span()=> {
                            {
                                fn iter_to_vals<T>(iterable: impl IntoIterator<Item = T>) -> impl Iterator<Item=::std::ffi::OsString>
                                where
                                    T: ::std::borrow::Borrow<#inner_type>
                                {
                                    iterable.into_iter().map(|val| val.borrow().into())
                                }

                                static DEFAULT_OS_STRINGS: clap::__derive_refs::once_cell::sync::Lazy<Vec<::std::ffi::OsString>> = clap::__derive_refs::once_cell::sync::Lazy::new(|| {
                                    iter_to_vals(#expr).collect()
                                });

                                static DEFAULT_VALUES: clap::__derive_refs::once_cell::sync::Lazy<Vec<&::std::ffi::OsStr>> = clap::__derive_refs::once_cell::sync::Lazy::new(|| {
                                    DEFAULT_OS_STRINGS.iter().map(::std::ffi::OsString::as_os_str).collect()
                                });
                                DEFAULT_VALUES.iter().copied()
                            }
                        })
                    };

                    self.methods.push(Method::new(
                        Ident::new("default_values", attr.name.clone().span()),
                        val,
                    ));
                }

                Some(MagicAttrName::NextDisplayOrder) => {
                    assert_attr_kind(attr, &[AttrKind::Command])?;

                    let expr = attr.value_or_abort()?;
                    self.next_display_order = Some(Method::new(attr.name.clone(), quote!(#expr)));
                }

                Some(MagicAttrName::NextHelpHeading) => {
                    assert_attr_kind(attr, &[AttrKind::Command])?;

                    let expr = attr.value_or_abort()?;
                    self.next_help_heading = Some(Method::new(attr.name.clone(), quote!(#expr)));
                }

                Some(MagicAttrName::RenameAll) => {
                    let lit = attr.lit_str_or_abort()?;
                    self.casing = CasingStyle::from_lit(lit)?;
                }

                Some(MagicAttrName::RenameAllEnv) => {
                    assert_attr_kind(attr, &[AttrKind::Command, AttrKind::Arg])?;

                    let lit = attr.lit_str_or_abort()?;
                    self.env_casing = CasingStyle::from_lit(lit)?;
                }

                Some(MagicAttrName::Skip) if actual_attr_kind == AttrKind::Group => {
                    self.skip_group = true;
                }

                None
                // Magic only for the default, otherwise just forward to the builder
                | Some(MagicAttrName::Short)
                | Some(MagicAttrName::Long)
                | Some(MagicAttrName::Env)
                | Some(MagicAttrName::About)
                | Some(MagicAttrName::LongAbout)
                | Some(MagicAttrName::LongHelp)
                | Some(MagicAttrName::Author)
                | Some(MagicAttrName::Version)
                 => {
                    let expr = attr.value_or_abort()?;
                    self.push_method(*attr.kind.get(), attr.name.clone(), expr);
                }

                // Magic only for the default, otherwise just forward to the builder
                Some(MagicAttrName::ValueParser) | Some(MagicAttrName::Action) => {
                    let expr = attr.value_or_abort()?;
                    self.push_method(*attr.kind.get(), attr.name.clone(), expr);
                }

                // Directives that never receive a value
                Some(MagicAttrName::ValueEnum)
                | Some(MagicAttrName::VerbatimDocComment) => {
                    let expr = attr.value_or_abort()?;
                    abort!(expr, "attribute `{}` does not accept a value", attr.name);
                }

                // Kinds
                Some(MagicAttrName::FromGlobal)
                | Some(MagicAttrName::Subcommand)
                | Some(MagicAttrName::ExternalSubcommand)
                | Some(MagicAttrName::Flatten)
                | Some(MagicAttrName::Skip) => {
                }
            }
        }

        if self.has_explicit_methods() {
            if let Kind::Skip(_, attr) = &*self.kind {
                abort!(
                    self.methods[0].name.span(),
                    "`{}` cannot be used with `#[{}(skip)]",
                    self.methods[0].name,
                    attr.as_str(),
                );
            }
            if let Kind::FromGlobal(_) = &*self.kind {
                abort!(
                    self.methods[0].name.span(),
                    "`{}` cannot be used with `#[arg(from_global)]",
                    self.methods[0].name,
                );
            }
        }

        Ok(())
    }

    fn push_doc_comment(&mut self, attrs: &[Attribute], short_name: &str, long_name: Option<&str>) {
        let lines = extract_doc_comment(attrs);

        if !lines.is_empty() {
            let (short_help, long_help) =
                format_doc_comment(&lines, !self.verbatim_doc_comment, self.force_long_help);
            let short_name = format_ident!("{short_name}");
            let short = Method::new(
                short_name,
                short_help
                    .map(|h| quote!(#h))
                    .unwrap_or_else(|| quote!(None)),
            );
            self.doc_comment.push(short);
            if let Some(long_name) = long_name {
                let long_name = format_ident!("{long_name}");
                let long = Method::new(
                    long_name,
                    long_help
                        .map(|h| quote!(#h))
                        .unwrap_or_else(|| quote!(None)),
                );
                self.doc_comment.push(long);
            }
        }
    }

    fn set_kind(&mut self, kind: Sp<Kind>) -> Result<(), syn::Error> {
        match (self.kind.get(), kind.get()) {
            (Kind::Arg(_), Kind::FromGlobal(_))
            | (Kind::Arg(_), Kind::Subcommand(_))
            | (Kind::Arg(_), Kind::Flatten(_))
            | (Kind::Arg(_), Kind::Skip(_, _))
            | (Kind::Command(_), Kind::Subcommand(_))
            | (Kind::Command(_), Kind::Flatten(_))
            | (Kind::Command(_), Kind::Skip(_, _))
            | (Kind::Command(_), Kind::ExternalSubcommand)
            | (Kind::Value, Kind::Skip(_, _)) => {
                self.kind = kind;
            }

            (_, _) => {
                let old = self.kind.name();
                let new = kind.name();
                abort!(kind.span(), "`{new}` cannot be used with `{old}`");
            }
        }
        Ok(())
    }

    pub fn find_default_method(&self) -> Option<&Method> {
        self.methods
            .iter()
            .find(|m| m.name == "default_value" || m.name == "default_value_os")
    }

    /// generate methods from attributes on top of struct or enum
    pub fn initial_top_level_methods(&self) -> TokenStream {
        let next_display_order = self.next_display_order.as_ref().into_iter();
        let next_help_heading = self.next_help_heading.as_ref().into_iter();
        quote!(
            #(#next_display_order)*
            #(#next_help_heading)*
        )
    }

    pub fn final_top_level_methods(&self) -> TokenStream {
        let methods = &self.methods;
        let doc_comment = &self.doc_comment;

        quote!( #(#doc_comment)* #(#methods)*)
    }

    /// generate methods on top of a field
    pub fn field_methods(&self) -> proc_macro2::TokenStream {
        let methods = &self.methods;
        let doc_comment = &self.doc_comment;
        quote!( #(#doc_comment)* #(#methods)* )
    }

    pub fn group_id(&self) -> TokenStream {
        self.group_id.clone().raw()
    }

    pub fn group_methods(&self) -> TokenStream {
        let group_methods = &self.group_methods;
        quote!( #(#group_methods)* )
    }

    pub fn deprecations(&self) -> proc_macro2::TokenStream {
        let deprecations = &self.deprecations;
        quote!( #(#deprecations)* )
    }

    pub fn next_display_order(&self) -> TokenStream {
        let next_display_order = self.next_display_order.as_ref().into_iter();
        quote!( #(#next_display_order)* )
    }

    pub fn next_help_heading(&self) -> TokenStream {
        let next_help_heading = self.next_help_heading.as_ref().into_iter();
        quote!( #(#next_help_heading)* )
    }

    pub fn id(&self) -> TokenStream {
        self.name.clone().raw()
    }

    pub fn cased_name(&self) -> TokenStream {
        self.name.clone().translate(*self.casing)
    }

    pub fn value_name(&self) -> TokenStream {
        self.name.clone().translate(CasingStyle::ScreamingSnake)
    }

    pub fn value_parser(&self, field_type: &Type) -> Method {
        self.value_parser
            .clone()
            .map(|p| {
                let inner_type = inner_type(field_type);
                p.resolve(inner_type)
            })
            .unwrap_or_else(|| {
                let inner_type = inner_type(field_type);
                if let Some(action) = self.action.as_ref() {
                    let span = action.span();
                    default_value_parser(inner_type, span)
                } else {
                    let span = self
                        .action
                        .as_ref()
                        .map(|a| a.span())
                        .unwrap_or_else(|| self.kind.span());
                    default_value_parser(inner_type, span)
                }
            })
    }

    pub fn action(&self, field_type: &Type) -> Method {
        self.action
            .clone()
            .map(|p| p.resolve(field_type))
            .unwrap_or_else(|| {
                if let Some(value_parser) = self.value_parser.as_ref() {
                    let span = value_parser.span();
                    default_action(field_type, span)
                } else {
                    let span = self
                        .value_parser
                        .as_ref()
                        .map(|a| a.span())
                        .unwrap_or_else(|| self.kind.span());
                    default_action(field_type, span)
                }
            })
    }

    pub fn kind(&self) -> Sp<Kind> {
        self.kind.clone()
    }

    pub fn is_positional(&self) -> bool {
        self.is_positional
    }

    pub fn casing(&self) -> Sp<CasingStyle> {
        self.casing
    }

    pub fn env_casing(&self) -> Sp<CasingStyle> {
        self.env_casing
    }

    pub fn has_explicit_methods(&self) -> bool {
        self.methods
            .iter()
            .any(|m| m.name != "help" && m.name != "long_help")
    }

    pub fn skip_group(&self) -> bool {
        self.skip_group
    }
}

#[derive(Clone)]
enum ValueParser {
    Explicit(Method),
    Implicit(Ident),
}

impl ValueParser {
    fn resolve(self, _inner_type: &Type) -> Method {
        match self {
            Self::Explicit(method) => method,
            Self::Implicit(ident) => default_value_parser(_inner_type, ident.span()),
        }
    }

    fn span(&self) -> Span {
        match self {
            Self::Explicit(method) => method.name.span(),
            Self::Implicit(ident) => ident.span(),
        }
    }
}

fn default_value_parser(inner_type: &Type, span: Span) -> Method {
    let func = Ident::new("value_parser", span);
    Method::new(
        func,
        quote_spanned! { span=>
            clap::value_parser!(#inner_type)
        },
    )
}

#[derive(Clone)]
pub enum Action {
    Explicit(Method),
    Implicit(Ident),
}

impl Action {
    pub fn resolve(self, _field_type: &Type) -> Method {
        match self {
            Self::Explicit(method) => method,
            Self::Implicit(ident) => default_action(_field_type, ident.span()),
        }
    }

    pub fn span(&self) -> Span {
        match self {
            Self::Explicit(method) => method.name.span(),
            Self::Implicit(ident) => ident.span(),
        }
    }
}

fn default_action(field_type: &Type, span: Span) -> Method {
    let ty = Ty::from_syn_ty(field_type);
    let args = match *ty {
        Ty::Vec | Ty::OptionVec | Ty::VecVec | Ty::OptionVecVec => {
            quote_spanned! { span=>
                clap::ArgAction::Append
            }
        }
        Ty::Option | Ty::OptionOption => {
            quote_spanned! { span=>
                clap::ArgAction::Set
            }
        }
        _ => {
            if is_simple_ty(field_type, "bool") {
                quote_spanned! { span=>
                    clap::ArgAction::SetTrue
                }
            } else {
                quote_spanned! { span=>
                    clap::ArgAction::Set
                }
            }
        }
    };

    let func = Ident::new("action", span);
    Method::new(func, args)
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
pub enum Kind {
    Arg(Sp<Ty>),
    Command(Sp<Ty>),
    Value,
    FromGlobal(Sp<Ty>),
    Subcommand(Sp<Ty>),
    Flatten(Sp<Ty>),
    Skip(Option<AttrValue>, AttrKind),
    ExternalSubcommand,
}

impl Kind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Arg(_) => "arg",
            Self::Command(_) => "command",
            Self::Value => "value",
            Self::FromGlobal(_) => "from_global",
            Self::Subcommand(_) => "subcommand",
            Self::Flatten(_) => "flatten",
            Self::Skip(_, _) => "skip",
            Self::ExternalSubcommand => "external_subcommand",
        }
    }

    pub fn attr_kind(&self) -> AttrKind {
        match self {
            Self::Arg(_) => AttrKind::Arg,
            Self::Command(_) => AttrKind::Command,
            Self::Value => AttrKind::Value,
            Self::FromGlobal(_) => AttrKind::Arg,
            Self::Subcommand(_) => AttrKind::Command,
            Self::Flatten(_) => AttrKind::Command,
            Self::Skip(_, kind) => *kind,
            Self::ExternalSubcommand => AttrKind::Command,
        }
    }

    pub fn ty(&self) -> Option<&Sp<Ty>> {
        match self {
            Self::Arg(ty)
            | Self::Command(ty)
            | Self::Flatten(ty)
            | Self::FromGlobal(ty)
            | Self::Subcommand(ty) => Some(ty),
            Self::Value | Self::Skip(_, _) | Self::ExternalSubcommand => None,
        }
    }
}

#[derive(Clone)]
pub struct Method {
    name: Ident,
    args: TokenStream,
}

impl Method {
    pub fn new(name: Ident, args: TokenStream) -> Self {
        Method { name, args }
    }

    fn from_env(ident: Ident, env_var: &str) -> Result<Option<Self>, syn::Error> {
        let mut lit = match env::var(env_var) {
            Ok(val) => {
                if val.is_empty() {
                    return Ok(None);
                }
                LitStr::new(&val, ident.span())
            }
            Err(_) => {
                abort!(
                    ident,
                    "cannot derive `{}` from Cargo.toml\n\n= note: {note}\n\n= help: {help}\n\n",
                    ident,
                    note = format_args!("`{env_var}` environment variable is not set"),
                    help = format_args!("use `{ident} = \"...\"` to set {ident} manually")
                );
            }
        };

        if ident == "author" {
            let edited = process_author_str(&lit.value());
            lit = LitStr::new(&edited, lit.span());
        }

        Ok(Some(Method::new(ident, quote!(#lit))))
    }

    pub(crate) fn args(&self) -> &TokenStream {
        &self.args
    }
}

impl ToTokens for Method {
    fn to_tokens(&self, ts: &mut proc_macro2::TokenStream) {
        let Method { ref name, ref args } = self;

        let tokens = quote!( .#name(#args) );

        tokens.to_tokens(ts);
    }
}

#[derive(Clone)]
pub struct Deprecation {
    pub span: Span,
    pub id: &'static str,
    pub version: &'static str,
    pub description: String,
}

impl Deprecation {
    fn attribute(version: &'static str, old: AttrKind, new: AttrKind, span: Span) -> Self {
        Self {
            span,
            id: "old_attribute",
            version,
            description: format!(
                "Attribute `#[{}(...)]` has been deprecated in favor of `#[{}(...)]`",
                old.as_str(),
                new.as_str()
            ),
        }
    }
}

impl ToTokens for Deprecation {
    fn to_tokens(&self, ts: &mut proc_macro2::TokenStream) {
        let tokens = if cfg!(feature = "deprecated") {
            let Deprecation {
                span,
                id,
                version,
                description,
            } = self;
            let span = *span;
            let id = Ident::new(id, span);

            quote_spanned!(span=> {
                #[deprecated(since = #version, note = #description)]
                fn #id() {}
                #id();
            })
        } else {
            quote!()
        };

        tokens.to_tokens(ts);
    }
}

fn assert_attr_kind(attr: &ClapAttr, possible_kind: &[AttrKind]) -> Result<(), syn::Error> {
    if *attr.kind.get() == AttrKind::Clap || *attr.kind.get() == AttrKind::StructOpt {
        // deprecated
    } else if !possible_kind.contains(attr.kind.get()) {
        let options = possible_kind
            .iter()
            .map(|k| format!("`#[{}({})]`", k.as_str(), attr.name))
            .collect::<Vec<_>>();
        abort!(
            attr.name,
            "Unknown `#[{}({})]` attribute ({} exists)",
            attr.kind.as_str(),
            attr.name,
            options.join(", ")
        );
    }
    Ok(())
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

/// Defines the casing for the attributes long representation.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
    /// Keep all letters lowercase and remove word boundaries.
    Lower,
    /// Keep all letters uppercase and remove word boundaries.
    Upper,
    /// Use the original attribute name defined in the code.
    Verbatim,
}

impl CasingStyle {
    fn from_lit(name: &LitStr) -> Result<Sp<Self>, syn::Error> {
        use self::CasingStyle::*;

        let normalized = name.value().to_upper_camel_case().to_lowercase();
        let cs = |kind| Sp::new(kind, name.span());

        let s = match normalized.as_ref() {
            "camel" | "camelcase" => cs(Camel),
            "kebab" | "kebabcase" => cs(Kebab),
            "pascal" | "pascalcase" => cs(Pascal),
            "screamingsnake" | "screamingsnakecase" => cs(ScreamingSnake),
            "snake" | "snakecase" => cs(Snake),
            "lower" | "lowercase" => cs(Lower),
            "upper" | "uppercase" => cs(Upper),
            "verbatim" | "verbatimcase" => cs(Verbatim),
            s => abort!(name, "unsupported casing: `{s}`"),
        };
        Ok(s)
    }
}

#[derive(Clone)]
pub enum Name {
    Derived(Ident),
    Assigned(TokenStream),
}

impl Name {
    pub fn raw(self) -> TokenStream {
        match self {
            Name::Assigned(tokens) => tokens,
            Name::Derived(ident) => {
                let s = ident.unraw().to_string();
                quote_spanned!(ident.span()=> #s)
            }
        }
    }

    pub fn translate(self, style: CasingStyle) -> TokenStream {
        use CasingStyle::*;

        match self {
            Name::Assigned(tokens) => tokens,
            Name::Derived(ident) => {
                let s = ident.unraw().to_string();
                let s = match style {
                    Pascal => s.to_upper_camel_case(),
                    Kebab => s.to_kebab_case(),
                    Camel => s.to_lower_camel_case(),
                    ScreamingSnake => s.to_shouty_snake_case(),
                    Snake => s.to_snake_case(),
                    Lower => s.to_snake_case().replace('_', ""),
                    Upper => s.to_shouty_snake_case().replace('_', ""),
                    Verbatim => s,
                };
                quote_spanned!(ident.span()=> #s)
            }
        }
    }

    pub fn translate_char(self, style: CasingStyle) -> TokenStream {
        use CasingStyle::*;

        match self {
            Name::Assigned(tokens) => quote!( (#tokens).chars().next().unwrap() ),
            Name::Derived(ident) => {
                let s = ident.unraw().to_string();
                let s = match style {
                    Pascal => s.to_upper_camel_case(),
                    Kebab => s.to_kebab_case(),
                    Camel => s.to_lower_camel_case(),
                    ScreamingSnake => s.to_shouty_snake_case(),
                    Snake => s.to_snake_case(),
                    Lower => s.to_snake_case(),
                    Upper => s.to_shouty_snake_case(),
                    Verbatim => s,
                };

                let s = s.chars().next().unwrap();
                quote_spanned!(ident.span()=> #s)
            }
        }
    }
}
