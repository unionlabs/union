use std::convert;

use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{Attribute, Data, DeriveInput, Fields, LitStr, parse_macro_input, spanned::Spanned};

#[proc_macro_derive(Event, attributes(event))]
pub fn event(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_event(parse_macro_input!(ts as DeriveInput))
        // .inspect(|x| println!("{x}"))
        .map_err(|e| e.into_compile_error())
        .unwrap_or_else(convert::identity)
        .into()
}

fn derive_event(
    DeriveInput {
        attrs,
        ident,
        generics,
        data,
        ..
    }: DeriveInput,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    match data {
        Data::Struct(s) => {
            let Fields::Named(fields) = s.fields else {
                return Err(syn::Error::new(
                    s.fields.span(),
                    "only named fields are supported",
                ));
            };

            let attr_names = fields
                .named
                .iter()
                .map(|field| {
                    let ident = field.ident.as_ref().unwrap();
                    let attr_name = parse_name(field.span(), &field.attrs)?
                        .map_or(ident.to_string(), |s| s.value());
                    Ok::<_, syn::Error>((ident, attr_name))
                })
                .collect::<Result<Vec<_>, _>>()?;

            let attr_methods = attr_names.iter().map(|(_, attr)| {
                let ident = format_ident!("{attr}_attr_key");
                quote! {
                    pub const fn #ident() -> &'static str {
                        #attr
                    }
                }
            });

            let field_add_attributes = attr_names.iter().map(|(ident, attr_name)| {
                quote! {
                    .add_attribute(#attr_name, event.#ident.to_string())
                }
            });

            let kind = parse_name(ident.span(), &attrs)?.ok_or(syn::Error::new(
                ident.span(),
                "must provide one #[event(\"name\")]",
            ))?;

            let wasm_kind = format!("wasm-{}", kind.value());

            Ok(quote! {
                impl #impl_generics ::core::convert::From<#ident #ty_generics> for ::cosmwasm_std::Event #where_clause {
                    fn from(event: #ident #ty_generics) -> Self {
                        ::cosmwasm_std::Event::new(#kind)
                            #(#field_add_attributes)*
                    }
                }

                impl #impl_generics #ident #ty_generics #where_clause {
                    pub const fn wasm_ty() -> &'static str {
                        #wasm_kind
                    }

                    pub const fn ty() -> &'static str {
                        #kind
                    }

                    #(#attr_methods)*
                }
            })
        }
        _ => Err(syn::Error::new(ident.span(), "only structs are supported")),
    }
}

fn parse_name(span: Span, attrs: &[Attribute]) -> Result<Option<LitStr>, syn::Error> {
    let mut maybe_name = attrs
        .iter()
        .filter_map(|attr| {
            if attr.meta.path().is_ident("event") {
                Some(attr.parse_args::<LitStr>())
            } else {
                None
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    match maybe_name.len() {
        0 | 1 => Ok(maybe_name.pop()),
        _ => Err(syn::Error::new(
            span,
            "must provide only one #[event(\"name\")]",
        )),
    }
}
