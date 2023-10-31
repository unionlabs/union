// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This crate is custom derive for `StructOpt`. It should not be used
//! directly. See [structopt documentation](https://docs.rs/structopt)
//! for the usage of `#[derive(StructOpt)]`.

#![allow(clippy::large_enum_variant)]
// FIXME: remove when and if our MSRV hits 1.42
#![allow(clippy::match_like_matches_macro)]
#![forbid(unsafe_code)]

extern crate proc_macro;

mod attrs;
mod doc_comments;
mod parse;
mod spanned;
mod ty;

use crate::{
    attrs::{Attrs, CasingStyle, Kind, Name, ParserKind},
    spanned::Sp,
    ty::{is_simple_ty, sub_type, subty_if_name, Ty},
};

use proc_macro2::{Span, TokenStream};
use proc_macro_error::{abort, abort_call_site, proc_macro_error, set_dummy};
use quote::{format_ident, quote, quote_spanned};
use syn::{punctuated::Punctuated, spanned::Spanned, token::Comma, *};

/// Default casing style for generated arguments.
const DEFAULT_CASING: CasingStyle = CasingStyle::Kebab;

/// Default casing style for environment variables
const DEFAULT_ENV_CASING: CasingStyle = CasingStyle::ScreamingSnake;

/// Output for the `gen_xxx()` methods were we need more than a simple stream of tokens.
///
/// The output of a generation method is not only the stream of new tokens but also the attribute
/// information of the current element. These attribute information may contain valuable information
/// for any kind of child arguments.
struct GenOutput {
    tokens: TokenStream,
    attrs: Attrs,
}

/// Generates the `StructOpt` impl.
#[proc_macro_derive(StructOpt, attributes(structopt))]
#[proc_macro_error]
pub fn structopt(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    let gen = impl_structopt(&input);
    gen.into()
}

