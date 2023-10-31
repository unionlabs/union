extern crate proc_macro;

use proc_macro2::TokenStream;
use proc_macro_error::{abort, proc_macro_error, set_dummy};
use quote::{quote, quote_spanned, ToTokens};
use syn::{spanned::Spanned, *};

mod parse;

use crate::parse::WasmerAttr;

#[proc_macro_error]
#[proc_macro_derive(WasmerEnv, attributes(wasmer))]
pub fn derive_wasmer_env(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    let gen = impl_wasmer_env(&input);
    gen.into()
}

fn impl_wasmer_env_for_struct(
    name: &Ident,
    data: &DataStruct,
    generics: &Generics,
    _attrs: &[Attribute],
) -> TokenStream {
    let (trait_methods, helper_methods) = derive_struct_fields(data);
    let lifetimes_and_generics = generics.params.clone();
    let where_clause = generics.where_clause.clone();
    quote! {
        impl < #lifetimes_and_generics > ::wasmer::WasmerEnv for #name < #lifetimes_and_generics > #where_clause{
            #trait_methods
        }

        #[allow(dead_code)]
        impl < #lifetimes_and_generics > #name < #lifetimes_and_generics > #where_clause {
            #helper_methods
        }
    }
}

fn impl_wasmer_env(input: &DeriveInput) -> TokenStream {
    let struct_name = &input.ident;

    set_dummy(quote! {
        impl ::wasmer::WasmerEnv for #struct_name {
            fn init_with_instance(&mut self, instance: &::wasmer::Instance) -> ::core::result::Result<(), ::wasmer::HostEnvInitError> {
                Ok(())
            }
        }
    });

    match &input.data {
        Data::Struct(ds) => {
            impl_wasmer_env_for_struct(struct_name, ds, &input.generics, &input.attrs)
        }
        _ => todo!(),
    }
    /*match input.data {
        Struct(ds /*DataStruct {
            fields: syn::Fields::Named(ref fields),
            ..
        }*/) => ,
        Enum(ref e) => impl_wasmer_env_for_enum(struct_name, &e.variants, &input.attrs),
        _ => abort_call_site!("Clap only supports non-tuple structs and enums"),
    }*/
}

