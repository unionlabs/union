use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{Fields, Ident, ItemStruct};

use crate::helpers::{contains_skip, declaration, quote_where_clause};

pub fn process_struct(input: &ItemStruct, cratename: Ident) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let name_str = name.to_token_stream().to_string();
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    // Generate function that returns the name of the type.
    let (declaration, mut where_clause_additions) =
        declaration(&name_str, &input.generics, cratename.clone());

    // Generate function that returns the schema of required types.
    let mut fields_vec = vec![];
    let mut struct_fields = TokenStream2::new();
    let mut add_definitions_recursively_rec = TokenStream2::new();
    match &input.fields {
        Fields::Named(fields) => {
            for field in &fields.named {
                if contains_skip(&field.attrs) {
                    continue;
                }
                let field_name = field.ident.as_ref().unwrap().to_token_stream().to_string();
                let field_type = &field.ty;
                fields_vec.push(quote! {
                    (#field_name.to_string(), <#field_type>::declaration())
                });
                add_definitions_recursively_rec.extend(quote! {
                    <#field_type>::add_definitions_recursively(definitions);
                });
                where_clause_additions.push(quote! {
                    #field_type: #cratename::BorshSchema
                });
            }
            if !fields_vec.is_empty() {
                struct_fields = quote! {
                    let fields = #cratename::schema::Fields::NamedFields(#cratename::maybestd::vec![#(#fields_vec),*]);
                };
            }
        }
        Fields::Unnamed(fields) => {
            for field in &fields.unnamed {
                if contains_skip(&field.attrs) {
                    continue;
                }
                let field_type = &field.ty;
                fields_vec.push(quote! {
                    <#field_type>::declaration()
                });
                add_definitions_recursively_rec.extend(quote! {
                    <#field_type>::add_definitions_recursively(definitions);
                });
                where_clause_additions.push(quote! {
                    #field_type: #cratename::BorshSchema
                });
            }
            if !fields_vec.is_empty() {
                struct_fields = quote! {
                    let fields = #cratename::schema::Fields::UnnamedFields(#cratename::maybestd::vec![#(#fields_vec),*]);
                };
            }
        }
        Fields::Unit => {}
    }

    if fields_vec.is_empty() {
        struct_fields = quote! {
            let fields = #cratename::schema::Fields::Empty;
        };
    }

    let add_definitions_recursively = quote! {
        fn add_definitions_recursively(definitions: &mut #cratename::maybestd::collections::HashMap<#cratename::schema::Declaration, #cratename::schema::Definition>) {
            #struct_fields
            let definition = #cratename::schema::Definition::Struct { fields };
            Self::add_definition(Self::declaration(), definition, definitions);
            #add_definitions_recursively_rec
        }
    };
    let where_clause = quote_where_clause(where_clause, where_clause_additions);
    Ok(quote! {
        impl #impl_generics #cratename::BorshSchema for #name #ty_generics #where_clause {
            fn declaration() -> #cratename::schema::Declaration {
                #declaration
            }
            #add_definitions_recursively
        }
    })
}

// Rustfmt removes comas.
#[rustfmt::skip::macros(quote)]
#[cfg(test)]
mod tests {
    use super::*;

    fn assert_eq(expected: TokenStream2, actual: TokenStream2) {
        pretty_assertions::assert_eq!(expected.to_string(), actual.to_string())
    }

    #[test]
    fn unit_struct() {
        let item_struct: ItemStruct = syn::parse2(quote!{
            struct A;
        })
        .unwrap();

        let actual = process_struct(
            &item_struct,
            Ident::new("borsh", proc_macro2::Span::call_site()),
        )
        .unwrap();
        let expected = quote!{
            impl borsh::BorshSchema for A
            {
                fn declaration() -> borsh::schema::Declaration {
                    "A".to_string()
                }
                fn add_definitions_recursively(definitions: &mut borsh::maybestd::collections::HashMap<borsh::schema::Declaration, borsh::schema::Definition>) {
                    let fields = borsh::schema::Fields::Empty;
                    let definition = borsh::schema::Definition::Struct { fields };
                    Self::add_definition(Self::declaration(), definition, definitions);
                }
            }
        };
        assert_eq(expected, actual);
    }