/// Generate a block of code to add arguments/subcommands corresponding to
/// the `fields` to an app.
fn gen_augmentation(
    fields: &Punctuated<Field, Comma>,
    app_var: &Ident,
    parent_attribute: &Attrs,
) -> TokenStream {
    let mut subcmds = fields.iter().filter_map(|field| {
        let attrs = Attrs::from_field(
            field,
            Some(parent_attribute),
            parent_attribute.casing(),
            parent_attribute.env_casing(),
        );
        let kind = attrs.kind();
        if let Kind::Subcommand(ty) = &*kind {
            let subcmd_type = match (**ty, sub_type(&field.ty)) {
                (Ty::Option, Some(sub_type)) => sub_type,
                _ => &field.ty,
            };
            let required = if **ty == Ty::Option {
                quote!()
            } else {
                quote_spanned! { kind.span()=>
                    let #app_var = #app_var.setting(
                        ::structopt::clap::AppSettings::SubcommandRequiredElseHelp
                    );
                }
            };

            let span = field.span();
            let ts = quote! {
                let #app_var = <#subcmd_type as ::structopt::StructOptInternal>::augment_clap(
                    #app_var
                );
                #required
            };
            Some((span, ts))
        } else {
            None
        }
    });

    let subcmd = subcmds.next().map(|(_, ts)| ts);
    if let Some((span, _)) = subcmds.next() {
        abort!(
            span,
            "multiple subcommand sets are not allowed, that's the second"
        );
    }

    let args = fields.iter().filter_map(|field| {
        let attrs = Attrs::from_field(
            field,
            Some(parent_attribute),
            parent_attribute.casing(),
            parent_attribute.env_casing(),
        );
        let kind = attrs.kind();
        match &*kind {
            Kind::ExternalSubcommand => abort!(
                kind.span(),
                "`external_subcommand` is only allowed on enum variants"
            ),
            Kind::Subcommand(_) | Kind::Skip(_) => None,
            Kind::Flatten => {
                let ty = &field.ty;
                Some(quote_spanned! { kind.span()=>
                    let #app_var = <#ty as ::structopt::StructOptInternal>::augment_clap(#app_var);
                    let #app_var = if <#ty as ::structopt::StructOptInternal>::is_subcommand() {
                        #app_var.setting(::structopt::clap::AppSettings::SubcommandRequiredElseHelp)
                    } else {
                        #app_var
                    };
                })
            }
            Kind::Arg(ty) => {
                let convert_type = match **ty {
                    Ty::Vec | Ty::Option => sub_type(&field.ty).unwrap_or(&field.ty),
                    Ty::OptionOption | Ty::OptionVec => {
                        sub_type(&field.ty).and_then(sub_type).unwrap_or(&field.ty)
                    }
                    _ => &field.ty,
                };

                let occurrences = *attrs.parser().kind == ParserKind::FromOccurrences;
                let flag = *attrs.parser().kind == ParserKind::FromFlag;

                let parser = attrs.parser();
                let func = &parser.func;
                let validator = match *parser.kind {
                    ParserKind::TryFromStr => quote_spanned! { func.span()=>
                        .validator(|s| {
                            #func(s.as_str())
                            .map(|_: #convert_type| ())
                            .map_err(|e| e.to_string())
                        })
                    },
                    ParserKind::TryFromOsStr => quote_spanned! { func.span()=>
                        .validator_os(|s| #func(&s).map(|_: #convert_type| ()))
                    },
                    _ => quote!(),
                };

                let modifier = match **ty {
                    Ty::Bool => quote_spanned! { ty.span()=>
                        .takes_value(false)
                        .multiple(false)
                    },

                    Ty::Option => quote_spanned! { ty.span()=>
                        .takes_value(true)
                        .multiple(false)
                        #validator
                    },

                    Ty::OptionOption => quote_spanned! { ty.span()=>
                            .takes_value(true)
                            .multiple(false)
                            .min_values(0)
                            .max_values(1)
                            #validator
                    },

                    Ty::OptionVec => quote_spanned! { ty.span()=>
                        .takes_value(true)
                        .multiple(true)
                        .min_values(0)
                        #validator
                    },

                    Ty::Vec => quote_spanned! { ty.span()=>
                        .takes_value(true)
                        .multiple(true)
                        #validator
                    },

                    Ty::Other if occurrences => quote_spanned! { ty.span()=>
                        .takes_value(false)
                        .multiple(true)
                    },

                    Ty::Other if flag => quote_spanned! { ty.span()=>
                        .takes_value(false)
                        .multiple(false)
                    },

                    Ty::Other => {
                        let required = !attrs.has_method("default_value");
                        quote_spanned! { ty.span()=>
                            .takes_value(true)
                            .multiple(false)
                            .required(#required)
                            #validator
                        }
                    }
                };

                let name = attrs.cased_name();
                let methods = attrs.field_methods();

                Some(quote_spanned! { field.span()=>
                    let #app_var = #app_var.arg(
                        ::structopt::clap::Arg::with_name(#name)
                            #modifier
                            #methods
                    );
                })
            }
        }
    });

    let app_methods = parent_attribute.top_level_methods();
    let version = parent_attribute.version();
    quote! {{
        let #app_var = #app_var#app_methods;
        #( #args )*
        #subcmd
        #app_var#version
    }}
}

fn gen_constructor(fields: &Punctuated<Field, Comma>, parent_attribute: &Attrs) -> TokenStream {
    // This ident is used in several match branches below,
    // and the `quote[_spanned]` invocations have different spans.
    //
    // Given that this ident is used in several places and
    // that the branches are located inside of a loop, it is possible that
    // this ident will be given _different_ spans in different places, and
    // thus will not be the _same_ ident anymore. To make sure the `matches`
    // is always the same, we factor it out.
    let matches = format_ident!("matches");

    let fields = fields.iter().map(|field| {
        let attrs = Attrs::from_field(
            field,
            Some(parent_attribute),
            parent_attribute.casing(),
            parent_attribute.env_casing(),
        );
        let field_name = field.ident.as_ref().unwrap();
        let kind = attrs.kind();
        match &*kind {
            Kind::ExternalSubcommand => abort!(
                kind.span(),
                "`external_subcommand` is allowed only on enum variants"
            ),

            Kind::Subcommand(ty) => {
                let subcmd_type = match (**ty, sub_type(&field.ty)) {
                    (Ty::Option, Some(sub_type)) => sub_type,
                    _ => &field.ty,
                };
                let unwrapper = match **ty {
                    Ty::Option => quote!(),
                    _ => quote_spanned!( ty.span()=> .unwrap() ),
                };
                quote_spanned! { kind.span()=>
                    #field_name: <#subcmd_type as ::structopt::StructOptInternal>::from_subcommand(
                        #matches.subcommand())
                        #unwrapper
                }
            }

            Kind::Flatten => quote_spanned! { kind.span()=>
                #field_name: ::structopt::StructOpt::from_clap(#matches)
            },

            Kind::Skip(val) => match val {
                None => quote_spanned!(kind.span()=> #field_name: Default::default()),
                Some(val) => quote_spanned!(kind.span()=> #field_name: (#val).into()),
            },

            Kind::Arg(ty) => {
                use crate::attrs::ParserKind::*;

                let parser = attrs.parser();
                let func = &parser.func;
                let span = parser.kind.span();
                let (value_of, values_of, parse) = match *parser.kind {
                    FromStr => (
                        quote_spanned!(span=> value_of),
                        quote_spanned!(span=> values_of),
                        func.clone(),
                    ),
                    TryFromStr => (
                        quote_spanned!(span=> value_of),
                        quote_spanned!(span=> values_of),
                        quote_spanned!(func.span()=> |s| #func(s).unwrap()),
                    ),
                    FromOsStr => (
                        quote_spanned!(span=> value_of_os),
                        quote_spanned!(span=> values_of_os),
                        func.clone(),
                    ),
                    TryFromOsStr => (
                        quote_spanned!(span=> value_of_os),
                        quote_spanned!(span=> values_of_os),
                        quote_spanned!(func.span()=> |s| #func(s).unwrap()),
                    ),
                    FromOccurrences => (
                        quote_spanned!(span=> occurrences_of),
                        quote!(),
                        func.clone(),
                    ),
                    FromFlag => (quote!(), quote!(), func.clone()),
                };

                let flag = *attrs.parser().kind == ParserKind::FromFlag;
                let occurrences = *attrs.parser().kind == ParserKind::FromOccurrences;
                let name = attrs.cased_name();
                let convert_type = match **ty {
                    Ty::Vec | Ty::Option => sub_type(&field.ty).unwrap_or(&field.ty),
                    Ty::OptionOption | Ty::OptionVec => {
                        sub_type(&field.ty).and_then(sub_type).unwrap_or(&field.ty)
                    }
                    _ => &field.ty,
                };
                let field_value = match **ty {
                    Ty::Bool => quote_spanned!(ty.span()=> #matches.is_present(#name)),

                    Ty::Option => quote_spanned! { ty.span()=>
                        #matches.#value_of(#name)
                            .map(#parse)
                    },

                    Ty::OptionOption => quote_spanned! { ty.span()=>
                        if #matches.is_present(#name) {
                            Some(#matches.#value_of(#name).map(#parse))
                        } else {
                            None
                        }
                    },

                    Ty::OptionVec => quote_spanned! { ty.span()=>
                        if #matches.is_present(#name) {
                            Some(#matches.#values_of(#name)
                                 .map_or_else(Vec::new, |v| v.map::<#convert_type, _>(#parse).collect()))
                        } else {
                            None
                        }
                    },

                    Ty::Vec => quote_spanned! { ty.span()=>
                        #matches.#values_of(#name)
                            .map_or_else(Vec::new, |v| v.map::<#convert_type, _>(#parse).collect())
                    },

                    Ty::Other if occurrences => quote_spanned! { ty.span()=>
                        #parse(#matches.#value_of(#name))
                    },

                    Ty::Other if flag => quote_spanned! { ty.span()=>
                        #parse(#matches.is_present(#name))
                    },

                    Ty::Other => quote_spanned! { ty.span()=>
                        #matches.#value_of(#name)
                            .map(#parse)
                            .unwrap()
                    },
                };

                quote_spanned!(field.span()=> #field_name: #field_value )
            }
        }
    });

    quote! {{
        #( #fields ),*
    }}
}

fn gen_from_clap(
    struct_name: &Ident,
    fields: &Punctuated<Field, Comma>,
    parent_attribute: &Attrs,
) -> TokenStream {
    let field_block = gen_constructor(fields, parent_attribute);

    quote! {
        fn from_clap(matches: &::structopt::clap::ArgMatches) -> Self {
            #struct_name #field_block
        }
    }
}

fn gen_clap(attrs: &[Attribute]) -> GenOutput {
    let name = std::env::var("CARGO_PKG_NAME").ok().unwrap_or_default();

    let attrs = Attrs::from_struct(
        Span::call_site(),
        attrs,
        Name::Assigned(quote!(#name)),
        None,
        Sp::call_site(DEFAULT_CASING),
        Sp::call_site(DEFAULT_ENV_CASING),
        false,
    );
    let tokens = {
        let name = attrs.cased_name();
        quote!(::structopt::clap::App::new(#name))
    };

    GenOutput { tokens, attrs }
}

fn gen_clap_struct(struct_attrs: &[Attribute]) -> GenOutput {
    let initial_clap_app_gen = gen_clap(struct_attrs);
    let clap_tokens = initial_clap_app_gen.tokens;

    let augmented_tokens = quote! {
        fn clap<'a, 'b>() -> ::structopt::clap::App<'a, 'b> {
            let app = #clap_tokens;
            <Self as ::structopt::StructOptInternal>::augment_clap(app)
        }
    };

    GenOutput {
        tokens: augmented_tokens,
        attrs: initial_clap_app_gen.attrs,
    }
}

fn gen_augment_clap(fields: &Punctuated<Field, Comma>, parent_attribute: &Attrs) -> TokenStream {
    let app_var = Ident::new("app", Span::call_site());
    let augmentation = gen_augmentation(fields, &app_var, parent_attribute);
    quote! {
        fn augment_clap<'a, 'b>(
            #app_var: ::structopt::clap::App<'a, 'b>
        ) -> ::structopt::clap::App<'a, 'b> {
            #augmentation
        }
    }
}

fn gen_clap_enum(enum_attrs: &[Attribute]) -> GenOutput {
    let initial_clap_app_gen = gen_clap(enum_attrs);
    let clap_tokens = initial_clap_app_gen.tokens;

    let tokens = quote! {
        fn clap<'a, 'b>() -> ::structopt::clap::App<'a, 'b> {
            let app = #clap_tokens
                .setting(::structopt::clap::AppSettings::SubcommandRequiredElseHelp);
            <Self as ::structopt::StructOptInternal>::augment_clap(app)
        }
    };

    GenOutput {
        tokens,
        attrs: initial_clap_app_gen.attrs,
    }
}

fn gen_augment_clap_enum(
    variants: &Punctuated<Variant, Comma>,
    parent_attribute: &Attrs,
) -> TokenStream {
    use syn::Fields::*;

    let subcommands = variants.iter().filter_map(|variant| {
        let attrs = Attrs::from_struct(
            variant.span(),
            &variant.attrs,
            Name::Derived(variant.ident.clone()),
            Some(parent_attribute),
            parent_attribute.casing(),
            parent_attribute.env_casing(),
            true,
        );

        let kind = attrs.kind();
        match &*kind {
            Kind::Skip(_) => None,

            Kind::ExternalSubcommand => {
                let app_var = Ident::new("app", Span::call_site());
                Some(quote_spanned! { attrs.kind().span()=>
                    let #app_var = #app_var.setting(
                        ::structopt::clap::AppSettings::AllowExternalSubcommands
                    );
                })
            },

            Kind::Flatten => {
                match variant.fields {
                    Unnamed(FieldsUnnamed { ref unnamed, .. }) if unnamed.len() == 1 => {
                        let ty = &unnamed[0];
                        Some(quote! {
                            let app = <#ty as ::structopt::StructOptInternal>::augment_clap(app);
                        })
                    },
                    _ => abort!(
                        variant,
                        "`flatten` is usable only with single-typed tuple variants"
                    ),
                }
            },

            _ => {
                let app_var = Ident::new("subcommand", Span::call_site());
                let from_attrs = attrs.top_level_methods();
                let version = attrs.version();

                let arg_block = match variant.fields {
                    // If the variant is named, then gen_augmentation already generates the
                    // top level methods (#from_attrs) and version.
                    Named(ref fields) => gen_augmentation(&fields.named, &app_var, &attrs),
                    Unit => quote!( #app_var#from_attrs#version ),
                    Unnamed(FieldsUnnamed { ref unnamed, .. }) if unnamed.len() == 1 => {
                        let ty = &unnamed[0];
                        quote_spanned! { ty.span()=>
                            {
                                let #app_var = <#ty as ::structopt::StructOptInternal>::augment_clap(
                                    #app_var
                                );
                                if <#ty as ::structopt::StructOptInternal>::is_subcommand() {
                                    #app_var.setting(
                                        ::structopt::clap::AppSettings::SubcommandRequiredElseHelp
                                    )
                                } else {
                                    #app_var
                                }#from_attrs#version
                            }
                        }
                    }
                    Unnamed(..) => abort!(variant, "non single-typed tuple enums are not supported"),
                };

                let name = attrs.cased_name();
                Some(quote! {
                    let app = app.subcommand({
                        let #app_var = ::structopt::clap::SubCommand::with_name(#name);
                        #arg_block
                    });
                })
            },
        }
    });

    let app_methods = parent_attribute.top_level_methods();
    let version = parent_attribute.version();
    quote! {
        fn augment_clap<'a, 'b>(
            app: ::structopt::clap::App<'a, 'b>
        ) -> ::structopt::clap::App<'a, 'b> {
            let app = app #app_methods;
            #( #subcommands )*;
            app #version
        }
    }
}

fn gen_from_clap_enum() -> TokenStream {
    quote! {
        fn from_clap(matches: &::structopt::clap::ArgMatches) -> Self {
            <Self as ::structopt::StructOptInternal>::from_subcommand(matches.subcommand())
                .expect("structopt misuse: You likely tried to #[flatten] a struct \
                         that contains #[subcommand]. This is forbidden.")
        }
    }
}

fn gen_from_subcommand(
    name: &Ident,
    variants: &Punctuated<Variant, Comma>,
    parent_attribute: &Attrs,
) -> TokenStream {
    use syn::Fields::*;

    let mut ext_subcmd = None;

    let (flatten_variants, variants): (Vec<_>, Vec<_>) = variants
        .iter()
        .filter_map(|variant| {
            let attrs = Attrs::from_struct(
                variant.span(),
                &variant.attrs,
                Name::Derived(variant.ident.clone()),
                Some(parent_attribute),
                parent_attribute.casing(),
                parent_attribute.env_casing(),
                true,
            );

            let variant_name = &variant.ident;

            match *attrs.kind() {
                Kind::ExternalSubcommand => {
                    if ext_subcmd.is_some() {
                        abort!(
                            attrs.kind().span(),
                            "Only one variant can be marked with `external_subcommand`, \
                         this is the second"
                        );
                    }

                    let ty = match variant.fields {
                        Unnamed(ref fields) if fields.unnamed.len() == 1 => &fields.unnamed[0].ty,

                        _ => abort!(
                            variant,
                            "The enum variant marked with `external_attribute` must be \
                         a single-typed tuple, and the type must be either `Vec<String>` \
                         or `Vec<OsString>`."
                        ),
                    };

                    let (span, str_ty, values_of) = match subty_if_name(ty, "Vec") {
                        Some(subty) => {
                            if is_simple_ty(subty, "String") {
                                (
                                    subty.span(),
                                    quote!(::std::string::String),
                                    quote!(values_of),
                                )
                            } else {
                                (
                                    subty.span(),
                                    quote!(::std::ffi::OsString),
                                    quote!(values_of_os),
                                )
                            }
                        }

                        None => abort!(
                            ty,
                            "The type must be either `Vec<String>` or `Vec<OsString>` \
                         to be used with `external_subcommand`."
                        ),
                    };

                    ext_subcmd = Some((span, variant_name, str_ty, values_of));
                    None
                }
                Kind::Skip(_) => None,
                _ => Some((variant, attrs)),
            }
        })
        .partition(|(_, attrs)| match &*attrs.kind() {
            Kind::Flatten => true,
            _ => false,
        });

    let other = format_ident!("other");
    let matches = format_ident!("matches");

    let external = match ext_subcmd {
        Some((span, var_name, str_ty, values_of)) => quote_spanned! { span=>
            match #other {
                ("", ::std::option::Option::None) => None,

                (external, Some(#matches)) => {
                    ::std::option::Option::Some(#name::#var_name(
                        ::std::iter::once(#str_ty::from(external))
                        .chain(
                            #matches.#values_of("").into_iter().flatten().map(#str_ty::from)
                        )
                        .collect::<::std::vec::Vec<_>>()
                    ))
                }

                (external, None) => {
                    ::std::option::Option::Some(#name::#var_name(
                        ::std::iter::once(#str_ty::from(external))
                            .collect::<::std::vec::Vec<_>>()
                    ))
                }
            }
        },

        None => quote!(None),
    };

    let match_arms = variants.iter().map(|(variant, attrs)| {
        let sub_name = attrs.cased_name();
        let variant_name = &variant.ident;
        let constructor_block = match variant.fields {
            Named(ref fields) => gen_constructor(&fields.named, &attrs),
            Unit => quote!(),
            Unnamed(ref fields) if fields.unnamed.len() == 1 => {
                let ty = &fields.unnamed[0];
                quote!( ( <#ty as ::structopt::StructOpt>::from_clap(#matches) ) )
            }
            Unnamed(..) => abort!(
                variant.ident,
                "non single-typed tuple enums are not supported"
            ),
        };

        quote! {
            (#sub_name, Some(#matches)) => {
                Some(#name :: #variant_name #constructor_block)
            }
        }
    });

    let child_subcommands = flatten_variants.iter().map(|(variant, _attrs)| {
        let variant_name = &variant.ident;
        match variant.fields {
            Unnamed(ref fields) if fields.unnamed.len() == 1 => {
                let ty = &fields.unnamed[0];
                quote! {
                    if let Some(res) =
                        <#ty as ::structopt::StructOptInternal>::from_subcommand(#other)
                    {
                        return Some(#name :: #variant_name (res));
                    }
                }
            }
            _ => abort!(
                variant,
                "`flatten` is usable only with single-typed tuple variants"
            ),
        }
    });

    quote! {
        fn from_subcommand<'a, 'b>(
            sub: (&'b str, Option<&'b ::structopt::clap::ArgMatches<'a>>)
        ) -> Option<Self> {
            match sub {
                #( #match_arms, )*
                #other => {
                    #( #child_subcommands )else*;
                    #external
                }
            }
        }
    }
}

#[cfg(feature = "paw")]
fn gen_paw_impl(
    impl_generics: &ImplGenerics,
    name: &Ident,
    ty_generics: &TypeGenerics,
    where_clause: &TokenStream,
) -> TokenStream {
    quote! {
        impl #impl_generics ::structopt::paw::ParseArgs for #name #ty_generics #where_clause {
            type Error = std::io::Error;

            fn parse_args() -> std::result::Result<Self, Self::Error> {
                Ok(<#name as ::structopt::StructOpt>::from_args())
            }
        }
    }
}
#[cfg(not(feature = "paw"))]
fn gen_paw_impl(_: &ImplGenerics, _: &Ident, _: &TypeGenerics, _: &TokenStream) -> TokenStream {
    TokenStream::new()
}

fn split_structopt_generics_for_impl(
    generics: &Generics,
) -> (ImplGenerics, TypeGenerics, TokenStream) {
    use syn::{token::Add, TypeParamBound::Trait};

    fn path_ends_with(path: &Path, ident: &str) -> bool {
        path.segments.last().unwrap().ident == ident
    }

    fn type_param_bounds_contains(bounds: &Punctuated<TypeParamBound, Add>, ident: &str) -> bool {
        for bound in bounds {
            if let Trait(bound) = bound {
                if path_ends_with(&bound.path, ident) {
                    return true;
                }
            }
        }
        return false;
    }

    struct TraitBoundAmendments {
        tokens: TokenStream,
        need_where: bool,
        need_comma: bool,
    }

    impl TraitBoundAmendments {
        fn new(where_clause: Option<&WhereClause>) -> Self {
            let tokens = TokenStream::new();
            let (need_where, need_comma) = if let Some(where_clause) = where_clause {
                if where_clause.predicates.trailing_punct() {
                    (false, false)
                } else {
                    (false, true)
                }
            } else {
                (true, false)
            };
            Self {
                tokens,
                need_where,
                need_comma,
            }
        }

        fn add(&mut self, amendment: TokenStream) {
            if self.need_where {
                self.tokens.extend(quote! { where });
                self.need_where = false;
            }
            if self.need_comma {
                self.tokens.extend(quote! { , });
            }
            self.tokens.extend(amendment);
            self.need_comma = true;
        }

        fn into_tokens(self) -> TokenStream {
            self.tokens
        }
    }

    let mut trait_bound_amendments = TraitBoundAmendments::new(generics.where_clause.as_ref());

    for param in &generics.params {
        if let GenericParam::Type(param) = param {
            let param_ident = &param.ident;
            if type_param_bounds_contains(&param.bounds, "StructOpt") {
                trait_bound_amendments
                    .add(quote! { #param_ident : ::structopt::StructOptInternal });
            }
        }
    }

    if let Some(where_clause) = &generics.where_clause {
        for predicate in &where_clause.predicates {
            if let WherePredicate::Type(predicate) = predicate {
                let predicate_bounded_ty = &predicate.bounded_ty;
                if type_param_bounds_contains(&predicate.bounds, "StructOpt") {
                    trait_bound_amendments
                        .add(quote! { #predicate_bounded_ty : ::structopt::StructOptInternal });
                }
            }
        }
    }

    let trait_bound_amendments = trait_bound_amendments.into_tokens();

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let where_clause = quote! { #where_clause #trait_bound_amendments };

    (impl_generics, ty_generics, where_clause)
}

fn impl_structopt_for_struct(
    name: &Ident,
    fields: &Punctuated<Field, Comma>,
    attrs: &[Attribute],
    generics: &Generics,
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = split_structopt_generics_for_impl(&generics);

    let basic_clap_app_gen = gen_clap_struct(attrs);
    let augment_clap = gen_augment_clap(fields, &basic_clap_app_gen.attrs);
    let from_clap = gen_from_clap(name, fields, &basic_clap_app_gen.attrs);
    let paw_impl = gen_paw_impl(&impl_generics, name, &ty_generics, &where_clause);

    let clap_tokens = basic_clap_app_gen.tokens;
    quote! {
        #[allow(unused_variables)]
        #[allow(unknown_lints)]
        #[allow(
            clippy::style,
            clippy::complexity,
            clippy::pedantic,
            clippy::restriction,
            clippy::perf,
            clippy::deprecated,
            clippy::nursery,
            clippy::cargo
        )]
        #[deny(clippy::correctness)]
        #[allow(dead_code, unreachable_code)]
        impl #impl_generics ::structopt::StructOpt for #name #ty_generics #where_clause {
            #clap_tokens
            #from_clap
        }

        #[allow(unused_variables)]
        #[allow(unknown_lints)]
        #[allow(
            clippy::style,
            clippy::complexity,
            clippy::pedantic,
            clippy::restriction,
            clippy::perf,
            clippy::deprecated,
            clippy::nursery,
            clippy::cargo
        )]
        #[deny(clippy::correctness)]
        #[allow(dead_code, unreachable_code)]
        impl #impl_generics ::structopt::StructOptInternal for #name #ty_generics #where_clause {
            #augment_clap
            fn is_subcommand() -> bool { false }
        }

        #paw_impl
    }
}

fn impl_structopt_for_enum(
    name: &Ident,
    variants: &Punctuated<Variant, Comma>,
    attrs: &[Attribute],
    generics: &Generics,
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = split_structopt_generics_for_impl(&generics);

    let basic_clap_app_gen = gen_clap_enum(attrs);
    let clap_tokens = basic_clap_app_gen.tokens;
    let attrs = basic_clap_app_gen.attrs;

    let augment_clap = gen_augment_clap_enum(variants, &attrs);
    let from_clap = gen_from_clap_enum();
    let from_subcommand = gen_from_subcommand(name, variants, &attrs);
    let paw_impl = gen_paw_impl(&impl_generics, name, &ty_generics, &where_clause);

    quote! {
        #[allow(unknown_lints)]
        #[allow(unused_variables, dead_code, unreachable_code)]
        #[allow(
            clippy::style,
            clippy::complexity,
            clippy::pedantic,
            clippy::restriction,
            clippy::perf,
            clippy::deprecated,
            clippy::nursery,
            clippy::cargo
        )]
        #[deny(clippy::correctness)]
        impl #impl_generics ::structopt::StructOpt for #name #ty_generics #where_clause {
            #clap_tokens
            #from_clap
        }

        #[allow(unused_variables)]
        #[allow(unknown_lints)]
        #[allow(
            clippy::style,
            clippy::complexity,
            clippy::pedantic,
            clippy::restriction,
            clippy::perf,
            clippy::deprecated,
            clippy::nursery,
            clippy::cargo
        )]
        #[deny(clippy::correctness)]
        #[allow(dead_code, unreachable_code)]
        impl #impl_generics ::structopt::StructOptInternal for #name #ty_generics #where_clause {
            #augment_clap
            #from_subcommand
            fn is_subcommand() -> bool { true }
        }

        #paw_impl
    }
}

fn impl_structopt(input: &DeriveInput) -> TokenStream {
    use syn::Data::*;

    let struct_name = &input.ident;

    set_dummy(quote! {
        impl ::structopt::StructOpt for #struct_name {
            fn clap<'a, 'b>() -> ::structopt::clap::App<'a, 'b> {
                unimplemented!()
            }
            fn from_clap(_matches: &::structopt::clap::ArgMatches) -> Self {
                unimplemented!()
            }
        }

        impl ::structopt::StructOptInternal for #struct_name {}
    });

    match input.data {
        Struct(DataStruct {
            fields: syn::Fields::Named(ref fields),
            ..
        }) => impl_structopt_for_struct(struct_name, &fields.named, &input.attrs, &input.generics),
        Enum(ref e) => {
            impl_structopt_for_enum(struct_name, &e.variants, &input.attrs, &input.generics)
        }
        _ => abort_call_site!("structopt only supports non-tuple structs and enums"),
    }
}
