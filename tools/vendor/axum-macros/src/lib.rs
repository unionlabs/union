//! Macros for [`axum`].
//!
//! [`axum`]: https://crates.io/crates/axum

#![warn(
    clippy::all,
    clippy::dbg_macro,
    clippy::todo,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::mem_forget,
    clippy::unused_self,
    clippy::filter_map_next,
    clippy::needless_continue,
    clippy::needless_borrow,
    clippy::match_wildcard_for_single_variants,
    clippy::if_let_mutex,
    clippy::mismatched_target_os,
    clippy::await_holding_lock,
    clippy::match_on_vec_items,
    clippy::imprecise_flops,
    clippy::suboptimal_flops,
    clippy::lossy_float_literal,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::fn_params_excessive_bools,
    clippy::exit,
    clippy::inefficient_to_string,
    clippy::linkedlist,
    clippy::macro_use_imports,
    clippy::option_option,
    clippy::verbose_file_reads,
    clippy::unnested_or_patterns,
    clippy::str_to_string,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style,
    missing_debug_implementations,
    missing_docs
)]
#![deny(unreachable_pub, private_in_public)]
#![allow(elided_lifetimes_in_paths, clippy::type_complexity)]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(test, allow(clippy::float_cmp))]

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::Parse;

mod debug_handler;
mod from_request;
mod typed_path;

