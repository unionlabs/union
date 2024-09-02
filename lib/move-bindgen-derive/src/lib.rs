use std::convert;

use quote::quote;
use syn::{
    parse_macro_input, parse_quote, parse_quote_spanned, spanned::Spanned, Data, DeriveInput, Expr,
    Field, MetaNameValue, WhereClause, WherePredicate,
};

#[proc_macro_derive(TypeTagged, attributes(type_tag))]
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
        .filter(|attr| attr.path().is_ident("type_tag"))
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

    let ident_str = ident.to_string();
    let module_str = module.to_string();

    let body = match data {
        Data::Struct(_) => {
            quote! {
                ::move_bindgen::move_core_types::language_storage::TypeTag::Struct(
                    ::std::boxed::Box::new(
                        ::move_bindgen::move_core_types::language_storage::StructTag {
                            address: ctx,
                            module: ::move_bindgen::move_core_types::ident_str!(#module_str).into(),
                            name: ::move_bindgen::move_core_types::ident_str!(#ident_str).into(),
                            type_args: vec![],
                        }
                    )
                )
            }
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
            impl #impl_generics ::move_bindgen::TypeTagged for #ident #ty_generics #where_clause {
                // type Ctx = (#(#ctxs,)* ::move_bindgen::move_core_types::account_address::AccountAddress);
                type Ctx = ::move_bindgen::move_core_types::account_address::AccountAddress;

                fn type_tag(ctx: Self::Ctx) -> ::move_bindgen::move_core_types::language_storage::TypeTag {
                    #body
                }
            }

            #[automatically_derived]
            impl #impl_generics #ident #ty_generics #original_where_clause {
                pub fn with_address(self, address: ::move_bindgen::move_core_types::account_address::AccountAddress) -> (Self, ::move_bindgen::move_core_types::account_address::AccountAddress) {
                    (self, address)
                }
            }
        };
    })
}

fn mk_where_clause(data: &Data) -> Vec<WherePredicate> {
    let f = |Field { ty, .. }: &Field| parse_quote_spanned!(ty.span()=> #ty: ::move_bindgen::TypeTagged);

    match data {
        Data::Struct(s) => s.fields.iter().map(f).collect(),
        Data::Enum(e) => e.variants.iter().flat_map(|v| &v.fields).map(f).collect(),
        Data::Union(_) => panic!(),
    }
}
