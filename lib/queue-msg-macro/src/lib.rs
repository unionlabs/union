use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote_spanned, ToTokens};
use syn::{parse_macro_input, parse_quote, spanned::Spanned, DeriveInput, Type};

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