    #[test]
    fn wrapper_struct() {
        let item_struct: ItemStruct = syn::parse2(quote!{
            struct A<T>(T);
        })
        .unwrap();

        let actual = process_struct(
            &item_struct,
            Ident::new("borsh", proc_macro2::Span::call_site()),
        )
        .unwrap();
        let expected = quote!{
            impl<T> borsh::BorshSchema for A<T>
            where
                T: borsh::BorshSchema,
                T: borsh::BorshSchema
            {
                fn declaration() -> borsh::schema::Declaration {
                    let params = borsh::maybestd::vec![<T>::declaration()];
                    format!(r#"{}<{}>"#, "A", params.join(", "))
                }
                fn add_definitions_recursively(
                    definitions: &mut borsh::maybestd::collections::HashMap<
                        borsh::schema::Declaration,
                        borsh::schema::Definition
                    >
                ) {
                    let fields = borsh::schema::Fields::UnnamedFields(borsh::maybestd::vec![<T>::declaration()]);
                    let definition = borsh::schema::Definition::Struct { fields };
                    Self::add_definition(Self::declaration(), definition, definitions);
                    <T>::add_definitions_recursively(definitions);
                }
            }
        };
        assert_eq(expected, actual);
    }

    #[test]
    fn tuple_struct() {
        let item_struct: ItemStruct = syn::parse2(quote!{
            struct A(u64, String);
        })
        .unwrap();

        let actual = process_struct(
            &item_struct,
            Ident::new("borsh", proc_macro2::Span::call_site()),
        )
        .unwrap();
        let expected = quote!{
            impl borsh::BorshSchema for A
            where
                u64: borsh::BorshSchema,
                String: borsh::BorshSchema
            {
                fn declaration() -> borsh::schema::Declaration {
                    "A".to_string()
                }
                fn add_definitions_recursively(
                    definitions: &mut borsh::maybestd::collections::HashMap<
                        borsh::schema::Declaration,
                        borsh::schema::Definition
                    >
                ) {
                    let fields = borsh::schema::Fields::UnnamedFields(borsh::maybestd::vec![
                        <u64>::declaration(),
                        <String>::declaration()
                    ]);
                    let definition = borsh::schema::Definition::Struct { fields };
                    Self::add_definition(Self::declaration(), definition, definitions);
                    <u64>::add_definitions_recursively(definitions);
                    <String>::add_definitions_recursively(definitions);
                }
            }
        };
        assert_eq(expected, actual);
    }

    #[test]
    fn tuple_struct_params() {
        let item_struct: ItemStruct = syn::parse2(quote!{
            struct A<K, V>(K, V);
        })
        .unwrap();

        let actual = process_struct(
            &item_struct,
            Ident::new("borsh", proc_macro2::Span::call_site()),
        )
        .unwrap();
        let expected = quote!{
            impl<K, V> borsh::BorshSchema for A<K, V>
            where
                K: borsh::BorshSchema,
                V: borsh::BorshSchema,
                K: borsh::BorshSchema,
                V: borsh::BorshSchema
            {
                fn declaration() -> borsh::schema::Declaration {
                    let params = borsh::maybestd::vec![<K>::declaration(), <V>::declaration()];
                    format!(r#"{}<{}>"#, "A", params.join(", "))
                }
                fn add_definitions_recursively(
                    definitions: &mut borsh::maybestd::collections::HashMap<
                        borsh::schema::Declaration,
                        borsh::schema::Definition
                    >
                ) {
                    let fields =
                        borsh::schema::Fields::UnnamedFields(borsh::maybestd::vec![<K>::declaration(), <V>::declaration()]);
                    let definition = borsh::schema::Definition::Struct { fields };
                    Self::add_definition(Self::declaration(), definition, definitions);
                    <K>::add_definitions_recursively(definitions);
                    <V>::add_definitions_recursively(definitions);
                }
            }
        };
        assert_eq(expected, actual);
    }

    #[test]
    fn simple_struct() {
        let item_struct: ItemStruct = syn::parse2(quote!{
            struct A {
                x: u64,
                y: String,
            }
        })
        .unwrap();

        let actual = process_struct(
            &item_struct,
            Ident::new("borsh", proc_macro2::Span::call_site()),
        )
        .unwrap();
        let expected = quote!{
            impl borsh::BorshSchema for A
            where
                u64: borsh::BorshSchema,
                String: borsh::BorshSchema
            {
                fn declaration() -> borsh::schema::Declaration {
                    "A".to_string()
                }
                fn add_definitions_recursively(
                    definitions: &mut borsh::maybestd::collections::HashMap<
                        borsh::schema::Declaration,
                        borsh::schema::Definition
                    >
                ) {
                    let fields = borsh::schema::Fields::NamedFields(borsh::maybestd::vec![
                        ("x".to_string(), <u64>::declaration()),
                        ("y".to_string(), <String>::declaration())
                    ]);
                    let definition = borsh::schema::Definition::Struct { fields };
                    Self::add_definition(Self::declaration(), definition, definitions);
                    <u64>::add_definitions_recursively(definitions);
                    <String>::add_definitions_recursively(definitions);
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
        })
        .unwrap();

        let actual = process_struct(
            &item_struct,
            Ident::new("borsh", proc_macro2::Span::call_site()),
        )
        .unwrap();
        let expected = quote!{
            impl<K, V> borsh::BorshSchema for A<K, V>
            where
                K: borsh::BorshSchema,
                V: borsh::BorshSchema,
                HashMap<K, V>: borsh::BorshSchema,
                String: borsh::BorshSchema
            {
                fn declaration() -> borsh::schema::Declaration {
                    let params = borsh::maybestd::vec![<K>::declaration(), <V>::declaration()];
                    format!(r#"{}<{}>"#, "A", params.join(", "))
                }
                fn add_definitions_recursively(
                    definitions: &mut borsh::maybestd::collections::HashMap<
                        borsh::schema::Declaration,
                        borsh::schema::Definition
                    >
                ) {
                    let fields = borsh::schema::Fields::NamedFields(borsh::maybestd::vec![
                        ("x".to_string(), <HashMap<K, V> >::declaration()),
                        ("y".to_string(), <String>::declaration())
                    ]);
                    let definition = borsh::schema::Definition::Struct { fields };
                    Self::add_definition(Self::declaration(), definition, definitions);
                    <HashMap<K, V> >::add_definitions_recursively(definitions);
                    <String>::add_definitions_recursively(definitions);
                }
            }
        };
        assert_eq(expected, actual);
    }

    #[test]
    fn trailing_comma_generics() {
        let item_struct: ItemStruct = syn::parse2(quote!{
            struct A<K, V>
            where
                K: Display + Debug,
            {
                x: HashMap<K, V>,
                y: String,
            }
        })
        .unwrap();

        let actual = process_struct(
            &item_struct,
            Ident::new("borsh", proc_macro2::Span::call_site()),
        )
        .unwrap();
        let expected = quote!{
            impl<K, V> borsh::BorshSchema for A<K, V>
            where
                K: Display + Debug,
                K: borsh::BorshSchema,
                V: borsh::BorshSchema,
                HashMap<K, V>: borsh::BorshSchema,
                String: borsh::BorshSchema
            {
                fn declaration() -> borsh::schema::Declaration {
                    let params = borsh::maybestd::vec![<K>::declaration(), <V>::declaration()];
                    format!(r#"{}<{}>"#, "A", params.join(", "))
                }
                fn add_definitions_recursively(
                    definitions: &mut borsh::maybestd::collections::HashMap<
                        borsh::schema::Declaration,
                        borsh::schema::Definition
                    >
                ) {
                    let fields = borsh::schema::Fields::NamedFields(borsh::maybestd::vec![
                        ("x".to_string(), <HashMap<K, V> >::declaration()),
                        ("y".to_string(), <String>::declaration())
                    ]);
                    let definition = borsh::schema::Definition::Struct { fields };
                    Self::add_definition(Self::declaration(), definition, definitions);
                    <HashMap<K, V> >::add_definitions_recursively(definitions);
                    <String>::add_definitions_recursively(definitions);
                }
            }
        };
        assert_eq(expected, actual);
    }

    #[test]
    fn tuple_struct_whole_skip() {
        let item_struct: ItemStruct = syn::parse2(quote!{
            struct A(#[borsh_skip] String);
        })
        .unwrap();

        let actual = process_struct(
            &item_struct,
            Ident::new("borsh", proc_macro2::Span::call_site()),
        )
        .unwrap();
        let expected = quote!{
            impl borsh::BorshSchema for A {
                fn declaration() -> borsh::schema::Declaration {
                    "A".to_string()
                }
                fn add_definitions_recursively(
                    definitions: &mut borsh::maybestd::collections::HashMap<
                        borsh::schema::Declaration,
                        borsh::schema::Definition
                    >
                ) {
                    let fields = borsh::schema::Fields::Empty;
                    let definition = borsh::schema::Definition::Struct { fields };
                    Self::add_definition(Self::declaration(), definition, definitions);
                }
            }
        };
        assert_eq(expected, actual);
    }

    #[test]
    fn tuple_struct_partial_skip() {
        let item_struct: ItemStruct = syn::parse2(quote!{
            struct A(#[borsh_skip] u64, String);
        })
        .unwrap();

        let actual = process_struct(
            &item_struct,
            Ident::new("borsh", proc_macro2::Span::call_site()),
        )
        .unwrap();
        let expected = quote!{
            impl borsh::BorshSchema for A
            where
                String: borsh::BorshSchema
            {
                fn declaration() -> borsh::schema::Declaration {
                    "A".to_string()
                }
                fn add_definitions_recursively(
                    definitions: &mut borsh::maybestd::collections::HashMap<
                        borsh::schema::Declaration,
                        borsh::schema::Definition
                    >
                ) {
                    let fields = borsh::schema::Fields::UnnamedFields(borsh::maybestd::vec![<String>::declaration()]);
                    let definition = borsh::schema::Definition::Struct { fields };
                    Self::add_definition(Self::declaration(), definition, definitions);
                    <String>::add_definitions_recursively(definitions);
                }
            }
        };
        assert_eq(expected, actual);
    }
}
