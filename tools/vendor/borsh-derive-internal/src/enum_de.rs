use core::convert::TryFrom;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Fields, Ident, ItemEnum, WhereClause};

use crate::attribute_helpers::{contains_initialize_with, contains_skip};

pub fn enum_de(input: &ItemEnum, cratename: Ident) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut where_clause = where_clause.map_or_else(
        || WhereClause {
            where_token: Default::default(),
            predicates: Default::default(),
        },
        Clone::clone,
    );
    let init_method = contains_initialize_with(&input.attrs)?;
    let mut variant_arms = TokenStream2::new();
    for (variant_idx, variant) in input.variants.iter().enumerate() {
        let variant_idx = u8::try_from(variant_idx).expect("up to 256 enum variants are supported");
        let variant_ident = &variant.ident;
        let mut variant_header = TokenStream2::new();
        match &variant.fields {
            Fields::Named(fields) => {
                for field in &fields.named {
                    let field_name = field.ident.as_ref().unwrap();
                    if contains_skip(&field.attrs) {
                        variant_header.extend(quote! {
                            #field_name: Default::default(),
                        });
                    } else {
                        let field_type = &field.ty;
                        where_clause.predicates.push(
                            syn::parse2(quote! {
                                #field_type: #cratename::BorshDeserialize
                            })
                            .unwrap(),
                        );

                        variant_header.extend(quote! {
                            #field_name: #cratename::BorshDeserialize::deserialize_reader(reader)?,
                        });
                    }
                }
                variant_header = quote! { { #variant_header }};
            }
            Fields::Unnamed(fields) => {
                for field in fields.unnamed.iter() {
                    if contains_skip(&field.attrs) {
                        variant_header.extend(quote! { Default::default(), });
                    } else {
                        let field_type = &field.ty;
                        where_clause.predicates.push(
                            syn::parse2(quote! {
                                #field_type: #cratename::BorshDeserialize
                            })
                            .unwrap(),
                        );

                        variant_header.extend(
                            quote! { #cratename::BorshDeserialize::deserialize_reader(reader)?, },
                        );
                    }
                }
                variant_header = quote! { ( #variant_header )};
            }
            Fields::Unit => {}
        }
        variant_arms.extend(quote! {
            #variant_idx => #name::#variant_ident #variant_header ,
        });
    }

    let init = if let Some(method_ident) = init_method {
        quote! {
            return_value.#method_ident();
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        impl #impl_generics #cratename::de::BorshDeserialize for #name #ty_generics #where_clause {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(reader: &mut R) -> ::core::result::Result<Self, #cratename::maybestd::io::Error> {
                let tag = <u8 as #cratename::de::BorshDeserialize>::deserialize_reader(reader)?;
                <Self as #cratename::de::EnumExt>::deserialize_variant(reader, tag)
            }
        }

        impl #impl_generics #cratename::de::EnumExt for #name #ty_generics #where_clause {
            fn deserialize_variant<R: borsh::maybestd::io::Read>(
                reader: &mut R,
                variant_idx: u8,
            ) -> ::core::result::Result<Self, #cratename::maybestd::io::Error> {
                let mut return_value = match variant_idx {
                    #variant_arms
                    _ => return Err(#cratename::maybestd::io::Error::new(
                        #cratename::maybestd::io::ErrorKind::InvalidInput,
                        #cratename::maybestd::format!("Unexpected variant index: {:?}", variant_idx),
                    ))
                };
                #init
                Ok(return_value)
            }
        }
    })
}
