#![feature(proc_macro_quote)]

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Attribute, Data::Struct, DataStruct,
    DeriveInput, Error, Fields, GenericParam, Generics, Meta,
};

#[proc_macro_attribute]
pub fn msg_struct(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let result = apply_item(input).unwrap_or_else(|error| error.to_compile_error());

    result.into()
}

fn apply_item(derive_input: DeriveInput) -> Result<proc_macro2::TokenStream, Error> {
    let derive_input = &mut derive_input.clone();

    match &mut derive_input.data {
        Struct(data_struct) => {
            match &mut data_struct.fields {
                Fields::Named(field_named) => {
                    let struct_ident = &derive_input.ident.clone();

                    // copy the fields, before we're adding them in 'cover_types' below
                    let fields = &field_named
                        .named
                        .iter()
                        .map(|field| field.clone().ident.expect("a field Ident for Named field"))
                        .collect::<Vec<Ident>>();

                    let type_params_to_cover = extract_covered_types(&mut derive_input.generics);

                    let clone_marker_fields = if let Some(type_params_to_cover) =
                        &type_params_to_cover
                    {
                        add_marker_field_to_struct(data_struct, type_params_to_cover);

                        quote! {
                            __marker: ::core::marker::PhantomData::<fn() -> (#(#type_params_to_cover),*)>
                        }
                    } else {
                        proc_macro2::TokenStream::new()
                    };

                    let (impl_generics, ty_generics, where_clause) =
                        &derive_input.generics.split_for_impl();

                    Ok(parse_quote!(
                        #[derive(::serde::Serialize, ::serde::Deserialize)]
                        #[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
                        #[cfg_attr(
                            feature = "arbitrary",
                            derive(arbitrary::Arbitrary),
                            arbitrary(bound = "")
                        )]

                        #derive_input

                        const _: () = {
                            impl #impl_generics ::core::fmt::Debug for #struct_ident #ty_generics #where_clause {
                                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                                    f.debug_struct(stringify!(#struct_ident))
                                        #(.field(stringify!(#fields), &self.#fields))*
                                        .finish()
                                }
                            }

                            impl #impl_generics ::core::clone::Clone for #struct_ident #ty_generics #where_clause {
                                fn clone(&self) -> Self {
                                    Self {
                                        #(#fields: ::core::clone::Clone::clone(&self.#fields),)*
                                        #clone_marker_fields
                                    }
                                }
                            }

                            impl #impl_generics ::core::cmp::PartialEq for #struct_ident #ty_generics #where_clause {
                                fn eq(&self, other: &Self) -> bool {
                                    let _other = other;
                                    true #(&& self.#fields == _other.#fields)*
                                }
                            }
                        };
                    ))
                }
                Fields::Unnamed(field_unnamed) => {
                    if field_unnamed.unnamed.len() != 1 {
                        return Err(Error::new(
                            field_unnamed.span(),
                            "Only support one parameter",
                        ));
                    }

                    let struct_ident = derive_input.ident.clone();

                    let (impl_generics, ty_generics, where_clause) =
                        &derive_input.generics.split_for_impl();

                    Ok(parse_quote!(
                        #[derive(::serde::Serialize, ::serde::Deserialize)]
                        #[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields, transparent)]
                        #[cfg_attr(
                            feature = "arbitrary",
                            derive(arbitrary::Arbitrary),
                            arbitrary(bound = "")
                        )]

                        #derive_input

                        const _: () = {
                            impl #impl_generics ::core::fmt::Debug for #struct_ident #ty_generics #where_clause {
                                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                                    f.debug_tuple(stringify!(#struct_ident))
                                        .field(&self.0)
                                        .finish()
                                }
                            }

                            impl #impl_generics ::core::clone::Clone for #struct_ident #ty_generics #where_clause {
                                fn clone(&self) -> Self {
                                    Self(
                                        ::core::clone::Clone::clone(&self.0)
                                    )
                                }
                            }

                            impl #impl_generics ::core::cmp::PartialEq for #struct_ident #ty_generics #where_clause {
                                fn eq(&self, other: &Self) -> bool {
                                    self.0 == other.0
                                }
                            }
                        };
                    ))
                }
                _ => Err(Error::new(
                    Span::call_site(),
                    "Only supports Named and Unnamed fields",
                )),
            }
        }
        _ => Err(Error::new(Span::call_site(), "Only support Structs")),
    }
}

// inject `__marker: PhantomData<fn -> ({type params})>` fields into struct
fn add_marker_field_to_struct(data_struct: &mut DataStruct, type_params_to_cover: &Vec<Ident>) {
    match &mut data_struct.fields {
        Fields::Named(fields_named) => {
            fields_named.named.push(parse_quote! {
                #[serde(skip)]
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
