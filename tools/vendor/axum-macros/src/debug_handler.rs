use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{parse::Parse, spanned::Spanned, FnArg, ItemFn, Token, Type};

pub(crate) fn expand(attr: Attrs, item_fn: ItemFn) -> TokenStream {
    let check_extractor_count = check_extractor_count(&item_fn);
    let check_request_last_extractor = check_request_last_extractor(&item_fn);
    let check_path_extractor = check_path_extractor(&item_fn);
    let check_multiple_body_extractors = check_multiple_body_extractors(&item_fn);

    let check_inputs_impls_from_request = check_inputs_impls_from_request(&item_fn, &attr.body_ty);
    let check_output_impls_into_response = check_output_impls_into_response(&item_fn);
    let check_future_send = check_future_send(&item_fn);

    quote! {
        #item_fn
        #check_extractor_count
        #check_request_last_extractor
        #check_path_extractor
        #check_multiple_body_extractors
        #check_inputs_impls_from_request
        #check_output_impls_into_response
        #check_future_send
    }
}

pub(crate) struct Attrs {
    body_ty: Type,
}

impl Parse for Attrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut body_ty = None;

        while !input.is_empty() {
            let ident = input.parse::<syn::Ident>()?;
            if ident == "body" {
                input.parse::<Token![=]>()?;
                body_ty = Some(input.parse()?);
            } else {
                return Err(syn::Error::new_spanned(ident, "unknown argument"));
            }

            let _ = input.parse::<Token![,]>();
        }

        let body_ty = body_ty.unwrap_or_else(|| syn::parse_quote!(axum::body::Body));

        Ok(Self { body_ty })
    }
}

fn check_extractor_count(item_fn: &ItemFn) -> Option<TokenStream> {
    let max_extractors = 16;
    if item_fn.sig.inputs.len() <= max_extractors {
        None
    } else {
        let error_message = format!(
            "Handlers cannot take more than {} arguments. \
            Use `(a, b): (ExtractorA, ExtractorA)` to further nest extractors",
            max_extractors,
        );
        let error = syn::Error::new_spanned(&item_fn.sig.inputs, error_message).to_compile_error();
        Some(error)
    }
}

fn extractor_idents(item_fn: &ItemFn) -> impl Iterator<Item = (usize, &syn::FnArg, &syn::Ident)> {
    item_fn
        .sig
        .inputs
        .iter()
        .enumerate()
        .filter_map(|(idx, fn_arg)| match fn_arg {
            FnArg::Receiver(_) => None,
            FnArg::Typed(pat_type) => {
                if let Type::Path(type_path) = &*pat_type.ty {
                    type_path
                        .path
                        .segments
                        .last()
                        .map(|segment| (idx, fn_arg, &segment.ident))
                } else {
                    None
                }
            }
        })
}

fn check_request_last_extractor(item_fn: &ItemFn) -> Option<TokenStream> {
    let request_extractor_ident =
        extractor_idents(item_fn).find(|(_, _, ident)| *ident == "Request");

    if let Some((idx, fn_arg, _)) = request_extractor_ident {
        if idx != item_fn.sig.inputs.len() - 1 {
            return Some(
                syn::Error::new_spanned(fn_arg, "`Request` extractor should always be last")
                    .to_compile_error(),
            );
        }
    }

    None
}

fn check_path_extractor(item_fn: &ItemFn) -> TokenStream {
    let path_extractors = extractor_idents(item_fn)
        .filter(|(_, _, ident)| *ident == "Path")
        .collect::<Vec<_>>();

    if path_extractors.len() > 1 {
        path_extractors
            .into_iter()
            .map(|(_, arg, _)| {
                syn::Error::new_spanned(
                    arg,
                    "Multiple parameters must be extracted with a tuple \
                    `Path<(_, _)>` or a struct `Path<YourParams>`, not by applying \
                    multiple `Path<_>` extractors",
                )
                .to_compile_error()
            })
            .collect()
    } else {
        quote! {}
    }
}

fn check_multiple_body_extractors(item_fn: &ItemFn) -> TokenStream {
    let body_extractors = extractor_idents(item_fn)
        .filter(|(_, _, ident)| {
            *ident == "String"
                || *ident == "Bytes"
                || *ident == "Json"
                || *ident == "RawBody"
                || *ident == "BodyStream"
                || *ident == "Multipart"
                || *ident == "Request"
        })
        .collect::<Vec<_>>();

    if body_extractors.len() > 1 {
        body_extractors
            .into_iter()
            .map(|(_, arg, _)| {
                syn::Error::new_spanned(arg, "Only one body extractor can be applied")
                    .to_compile_error()
            })
            .collect()
    } else {
        quote! {}
    }
}