/// Derive an implementation of [`FromRequest`].
///
/// Supports generating two kinds of implementations:
/// 1. One that extracts each field individually.
/// 2. Another that extracts the whole type at once via another extractor.
///
/// # Each field individually
///
/// By default `#[derive(FromRequest)]` will call `FromRequest::from_request` for each field:
///
/// ```
/// use axum_macros::FromRequest;
/// use axum::{
///     extract::{Extension, TypedHeader},
///     headers::ContentType,
///     body::Bytes,
/// };
///
/// #[derive(FromRequest)]
/// struct MyExtractor {
///     state: Extension<State>,
///     content_type: TypedHeader<ContentType>,
///     request_body: Bytes,
/// }
///
/// #[derive(Clone)]
/// struct State {
///     // ...
/// }
///
/// async fn handler(extractor: MyExtractor) {}
/// ```
///
/// This requires that each field is an extractor (i.e. implements [`FromRequest`]).
///
/// ## Extracting via another extractor
///
/// You can use `#[from_request(via(...))]` to extract a field via another extractor, meaning the
/// field itself doesn't need to implement `FromRequest`:
///
/// ```
/// use axum_macros::FromRequest;
/// use axum::{
///     extract::{Extension, TypedHeader},
///     headers::ContentType,
///     body::Bytes,
/// };
///
/// #[derive(FromRequest)]
/// struct MyExtractor {
///     // This will extracted via `Extension::<State>::from_request`
///     #[from_request(via(Extension))]
///     state: State,
///     // and this via `TypedHeader::<ContentType>::from_request`
///     #[from_request(via(TypedHeader))]
///     content_type: ContentType,
///     // Can still be combined with other extractors
///     request_body: Bytes,
/// }
///
/// #[derive(Clone)]
/// struct State {
///     // ...
/// }
///
/// async fn handler(extractor: MyExtractor) {}
/// ```
///
/// Note this requires the via extractor to be a generic newtype struct (a tuple struct with
/// exactly one public field) that implements `FromRequest`:
///
/// ```
/// pub struct ViaExtractor<T>(pub T);
///
/// // impl<T, B> FromRequest<B> for ViaExtractor<T> { ... }
/// ```
///
/// More complex via extractors are not supported and require writing a manual implementation.
///
/// ## Optional fields
///
/// `#[from_request(via(...))]` supports `Option<_>` and `Result<_, _>` to make fields optional:
///
/// ```
/// use axum_macros::FromRequest;
/// use axum::{
///     extract::{TypedHeader, rejection::TypedHeaderRejection},
///     headers::{ContentType, UserAgent},
/// };
///
/// #[derive(FromRequest)]
/// struct MyExtractor {
///     // This will extracted via `Option::<TypedHeader<ContentType>>::from_request`
///     #[from_request(via(TypedHeader))]
///     content_type: Option<ContentType>,
///     // This will extracted via
///     // `Result::<TypedHeader<UserAgent>, TypedHeaderRejection>::from_request`
///     #[from_request(via(TypedHeader))]
///     user_agent: Result<UserAgent, TypedHeaderRejection>,
/// }
///
/// async fn handler(extractor: MyExtractor) {}
/// ```
///
/// ## The rejection
///
/// A rejection enum is also generated. It has a variant for each field:
///
/// ```
/// use axum_macros::FromRequest;
/// use axum::{
///     extract::{Extension, TypedHeader},
///     headers::ContentType,
///     body::Bytes,
/// };
///
/// #[derive(FromRequest)]
/// struct MyExtractor {
///     #[from_request(via(Extension))]
///     state: State,
///     #[from_request(via(TypedHeader))]
///     content_type: ContentType,
///     request_body: Bytes,
/// }
///
/// // also generates
/// //
/// // #[derive(Debug)]
/// // enum MyExtractorRejection {
/// //     State(ExtensionRejection),
/// //     ContentType(TypedHeaderRejection),
/// //     RequestBody(BytesRejection),
/// // }
/// //
/// // impl axum::response::IntoResponse for MyExtractor { ... }
/// //
/// // impl std::fmt::Display for MyExtractor { ... }
/// //
/// // impl std::error::Error for MyExtractor { ... }
///
/// #[derive(Clone)]
/// struct State {
///     // ...
/// }
/// ```
///
/// The rejection's `std::error::Error::source` implementation returns the inner rejection. This
/// can be used to access source errors for example to customize rejection responses. Note this
/// means the inner rejection types must themselves implement `std::error::Error`. All extractors
/// in axum does this.
///
/// You can opt out of this using `#[from_request(rejection_derive(...))]`:
///
/// ```
/// use axum_macros::FromRequest;
/// use axum::{
///     extract::{FromRequest, RequestParts},
///     http::StatusCode,
///     headers::ContentType,
///     body::Bytes,
///     async_trait,
/// };
///
/// #[derive(FromRequest)]
/// #[from_request(rejection_derive(!Display, !Error))]
/// struct MyExtractor {
///     other: OtherExtractor,
/// }
///
/// struct OtherExtractor;
///
/// #[async_trait]
/// impl<B> FromRequest<B> for OtherExtractor
/// where
///     B: Send + 'static,
/// {
///     // this rejection doesn't implement `Display` and `Error`
///     type Rejection = (StatusCode, String);
///
///     async fn from_request(_req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
///         // ...
///         # unimplemented!()
///     }
/// }
/// ```
///
/// # The whole type at once
///
/// By using `#[from_request(via(...))]` on the container you can extract the whole type at once,
/// instead of each field individually:
///
/// ```
/// use axum_macros::FromRequest;
/// use axum::extract::Extension;
///
/// // This will extracted via `Extension::<State>::from_request`
/// #[derive(Clone, FromRequest)]
/// #[from_request(via(Extension))]
/// struct State {
///     // ...
/// }
///
/// async fn handler(state: State) {}
/// ```
///
/// The rejection will be the "via extractors"'s rejection. For the previous example that would be
/// [`axum::extract::rejection::ExtensionRejection`].
///
/// # Known limitations
///
/// Generics are currently not supported:
///
/// ```compile_fail
/// #[derive(axum_macros::FromRequest)]
/// struct MyExtractor<T> {
///     thing: Option<T>,
/// }
/// ```
///
/// [`FromRequest`]: https://docs.rs/axum/latest/axum/extract/trait.FromRequest.html
/// [`axum::extract::rejection::ExtensionRejection`]: https://docs.rs/axum/latest/axum/extract/rejection/enum.ExtensionRejection.html
#[proc_macro_derive(FromRequest, attributes(from_request))]
pub fn derive_from_request(item: TokenStream) -> TokenStream {
    expand_with(item, from_request::expand)
}

