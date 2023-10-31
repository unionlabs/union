use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{parse::Parse, ItemStruct, LitStr, Token};

pub(crate) fn expand(item_struct: ItemStruct) -> syn::Result<TokenStream> {
    let ItemStruct {
        attrs,
        ident,
        generics,
        fields,
        ..
    } = &item_struct;

    if !generics.params.is_empty() || generics.where_clause.is_some() {
        return Err(syn::Error::new_spanned(
            generics,
            "`#[derive(TypedPath)]` doesn't support generics",
        ));
    }

    let Attrs { path, rejection } = parse_attrs(attrs)?;

    match fields {
        syn::Fields::Named(_) => {
            let segments = parse_path(&path)?;
            Ok(expand_named_fields(ident, path, &segments, rejection))
        }
        syn::Fields::Unnamed(fields) => {
            let segments = parse_path(&path)?;
            expand_unnamed_fields(fields, ident, path, &segments, rejection)
        }
        syn::Fields::Unit => expand_unit_fields(ident, path, rejection),
    }
}

mod kw {
    syn::custom_keyword!(rejection);
}

struct Attrs {
    path: LitStr,
    rejection: Option<syn::Path>,
}

impl Parse for Attrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let path = input.parse()?;

        let rejection = if input.is_empty() {
            None
        } else {
            let _: Token![,] = input.parse()?;
            let _: kw::rejection = input.parse()?;

            let content;
            syn::parenthesized!(content in input);
            Some(content.parse()?)
        };

        Ok(Self { path, rejection })
    }
}

fn parse_attrs(attrs: &[syn::Attribute]) -> syn::Result<Attrs> {
    let mut out = None;

    for attr in attrs {
        if attr.path.is_ident("typed_path") {
            if out.is_some() {
                return Err(syn::Error::new_spanned(
                    attr,
                    "`typed_path` specified more than once",
                ));
            } else {
                out = Some(attr.parse_args()?);
            }
        }
    }

    out.ok_or_else(|| {
        syn::Error::new(
            Span::call_site(),
            "missing `#[typed_path(\"...\")]` attribute",
        )
    })
}

fn expand_named_fields(
    ident: &syn::Ident,
    path: LitStr,
    segments: &[Segment],
    rejection: Option<syn::Path>,
) -> TokenStream {
    let format_str = format_str_from_path(segments);
    let captures = captures_from_path(segments);

    let typed_path_impl = quote_spanned! {path.span()=>
        #[automatically_derived]
        impl ::axum_extra::routing::TypedPath for #ident {
            const PATH: &'static str = #path;
        }
    };

    let display_impl = quote_spanned! {path.span()=>
        #[automatically_derived]
        impl ::std::fmt::Display for #ident {
            #[allow(clippy::unnecessary_to_owned)]
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                let Self { #(#captures,)* } = self;
                write!(
                    f,
                    #format_str,
                    #(
                        #captures = ::axum_extra::__private::utf8_percent_encode(
                            &#captures.to_string(),
                            ::axum_extra::__private::PATH_SEGMENT,
                        )
                    ),*
                )
            }
        }
    };

    let rejection_assoc_type = rejection_assoc_type(&rejection);
    let map_err_rejection = map_err_rejection(&rejection);

    let from_request_impl = quote! {
        #[::axum::async_trait]
        #[automatically_derived]
        impl<B> ::axum::extract::FromRequest<B> for #ident
        where
            B: Send,
        {
            type Rejection = #rejection_assoc_type;

            async fn from_request(req: &mut ::axum::extract::RequestParts<B>) -> ::std::result::Result<Self, Self::Rejection> {
                ::axum::extract::Path::from_request(req)
                    .await
                    .map(|path| path.0)
                    #map_err_rejection
            }
        }
    };

    quote! {
        #typed_path_impl
        #display_impl
        #from_request_impl
    }
}

fn expand_unnamed_fields(
    fields: &syn::FieldsUnnamed,
    ident: &syn::Ident,
    path: LitStr,
    segments: &[Segment],
    rejection: Option<syn::Path>,
) -> syn::Result<TokenStream> {
    let num_captures = segments
        .iter()
        .filter(|segment| match segment {
            Segment::Capture(_, _) => true,
            Segment::Static(_) => false,
        })
        .count();
    let num_fields = fields.unnamed.len();
    if num_fields != num_captures {
        return Err(syn::Error::new_spanned(
            fields,
            format!(
                "Mismatch in number of captures and fields. Path has {} but struct has {}",
                simple_pluralize(num_captures, "capture"),
                simple_pluralize(num_fields, "field"),
            ),
        ));
    }

    let destructure_self = segments
        .iter()
        .filter_map(|segment| match segment {
            Segment::Capture(capture, _) => Some(capture),
            Segment::Static(_) => None,
        })
        .enumerate()
        .map(|(idx, capture)| {
            let idx = syn::Index {
                index: idx as _,
                span: Span::call_site(),
            };
            let capture = format_ident!("{}", capture, span = path.span());
            quote_spanned! {path.span()=>
                #idx: #capture,
            }
        });

    let format_str = format_str_from_path(segments);
    let captures = captures_from_path(segments);

    let typed_path_impl = quote_spanned! {path.span()=>
        #[automatically_derived]
        impl ::axum_extra::routing::TypedPath for #ident {
            const PATH: &'static str = #path;
        }
    };

    let display_impl = quote_spanned! {path.span()=>
        #[automatically_derived]
        impl ::std::fmt::Display for #ident {
            #[allow(clippy::unnecessary_to_owned)]
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                let Self { #(#destructure_self)* } = self;
                write!(
                    f,
                    #format_str,
                    #(
                        #captures = ::axum_extra::__private::utf8_percent_encode(
                            &#captures.to_string(),
                            ::axum_extra::__private::PATH_SEGMENT,
                        )
                    ),*
                )
            }
        }
    };

    let rejection_assoc_type = rejection_assoc_type(&rejection);
    let map_err_rejection = map_err_rejection(&rejection);

    let from_request_impl = quote! {
        #[::axum::async_trait]
        #[automatically_derived]
        impl<B> ::axum::extract::FromRequest<B> for #ident
        where
            B: Send,
        {
            type Rejection = #rejection_assoc_type;

            async fn from_request(req: &mut ::axum::extract::RequestParts<B>) -> ::std::result::Result<Self, Self::Rejection> {
                ::axum::extract::Path::from_request(req)
                    .await
                    .map(|path| path.0)
                    #map_err_rejection
            }
        }
    };

    Ok(quote! {
        #typed_path_impl
        #display_impl
        #from_request_impl
    })
}

