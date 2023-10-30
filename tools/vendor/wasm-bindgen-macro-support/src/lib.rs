//! This crate contains the part of the implementation of the `#[wasm_bindgen]` optsibute that is
//! not in the shared backend crate.

#![doc(html_root_url = "https://docs.rs/wasm-bindgen-macro-support/0.2")]

extern crate proc_macro2;
extern crate quote;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate wasm_bindgen_backend as backend;
extern crate wasm_bindgen_shared as shared;

pub use crate::parser::BindgenAttrs;
use crate::parser::MacroParse;
use backend::{Diagnostic, TryToTokens};
use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::TokenStreamExt;
use syn::parse::{Parse, ParseStream, Result as SynResult};

mod parser;

/// Takes the parsed input from a `#[wasm_bindgen]` macro and returns the generated bindings
pub fn expand(attr: TokenStream, input: TokenStream) -> Result<TokenStream, Diagnostic> {
    parser::reset_attrs_used();
    let item = syn::parse2::<syn::Item>(input)?;
    let opts = syn::parse2(attr)?;

    let mut tokens = proc_macro2::TokenStream::new();
    let mut program = backend::ast::Program::default();
    item.macro_parse(&mut program, (Some(opts), &mut tokens))?;
    program.try_to_tokens(&mut tokens)?;

    // If we successfully got here then we should have used up all attributes
    // and considered all of them to see if they were used. If one was forgotten
    // that's a bug on our end, so sanity check here.
    parser::check_unused_attrs(&mut tokens);

    Ok(tokens)
}

/// Takes the parsed input from a `wasm_bindgen::link_to` macro and returns the generated link
pub fn expand_link_to(input: TokenStream) -> Result<TokenStream, Diagnostic> {
    parser::reset_attrs_used();
    let opts = syn::parse2(input)?;

    let mut tokens = proc_macro2::TokenStream::new();
    let link = parser::link_to(opts)?;
    link.try_to_tokens(&mut tokens)?;

    Ok(tokens)
}

/// Takes the parsed input from a `#[wasm_bindgen]` macro and returns the generated bindings
pub fn expand_class_marker(
    attr: TokenStream,
    input: TokenStream,
) -> Result<TokenStream, Diagnostic> {
    parser::reset_attrs_used();
    let mut item = syn::parse2::<syn::ImplItemFn>(input)?;
    let opts: ClassMarker = syn::parse2(attr)?;

    let mut program = backend::ast::Program::default();
    item.macro_parse(&mut program, &opts)?;

    // This is where things are slightly different, we are being expanded in the
    // context of an impl so we can't inject arbitrary item-like tokens into the
    // output stream. If we were to do that then it wouldn't parse!
    //
    // Instead what we want to do is to generate the tokens for `program` into
    // the header of the function. This'll inject some no_mangle functions and
    // statics and such, and they should all be valid in the context of the
    // start of a function.
    //
    // We manually implement `ToTokens for ImplItemFn` here, injecting our
    // program's tokens before the actual method's inner body tokens.
    let mut tokens = proc_macro2::TokenStream::new();
    tokens.append_all(
        item.attrs
            .iter()
            .filter(|attr| matches!(attr.style, syn::AttrStyle::Outer)),
    );
    item.vis.to_tokens(&mut tokens);
    item.sig.to_tokens(&mut tokens);
    let mut err = None;
    item.block.brace_token.surround(&mut tokens, |tokens| {
        if let Err(e) = program.try_to_tokens(tokens) {
            err = Some(e);
        }
        parser::check_unused_attrs(tokens); // same as above
        tokens.append_all(
            item.attrs
                .iter()
                .filter(|attr| matches!(attr.style, syn::AttrStyle::Inner(_))),
        );
        tokens.append_all(&item.block.stmts);
    });

    if let Some(err) = err {
        return Err(err);
    }

    Ok(tokens)
}

struct ClassMarker {
    class: syn::Ident,
    js_class: String,
    wasm_bindgen: syn::Path,
    wasm_bindgen_futures: syn::Path,
}

impl Parse for ClassMarker {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let class = input.parse::<syn::Ident>()?;
        input.parse::<Token![=]>()?;
        let js_class = input.parse::<syn::LitStr>()?.value();

        let mut wasm_bindgen = None;
        let mut wasm_bindgen_futures = None;

        loop {
            if input.parse::<Option<Token![,]>>()?.is_some() {
                let ident = input.parse::<syn::Ident>()?;

                if ident == "wasm_bindgen" {
                    if wasm_bindgen.is_some() {
                        return Err(syn::Error::new(
                            ident.span(),
                            "found duplicate `wasm_bindgen`",
                        ));
                    }

                    input.parse::<Token![=]>()?;
                    wasm_bindgen = Some(input.parse::<syn::Path>()?);
                } else if ident == "wasm_bindgen_futures" {
                    if wasm_bindgen_futures.is_some() {
                        return Err(syn::Error::new(
                            ident.span(),
                            "found duplicate `wasm_bindgen_futures`",
                        ));
                    }

                    input.parse::<Token![=]>()?;
                    wasm_bindgen_futures = Some(input.parse::<syn::Path>()?);
                } else {
                    return Err(syn::Error::new(
                        ident.span(),
                        "expected `wasm_bindgen` or `wasm_bindgen_futures`",
                    ));
                }
            } else {
                break;
            }
        }

        Ok(ClassMarker {
            class,
            js_class,
            wasm_bindgen: wasm_bindgen.unwrap_or_else(|| syn::parse_quote! { wasm_bindgen }),
            wasm_bindgen_futures: wasm_bindgen_futures
                .unwrap_or_else(|| syn::parse_quote! { wasm_bindgen_futures }),
        })
    }
}
