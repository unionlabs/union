use std::{collections::HashMap, convert};

use proc_macro::{Delimiter, Group, Punct, Spacing, TokenStream, TokenTree};
use proc_macro2::{Literal, Span};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{
    fold::Fold,
    parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote, parse_quote_spanned,
    punctuated::Punctuated,
    spanned::Spanned,
    Attribute, Data, DeriveInput, Expr, ExprPath, Field, Fields, GenericParam, Generics, Ident,
    Item, ItemEnum, ItemStruct, LitStr, MacroDelimiter, Meta, MetaList, Path, Token, Type, Variant,
    WhereClause, WherePredicate,
};

#[proc_macro_attribute]
pub fn apply(meta: TokenStream, ts: TokenStream) -> TokenStream {
    let [ident @ TokenTree::Ident(_)]: [TokenTree; 1] =
        meta.into_iter().collect::<Vec<_>>().try_into().unwrap()
    else {
        panic!()
    };

    [
        ident,
        Punct::new('!', Spacing::Alone).into(),
        TokenTree::Group(Group::new(Delimiter::Brace, ts)),
    ]
    .into_iter()
    .collect()
}

#[proc_macro_derive(Debug, attributes(debug))]
pub fn debug(ts: TokenStream) -> TokenStream {
    derive_debug(parse_macro_input!(ts as DeriveInput))
        // .inspect(|x| println!("{x}"))
        .map_err(|e| e.into_compile_error())
        .unwrap_or_else(convert::identity)
        .into()
}