fn simple_pluralize(count: usize, word: &str) -> String {
    if count == 1 {
        format!("{} {}", count, word)
    } else {
        format!("{} {}s", count, word)
    }
}

fn expand_unit_fields(
    ident: &syn::Ident,
    path: LitStr,
    rejection: Option<syn::Path>,
) -> syn::Result<TokenStream> {
    for segment in parse_path(&path)? {
        match segment {
            Segment::Capture(_, span) => {
                return Err(syn::Error::new(
                    span,
                    "Typed paths for unit structs cannot contain captures",
                ));
            }
            Segment::Static(_) => {}
        }
    }

    let typed_path_impl = quote_spanned! {path.span()=>
        #[automatically_derived]
        impl ::axum_extra::routing::TypedPath for #ident {
            const PATH: &'static str = #path;
        }
    };

    let display_impl = quote_spanned! {path.span()=>
        #[automatically_derived]
        impl ::std::fmt::Display for #ident {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, #path)
            }
        }
    };

    let rejection_assoc_type = if let Some(rejection) = &rejection {
        quote! { #rejection }
    } else {
        quote! { ::axum::http::StatusCode }
    };
    let create_rejection = if let Some(rejection) = &rejection {
        quote! {
            Err(<#rejection as ::std::default::Default>::default())
        }
    } else {
        quote! {
            Err(::axum::http::StatusCode::NOT_FOUND)
        }
    };

    let from_request_impl = quote! {
        #[::axum::async_trait]
        #[automatically_derived]
        impl<B> ::axum::extract::FromRequest<B> for #ident
        where
            B: Send,
        {
            type Rejection = #rejection_assoc_type;

            async fn from_request(req: &mut ::axum::extract::RequestParts<B>) -> ::std::result::Result<Self, Self::Rejection> {
                if req.uri().path() == <Self as ::axum_extra::routing::TypedPath>::PATH {
                    Ok(Self)
                } else {
                    #create_rejection
                }
            }
        }
    };

    Ok(quote! {
        #typed_path_impl
        #display_impl
        #from_request_impl
    })
}

fn format_str_from_path(segments: &[Segment]) -> String {
    segments
        .iter()
        .map(|segment| match segment {
            Segment::Capture(capture, _) => format!("{{{}}}", capture),
            Segment::Static(segment) => segment.to_owned(),
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn captures_from_path(segments: &[Segment]) -> Vec<syn::Ident> {
    segments
        .iter()
        .filter_map(|segment| match segment {
            Segment::Capture(capture, span) => Some(format_ident!("{}", capture, span = *span)),
            Segment::Static(_) => None,
        })
        .collect::<Vec<_>>()
}

fn parse_path(path: &LitStr) -> syn::Result<Vec<Segment>> {
    let value = path.value();
    if value.is_empty() {
        return Err(syn::Error::new_spanned(
            path,
            "paths must start with a `/`. Use \"/\" for root routes",
        ));
    } else if !path.value().starts_with('/') {
        return Err(syn::Error::new_spanned(path, "paths must start with a `/`"));
    }

    path.value()
        .split('/')
        .map(|segment| {
            if let Some(capture) = segment
                .strip_prefix(':')
                .or_else(|| segment.strip_prefix('*'))
            {
                Ok(Segment::Capture(capture.to_owned(), path.span()))
            } else {
                Ok(Segment::Static(segment.to_owned()))
            }
        })
        .collect()
}

enum Segment {
    Capture(String, Span),
    Static(String),
}

fn path_rejection() -> TokenStream {
    quote! {
        <::axum::extract::Path<Self> as ::axum::extract::FromRequest<B>>::Rejection
    }
}

fn rejection_assoc_type(rejection: &Option<syn::Path>) -> TokenStream {
    match rejection {
        Some(rejection) => quote! { #rejection },
        None => path_rejection(),
    }
}

fn map_err_rejection(rejection: &Option<syn::Path>) -> TokenStream {
    rejection
        .as_ref()
        .map(|rejection| {
            let path_rejection = path_rejection();
            quote! {
                .map_err(|rejection| {
                    <#rejection as ::std::convert::From<#path_rejection>>::from(rejection)
                })
            }
        })
        .unwrap_or_default()
}

#[test]
fn ui() {
    #[rustversion::stable]
    fn go() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/typed_path/fail/*.rs");
        t.pass("tests/typed_path/pass/*.rs");
    }

    #[rustversion::not(stable)]
    fn go() {}

    go();
}