/// Generates better error messages when applied handler functions.
///
/// While using [`axum`], you can get long error messages for simple mistakes. For example:
///
/// ```compile_fail
/// use axum::{routing::get, Router};
///
/// #[tokio::main]
/// async fn main() {
///     let app = Router::new().route("/", get(handler));
///
///     axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
///         .serve(app.into_make_service())
///         .await
///         .unwrap();
/// }
///
/// fn handler() -> &'static str {
///     "Hello, world"
/// }
/// ```
///
/// You will get a long error message about function not implementing [`Handler`] trait. But why
/// does this function not implement it? To figure it out, the [`debug_handler`] macro can be used.
///
/// ```compile_fail
/// # use axum::{routing::get, Router};
/// # use axum_macros::debug_handler;
/// #
/// # #[tokio::main]
/// # async fn main() {
/// #     let app = Router::new().route("/", get(handler));
/// #
/// #     axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
/// #         .serve(app.into_make_service())
/// #         .await
/// #         .unwrap();
/// # }
/// #
/// #[debug_handler]
/// fn handler() -> &'static str {
///     "Hello, world"
/// }
/// ```
///
/// ```text
/// error: handlers must be async functions
///   --> main.rs:xx:1
///    |
/// xx | fn handler() -> &'static str {
///    | ^^
/// ```
///
/// As the error message says, handler function needs to be async.
///
/// ```
/// use axum::{routing::get, Router};
/// use axum_macros::debug_handler;
///
/// #[tokio::main]
/// async fn main() {
///     # async {
///     let app = Router::new().route("/", get(handler));
///
///     axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
///         .serve(app.into_make_service())
///         .await
///         .unwrap();
///     # };
/// }
///
/// #[debug_handler]
/// async fn handler() -> &'static str {
///     "Hello, world"
/// }
/// ```
///
/// # Changing request body type
///
/// By default `#[debug_handler]` assumes your request body type is `axum::body::Body`. This will
/// work for most extractors but, for example, it wont work for `Request<axum::body::BoxBody>`,
/// which only implements `FromRequest<BoxBody>` and _not_ `FromRequest<Body>`.
///
/// To work around that the request body type can be customized like so:
///
/// ```
/// use axum::{body::BoxBody, http::Request};
/// # use axum_macros::debug_handler;
///
/// #[debug_handler(body = BoxBody)]
/// async fn handler(request: Request<BoxBody>) {}
/// ```
///
/// # Performance
///
/// This macro has no effect when compiled with the release profile. (eg. `cargo build --release`)
///
/// [`axum`]: https://docs.rs/axum/latest
/// [`Handler`]: https://docs.rs/axum/latest/axum/handler/trait.Handler.html
/// [`debug_handler`]: macro@debug_handler
#[proc_macro_attribute]
pub fn debug_handler(_attr: TokenStream, input: TokenStream) -> TokenStream {
    #[cfg(not(debug_assertions))]
    return input;

    #[cfg(debug_assertions)]
    return expand_attr_with(_attr, input, debug_handler::expand);
}

/// Derive an implementation of [`axum_extra::routing::TypedPath`].
///
/// See that trait for more details.
///
/// [`axum_extra::routing::TypedPath`]: https://docs.rs/axum-extra/latest/axum_extra/routing/trait.TypedPath.html
#[proc_macro_derive(TypedPath, attributes(typed_path))]
pub fn derive_typed_path(input: TokenStream) -> TokenStream {
    expand_with(input, typed_path::expand)
}

fn expand_with<F, I, K>(input: TokenStream, f: F) -> TokenStream
where
    F: FnOnce(I) -> syn::Result<K>,
    I: Parse,
    K: ToTokens,
{
    expand(syn::parse(input).and_then(f))
}

fn expand_attr_with<F, A, I, K>(attr: TokenStream, input: TokenStream, f: F) -> TokenStream
where
    F: FnOnce(A, I) -> K,
    A: Parse,
    I: Parse,
    K: ToTokens,
{
    let expand_result = (|| {
        let attr = syn::parse(attr)?;
        let input = syn::parse(input)?;
        Ok(f(attr, input))
    })();
    expand(expand_result)
}

fn expand<T>(result: syn::Result<T>) -> TokenStream
where
    T: ToTokens,
{
    match result {
        Ok(tokens) => {
            let tokens = (quote! { #tokens }).into();
            if std::env::var_os("AXUM_MACROS_DEBUG").is_some() {
                eprintln!("{}", tokens);
            }
            tokens
        }
        Err(err) => err.into_compile_error().into(),
    }
}