fn derive_debug(
    DeriveInput {
        attrs,
        ident,
        generics,
        data,
        ..
    }: DeriveInput,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let container_attrs = parse_debug_meta(attrs.iter())?;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let bounds = match container_attrs.bound {
        Some(bound) => bound.into_iter().collect(),
        None => mk_where_clause(&data)?,
    };

    let mut where_clause = where_clause.cloned().unwrap_or_else(|| WhereClause {
        where_token: parse_quote!(where),
        predicates: Default::default(),
    });
    where_clause.predicates.extend(bounds);

    let mk_binding_pat = |idx, ident: &Option<Ident>| {
        ident.as_ref().map_or_else(
            || Literal::usize_unsuffixed(idx).into_token_stream(),
            |i| i.to_token_stream(),
        )
    };

    let mk_self_pat = |fields: &Fields| {
        fields
            .iter()
            .enumerate()
            .filter_map(|(idx, Field { ident, attrs, .. })| {
                let pat_ident = mk_binding_pat(idx, ident);

                parse_debug_meta(attrs)
                    .map(|meta| {
                        let binding = format_ident!("__binding_{idx}");

                        match meta.fmt {
                            Some(DebugMetaFmt::Skip(_)) => None,
                            Some(_) | None => Some(quote! {
                                #pat_ident: #binding,
                            }),
                        }
                    })
                    .transpose()
            })
            .collect::<Result<proc_macro2::TokenStream, _>>()
            .map(|bindings| {
                quote! {
                    { #bindings .. }
                }
            })
    };

    let mk_field_debugs = |(idx, Field { ident, attrs, .. }): (usize, &Field)| {
        parse_debug_meta(attrs)
            .and_then(|meta| {
                let binding = format_ident!("__binding_{idx}");

                if let Some(meta_fmt) = &meta.fmt {
                    if container_attrs.fmt.is_some() {
                        return Err(syn::Error::new(
                            meta_fmt.span(),
                            "container and field `#[debug(...)]` attributes cannot be combined"
                        ))
                    }
                }

                if let Some(meta_bound) = &meta.bound {
                    return Err(syn::Error::new(
                        meta_bound.span(),
                        "`#[debug(bound(...))]` is only valid as a container attribute"
                    ))
                }

                let expr = match &meta.fmt {
                    Some(DebugMetaFmt::Format(lit, exprs)) => {
                        let exprs = exprs.iter().map(|expr| {
                            ReplacePath {
                                from: ident.clone().unwrap_or_else(|| format_ident!("_{idx}")),
                                to: binding.clone(),
                            }
                            .fold_expr(expr.clone())
                        });

                        quote! {{
                            // yes, write to a string and then display the string
                            // fight me, it's debug
                            struct DebugAsDisplay<T>(T);
                            impl<T: ::core::fmt::Display> ::core::fmt::Debug for DebugAsDisplay<T> {
                                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                                    f.write_fmt(format_args!("{0}", self.0))
                                }
                            }

                            DebugAsDisplay(format!(#lit, #(#exprs,)*))
                        }}
                    }
                    Some(DebugMetaFmt::Wrap(path)) => {
                        quote! { (#path)(#binding) }
                    }
                    Some(DebugMetaFmt::Skip(_)) => {
                        quote! {}
                    }
                    None => {
                        quote! { #binding }
                    }
                };

                Ok(match meta.fmt {
                    Some(DebugMetaFmt::Skip(_)) => None,
                    Some(_) | None => Some(match ident {
                        Some(ident) => {
                            quote! {
                                debug_builder.field(stringify!(#ident), &#expr);
                            }
                        }
                        None => quote! {
                            debug_builder.field(&#expr);
                        },
                    }),
                })
            })
            .transpose()
    };

    let mk_builder = |ident: &Ident, fields: &Fields| match fields {
        Fields::Unit | Fields::Named(_) => quote! { f.debug_struct(stringify!(#ident)); },
        Fields::Unnamed(_) => quote! { f.debug_tuple(stringify!(#ident)); },
    };

    let body = match (data, container_attrs.fmt.as_ref()) {
        (_, Some(DebugMetaFmt::Skip(skip))) => Err(syn::Error::new(
            skip.span(),
            "`skip` is only valid as a field attribute",
        )),
        (Data::Struct(s), None) => {
            let field_debugs = s
                .fields
                .iter()
                .enumerate()
                .filter_map(mk_field_debugs)
                .collect::<Result<Vec<_>, _>>()?;

            let builder = mk_builder(&ident, &s.fields);

            let pat = mk_self_pat(&s.fields)?;

            Ok(quote! {
                let Self #pat = self;

                let mut debug_builder = #builder
                #(#field_debugs)*
                debug_builder.finish()
            })
        }
        (Data::Enum(e), None) => {
            if e.variants.is_empty() {
                Ok(quote!(unreachable!()))
            } else {
                let variant_debugs = e
                .variants
                .iter()
                .map(
                    |Variant {
                         attrs,
                         ident,
                         fields,
                         ..
                     }| {
                        parse_debug_meta(attrs)
                            .and_then(|m| {
                                m.bound
                                    .map_or_else(
                                        || Ok(()),
                                        |m| {
                                            Err(syn::Error::new(
                                                m.span(),
                                                "`#[debug(bound(...))]` is only valid as a container attribute",
                                            ))
                                        },
                                    )
                                    .and_then(|()| {
                                        m.fmt.map_or_else(
                                            || Ok(()),
                                            |m| {
                                                Err(syn::Error::new(
                                                    m.span(),
                                                    // arbitrary limitation bc we don't need it right now and i'm lazy
                                                    "`#[debug(...)]` cannot be used on variants",
                                                ))
                                            },
                                        )
                                    })
                            })
                            .and_then(|()| -> Result<_, _> {
                                let field_debugs = fields
                                    .iter()
                                    .enumerate()
                                    .filter_map(mk_field_debugs)
                                    .collect::<Result<Vec<_>, _>>()?;

                                let builder = mk_builder(ident, fields);

                                let pat = mk_self_pat(fields)?;

                                Ok(quote! {
                                    Self::#ident #pat => {
                                        let mut debug_builder = #builder
                                        #(#field_debugs)*
                                        debug_builder.finish()
                                    }
                                })
                            })
                    },
                )
                .collect::<Result<Vec<_>, _>>()?;

                Ok(quote! {
                    match self {
                        #(#variant_debugs)*
                    }
                })
            }
        }
        (Data::Struct(_) | Data::Enum(_), Some(DebugMetaFmt::Format(lit, exprs))) => Ok(quote! {
            write!(f, #lit, #(#exprs,)*)
        }),
        (Data::Struct(_) | Data::Enum(_), Some(DebugMetaFmt::Wrap(path))) => Ok(quote! {
            ::core::fmt::Debug::fmt((#path)(self), f)
        }),
        (Data::Union(_), _) => panic!(),
    }?;

    Ok(quote! {
        const _: () = {
            #[automatically_derived]
            impl #impl_generics ::core::fmt::Debug for #ident #ty_generics #where_clause {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    #body
                }
            }
        };
    })
}

struct ReplacePath {
    from: Ident,
    to: Ident,
}

impl Fold for ReplacePath {
    fn fold_expr_path(&mut self, i: ExprPath) -> ExprPath {
        ExprPath {
            attrs: i
                .attrs
                .into_iter()
                .map(|it| self.fold_attribute(it))
                .collect(),
            qself: i.qself.map(|q| self.fold_qself(q)),
            path: if i.path.is_ident(&self.from) {
                Path::from(self.to.clone())
            } else {
                i.path
            },
        }
    }
}

fn parse_debug_meta<'a>(
    attrs: impl IntoIterator<Item = &'a Attribute>,
) -> Result<DebugMeta, syn::Error> {
    attrs
        .into_iter()
        .filter(|attr| attr.path().is_ident("debug"))
        .map(DebugMeta::try_from_attribute)
        .try_fold(
            DebugMeta {
                bound: None,
                fmt: None,
            },
            |curr, acc| {
                let acc = acc?;

                let bound = match (curr.bound, acc.bound) {
                    (None, acc) => acc,
                    (new @ Some(_), None) => new,
                    (Some(new), Some(_)) => {
                        return Err(syn::Error::new(
                            new.span(),
                            "only one `#[debug(bound(...))]` attribute is allowed",
                        ))
                    }
                };

                let fmt = match (curr.fmt, acc.fmt) {
                    (None, acc) => acc,
                    (new @ Some(_), None) => new,
                    (Some(new), Some(_)) => {
                        return Err(syn::Error::new(
                            new.span(),
                            "only one `#[debug(...)]` formatting attribute is allowed",
                        ))
                    }
                };

                Ok(DebugMeta { bound, fmt })
            },
        )
}

fn mk_where_clause(data: &Data) -> Result<Vec<WherePredicate>, syn::Error> {
    let f = |Field { ty, attrs, .. }: &Field| {
        parse_debug_meta(attrs.iter())
            .map(|m| {
                m.fmt
                    .is_none()
                    .then(|| parse_quote_spanned!(ty.span()=> #ty: ::core::fmt::Debug))
            })
            .transpose()
    };

    match data {
        Data::Struct(s) => s.fields.iter().filter_map(f).collect(),
        Data::Enum(e) => e
            .variants
            .iter()
            .flat_map(|v| &v.fields)
            .filter_map(f)
            .collect(),
        Data::Union(_) => panic!(),
    }
}

#[derive(core::fmt::Debug)]
struct DebugMeta {
    bound: Option<Punctuated<WherePredicate, Token![,]>>,
    fmt: Option<DebugMetaFmt>,
}

#[derive(core::fmt::Debug)]
enum DebugMetaFmt {
    Skip(Span),
    Format(LitStr, Vec<Expr>),
    Wrap(Path),
}

impl DebugMetaFmt {
    fn span(&self) -> Span {
        match self {
            Self::Skip(span) => span.span(),
            Self::Format(lit, _exprs) => lit.span(),
            Self::Wrap(path) => path.span(),
        }
    }
}

impl DebugMeta {
    fn try_from_attribute(attr: &Attribute) -> syn::Result<Self> {
        syn::custom_keyword!(skip);
        syn::custom_keyword!(wrap);
        syn::custom_keyword!(bound);

        let mut debug_meta = Self {
            bound: None,
            fmt: None,
        };

        attr.parse_args_with(|input: ParseStream| {
            if let Some(_kw) = input.parse::<Option<bound>>()? {
                if debug_meta.bound.is_some() {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "duplicate #[debug(bound(...))] attribute",
                    ));
                }

                let content;
                parenthesized!(content in input);

                debug_meta.bound = Some(Punctuated::parse_terminated(&content)?);
                return Ok(());
            }

            if let Some(kw) = input.parse::<Option<skip>>()? {
                if debug_meta.fmt.is_some() {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "duplicate #[debug(skip)] attribute",
                    ));
                }
                debug_meta.fmt = Some(DebugMetaFmt::Skip(kw.span));
                return Ok(());
            }

            if let Some(_kw) = input.parse::<Option<wrap>>()? {
                if debug_meta.fmt.is_some() {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "duplicate #[debug(wrap)] attribute",
                    ));
                }

                let _eq = input.parse::<Token![=]>()?;

                debug_meta.fmt = Some(DebugMetaFmt::Wrap(input.parse()?));
                return Ok(());
            }

            let fmt: LitStr = input.parse()?;

            let args = match input.parse::<Option<Token![,]>>()? {
                Some(_token) => Punctuated::<Expr, Token![,]>::parse_terminated(input)?,
                None => Punctuated::default(),
            };

            let prev = debug_meta
                .fmt
                .replace(DebugMetaFmt::Format(fmt, args.into_iter().collect()));

            assert!(prev.is_none());

            Ok(())
        })?;

        Ok(debug_meta)
    }
}

fn mk_proto(
    FromRawAttrs {
        raw,
        into,
        from,
        no_static_assert,
    }: FromRawAttrs,
    ident: &Ident,
    generics: &Generics,
) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut from_where_clause = where_clause.cloned().unwrap_or_else(|| WhereClause {
        where_token: parse_quote!(where),
        predicates: Default::default(),
    });

    let mut into_where_clause = where_clause.cloned().unwrap_or_else(|| WhereClause {
        where_token: parse_quote!(where),
        predicates: Default::default(),
    });

    if !generics.params.is_empty() {
        into_where_clause
            .predicates
            .push(parse_quote!(#ident #ty_generics: Into<#raw>));

        from_where_clause.predicates.push(parse_quote!(
            #ident #ty_generics: TryFrom<#raw>
        ));
        from_where_clause.predicates.push(parse_quote!(
            <#ident #ty_generics as TryFrom<#raw>>::Error: ::core::fmt::Debug + ::core::marker::Send + ::core::marker::Sync
        ));
    }

    let mut output = quote! {
        impl #impl_generics crate::TypeUrl for #ident #ty_generics #where_clause {
            fn type_url() -> String {
                <#raw as ::prost::Name>::type_url()
            }
        }
    };

    let assert_impl_generics = &generics
        .params
        .clone()
        .into_iter()
        .map(|mut param| {
            match &mut param {
                GenericParam::Type(param) => param.default = None,
                GenericParam::Const(param) => param.default = None,
                _ => {}
            };
            param
        })
        .collect::<Punctuated<GenericParam, Token![,]>>();

    if into {
        output.extend(quote! {
            #[automatically_derived]
            impl #impl_generics crate::encoding::Decode<crate::encoding::Proto> for #ident #ty_generics #from_where_clause {
                type Error = crate::TryFromProtoBytesError<<#ident #ty_generics as TryFrom<#raw>>::Error>;

                fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
                    <#raw as ::prost::Message>::decode(bytes)
                        .map_err(crate::TryFromProtoBytesError::Decode)
                        .and_then(|proto| {
                            proto
                                .try_into()
                                .map_err(crate::TryFromProtoBytesError::TryFromProto)
                        })
                }
            }
        });

        if !no_static_assert {
            output.extend(quote! {
                ::static_assertions::assert_impl!(for(#assert_impl_generics) <#ident #ty_generics as TryFrom<#raw>>::Error: (::core::marker::Send) & (::core::marker::Sync) & (::core::fmt::Debug));
            });
        }
    }

    if from {
        output.extend(quote! {
            #[automatically_derived]
            impl #impl_generics crate::encoding::Encode<crate::encoding::Proto> for #ident #ty_generics #into_where_clause {
                fn encode(self) -> Vec<u8> {
                    ::prost::Message::encode_to_vec(&Into::<#raw>::into(self))
                }
            }
        });
    }

    output
}

fn mk_ethabi(
    FromRawAttrs {
        raw,
        into,
        from,
        no_static_assert: _,
    }: FromRawAttrs,
    ident: &Ident,
    generics: &Generics,
) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut from_where_clause = where_clause.cloned().unwrap_or_else(|| WhereClause {
        where_token: parse_quote!(where),
        predicates: Default::default(),
    });

    let mut into_where_clause = where_clause.cloned().unwrap_or_else(|| WhereClause {
        where_token: parse_quote!(where),
        predicates: Default::default(),
    });

    if !generics.params.is_empty() {
        into_where_clause
            .predicates
            .push(parse_quote!(#ident #ty_generics: Into<#raw>));

        from_where_clause.predicates.push(parse_quote!(
            #ident #ty_generics: TryFrom<#raw>
        ));
        from_where_clause.predicates.push(parse_quote!(
            <#ident #ty_generics as TryFrom<#raw>>::Error: ::core::fmt::Debug + ::core::marker::Send + ::core::marker::Sync
        ));
    }

    let mut output = proc_macro2::TokenStream::new();

    if into {
        output.extend(quote! {
            #[cfg(feature = "ethabi")]
            #[automatically_derived]
            impl #impl_generics crate::encoding::Decode<crate::encoding::EthAbi> for #ident #ty_generics #from_where_clause {
                type Error = crate::TryFromEthAbiBytesError<<#ident #ty_generics as TryFrom<#raw>>::Error>;

                fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
                    <#raw as ethers_core::abi::AbiDecode>::decode(bytes)
                        .map_err(crate::TryFromEthAbiBytesError::Decode)
                        .and_then(|proto| {
                            proto
                                .try_into()
                                .map_err(crate::TryFromEthAbiBytesError::TryFromEthAbi)
                        })
                }
            }
        });
    }

    if from {
        output.extend(quote! {
            #[cfg(feature = "ethabi")]
            #[automatically_derived]
            impl #impl_generics crate::encoding::Encode<crate::encoding::EthAbi> for #ident #ty_generics #into_where_clause {
                fn encode(self) -> Vec<u8> {
                    ethers_core::abi::AbiEncode::encode(Into::<#raw>::into(self))
                }
            }
        });
    }

    output
}

#[proc_macro_attribute]
pub fn model(meta: TokenStream, ts: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(ts.clone());

    let item = syn::parse_macro_input!(ts as Item);
    let Model {
        proto,
        ethabi,
        no_serde,
    } = syn::parse_macro_input!(meta as Model);

    let output = match &item {
        Item::Enum(ItemEnum {
            ident, generics, ..
        })
        | Item::Struct(ItemStruct {
            ident, generics, ..
        }) => {
            let proto = proto.map(|from_raw| mk_proto(from_raw, ident, generics));
            let ethabi = ethabi.map(|from_raw| mk_ethabi(from_raw, ident, generics));

            quote! { #proto #ethabi }
        }
        _ => panic!(),
    };

    let attributes = match item {
        Item::Enum(ItemEnum {
            variants, attrs, ..
        }) => {
            let debug_derive_crate: Path = variants
                .iter()
                .flat_map(|v| &v.fields)
                .flat_map(|f| &f.attrs)
                .chain(&attrs)
                .any(|a| a.path().is_ident("debug"))
                .then_some(parse_quote!(::macros))
                .unwrap_or(parse_quote!(::core::fmt));

            let serde = (!no_serde).then(|| {
                quote! {
                    #[derive(::serde::Serialize, ::serde::Deserialize)]
                    #[serde(
                        deny_unknown_fields,
                        tag = "@type",
                        content = "@value",
                        rename_all = "snake_case"
                    )]
                }
            });

            quote! {
                #[derive(
                    #debug_derive_crate::Debug,
                    ::core::clone::Clone,
                    ::core::cmp::PartialEq,
                    ::core::cmp::Eq,
                )]
                #serde
            }
        }
        Item::Struct(ItemStruct { fields, attrs, .. }) => {
            let debug_derive_crate: Path = fields
                .iter()
                .flat_map(|f| &f.attrs)
                .chain(&attrs)
                .any(|a| a.path().is_ident("debug"))
                .then_some(parse_quote!(::macros))
                .unwrap_or(parse_quote!(::core::fmt));

            let serde = (!no_serde).then(|| {
                quote! {
                    #[derive(::serde::Serialize, ::serde::Deserialize)]
                    #[serde(
                        deny_unknown_fields,
                        rename_all = "snake_case"
                    )]
                }
            });

            quote! {
                #[derive(
                    #debug_derive_crate::Debug,
                    ::core::clone::Clone,
                    ::core::cmp::PartialEq,
                    ::core::cmp::Eq,
                )]
                #serde
            }
        }
        _ => panic!(),
    };

    quote! {
        #attributes
        #input

        #output
    }
    .into()
}

struct Model {
    proto: Option<FromRawAttrs>,
    ethabi: Option<FromRawAttrs>,
    no_serde: bool,
}

impl Parse for Model {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        const INVALID_ATTR_MSG: &str =
            "invalid attribute, valid attributes are `no_serde`, `proto(...)` and `ethabi(...)`";

        let meta = <Punctuated<Meta, Token![,]>>::parse_terminated(input)?;

        let mut proto = None;
        let mut ethabi = None;
        let mut no_serde = false;

        for meta in meta {
            match meta {
                Meta::List(MetaList {
                    path,
                    delimiter: MacroDelimiter::Paren(_),
                    tokens,
                }) => match &*path.require_ident()?.to_string() {
                    "proto" => {
                        if proto.is_some() {
                            return Err(syn::Error::new_spanned(
                                path,
                                "duplicate `proto(...)` attribute",
                            ));
                        } else {
                            proto = Some(syn::parse2(tokens)?);
                        }
                    }
                    "ethabi" => {
                        if ethabi.is_some() {
                            return Err(syn::Error::new_spanned(
                                path,
                                "duplicate `ethabi(...)` attribute",
                            ));
                        } else {
                            ethabi = Some(syn::parse2(tokens)?);
                        }
                    }
                    _ => return Err(syn::Error::new_spanned(path, INVALID_ATTR_MSG)),
                },
                Meta::Path(path) => match &*path.require_ident()?.to_string() {
                    "no_serde" => {
                        if no_serde {
                            return Err(syn::Error::new_spanned(
                                path,
                                "duplicate `no_serde` attribute",
                            ));
                        } else {
                            no_serde = true;
                        }
                    }
                    _ => return Err(syn::Error::new_spanned(path, INVALID_ATTR_MSG)),
                },
                _ => return Err(syn::Error::new_spanned(meta, INVALID_ATTR_MSG)),
            }
        }

        Ok(Model {
            proto,
            ethabi,
            no_serde,
        })
    }
}

