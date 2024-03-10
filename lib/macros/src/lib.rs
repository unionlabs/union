use proc_macro::{Delimiter, Group, Punct, Spacing, TokenStream, TokenTree};
use quote::quote;
use syn::{
    parse::Parse, parse_quote, punctuated::Punctuated, spanned::Spanned, Generics, Ident, Item,
    ItemEnum, ItemStruct, MacroDelimiter, Meta, MetaList, Path, Token, WhereClause,
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

fn mk_proto(
    FromRawAttrs { raw, into, from }: FromRawAttrs,
    ident: &Ident,
    generics: &Generics,
) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut from_where_clause = where_clause.cloned().unwrap_or_else(|| WhereClause {
        where_token: parse_quote!(where),
        predicates: Default::default(),
    });

    from_where_clause.predicates.push(parse_quote!(
        #ident #ty_generics: TryFrom<#raw>
    ));
    from_where_clause.predicates.push(parse_quote!(
        <#ident #ty_generics as TryFrom<#raw>>::Error: ::core::fmt::Debug
    ));

    let mut into_where_clause = where_clause.cloned().unwrap_or_else(|| WhereClause {
        where_token: parse_quote!(where),
        predicates: Default::default(),
    });

    into_where_clause
        .predicates
        .push(parse_quote!(#ident #ty_generics: Into<#raw>));

    let mut output = quote! {
        impl #impl_generics crate::TypeUrl for #ident #ty_generics #where_clause {
            fn type_url() -> String {
                <#raw as ::prost::Name>::type_url()
            }
        }
    };

    if into {
        output.extend(quote! {
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
    }

    if from {
        output.extend(quote! {
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
    FromRawAttrs { raw, into, from }: FromRawAttrs,
    ident: &Ident,
    generics: &Generics,
) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut from_where_clause = where_clause.cloned().unwrap_or_else(|| WhereClause {
        where_token: parse_quote!(where),
        predicates: Default::default(),
    });

    from_where_clause.predicates.push(parse_quote!(
        #ident #ty_generics: TryFrom<#raw>
    ));
    from_where_clause.predicates.push(parse_quote!(
        <#ident #ty_generics as TryFrom<#raw>>::Error: ::core::fmt::Debug
    ));

    let mut into_where_clause = where_clause.cloned().unwrap_or_else(|| WhereClause {
        where_token: parse_quote!(where),
        predicates: Default::default(),
    });

    into_where_clause
        .predicates
        .push(parse_quote!(#ident #ty_generics: Into<#raw>));

    let mut output = proc_macro2::TokenStream::new();

    if into {
        output.extend(quote! {
            #[cfg(feature = "ethabi")]
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
    let input = ts.clone();

    let item = syn::parse_macro_input!(ts as Item);
    let Model { proto, ethabi } = syn::parse_macro_input!(meta as Model);

    let output = match item {
        Item::Enum(ItemEnum {
            ident, generics, ..
        })
        | Item::Struct(ItemStruct {
            ident, generics, ..
        }) => {
            let proto = proto.map(|from_raw| mk_proto(from_raw, &ident, &generics));
            let ethabi = ethabi.map(|from_raw| mk_ethabi(from_raw, &ident, &generics));

            quote! { #proto #ethabi }
        }
        _ => panic!(),
    };

    input
        .into_iter()
        .chain::<proc_macro::TokenStream>(output.into())
        .collect()
}

struct Model {
    proto: Option<FromRawAttrs>,
    ethabi: Option<FromRawAttrs>,
}

impl Parse for Model {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let meta = <Punctuated<Meta, Token![,]>>::parse_terminated(input)?;

        let mut proto = None;
        let mut ethabi = None;

        for meta in meta {
            match meta {
                // Meta::Path(path) => {}
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
                    _ => return Err(syn::Error::new_spanned(
                        path,
                        "invalid attribute, valid attributes are `proto(...)` and `ethabi(...)`",
                    )),
                },
                _ => {
                    return Err(syn::Error::new_spanned(
                        meta,
                        "invalid attribute, valid attributes are `proto(...)` and `ethabi(...)`",
                    ))
                }
            }
        }

        Ok(Model { proto, ethabi })
    }
}

struct FromRawAttrs {
    raw: Path,
    into: bool,
    from: bool,
}

impl Parse for FromRawAttrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let meta = <Punctuated<Meta, Token![,]>>::parse_terminated(input)?;

        let meta_span = meta.span();

        let mut raw = None;
        let mut into = false;
        let mut from = false;

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
                    } else {
                        return Err(syn::Error::new_spanned(
                            path,
                            "invalid attribute, valid attributes are `raw(...)`, `into`, `from`",
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
                            "invalid attribute, must be `raw = path::to::raw::Type`",
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
                        "invalid attribute, valid attributes are `raw(...)`, `into`, `from`",
                    ))
                }
            }
        }

        if let Some(raw) = raw {
            Ok(FromRawAttrs { raw, into, from })
        } else {
            Err(syn::Error::new(meta_span, "`raw(...)` is required"))
        }
    }
}
