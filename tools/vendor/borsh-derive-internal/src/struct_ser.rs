use core::convert::TryFrom;

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Fields, Ident, Index, ItemStruct, WhereClause};

use crate::attribute_helpers::contains_skip;

pub fn struct_ser(input: &ItemStruct, cratename: Ident) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut where_clause = where_clause.map_or_else(
        || WhereClause {
            where_token: Default::default(),
            predicates: Default::default(),
        },
        Clone::clone,
    );
    let mut body = TokenStream2::new();
    match &input.fields {
        Fields::Named(fields) => {
            for field in &fields.named {
                if contains_skip(&field.attrs) {
                    continue;
                }
                let field_name = field.ident.as_ref().unwrap();
                let delta = quote! {
                    #cratename::BorshSerialize::serialize(&self.#field_name, writer)?;
                };
                body.extend(delta);

                let field_type = &field.ty;
                where_clause.predicates.push(
                    syn::parse2(quote! {
                        #field_type: #cratename::ser::BorshSerialize
                    })
                    .unwrap(),
                );
            }
        }
        Fields::Unnamed(fields) => {
            for field_idx in 0..fields.unnamed.len() {
                let field_idx = Index {
                    index: u32::try_from(field_idx).expect("up to 2^32 fields are supported"),
                    span: Span::call_site(),
                };
                let delta = quote! {
                    #cratename::BorshSerialize::serialize(&self.#field_idx, writer)?;
                };
                body.extend(delta);
            }
        }
        Fields::Unit => {}
    }
    Ok(quote! {
        impl #impl_generics #cratename::ser::BorshSerialize for #name #ty_generics #where_clause {
            fn serialize<W: #cratename::maybestd::io::Write>(&self, writer: &mut W) -> ::core::result::Result<(), #cratename::maybestd::io::Error> {
                #body
                Ok(())
            }
        }
    })
}

// Rustfmt removes comas.
#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use super::*;

    fn assert_eq(expected: TokenStream2, actual: TokenStream2) {
        assert_eq!(expected.to_string(), actual.to_string())
    }

    #[test]
    fn simple_struct() {
        let item_struct: ItemStruct = syn::parse2(quote!{
            struct A {
                x: u64,
                y: String,
            }
        }).unwrap();

        let actual = struct_ser(&item_struct, Ident::new("borsh", Span::call_site())).unwrap();
        let expected = quote!{
            impl borsh::ser::BorshSerialize for A
            where
                u64: borsh::ser::BorshSerialize,
                String: borsh::ser::BorshSerialize
            {
                fn serialize<W: borsh::maybestd::io::Write>(&self, writer: &mut W) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                    borsh::BorshSerialize::serialize(&self.x, writer)?;
                    borsh::BorshSerialize::serialize(&self.y, writer)?;
                    Ok(())
                }
            }
        };
        assert_eq(expected, actual);
    }

    #[test]
    fn simple_generics() {
        let item_struct: ItemStruct = syn::parse2(quote!{
            struct A<K, V> {
                x: HashMap<K, V>,
                y: String,
            }
        }).unwrap();

        let actual = struct_ser(&item_struct, Ident::new("borsh", Span::call_site())).unwrap();
        let expected = quote!{
            impl<K, V> borsh::ser::BorshSerialize for A<K, V>
            where
                HashMap<K, V>: borsh::ser::BorshSerialize,
                String: borsh::ser::BorshSerialize
            {
                fn serialize<W: borsh::maybestd::io::Write>(&self, writer: &mut W) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                    borsh::BorshSerialize::serialize(&self.x, writer)?;
                    borsh::BorshSerialize::serialize(&self.y, writer)?;
                    Ok(())
                }
            }
        };
        assert_eq(expected, actual);
    }

    #[test]
    fn bound_generics() {
        let item_struct: ItemStruct = syn::parse2(quote!{
            struct A<K: Key, V> where V: Value {
                x: HashMap<K, V>,
                y: String,
            }
        }).unwrap();

        let actual = struct_ser(&item_struct, Ident::new("borsh", Span::call_site())).unwrap();
        let expected = quote!{
            impl<K: Key, V> borsh::ser::BorshSerialize for A<K, V>
            where
                V: Value,
                HashMap<K, V>: borsh::ser::BorshSerialize,
                String: borsh::ser::BorshSerialize
            {
                fn serialize<W: borsh::maybestd::io::Write>(&self, writer: &mut W) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                    borsh::BorshSerialize::serialize(&self.x, writer)?;
                    borsh::BorshSerialize::serialize(&self.y, writer)?;
                    Ok(())
                }
            }
        };
        assert_eq(expected, actual);
    }
}