struct FromRawAttrs {
    raw: Path,
    into: bool,
    from: bool,
    // TODO: This is a stop gap solution until i figure out a better way to do these assertions with a custom bound
    no_static_assert: bool,
}

impl Parse for FromRawAttrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let meta = <Punctuated<Meta, Token![,]>>::parse_terminated(input)?;

        let meta_span = meta.span();

        let mut raw = None;
        let mut into = false;
        let mut from = false;
        let mut no_static_assert = false;

        for meta in meta {
            match meta {
                Meta::Path(path) => {
                    if path.is_ident("into") {
                        if into {
                            return Err(syn::Error::new_spanned(
                                path,
                                "duplicate `into` attribute",
                            ));
                        } else {
                            into = true;
                        }
                    } else if path.is_ident("from") {
                        if from {
                            return Err(syn::Error::new_spanned(
                                path,
                                "duplicate `from` attribute",
                            ));
                        } else {
                            from = true;
                        }
                    } else if path.is_ident("no_static_assert") {
                        if no_static_assert {
                            return Err(syn::Error::new_spanned(
                                path,
                                "duplicate `no_static_assert` attribute",
                            ));
                        } else {
                            no_static_assert = true;
                        }
                    } else {
                        return Err(syn::Error::new_spanned(
                            path,
                            "invalid attribute, valid attributes are `raw(...)`, `into`, `from`, `no_static_assert`",
                        ));
                    }
                }
                Meta::List(MetaList {
                    path,
                    delimiter: MacroDelimiter::Paren(_),
                    tokens,
                }) => {
                    if !path.is_ident("raw") {
                        return Err(syn::Error::new_spanned(
                            path,
                            "invalid attribute, must be `raw(path::to::raw::Type)`",
                        ));
                    } else if raw.is_some() {
                        return Err(syn::Error::new_spanned(
                            path,
                            "duplicate `raw(...)` attribute",
                        ));
                    } else {
                        raw = Some(syn::parse2(tokens)?);
                    }
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        meta,
                        "invalid attribute, valid attributes are `raw(...)`, `into`, `from`, `no_static_assert`",
                    ))
                }
            }
        }

        if let Some(raw) = raw {
            Ok(FromRawAttrs {
                raw,
                into,
                from,
                no_static_assert,
            })
        } else {
            Err(syn::Error::new(meta_span, "`raw(...)` is required"))
        }
    }
}

