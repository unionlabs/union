#![feature(proc_macro_quote)]

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Attribute, Data, Data::Struct, DataStruct,
    DeriveInput, Error, Fields, GenericParam, Generics, Meta,
};

#[proc_macro_attribute]
pub fn queue_msg(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let result = apply_item(input).unwrap_or_else(|error| error.to_compile_error());

    result.into()
}

fn apply_item(derive_input: DeriveInput) -> Result<proc_macro2::TokenStream, Error> {
    let derive_input = &mut derive_input.clone();

    let derives = if derive_input.generics.params.is_empty() {
        quote! {
            #[derive(
                ::macros::Debug,
                ::core::clone::Clone,
                ::core::cmp::PartialEq,
                ::serde::Serialize,
                ::serde::Deserialize,
            )]
        }
    } else {
        quote! {
            #[derive(
                ::macros::Debug,
                ::frame_support_procedural::CloneNoBound,
                ::frame_support_procedural::PartialEqNoBound,
                ::serde::Serialize,
                ::serde::Deserialize,
            )]
        }
    };

    match &mut derive_input.data {
        Struct(data_struct) => match &mut data_struct.fields {
            Fields::Named(_) => {
                let type_params_to_cover = extract_covered_types(&mut derive_input.generics);

                if let Some(type_params_to_cover) = &type_params_to_cover {
                    add_marker_field_to_struct(data_struct, type_params_to_cover);
                };

                Ok(quote! {
                    #derives

                    #[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
                    #[cfg_attr(
                        feature = "arbitrary",
                        derive(arbitrary::Arbitrary),
                        arbitrary(bound = "")
                    )]

                    #derive_input
                })
            }
            Fields::Unnamed(field_unnamed) => {
                if field_unnamed.unnamed.len() != 1 {
                    return Err(Error::new(
                        field_unnamed.span(),
                        "only newtype structs are supported",
                    ));
                }

                Ok(quote! {
                    #derives

                    #[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields, transparent)]
                    #[cfg_attr(
                        feature = "arbitrary",
                        derive(arbitrary::Arbitrary),
                        arbitrary(bound = "")
                    )]

                    #derive_input
                })
            }
            _ => Err(Error::new_spanned(
                data_struct.struct_token,
                "queue-msg only supports Named and Unnamed Struct fields",
            )),
        },
        Data::Enum(_) => Ok(quote! {
            #derives

            #[serde(
                tag = "@type",
                content = "@value",
                rename_all = "snake_case",
                bound(serialize = "", deserialize = ""),
                deny_unknown_fields
            )]
            #[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
            #[allow(clippy::large_enum_variant)]

            #derive_input
        }),
        Data::Union(data_union) => Err(Error::new_spanned(
            data_union.union_token,
            "queue-msg only supports Enum and Struct",
        )),
    }
}

// inject `__marker: PhantomData<fn -> ({type params})>` fields into struct
fn add_marker_field_to_struct(data_struct: &mut DataStruct, type_params_to_cover: &Vec<Ident>) {
    match &mut data_struct.fields {
        Fields::Named(fields_named) => {
            fields_named.named.push(parse_quote! {
                #[serde(skip)]
                #[debug(skip)]
                #[cfg_attr(feature = "arbitrary", arbitrary(default))]
                pub __marker: ::core::marker::PhantomData<fn() -> (#(#type_params_to_cover),*)>
            });
        }
        _ => panic!("Expecting only Named fields"),
    }
}

fn extract_covered_types(generics: &mut Generics) -> Option<Vec<Ident>> {
    let type_params_to_cover = &mut generics
        .params
        .iter_mut()
        .filter_map(|generic_param| {
            if let GenericParam::Type(type_param) = generic_param {
                let cover_attributes = &type_param
                    .attrs
                    .iter()
                    .filter(|attr| {
                        if let Meta::Path(path) = &attr.meta {
                            path.is_ident("cover")
                        } else {
                            false
                        }
                    })
                    .cloned()
                    .collect::<Vec<Attribute>>();

                if cover_attributes.is_empty() {
                    None
                } else {
                    type_param
                        .attrs
                        .retain(|attr| !cover_attributes.contains(attr));

                    Some(type_param.ident.clone())
                }
            } else {
                None
            }
        })
        .collect::<Vec<Ident>>();

    if type_params_to_cover.is_empty() {
        None
    } else {
        Some(type_params_to_cover.clone())
    }
}
