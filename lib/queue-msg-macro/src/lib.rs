// #![feature(proc_macro_quote)]

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse_macro_input, parse_quote,
    spanned::Spanned,
    Attribute,
    Data::{self, Struct},
    DataStruct, DeriveInput, Error, Fields, GenericParam, Generics, Meta, Type,
};

#[proc_macro_attribute]
pub fn queue_msg(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let result = apply_item(input).unwrap_or_else(|error| error.to_compile_error());

    result.into()
}

fn apply_item(derive_input: DeriveInput) -> Result<proc_macro2::TokenStream, Error> {
    let derive_input = &mut derive_input.clone();

    // let derives = if derive_input.generics.params.is_empty() {
    //     quote! {
    //         #[derive(
    //             ::macros::Debug,
    //             ::core::clone::Clone,
    //             ::core::cmp::PartialEq,
    //             ::serde::Serialize,
    //             ::serde::Deserialize,
    //         )]
    //     }
    // } else {
    //     quote! {
    //         #[derive(
    //             ::macros::Debug,
    //             ::frame_support_procedural::CloneNoBound,
    //             ::frame_support_procedural::PartialEqNoBound,
    //             ::serde::Serialize,
    //             ::serde::Deserialize,
    //         )]
    //     }
    // };

    let derives = quote! {
        #[derive(
            ::macros::Debug,
            ::core::clone::Clone,
            ::core::cmp::PartialEq,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
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

                    #[serde(
                        // bound(serialize = "", deserialize = ""),
                        deny_unknown_fields
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

                    #[serde(
                        // bound(serialize = "", deserialize = ""),
                        deny_unknown_fields,
                        transparent
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
                // bound(serialize = "", deserialize = ""),
                deny_unknown_fields
            )]
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

#[proc_macro_derive(SubsetOf, attributes(subset_of))]
pub fn subset_of(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let enm = match input.data {
        syn::Data::Struct(struct_) => {
            return syn::Error::new_spanned(
                struct_.struct_token,
                "SubsetOf can only be derived on enums",
            )
            .into_compile_error()
            .into()
        }
        syn::Data::Enum(enm) => enm,
        syn::Data::Union(union) => {
            return syn::Error::new_spanned(
                union.union_token,
                "SubsetOf can only be derived on enums",
            )
            .into_compile_error()
            .into()
        }
    };

    let impl_generics: (
        syn::ImplGenerics<'_>,
        syn::TypeGenerics<'_>,
        Option<&syn::WhereClause>,
    ) = input.generics.split_for_impl();

    let impls = enm
        .variants
        .into_iter()
        .filter(|x| {
            x.attrs
                .iter()
                .all(|x| x.meta != parse_quote!(subset_of(ignore)))
        })
        .map(|x| match x.fields {
            syn::Fields::Unnamed(mut unnamed) => {
                let fields_span = unnamed.span();
                if unnamed.unnamed.len() == 1 {
                    let field = unnamed.unnamed.pop().unwrap().into_value();
                    Ok(mk_impls(
                        &input.ident,
                        &x.ident,
                        &FieldName::Index(syn::Index::from(0)),
                        &field.ty,
                        &impl_generics,
                    ))
                } else {
                    Err(syn::Error::new(
                        fields_span,
                        "only newtype variants are supported",
                    ))
                }
            }
            fields => Err(syn::Error::new(
                fields.span(),
                "only newtype variants are supported",
            )),
        })
        .fold(
            (proc_macro2::TokenStream::new(), None::<syn::Error>),
            |mut acc, curr| {
                match curr {
                    Ok(ok) => acc.0.extend(ok),
                    Err(err) => match &mut acc.1 {
                        Some(errs) => {
                            errs.combine(err);
                        }
                        None => acc.1 = Some(err),
                    },
                }
                acc
            },
        );

    match impls {
        (_, Some(errs)) => errs.into_compile_error().into(),
        (impls, None) => impls.into(),
    }
}

fn mk_impls(
    enum_ident: &Ident,
    variant_name: &Ident,
    field_name: &FieldName,
    field_type: &Type,
    (impl_generics, ty_generics, where_clause): &(
        syn::ImplGenerics<'_>,
        syn::TypeGenerics<'_>,
        Option<&syn::WhereClause>,
    ),
) -> proc_macro2::TokenStream {
    quote_spanned! {field_type.span()=>
        #[automatically_derived]
        impl #impl_generics queue_msg::aggregation::SubsetOf<#enum_ident #ty_generics> for #field_type #where_clause {
            fn try_from_super(t: #enum_ident #ty_generics) -> Result<Self, #enum_ident #ty_generics> {
                match t {
                    #enum_ident::#variant_name { #field_name: t, .. } => ::std::result::Result::Ok(t),
                    #[allow(unreachable_patterns)] // triggers on enums with one variant
                    _ => ::std::result::Result::Err(t),
                }
            }

            fn into_super(self) -> #enum_ident #ty_generics {
                #[allow(clippy::init_numbered_fields)]
                #enum_ident::#variant_name { #field_name: self }
            }
        }

        // #[automatically_derived]
        // impl #impl_generics ::std::convert::From<#field_type> for #enum_ident #ty_generics #where_clause {
        //     fn from(value: #field_type) -> Self {
        //         #[allow(clippy::init_numbered_fields)]
        //         #enum_ident::#variant_name { #field_name: value }
        //     }
        // }
    }
}

enum FieldName {
    Index(syn::Index),
    // Ident(&'a syn::Ident),
}

impl ToTokens for FieldName {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            FieldName::Index(i) => i.to_tokens(tokens),
            // FieldName::Ident(i) => i.to_tokens(tokens),
        }
    }
}