/// NOTE: Doesn't suport generics. Generics this low in the stack is a dark path I do not wish to explore again
#[proc_macro_attribute]
pub fn ibc_path(meta: TokenStream, ts: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(ts as ItemStruct);
    let IbcPathMeta { path, comma: _, ty } = parse_macro_input!(meta as IbcPathMeta);

    let segments = parse_ibc_path(path.clone());

    let Fields::Named(ref fields) = item_struct.fields else {
        panic!("expected named fields")
    };

    assert_eq!(
        fields
            .named
            .iter()
            .map(|x| x.ident.as_ref().unwrap())
            .collect::<Vec<_>>(),
        segments
            .iter()
            .filter_map(|x| match x {
                Segment::Static(_) => None,
                Segment::Variable(x) => Some(x),
            })
            .collect::<Vec<_>>()
    );

    let fields_map = fields
        .named
        .iter()
        .map(|f| (f.ident.as_ref().unwrap(), &f.ty))
        .collect::<HashMap<_, _>>();

    let parse_body = segments
        .iter()
        .map(|x| match x {
            Segment::Static(static_seg) => quote! {
                match it.next() {
                    Some(segment) => {
                        if segment != #static_seg {
                            return Err(PathParseError::InvalidStaticSegment {
                                expected: #static_seg,
                                found: segment.to_string(),
                            })
                        }
                    }
                    None => return Err(PathParseError::MissingStaticSegment(#static_seg)),
                }
            },
            Segment::Variable(variable_seg) => {
                let ty = fields_map[variable_seg];
                quote! {
                    let #variable_seg = match it.next() {
                        Some(segment) => segment
                            .parse()
                            .map_err(|e: <#ty as ::core::str::FromStr>::Err| PathParseError::Parse(e.to_string()))?,
                        None => return Err(PathParseError::MissingSegment),
                    };
                }
            }
        })
        .collect::<proc_macro2::TokenStream>();

    let display_body = segments
        .iter()
        .map(|x| match x {
            Segment::Static(_) => quote! {},
            Segment::Variable(variable_seg) => quote_spanned! {fields_map[variable_seg].span()=>
                let #variable_seg = &self.#variable_seg;
            },
        })
        .collect::<proc_macro2::TokenStream>();

    let field_pat = segments.iter().filter_map(|seg| match seg {
        Segment::Static(_) => None,
        // use the span of the input tokens
        Segment::Variable(variable_seg) => Some(fields_map.get_key_value(variable_seg).unwrap().0),
    });

    let ident = &item_struct.ident;

    quote! {
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash, ::clap::Args)]
        #[serde(deny_unknown_fields)]
        #item_struct

        const _: () = {
            #[automatically_derived]
            impl ::core::fmt::Display for #ident {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    #display_body

                    write!(f, #path)
                }
            }
        };

        const _: () = {
            #[automatically_derived]
            impl ::core::str::FromStr for #ident {
                type Err = PathParseError;

                fn from_str(s: &str) -> ::core::result::Result<Self, Self::Err> {
                    let mut it = s.split('/');

                    #parse_body

                    if it.next().is_some() {
                        return Err(PathParseError::TooManySegments);
                    }

                    Ok(Self { #(#field_pat),* })
                }
            }
        };

        const _: () = {
            impl IbcPath for #ident {
                type Value = #ty;
            }
        };
    }
    .into()
}

struct IbcPathMeta {
    path: LitStr,
    #[allow(unused)]
    comma: Token![,],
    ty: Type,
}

impl Parse for IbcPathMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            path: input.parse()?,
            comma: input.parse()?,
            ty: input.parse()?,
        })
    }
}

enum Segment {
    Static(String),
    Variable(syn::Ident),
}

fn parse_ibc_path(path: LitStr) -> Vec<Segment> {
    path.value()
        .split('/')
        .map(|segment| {
            segment
                .strip_prefix('{')
                .map(|s| {
                    s.strip_suffix('}')
                        .expect("unclosed `{` in variable interpolation")
                })
                .map_or_else(
                    || Segment::Static(segment.to_string()),
                    |s| Segment::Variable(Ident::new(s, path.span())),
                )
        })
        .collect()
}
