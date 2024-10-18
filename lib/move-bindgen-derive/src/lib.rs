use std::convert;

use quote::{format_ident, quote};
use syn::{
    parse_macro_input, parse_quote, parse_quote_spanned, spanned::Spanned, Data, DeriveInput, Expr,
    Field, MetaNameValue, WhereClause, WherePredicate,
};

#[proc_macro_derive(MoveOutputType, attributes(move_output_type))]
pub fn type_tagged(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_type_tagged(parse_macro_input!(ts as DeriveInput))
        // .inspect(|x| println!("{x}"))
        .map_err(|e| e.into_compile_error())
        .unwrap_or_else(convert::identity)
        .into()
}

fn derive_type_tagged(
    DeriveInput {
        attrs,
        ident,
        generics,
        data,
        ..
    }: DeriveInput,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let (impl_generics, ty_generics, original_where_clause) = generics.split_for_impl();

    let bounds = mk_where_clause(&data);

    let mut where_clause = original_where_clause
        .cloned()
        .unwrap_or_else(|| WhereClause {
            where_token: parse_quote!(where),
            predicates: Default::default(),
        });
    where_clause.predicates.extend(bounds.clone());

    let [module] = attrs
        .into_iter()
        .filter(|attr| attr.path().is_ident("move_output_type"))
        .map(|attr| {
            let mnv =
                syn::parse2::<MetaNameValue>(attr.meta.require_list().unwrap().tokens.clone())
                    .unwrap();

            assert!(mnv.path.is_ident("module"));
            match &mnv.value {
                Expr::Path(expr) => expr.path.require_ident().unwrap().clone(),
                _ => panic!("???"),
            }
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    let (raw_ident, raw, from_raw, into_raw) = match data {
        Data::Struct(data) => {
            let raw_ident = format_ident!("Raw{ident}");

            let from_raw = {
                let fields = data.fields.iter().map(|field| {
                    let name = field
                        .ident
                        .as_ref()
                        .expect("tuple structs are not supported by move");
                    let ty = &field.ty;
                    quote! {
                        #name: <#ty as ::move_bindgen::MoveOutputType>::from_raw(raw.#name)
                    }
                });

                quote! {
                    #ident {
                        #(#fields,)*
                    }
                }
            };

            let into_raw = {
                let fields = data.fields.iter().map(|field| {
                    let name = field
                        .ident
                        .as_ref()
                        .expect("tuple structs are not supported by move");
                    let ty = &field.ty;
                    quote! {
                        #name: <#ty as ::move_bindgen::MoveOutputType>::into_raw(self.#name)
                    }
                });

                quote! {
                    #raw_ident {
                        #(#fields,)*
                    }
                }
            };

            let fields = data.fields.iter().map(|field| {
                let name = field
                    .ident
                    .as_ref()
                    .expect("tuple structs are not supported by move");
                let ty = &field.ty;
                quote! {
                    pub #name: <#ty as ::move_bindgen::MoveOutputType>::Raw
                }
            });

            let raw = quote! {
                #[derive(
                    Debug,
                    ::move_bindgen::serde::Serialize,
                    ::move_bindgen::serde::Deserialize,
                )]
                #[serde(crate = "::move_bindgen::serde")]
                pub struct #raw_ident {
                    #(#fields,)*
                }
            };

            (raw_ident, raw, from_raw, into_raw)
        }
        _ => panic!(),
    };

    // let ctxs = bounds.iter().filter_map(|wp| match wp {
    //     WherePredicate::Type(PredicateType { bounded_ty, .. }) => {
    //         Some(quote!(<#bounded_ty as ::move_bindgen::TypeTagged>::Ctx))
    //     }
    //     _ => None,
    // });

    Ok(quote! {
        const _: () = {
            #[automatically_derived]
            impl #impl_generics ::move_bindgen::MoveOutputType for #ident #ty_generics #where_clause {
                type Raw = #raw_ident;

                fn from_raw(raw: Self::Raw) -> Self {
                    #from_raw
                }

                fn into_raw(self) -> Self::Raw {
                    #into_raw
                }
            }

            // #[automatically_derived]
            // impl #impl_generics #ident #ty_generics #original_where_clause {
            //     pub fn with_address(self, address: ::move_bindgen::move_core_types::account_address::AccountAddress) -> (Self, ::move_bindgen::move_core_types::account_address::AccountAddress) {
            //         (self, address)
            //     }
            // }
        };

        #raw
    })
}

fn mk_where_clause(data: &Data) -> Vec<WherePredicate> {
    let f = |Field { ty, .. }: &Field| parse_quote_spanned!(ty.span()=> #ty: ::move_bindgen::MoveOutputType);

    match data {
        Data::Struct(s) => s.fields.iter().map(f).collect(),
        Data::Enum(e) => e.variants.iter().flat_map(|v| &v.fields).map(f).collect(),
        Data::Union(_) => panic!(),
    }
}