fn check_inputs_impls_from_request(item_fn: &ItemFn, body_ty: &Type) -> TokenStream {
    if !item_fn.sig.generics.params.is_empty() {
        return syn::Error::new_spanned(
            &item_fn.sig.generics,
            "`#[axum_macros::debug_handler]` doesn't support generic functions",
        )
        .into_compile_error();
    }

    item_fn
        .sig
        .inputs
        .iter()
        .enumerate()
        .map(|(idx, arg)| {
            let (span, ty) = match arg {
                FnArg::Receiver(receiver) => {
                    if receiver.reference.is_some() {
                        return syn::Error::new_spanned(
                            receiver,
                            "Handlers must only take owned values",
                        )
                        .into_compile_error();
                    }

                    let span = receiver.span();
                    (span, syn::parse_quote!(Self))
                }
                FnArg::Typed(typed) => {
                    let ty = &typed.ty;
                    let span = ty.span();
                    (span, ty.clone())
                }
            };

            let name = format_ident!(
                "__axum_macros_check_{}_{}_from_request",
                item_fn.sig.ident,
                idx
            );
            quote_spanned! {span=>
                #[allow(warnings)]
                fn #name()
                where
                    #ty: ::axum::extract::FromRequest<#body_ty> + Send,
                {}
            }
        })
        .collect::<TokenStream>()
}

fn check_output_impls_into_response(item_fn: &ItemFn) -> TokenStream {
    let ty = match &item_fn.sig.output {
        syn::ReturnType::Default => return quote! {},
        syn::ReturnType::Type(_, ty) => ty,
    };
    let span = ty.span();

    let declare_inputs = item_fn
        .sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Receiver(_) => None,
            FnArg::Typed(pat_ty) => {
                let pat = &pat_ty.pat;
                let ty = &pat_ty.ty;
                Some(quote! {
                    let #pat: #ty = panic!();
                })
            }
        })
        .collect::<TokenStream>();

    let block = &item_fn.block;

    let make_value_name = format_ident!(
        "__axum_macros_check_{}_into_response_make_value",
        item_fn.sig.ident
    );

    let make = if item_fn.sig.asyncness.is_some() {
        quote_spanned! {span=>
            #[allow(warnings)]
            async fn #make_value_name() -> #ty {
                #declare_inputs
                #block
            }
        }
    } else {
        quote_spanned! {span=>
            #[allow(warnings)]
            fn #make_value_name() -> #ty {
                #declare_inputs
                #block
            }
        }
    };

    let name = format_ident!("__axum_macros_check_{}_into_response", item_fn.sig.ident);

    if let Some(receiver) = self_receiver(item_fn) {
        quote_spanned! {span=>
            #make

            #[allow(warnings)]
            async fn #name() {
                let value = #receiver #make_value_name().await;
                fn check<T>(_: T)
                    where T: ::axum::response::IntoResponse
                {}
                check(value);
            }
        }
    } else {
        quote_spanned! {span=>
            #[allow(warnings)]
            async fn #name() {
                #make

                let value = #make_value_name().await;

                fn check<T>(_: T)
                where T: ::axum::response::IntoResponse
                {}

                check(value);
            }
        }
    }
}

fn check_future_send(item_fn: &ItemFn) -> TokenStream {
    if item_fn.sig.asyncness.is_none() {
        match &item_fn.sig.output {
            syn::ReturnType::Default => {
                return syn::Error::new_spanned(
                    &item_fn.sig.fn_token,
                    "Handlers must be `async fn`s",
                )
                .into_compile_error();
            }
            syn::ReturnType::Type(_, ty) => ty,
        };
    }

    let span = item_fn.span();

    let handler_name = &item_fn.sig.ident;

    let args = item_fn.sig.inputs.iter().map(|_| {
        quote_spanned! {span=> panic!() }
    });

    let name = format_ident!("__axum_macros_check_{}_future", item_fn.sig.ident);

    if let Some(receiver) = self_receiver(item_fn) {
        quote_spanned! {span=>
            #[allow(warnings)]
            fn #name() {
                let future = #receiver #handler_name(#(#args),*);
                fn check<T>(_: T)
                    where T: ::std::future::Future + Send
                {}
                check(future);
            }
        }
    } else {
        quote_spanned! {span=>
            #[allow(warnings)]
            fn #name() {
                #item_fn

                let future = #handler_name(#(#args),*);
                fn check<T>(_: T)
                    where T: ::std::future::Future + Send
                {}
                check(future);
            }
        }
    }
}

fn self_receiver(item_fn: &ItemFn) -> Option<TokenStream> {
    let takes_self = item_fn
        .sig
        .inputs
        .iter()
        .any(|arg| matches!(arg, syn::FnArg::Receiver(_)));
    if takes_self {
        return Some(quote! { Self:: });
    }

    if let syn::ReturnType::Type(_, ty) = &item_fn.sig.output {
        if let syn::Type::Path(path) = &**ty {
            let segments = &path.path.segments;
            if segments.len() == 1 {
                if let Some(last) = segments.last() {
                    match &last.arguments {
                        syn::PathArguments::None if last.ident == "Self" => {
                            return Some(quote! { Self:: });
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    None
}

#[test]
fn ui() {
    #[rustversion::stable]
    fn go() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/debug_handler/fail/*.rs");
        t.pass("tests/debug_handler/pass/*.rs");
    }

    #[rustversion::not(stable)]
    fn go() {}

    go();
}