fn derive_struct_fields(data: &DataStruct) -> (TokenStream, TokenStream) {
    let mut finish = vec![];
    let mut helpers = vec![];
    //let mut assign_tokens = vec![];
    let mut touched_fields = vec![];
    let fields: Vec<Field> = match &data.fields {
        Fields::Named(ref fields) => fields.named.iter().cloned().collect(),
        Fields::Unit => vec![],
        Fields::Unnamed(fields) => fields.unnamed.iter().cloned().collect(),
    };
    for (field_num, f) in fields.into_iter().enumerate() {
        let field_idx = syn::Index::from(field_num);
        let name = f.ident.clone();
        let top_level_ty: &Type = &f.ty;
        touched_fields.push(name.clone());
        let mut wasmer_attr = None;
        for attr in &f.attrs {
            // if / filter
            if attr.path.is_ident(&Ident::new("wasmer", attr.span())) {
                let tokens = attr.tokens.clone();
                match syn::parse2(tokens) {
                    Ok(attr) => {
                        wasmer_attr = Some(attr);
                        break;
                    }
                    Err(e) => {
                        abort!(attr, "Failed to parse `wasmer` attribute: {}", e);
                    }
                }
            }
        }

        if let Some(wasmer_attr) = wasmer_attr {
            let inner_type = get_identifier(top_level_ty);
            if let Some(name) = &name {
                let name_ref_str = format!("{}_ref", name);
                let name_ref = syn::Ident::new(&name_ref_str, name.span());
                let name_ref_unchecked_str = format!("{}_ref_unchecked", name);
                let name_ref_unchecked = syn::Ident::new(&name_ref_unchecked_str, name.span());
                let helper_tokens = quote_spanned! {f.span()=>
                    /// Get access to the underlying data.
                    ///
                    /// If `WasmerEnv::finish` has been called, this function will never
                    /// return `None` unless the underlying data has been mutated manually.
                    pub fn #name_ref(&self) -> Option<&#inner_type> {
                        self.#name.get_ref()
                    }
                    /// Gets the item without checking if it's been initialized.
                    ///
                    /// # Safety
                    /// `WasmerEnv::finish` must have been called on this function or
                    /// this type manually initialized.
                    pub unsafe fn #name_ref_unchecked(&self) -> &#inner_type {
                        self.#name.get_unchecked()
                    }
                };
                helpers.push(helper_tokens);
            }
            match wasmer_attr {
                WasmerAttr::Export {
                    identifier,
                    optional,
                    aliases,
                    span,
                } => {
                    let finish_tokens = if let Some(name) = name {
                        let name_str = name.to_string();
                        let item_name =
                            identifier.unwrap_or_else(|| LitStr::new(&name_str, name.span()));
                        let mut access_expr = quote_spanned! {
                            f.span() =>
                                instance.exports.get_with_generics_weak::<#inner_type, _, _>(#item_name)
                        };
                        for alias in aliases {
                            access_expr = quote_spanned! {
                                f.span()=>
                                    #access_expr .or_else(|_| instance.exports.get_with_generics_weak::<#inner_type, _, _>(#alias))
                            };
                        }
                        if optional {
                            quote_spanned! {
                                f.span()=>
                                    match #access_expr {
                                        Ok(#name) => { self.#name.initialize(#name); },
                                        Err(_) => (),
                                    };
                            }
                        } else {
                            quote_spanned! {
                                f.span()=>
                                    let #name: #inner_type = #access_expr?;
                                    self.#name.initialize(#name);
                            }
                        }
                    } else {
                        if let Some(identifier) = identifier {
                            let mut access_expr = quote_spanned! {
                                f.span() =>
                                    instance.exports.get_with_generics_weak::<#inner_type, _, _>(#identifier)
                            };
                            for alias in aliases {
                                access_expr = quote_spanned! {
                                    f.span()=>
                                        #access_expr .or_else(|_| instance.exports.get_with_generics_weak::<#inner_type, _, _>(#alias))
                                };
                            }
                            let local_var =
                                Ident::new(&format!("field_{}", field_num), identifier.span());
                            if optional {
                                quote_spanned! {
                                    f.span()=>
                                        match #access_expr {
                                            Ok(#local_var) => {
                                                self.#field_idx.initialize(#local_var);
                                            },
                                            Err(_) => (),
                                        }
                                }
                            } else {
                                quote_spanned! {
                                    f.span()=>
                                        let #local_var: #inner_type = #access_expr?;
                                    self.#field_idx.initialize(#local_var);
                                }
                            }
                        } else {
                            abort!(
                                span,
                                "Expected `name` field on export attribute because field does not have a name. For example: `#[wasmer(export(name = \"wasm_ident\"))]`.",
                            );
                        }
                    };

                    finish.push(finish_tokens);
                }
            }
        }
    }

    let trait_methods = quote! {
        fn init_with_instance(&mut self, instance: &::wasmer::Instance) -> ::core::result::Result<(), ::wasmer::HostEnvInitError> {
            #(#finish)*
            Ok(())
        }
    };

    let helper_methods = quote! {
        #(#helpers)*
    };

    (trait_methods, helper_methods)
}

// TODO: name this something that makes sense
fn get_identifier(ty: &Type) -> TokenStream {
    match ty {
        Type::Path(TypePath {
            path: Path { segments, .. },
            ..
        }) => {
            if let Some(PathSegment { ident, arguments }) = segments.last() {
                if ident != "LazyInit" {
                    abort!(
                        ident,
                        "WasmerEnv derive expects all `export`s to be wrapped in `LazyInit`"
                    );
                }
                if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                    args, ..
                }) = arguments
                {
                    // TODO: proper error handling
                    assert_eq!(args.len(), 1);
                    if let GenericArgument::Type(Type::Path(TypePath {
                        path: Path { segments, .. },
                        ..
                    })) = &args[0]
                    {
                        segments
                            .last()
                            .expect("there must be at least one segment; TODO: error handling")
                            .to_token_stream()
                    } else {
                        abort!(
                            &args[0],
                            "unrecognized type in first generic position on `LazyInit`"
                        );
                    }
                } else {
                    abort!(arguments, "Expected a generic parameter on `LazyInit`");
                }
            } else {
                abort!(segments, "Unknown type found");
            }
        }
        _ => abort!(ty, "Unrecognized/unsupported type"),
    }
}
