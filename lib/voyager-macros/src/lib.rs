use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Item, ItemEnum, ItemStruct};

#[proc_macro_attribute]
pub fn model(
    meta: proc_macro::TokenStream,
    ts: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(ts.clone());

    let item = syn::parse_macro_input!(ts as Item);

    if !meta.is_empty() {
        return syn::Error::new(
            TokenStream::from(meta).span(),
            "#[model] takes no arguments",
        )
        .into_compile_error()
        .into();
    }

    let attributes = match item {
        Item::Enum(ItemEnum { .. }) => {
            let serde = quote! {
                #[derive(::serde::Serialize, ::serde::Deserialize)]
                #[serde(
                    deny_unknown_fields,
                    tag = "@type",
                    content = "@value",
                    rename_all = "snake_case"
                )]
            };

            quote! {
                #[derive(
                    ::core::fmt::Debug,
                    ::core::clone::Clone,
                    ::core::cmp::PartialEq,
                    ::core::cmp::Eq,
                )]
                #serde
            }
        }
        Item::Struct(ItemStruct { .. }) => {
            let serde = quote! {
                #[derive(::serde::Serialize, ::serde::Deserialize)]
                #[serde(
                    deny_unknown_fields,
                    rename_all = "snake_case"
                )]
            };

            quote! {
                #[derive(
                    ::core::fmt::Debug,
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
    }
    .into()
}
