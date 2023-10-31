# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

# Unreleased

- None.

# 0.2.9 (20. October, 2022)

- **added:** Add `DefaultBodyLimit::max` for changing the default body limit ([#1397])
- **added:** Add `Error::into_inner` for converting `Error` to `BoxError` without allocating ([#1476])

[#1397]: https://github.com/tokio-rs/axum/pull/1397
[#1476]: https://github.com/tokio-rs/axum/pull/1476

# 0.2.8 (10. September, 2022)

- **breaking:** Added default limit to how much data `Bytes::from_request` will
  consume. Previously it would attempt to consume the entire request body
  without checking its length. This meant if a malicious peer sent an large (or
  infinite) request body your server might run out of memory and crash.

  The default limit is at 2 MB and can be disabled by adding the new
  `DefaultBodyLimit::disable()` middleware. See its documentation for more
  details.

  This also applies to `String` which used `Bytes::from_request` internally.

  ([#1346])

[#1346]: https://github.com/tokio-rs/axum/pull/1346

# 0.2.7 (10. July, 2022)

- **fix:** Fix typos in `RequestParts` docs ([#1147])

[#1147]: https://github.com/tokio-rs/axum/pull/1147

# 0.2.6 (18. June, 2022)

- **change:** axum-core's MSRV is now 1.56 ([#1098])

[#1098]: https://github.com/tokio-rs/axum/pull/1098

# 0.2.5 (08. June, 2022)

- **added:** Automatically handle `http_body::LengthLimitError` in `FailedToBufferBody` and map
  such errors to `413 Payload Too Large` ([#1048])
- **fixed:** Use `impl IntoResponse` less in docs ([#1049])

[#1048]: https://github.com/tokio-rs/axum/pull/1048
[#1049]: https://github.com/tokio-rs/axum/pull/1049

# 0.2.4 (02. May, 2022)

- **added:** Implement `IntoResponse` and `IntoResponseParts` for `http::Extensions` ([#975])
- **added:** Implement `IntoResponse` for `(http::response::Parts, impl IntoResponse)` ([#950])
- **added:** Implement `IntoResponse` for `(http::response::Response<()>, impl IntoResponse)` ([#950])
- **added:** Implement `IntoResponse for (Parts | Request<()>, $(impl IntoResponseParts)+, impl IntoResponse)` ([#980])

[#950]: https://github.com/tokio-rs/axum/pull/950
[#975]: https://github.com/tokio-rs/axum/pull/975
[#980]: https://github.com/tokio-rs/axum/pull/980

# 0.2.3 (25. April, 2022)

- **added:** Add `response::ErrorResponse` and `response::Result` for
  `IntoResponse`-based error handling ([#921])

[#921]: https://github.com/tokio-rs/axum/pull/921 

# 0.2.2 (19. April, 2022)

- **added:** Add `AppendHeaders` for appending headers to a response rather than overriding them ([#927])

[#927]: https://github.com/tokio-rs/axum/pull/927

# 0.2.1 (03. April, 2022)

- **added:** Add `RequestParts::extract` which allows applying an extractor as a method call ([#897])

[#897]: https://github.com/tokio-rs/axum/pull/897

# 0.2.0 (31. March, 2022)

- **added:** Add `IntoResponseParts` trait which allows defining custom response
  types for adding headers or extensions to responses ([#797])
- **breaking:** Using `HeaderMap` as an extractor will no longer remove the headers and thus
  they'll still be accessible to other extractors, such as `axum::extract::Json`. Instead
  `HeaderMap` will clone the headers. You should prefer to use `TypedHeader` to extract only the
  headers you need ([#698])

  This includes these breaking changes:
    - `RequestParts::take_headers` has been removed.
    - `RequestParts::headers` returns `&HeaderMap`.
    - `RequestParts::headers_mut` returns `&mut HeaderMap`.
    - `HeadersAlreadyExtracted` has been removed.
    - The `HeadersAlreadyExtracted` variant has been removed from these rejections:
        - `RequestAlreadyExtracted`
        - `RequestPartsAlreadyExtracted`
    - `<HeaderMap as FromRequest<_>>::Rejection` has been changed to `std::convert::Infallible`.
- **breaking:** `axum::http::Extensions` is no longer an extractor (ie it
  doesn't implement `FromRequest`). The `axum::extract::Extension` extractor is
  _not_ impacted by this and works the same. This change makes it harder to
  accidentally remove all extensions which would result in confusing errors
  elsewhere ([#699])
  This includes these breaking changes:
    - `RequestParts::take_extensions` has been removed.
    - `RequestParts::extensions` returns `&Extensions`.
    - `RequestParts::extensions_mut` returns `&mut Extensions`.
    - `RequestAlreadyExtracted` has been removed.
    - `<Request as FromRequest>::Rejection` is now `BodyAlreadyExtracted`.
    - `<http::request::Parts as FromRequest>::Rejection` is now `Infallible`.
    - `ExtensionsAlreadyExtracted` has been removed.
- **breaking:** `RequestParts::body_mut` now returns `&mut Option<B>` so the
  body can be swapped ([#869])

[#698]: https://github.com/tokio-rs/axum/pull/698
[#699]: https://github.com/tokio-rs/axum/pull/699
[#797]: https://github.com/tokio-rs/axum/pull/797
[#869]: https://github.com/tokio-rs/axum/pull/869

# 0.1.2 (22. February, 2022)

- **added:** Implement `IntoResponse` for `bytes::BytesMut` and `bytes::Chain<T, U>` ([#767])

[#767]: https://github.com/tokio-rs/axum/pull/767

# 0.1.1 (06. December, 2021)

- **added:** `axum_core::response::Response` now exists as a shorthand for writing `Response<BoxBody>` ([#590])

[#590]: https://github.com/tokio-rs/axum/pull/590

# 0.1.0 (02. December, 2021)

- Initial release.
