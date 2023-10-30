# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

# Unreleased

- None.

# 0.5.17 (20. October, 2022)

- **fixed:** Annotate panicking functions with `#[track_caller]` so the error
  message points to where the user added the invalid router, rather than
  somewhere internally in axum ([#1248])
- **fixed:** Make `Multipart` extractor work with `RequestBodyLimit` middleware ([#1379])
- **added:** Add `DefaultBodyLimit::max` for changing the default body limit ([#1397])
- **added:** Various documentation improvements

[#1248]: https://github.com/tokio-rs/axum/pull/1248
[#1379]: https://github.com/tokio-rs/axum/pull/1379
[#1397]: https://github.com/tokio-rs/axum/pull/1397

# 0.5.16 (10. September, 2022)

## Security

- **breaking:** Added default limit to how much data `Bytes::from_request` will
  consume. Previously it would attempt to consume the entire request body
  without checking its length. This meant if a malicious peer sent an large (or
  infinite) request body your server might run out of memory and crash.

  The default limit is at 2 MB and can be disabled by adding the new
  `DefaultBodyLimit::disable()` middleware. See its documentation for more
  details.

  This also applies to these extractors which used `Bytes::from_request`
  internally:
  - `Form`
  - `Json`
  - `String`

  ([#1346])

[#1346]: https://github.com/tokio-rs/axum/pull/1346

# 0.5.15 (9. August, 2022)

- **fixed:** Don't expose internal type names in `QueryRejection` response. ([#1171])
- **fixed:** Improve performance of JSON serialization ([#1178])
- **fixed:** Improve build times by generating less IR ([#1192])

[#1171]: https://github.com/tokio-rs/axum/pull/1171
[#1178]: https://github.com/tokio-rs/axum/pull/1178
[#1192]: https://github.com/tokio-rs/axum/pull/1192

# 0.5.14 (25. July, 2022)

Yanked, as it contained an accidental breaking change.

# 0.5.13 (15. July, 2022)

- **fixed:** If `WebSocketUpgrade` cannot upgrade the connection it will return a
  `WebSocketUpgradeRejection::ConnectionNotUpgradable` rejection ([#1135])
- **changed:** `WebSocketUpgradeRejection` has a new variant `ConnectionNotUpgradable`
  variant ([#1135])

[#1135]: https://github.com/tokio-rs/axum/pull/1135

# 0.5.12 (10. July, 2022)

- **added:** Added `debug_handler` which is an attribute macro that improves
  type errors when applied to handler function. It is re-exported from
  `axum-macros` ([#1144])

[#1144]: https://github.com/tokio-rs/axum/pull/1144

# 0.5.11 (02. July, 2022)

- **added:** Implement `TryFrom<http::Method>` for `MethodFilter` and use new
  `NoMatchingMethodFilter` error in case of failure ([#1130])
- **added:** Document how to run extractors from middleware ([#1140])

[#1130]: https://github.com/tokio-rs/axum/pull/1130
[#1140]: https://github.com/tokio-rs/axum/pull/1140

# 0.5.10 (28. June, 2022)

- **fixed:** Make `Router` cheaper to clone ([#1123])
- **fixed:** Fix possible panic when doing trailing slash redirect ([#1124])

[#1123]: https://github.com/tokio-rs/axum/pull/1123
[#1124]: https://github.com/tokio-rs/axum/pull/1124

# 0.5.9 (20. June, 2022)

- **fixed:** Fix compile error when the `headers` is enabled and the `form`
  feature is disabled ([#1107])

[#1107]: https://github.com/tokio-rs/axum/pull/1107

# 0.5.8 (18. June, 2022)

- **added:** Support resolving host name via `Forwarded` header in `Host`
  extractor ([#1078])
- **added:** Implement `IntoResponse` for `Form` ([#1095])
- **change:** axum's MSRV is now 1.56 ([#1098])

[#1078]: https://github.com/tokio-rs/axum/pull/1078
[#1095]: https://github.com/tokio-rs/axum/pull/1095
[#1098]: https://github.com/tokio-rs/axum/pull/1098

# 0.5.7 (08. June, 2022)

- **added:** Implement `Default` for `Extension` ([#1043])
- **fixed:** Support deserializing `Vec<(String, String)>` in `extract::Path<_>` to get vector of
  key/value pairs ([#1059])
- **added:** Add `extract::ws::close_code` which contains constants for close codes ([#1067])
- **fixed:** Use `impl IntoResponse` less in docs ([#1049])

[#1043]: https://github.com/tokio-rs/axum/pull/1043
[#1049]: https://github.com/tokio-rs/axum/pull/1049
[#1059]: https://github.com/tokio-rs/axum/pull/1059
[#1067]: https://github.com/tokio-rs/axum/pull/1067

# 0.5.6 (15. May, 2022)

- **added:** Add `WebSocket::protocol` to return the selected WebSocket subprotocol, if there is one. ([#1022])
- **fixed:** Improve error message for `PathRejection::WrongNumberOfParameters` to hint at using
  `Path<(String, String)>` or `Path<SomeStruct>` ([#1023])
- **fixed:** `PathRejection::WrongNumberOfParameters` now uses `500 Internal Server Error` since
  it's a programmer error and not a client error ([#1023])
- **fixed:** Fix `InvalidFormContentType` mentioning the wrong content type

[#1022]: https://github.com/tokio-rs/axum/pull/1022
[#1023]: https://github.com/tokio-rs/axum/pull/1023

# 0.5.5 (10. May, 2022)

- **fixed:** Correctly handle `GET`, `HEAD`, and `OPTIONS` requests in `ContentLengthLimit`.
  Request with these methods are now accepted if they _do not_ have a `Content-Length` header, and
  the request body will not be checked. If they do have a `Content-Length` header they'll be
  rejected. This allows `ContentLengthLimit` to be used as middleware around several routes,
  including `GET` routes ([#989])
- **added:** Add `MethodRouter::{into_make_service, into_make_service_with_connect_info}` ([#1010])

[#989]: https://github.com/tokio-rs/axum/pull/989
[#1010]: https://github.com/tokio-rs/axum/pull/1010

# 0.5.4 (26. April, 2022)

- **added:** Add `response::ErrorResponse` and `response::Result` for
  `IntoResponse`-based error handling ([#921])
- **added:** Add `middleware::from_extractor` and deprecate `extract::extractor_middleware` ([#957])
- **changed:** Update to tower-http 0.3 ([#965])

[#921]: https://github.com/tokio-rs/axum/pull/921
[#957]: https://github.com/tokio-rs/axum/pull/957
[#965]: https://github.com/tokio-rs/axum/pull/965

# 0.5.3 (19. April, 2022)

- **added:** Add `AppendHeaders` for appending headers to a response rather than overriding them ([#927])
- **added:** Add `axum::extract::multipart::Field::chunk` method for streaming a single chunk from
  the field ([#901])
- **fixed:** Fix trailing slash redirection with query parameters ([#936])

[#901]: https://github.com/tokio-rs/axum/pull/901
[#927]: https://github.com/tokio-rs/axum/pull/927
[#936]: https://github.com/tokio-rs/axum/pull/936

# 0.5.2 (19. April, 2022)

Yanked, as it contained an accidental breaking change.

# 0.5.1 (03. April, 2022)

- **added:** Add `RequestParts::extract` which allows applying an extractor as a method call ([#897])

[#897]: https://github.com/tokio-rs/axum/pull/897

# 0.5.0 (31. March, 2022)

- **added:** Document sharing state between handler and middleware ([#783])
- **added:** `Extension<_>` can now be used in tuples for building responses, and will set an
  extension on the response ([#797])
- **added:** `extract::Host` for extracting the hostname of a request ([#827])
- **added:** Add `IntoResponseParts` trait which allows defining custom response
  types for adding headers or extensions to responses ([#797])
- **added:** `TypedHeader` implements the new `IntoResponseParts` trait so they
  can be returned from handlers as parts of a response ([#797])
- **changed:** `Router::merge` now accepts `Into<Router>` ([#819])
- **breaking:** `sse::Event` now accepts types implementing `AsRef<str>` instead of `Into<String>`
  as field values.
- **breaking:** `sse::Event` now panics if a setter method is called twice instead of silently
  overwriting old values.
- **breaking:** Require `Output = ()` on `WebSocketStream::on_upgrade` ([#644])
- **breaking:** Make `TypedHeaderRejectionReason` `#[non_exhaustive]` ([#665])
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
        - `JsonRejection`
        - `FormRejection`
        - `ContentLengthLimitRejection`
        - `WebSocketUpgradeRejection`
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
    - The `ExtensionsAlreadyExtracted` removed variant has been removed from these rejections:
        - `ExtensionRejection`
        - `PathRejection`
        - `MatchedPathRejection`
        - `WebSocketUpgradeRejection`
- **breaking:** `Redirect::found` has been removed ([#800])
- **breaking:** `AddExtensionLayer` has been removed. Use `Extension` instead. It now implements
  `tower::Layer` ([#807])
- **breaking:** `AddExtension` has been moved from the root module to `middleware`
- **breaking:** `.nest("/foo/", Router::new().route("/bar", _))` now does the right thing and
  results in a route at `/foo/bar` instead of `/foo//bar` ([#824])
- **breaking:** Routes are now required to start with `/`. Previously routes such as `:foo` would
  be accepted but most likely result in bugs ([#823])
- **breaking:** `Headers` has been removed. Arrays of tuples directly implement
  `IntoResponseParts` so `([("x-foo", "foo")], response)` now works ([#797])
- **breaking:** `InvalidJsonBody` has been replaced with `JsonDataError` to clearly signal that the
  request body was syntactically valid JSON but couldn't be deserialized into the target type
- **breaking:** `Handler` is no longer an `#[async_trait]` but instead has an
  associated `Future` type. That allows users to build their own `Handler` types
  without paying the cost of `#[async_trait]` ([#879])
- **changed:** New `JsonSyntaxError` variant added to `JsonRejection`. This is returned when the
  request body contains syntactically invalid JSON
- **fixed:** Correctly set the `Content-Length` header for response to `HEAD`
  requests ([#734])
- **fixed:** Fix wrong `content-length` for `HEAD` requests to endpoints that returns chunked
  responses ([#755])
- **fixed:** Fixed several routing bugs related to nested "opaque" tower services (i.e.
  non-`Router` services) ([#841] and [#842])
- **changed:** Update to tokio-tungstenite 0.17 ([#791])
- **breaking:** `Redirect::{to, temporary, permanent}` now accept `&str` instead
  of `Uri` ([#889])
- **breaking:** Remove second type parameter from `Router::into_make_service_with_connect_info`
  and `Handler::into_make_service_with_connect_info` to support `MakeService`s
  that accept multiple targets ([#892])

[#644]: https://github.com/tokio-rs/axum/pull/644
[#665]: https://github.com/tokio-rs/axum/pull/665
[#698]: https://github.com/tokio-rs/axum/pull/698
[#699]: https://github.com/tokio-rs/axum/pull/699
[#734]: https://github.com/tokio-rs/axum/pull/734
[#755]: https://github.com/tokio-rs/axum/pull/755
[#783]: https://github.com/tokio-rs/axum/pull/783
[#791]: https://github.com/tokio-rs/axum/pull/791
[#797]: https://github.com/tokio-rs/axum/pull/797
[#800]: https://github.com/tokio-rs/axum/pull/800
[#807]: https://github.com/tokio-rs/axum/pull/807
[#819]: https://github.com/tokio-rs/axum/pull/819
[#823]: https://github.com/tokio-rs/axum/pull/823
[#824]: https://github.com/tokio-rs/axum/pull/824
[#827]: https://github.com/tokio-rs/axum/pull/827
[#841]: https://github.com/tokio-rs/axum/pull/841
[#842]: https://github.com/tokio-rs/axum/pull/842
[#879]: https://github.com/tokio-rs/axum/pull/879
[#889]: https://github.com/tokio-rs/axum/pull/889
[#892]: https://github.com/tokio-rs/axum/pull/892

# 0.4.8 (2. March, 2022)

- Use correct path for `AddExtensionLayer` and `AddExtension::layer` deprecation
  notes ([#812])

[#812]: https://github.com/tokio-rs/axum/pull/812

# 0.4.7 (1. March, 2022)

- **added:** Implement `tower::Layer` for `Extension` ([#801])
- **changed:** Deprecate `AddExtensionLayer`. Use `Extension` instead ([#805])

[#801]: https://github.com/tokio-rs/axum/pull/801
[#805]: https://github.com/tokio-rs/axum/pull/805

# 0.4.6 (22. February, 2022)

- **added:** `middleware::from_fn` for creating middleware from async functions.
  This previously lived in axum-extra but has been moved to axum ([#719])
- **fixed:** Set `Allow` header when responding with `405 Method Not Allowed` ([#733])

[#719]: https://github.com/tokio-rs/axum/pull/719
[#733]: https://github.com/tokio-rs/axum/pull/733

# 0.4.5 (31. January, 2022)

- Reference [axum-macros] instead of [axum-debug]. The latter has been superseded by
  axum-macros and is deprecated ([#738])

[#738]: https://github.com/tokio-rs/axum/pull/738
[axum-debug]: https://docs.rs/axum-debug
[axum-macros]: https://docs.rs/axum-macros

# 0.4.4 (13. January, 2022)

- **fixed:** Fix using incorrect path prefix when nesting `Router`s at `/` ([#691])
- **fixed:** Make `nest("", service)` work and mean the same as `nest("/", service)` ([#691])
- **fixed:** Replace response code `301` with `308` for trailing slash redirects. Also deprecates
  `Redirect::found` (`302`) in favor of `Redirect::temporary` (`307`) or `Redirect::to` (`303`).
  This is to prevent clients from changing non-`GET` requests to `GET` requests ([#682])

[#691]: https://github.com/tokio-rs/axum/pull/691
[#682]: https://github.com/tokio-rs/axum/pull/682

# 0.4.3 (21. December, 2021)

- **added:** `axum::AddExtension::layer` ([#607])
- **added:** Re-export the headers crate when the headers feature is active ([#630])
- **fixed:** `sse::Event` will no longer drop the leading space of data, event ID and name values
  that have it ([#600])
- **fixed:** `sse::Event` is more strict about what field values it supports, disallowing any SSE
  events that break the specification (such as field values containing carriage returns) ([#599])
- **fixed:** Improve documentation of `sse::Event` ([#601])
- **fixed:** Make `Path` fail with `ExtensionsAlreadyExtracted` if another extractor (such as
  `Request`) has previously taken the request extensions. Thus `PathRejection` now contains a
  variant with `ExtensionsAlreadyExtracted`. This is not a breaking change since `PathRejection` is
  marked as `#[non_exhaustive]` ([#619])
- **fixed:** Fix misleading error message for `PathRejection` if extensions had
  previously been extracted ([#619])
- **fixed:** Use `AtomicU32` internally, rather than `AtomicU64`, to improve portability ([#616])

[#599]: https://github.com/tokio-rs/axum/pull/599
[#600]: https://github.com/tokio-rs/axum/pull/600
[#601]: https://github.com/tokio-rs/axum/pull/601
[#607]: https://github.com/tokio-rs/axum/pull/607
[#616]: https://github.com/tokio-rs/axum/pull/616
[#619]: https://github.com/tokio-rs/axum/pull/619
[#619]: https://github.com/tokio-rs/axum/pull/619
[#630]: https://github.com/tokio-rs/axum/pull/630

# 0.4.2 (06. December, 2021)

- **fix:** Depend on the correct version of `axum-core` ([#592])

[#592]: https://github.com/tokio-rs/axum/pull/592

# 0.4.1 (06. December, 2021)

- **added:** `axum::response::Response` now exists as a shorthand for writing `Response<BoxBody>` ([#590])

[#590]: https://github.com/tokio-rs/axum/pull/590

# 0.4.0 (02. December, 2021)

- **breaking:** New `MethodRouter` that works similarly to `Router`:
  - Route to handlers and services with the same type
  - Add middleware to some routes more easily with `MethodRouter::layer` and
    `MethodRouter::route_layer`.
  - Merge method routers with `MethodRouter::merge`
  - Customize response for unsupported methods with `MethodRouter::fallback`
- **breaking:** The default for the type parameter in `FromRequest` and
  `RequestParts` has been removed. Use `FromRequest<Body>` and
  `RequestParts<Body>` to get the previous behavior ([#564])
- **added:** `FromRequest` and `IntoResponse` are now defined in a new called
  `axum-core`. This crate is intended for library authors to depend on, rather
  than `axum` itself, if possible. `axum-core` has a smaller API and will thus
  receive fewer breaking changes. `FromRequest` and `IntoResponse` are
  re-exported from `axum` in the same location so nothing is changed for `axum`
  users ([#564])
- **breaking:** The previously deprecated `axum::body::box_body` function has
  been removed. Use `axum::body::boxed` instead.
- **fixed:** Adding the same route with different methods now works ie
  `.route("/", get(_)).route("/", post(_))`.
- **breaking:** `routing::handler_method_router` and
  `routing::service_method_router` has been removed in favor of
  `routing::{get, get_service, ..., MethodRouter}`.
- **breaking:** `HandleErrorExt` has been removed in favor of
  `MethodRouter::handle_error`.
- **breaking:** `HandleErrorLayer` now requires the handler function to be
  `async` ([#534])
- **added:** `HandleErrorLayer` now supports running extractors.
- **breaking:** The `Handler<B, T>` trait is now defined as `Handler<T, B =
  Body>`. That is the type parameters have been swapped and `B` defaults to
  `axum::body::Body` ([#527])
- **breaking:** `Router::merge` will panic if both routers have fallbacks.
  Previously the left side fallback would be silently discarded ([#529])
- **breaking:** `Router::nest` will panic if the nested router has a fallback.
  Previously it would be silently discarded ([#529])
- Update WebSockets to use tokio-tungstenite 0.16 ([#525])
- **added:** Default to return `charset=utf-8` for text content type. ([#554])
- **breaking:** The `Body` and `BodyError` associated types on the
  `IntoResponse` trait have been removed - instead, `.into_response()` will now
  always return `Response<BoxBody>` ([#571])
- **breaking:** `PathParamsRejection` has been renamed to `PathRejection` and its
  variants renamed to `FailedToDeserializePathParams` and `MissingPathParams`. This
  makes it more consistent with the rest of axum ([#574])
- **added:** `Path`'s rejection type now provides data about exactly which part of
  the path couldn't be deserialized ([#574])

[#525]: https://github.com/tokio-rs/axum/pull/525
[#527]: https://github.com/tokio-rs/axum/pull/527
[#529]: https://github.com/tokio-rs/axum/pull/529
[#534]: https://github.com/tokio-rs/axum/pull/534
[#554]: https://github.com/tokio-rs/axum/pull/554
[#564]: https://github.com/tokio-rs/axum/pull/564
[#571]: https://github.com/tokio-rs/axum/pull/571
[#574]: https://github.com/tokio-rs/axum/pull/574

# 0.3.4 (13. November, 2021)

- **change:** `box_body` has been renamed to `boxed`. `box_body` still exists
  but is deprecated ([#530])

[#530]: https://github.com/tokio-rs/axum/pull/530

# 0.3.3 (13. November, 2021)

- Implement `FromRequest` for [`http::request::Parts`] so it can be used an
  extractor ([#489])
- Implement `IntoResponse` for `http::response::Parts` ([#490])

[#489]: https://github.com/tokio-rs/axum/pull/489
[#490]: https://github.com/tokio-rs/axum/pull/490
[`http::request::Parts`]: https://docs.rs/http/latest/http/request/struct.Parts.html

# 0.3.2 (08. November, 2021)

- **added:** Add `Router::route_layer` for applying middleware that
  will only run on requests that match a route. This is useful for middleware
  that return early, such as authorization ([#474])

[#474]: https://github.com/tokio-rs/axum/pull/474

# 0.3.1 (06. November, 2021)

- **fixed:** Implement `Clone` for `IntoMakeServiceWithConnectInfo` ([#471])

[#471]: https://github.com/tokio-rs/axum/pull/471

# 0.3.0 (02. November, 2021)

- Overall:
  - **fixed:** All known compile time issues are resolved, including those with
    `boxed` and those introduced by Rust 1.56 ([#404])
  - **breaking:** The router's type is now always `Router` regardless of how many routes or
    middleware are applied ([#404])

    This means router types are all always nameable:

    ```rust
    fn my_routes() -> Router {
        Router::new().route(
            "/users",
            post(|| async { "Hello, World!" }),
        )
    }
    ```
  - **breaking:** Added feature flags for HTTP1 and JSON. This enables removing a
    few dependencies if your app only uses HTTP2 or doesn't use JSON. This is only a
    breaking change if you depend on axum with `default_features = false`. ([#286])
  - **breaking:** `Route::boxed` and `BoxRoute` have been removed as they're no longer
    necessary ([#404])
  - **breaking:** `Nested`, `Or` types are now private. They no longer had to be
    public because `Router` is internally boxed ([#404])
  - **breaking:** Remove `routing::Layered` as it didn't actually do anything and
    thus wasn't necessary
  - **breaking:** Vendor `AddExtensionLayer` and `AddExtension` to reduce public
    dependencies
  - **breaking:** `body::BoxBody` is now a type alias for
    `http_body::combinators::UnsyncBoxBody` and thus is no longer `Sync`. This
    is because bodies are streams and requiring streams to be `Sync` is
    unnecessary.
  - **added:** Implement `IntoResponse` for `http_body::combinators::UnsyncBoxBody`.
  - **added:** Add `Handler::into_make_service` for serving a handler without a
    `Router`.
  - **added:** Add `Handler::into_make_service_with_connect_info` for serving a
    handler without a `Router`, and storing info about the incoming connection.
  - **breaking:** axum's minimum supported rust version is now 1.56
- Routing:
  - Big internal refactoring of routing leading to several improvements ([#363])
    - **added:** Wildcard routes like `.route("/api/users/*rest", service)` are now supported.
    - **fixed:** The order routes are added in no longer matters.
    - **fixed:** Adding a conflicting route will now cause a panic instead of silently making
      a route unreachable.
    - **fixed:** Route matching is faster as number of routes increases.
    - **breaking:** Handlers for multiple HTTP methods must be added in the same
      `Router::route` call. So `.route("/", get(get_handler).post(post_handler))` and
      _not_ `.route("/", get(get_handler)).route("/", post(post_handler))`.
  - **fixed:** Correctly handle trailing slashes in routes:
    - If a route with a trailing slash exists and a request without a trailing
      slash is received, axum will send a 301 redirection to the route with the
      trailing slash.
    - Or vice versa if a route without a trailing slash exists and a request
      with a trailing slash is received.
    - This can be overridden by explicitly defining two routes: One with and one
      without a trailing slash.
  - **breaking:** Method routing for handlers has been moved from `axum::handler`
    to `axum::routing`. So `axum::handler::get` now lives at `axum::routing::get`
    ([#405])
  - **breaking:** Method routing for services has been moved from `axum::service`
    to `axum::routing::service_method_routing`. So `axum::service::get` now lives at
    `axum::routing::service_method_routing::get`, etc. ([#405])
  - **breaking:** `Router::or` renamed to `Router::merge` and will now panic on
    overlapping routes. It now only accepts `Router`s and not general `Service`s.
    Use `Router::fallback` for adding fallback routes ([#408])
  - **added:** `Router::fallback` for adding handlers for request that didn't
    match any routes. `Router::fallback` must be use instead of `nest("/", _)` ([#408])
  - **breaking:** `EmptyRouter` has been renamed to `MethodNotAllowed` as it's only
    used in method routers and not in path routers (`Router`)
  - **breaking:** Remove support for routing based on the `CONNECT` method. An
    example of combining axum with and HTTP proxy can be found [here][proxy] ([#428])
- Extractors:
  - **fixed:** Expand accepted content types for JSON requests ([#378])
  - **fixed:** Support deserializing `i128` and `u128` in `extract::Path`
  - **breaking:** Automatically do percent decoding in `extract::Path`
    ([#272])
  - **breaking:** Change `Connected::connect_info` to return `Self` and remove
    the associated type `ConnectInfo` ([#396])
  - **added:** Add `extract::MatchedPath` for accessing path in router that
    matched the request ([#412])
- Error handling:
  - **breaking:** Simplify error handling model ([#402]):
    - All services part of the router are now required to be infallible.
    - Error handling utilities have been moved to an `error_handling` module.
    - `Router::check_infallible` has been removed since routers are always
      infallible with the error handling changes.
    - Error handling closures must now handle all errors and thus always return
      something that implements `IntoResponse`.

    With these changes handling errors from fallible middleware is done like so:

    ```rust,no_run
    use axum::{
        routing::get,
        http::StatusCode,
        error_handling::HandleErrorLayer,
        response::IntoResponse,
        Router, BoxError,
    };
    use tower::ServiceBuilder;
    use std::time::Duration;

    let middleware_stack = ServiceBuilder::new()
        // Handle errors from middleware
        //
        // This middleware most be added above any fallible
        // ones if you're using `ServiceBuilder`, due to how ordering works
        .layer(HandleErrorLayer::new(handle_error))
        // Return an error after 30 seconds
        .timeout(Duration::from_secs(30));

    let app = Router::new()
        .route("/", get(|| async { /* ... */ }))
        .layer(middleware_stack);

    fn handle_error(_error: BoxError) -> impl IntoResponse {
        StatusCode::REQUEST_TIMEOUT
    }
    ```

    And handling errors from fallible leaf services is done like so:

    ```rust
    use axum::{
        Router, service,
        body::Body,
        routing::service_method_routing::get,
        response::IntoResponse,
        http::{Request, Response},
        error_handling::HandleErrorExt, // for `.handle_error`
    };
    use std::{io, convert::Infallible};
    use tower::service_fn;

    let app = Router::new()
        .route(
            "/",
            get(service_fn(|_req: Request<Body>| async {
                let contents = tokio::fs::read_to_string("some_file").await?;
                Ok::<_, io::Error>(Response::new(Body::from(contents)))
            }))
            .handle_error(handle_io_error),
        );

    fn handle_io_error(error: io::Error) -> impl IntoResponse {
        // ...
    }
    ```
- Misc:
  - `InvalidWebsocketVersionHeader` has been renamed to `InvalidWebSocketVersionHeader` ([#416])
  - `WebsocketKeyHeaderMissing` has been renamed to `WebSocketKeyHeaderMissing` ([#416])

[#339]: https://github.com/tokio-rs/axum/pull/339
[#286]: https://github.com/tokio-rs/axum/pull/286
[#272]: https://github.com/tokio-rs/axum/pull/272
[#378]: https://github.com/tokio-rs/axum/pull/378
[#363]: https://github.com/tokio-rs/axum/pull/363
[#396]: https://github.com/tokio-rs/axum/pull/396
[#402]: https://github.com/tokio-rs/axum/pull/402
[#404]: https://github.com/tokio-rs/axum/pull/404
[#405]: https://github.com/tokio-rs/axum/pull/405
[#408]: https://github.com/tokio-rs/axum/pull/408
[#412]: https://github.com/tokio-rs/axum/pull/412
[#416]: https://github.com/tokio-rs/axum/pull/416
[#428]: https://github.com/tokio-rs/axum/pull/428
[proxy]: https://github.com/tokio-rs/axum/blob/main/examples/http-proxy/src/main.rs

# 0.2.8 (07. October, 2021)

- Document debugging handler type errors with "axum-debug" ([#372])

[#372]: https://github.com/tokio-rs/axum/pull/372

# 0.2.7 (06. October, 2021)

- Bump minimum version of async-trait ([#370])

[#370]: https://github.com/tokio-rs/axum/pull/370

# 0.2.6 (02. October, 2021)

- Clarify that `handler::any` and `service::any` only accepts standard HTTP
  methods ([#337])
- Document how to customize error responses from extractors ([#359])

[#337]: https://github.com/tokio-rs/axum/pull/337
[#359]: https://github.com/tokio-rs/axum/pull/359

# 0.2.5 (18. September, 2021)

- Add accessors for `TypedHeaderRejection` fields ([#317])
- Improve docs for extractors ([#327])

[#317]: https://github.com/tokio-rs/axum/pull/317
[#327]: https://github.com/tokio-rs/axum/pull/327

# 0.2.4 (10. September, 2021)

- Document using `StreamExt::split` with `WebSocket` ([#291])
- Document adding middleware to multiple groups of routes ([#293])

[#291]: https://github.com/tokio-rs/axum/pull/291
[#293]: https://github.com/tokio-rs/axum/pull/293

# 0.2.3 (26. August, 2021)

- **fixed:** Fix accidental breaking change introduced by internal refactor.
  `BoxRoute` used to be `Sync` but was accidental made `!Sync` ([#273](https://github.com/tokio-rs/axum/pull/273))

# 0.2.2 (26. August, 2021)

- **fixed:** Fix URI captures matching empty segments. This means requests with
  URI `/` will no longer be matched by `/:key` ([#264](https://github.com/tokio-rs/axum/pull/264))
- **fixed:** Remove needless trait bounds from `Router::boxed` ([#269](https://github.com/tokio-rs/axum/pull/269))

# 0.2.1 (24. August, 2021)

- **added:** Add `Redirect::to` constructor ([#255](https://github.com/tokio-rs/axum/pull/255))
- **added:** Document how to implement `IntoResponse` for custom error type ([#258](https://github.com/tokio-rs/axum/pull/258))

# 0.2.0 (23. August, 2021)

- Overall:
  - **fixed:** Overall compile time improvements. If you're having issues with compile time
    please file an issue! ([#184](https://github.com/tokio-rs/axum/pull/184)) ([#198](https://github.com/tokio-rs/axum/pull/198)) ([#220](https://github.com/tokio-rs/axum/pull/220))
  - **changed:** Remove `prelude`. Explicit imports are now required ([#195](https://github.com/tokio-rs/axum/pull/195))
- Routing:
  - **added:** Add dedicated `Router` to replace the `RoutingDsl` trait ([#214](https://github.com/tokio-rs/axum/pull/214))
  - **added:** Add `Router::or` for combining routes ([#108](https://github.com/tokio-rs/axum/pull/108))
  - **fixed:** Support matching different HTTP methods for the same route that aren't defined
    together. So `Router::new().route("/", get(...)).route("/", post(...))` now
    accepts both `GET` and `POST`. Previously only `POST` would be accepted ([#224](https://github.com/tokio-rs/axum/pull/224))
  - **fixed:** `get` routes will now also be called for `HEAD` requests but will always have
    the response body removed ([#129](https://github.com/tokio-rs/axum/pull/129))
  - **changed:** Replace `axum::route(...)` with `axum::Router::new().route(...)`. This means
    there is now only one way to create a new router. Same goes for
    `axum::routing::nest`. ([#215](https://github.com/tokio-rs/axum/pull/215))
  - **changed:** Implement `routing::MethodFilter` via [`bitflags`](https://crates.io/crates/bitflags) ([#158](https://github.com/tokio-rs/axum/pull/158))
  - **changed:** Move `handle_error` from `ServiceExt` to `service::OnMethod` ([#160](https://github.com/tokio-rs/axum/pull/160))

  With these changes this app using 0.1:

  ```rust
  use axum::{extract::Extension, prelude::*, routing::BoxRoute, AddExtensionLayer};

  let app = route("/", get(|| async { "hi" }))
      .nest("/api", api_routes())
      .layer(AddExtensionLayer::new(state));

  fn api_routes() -> BoxRoute<Body> {
      route(
          "/users",
          post(|Extension(state): Extension<State>| async { "hi from nested" }),
      )
      .boxed()
  }
  ```

  Becomes this in 0.2:

  ```rust
  use axum::{
      extract::Extension,
      handler::{get, post},
      routing::BoxRoute,
      Router,
  };

  let app = Router::new()
      .route("/", get(|| async { "hi" }))
      .nest("/api", api_routes());

  fn api_routes() -> Router<BoxRoute> {
      Router::new()
          .route(
              "/users",
              post(|Extension(state): Extension<State>| async { "hi from nested" }),
          )
          .boxed()
  }
  ```
- Extractors:
  - **added:** Make `FromRequest` default to being generic over `body::Body` ([#146](https://github.com/tokio-rs/axum/pull/146))
  - **added:** Implement `std::error::Error` for all rejections ([#153](https://github.com/tokio-rs/axum/pull/153))
  - **added:** Add `OriginalUri` for extracting original request URI in nested services ([#197](https://github.com/tokio-rs/axum/pull/197))
  - **added:** Implement `FromRequest` for `http::Extensions` ([#169](https://github.com/tokio-rs/axum/pull/169))
  - **added:** Make `RequestParts::{new, try_into_request}` public so extractors can be used outside axum ([#194](https://github.com/tokio-rs/axum/pull/194))
  - **added:** Implement `FromRequest` for `axum::body::Body` ([#241](https://github.com/tokio-rs/axum/pull/241))
  - **changed:** Removed `extract::UrlParams` and `extract::UrlParamsMap`. Use `extract::Path` instead ([#154](https://github.com/tokio-rs/axum/pull/154))
  - **changed:** `extractor_middleware` now requires `RequestBody: Default` ([#167](https://github.com/tokio-rs/axum/pull/167))
  - **changed:** Convert `RequestAlreadyExtracted` to an enum with each possible error variant ([#167](https://github.com/tokio-rs/axum/pull/167))
  - **changed:** `extract::BodyStream` is no longer generic over the request body ([#234](https://github.com/tokio-rs/axum/pull/234))
  - **changed:** `extract::Body` has been renamed to `extract::RawBody` to avoid conflicting with `body::Body` ([#233](https://github.com/tokio-rs/axum/pull/233))
  - **changed:** `RequestParts` changes ([#153](https://github.com/tokio-rs/axum/pull/153))
      - `method` new returns an `&http::Method`
      - `method_mut` new returns an `&mut http::Method`
      - `take_method` has been removed
      - `uri` new returns an `&http::Uri`
      - `uri_mut` new returns an `&mut http::Uri`
      - `take_uri` has been removed
  - **changed:** Remove several rejection types that were no longer used ([#153](https://github.com/tokio-rs/axum/pull/153)) ([#154](https://github.com/tokio-rs/axum/pull/154))
- Responses:
  - **added:** Add `Headers` for easily customizing headers on a response ([#193](https://github.com/tokio-rs/axum/pull/193))
  - **added:** Add `Redirect` response ([#192](https://github.com/tokio-rs/axum/pull/192))
  - **added:** Add `body::StreamBody` for easily responding with a stream of byte chunks ([#237](https://github.com/tokio-rs/axum/pull/237))
  - **changed:** Add associated `Body` and `BodyError` types to `IntoResponse`. This is
    required for returning responses with bodies other than `hyper::Body` from
    handlers. See the docs for advice on how to implement `IntoResponse` ([#86](https://github.com/tokio-rs/axum/pull/86))
  - **changed:** `tower::util::Either` no longer implements `IntoResponse` ([#229](https://github.com/tokio-rs/axum/pull/229))

  This `IntoResponse` from 0.1:
  ```rust
  use axum::{http::Response, prelude::*, response::IntoResponse};

  struct MyResponse;

  impl IntoResponse for MyResponse {
      fn into_response(self) -> Response<Body> {
          Response::new(Body::empty())
      }
  }
  ```

  Becomes this in 0.2:
  ```rust
  use axum::{body::Body, http::Response, response::IntoResponse};

  struct MyResponse;

  impl IntoResponse for MyResponse {
      type Body = Body;
      type BodyError = <Self::Body as axum::body::HttpBody>::Error;

      fn into_response(self) -> Response<Self::Body> {
          Response::new(Body::empty())
      }
  }
  ```
- SSE:
  - **added:** Add `response::sse::Sse`. This implements SSE using a response rather than a service ([#98](https://github.com/tokio-rs/axum/pull/98))
  - **changed:** Remove `axum::sse`. It has been replaced by `axum::response::sse` ([#98](https://github.com/tokio-rs/axum/pull/98))

  Handler using SSE in 0.1:
  ```rust
  use axum::{
      prelude::*,
      sse::{sse, Event},
  };
  use std::convert::Infallible;

  let app = route(
      "/",
      sse(|| async {
          let stream = futures::stream::iter(vec![Ok::<_, Infallible>(
              Event::default().data("hi there!"),
          )]);
          Ok::<_, Infallible>(stream)
      }),
  );
  ```

  Becomes this in 0.2:

  ```rust
  use axum::{
      handler::get,
      response::sse::{Event, Sse},
      Router,
  };
  use std::convert::Infallible;

  let app = Router::new().route(
      "/",
      get(|| async {
          let stream = futures::stream::iter(vec![Ok::<_, Infallible>(
              Event::default().data("hi there!"),
          )]);
          Sse::new(stream)
      }),
  );
  ```
- WebSockets:
  - **changed:** Change WebSocket API to use an extractor plus a response ([#121](https://github.com/tokio-rs/axum/pull/121))
  - **changed:** Make WebSocket `Message` an enum ([#116](https://github.com/tokio-rs/axum/pull/116))
  - **changed:** `WebSocket` now uses `Error` as its error type ([#150](https://github.com/tokio-rs/axum/pull/150))

  Handler using WebSockets in 0.1:

  ```rust
  use axum::{
      prelude::*,
      ws::{ws, WebSocket},
  };

  let app = route(
      "/",
      ws(|socket: WebSocket| async move {
          // do stuff with socket
      }),
  );
  ```

  Becomes this in 0.2:

  ```rust
  use axum::{
      extract::ws::{WebSocket, WebSocketUpgrade},
      handler::get,
      Router,
  };

  let app = Router::new().route(
      "/",
      get(|ws: WebSocketUpgrade| async move {
          ws.on_upgrade(|socket: WebSocket| async move {
              // do stuff with socket
          })
      }),
  );
  ```
- Misc
  - **added:** Add default feature `tower-log` which exposes `tower`'s `log` feature. ([#218](https://github.com/tokio-rs/axum/pull/218))
  - **changed:** Replace `body::BoxStdError` with `axum::Error`, which supports downcasting ([#150](https://github.com/tokio-rs/axum/pull/150))
  - **changed:** `EmptyRouter` now requires the response body to implement `Send + Sync + 'static'` ([#108](https://github.com/tokio-rs/axum/pull/108))
  - **changed:** `Router::check_infallible` now returns a `CheckInfallible` service. This
    is to improve compile times ([#198](https://github.com/tokio-rs/axum/pull/198))
  - **changed:** `Router::into_make_service` now returns `routing::IntoMakeService` rather than
    `tower::make::Shared` ([#229](https://github.com/tokio-rs/axum/pull/229))
  - **changed:** All usage of `tower::BoxError` has been replaced with `axum::BoxError` ([#229](https://github.com/tokio-rs/axum/pull/229))
  - **changed:** Several response future types have been moved into dedicated
    `future` modules ([#133](https://github.com/tokio-rs/axum/pull/133))
  - **changed:** `EmptyRouter`, `ExtractorMiddleware`, `ExtractorMiddlewareLayer`,
    and `QueryStringMissing` no longer implement `Copy` ([#132](https://github.com/tokio-rs/axum/pull/132))
  - **changed:** `service::OnMethod`, `handler::OnMethod`, and `routing::Nested` have new response future types ([#157](https://github.com/tokio-rs/axum/pull/157))

# 0.1.3 (06. August, 2021)

- Fix stripping prefix when nesting services at `/` ([#91](https://github.com/tokio-rs/axum/pull/91))
- Add support for WebSocket protocol negotiation ([#83](https://github.com/tokio-rs/axum/pull/83))
- Use `pin-project-lite` instead of `pin-project` ([#95](https://github.com/tokio-rs/axum/pull/95))
- Re-export `http` crate and `hyper::Server` ([#110](https://github.com/tokio-rs/axum/pull/110))
- Fix `Query` and `Form` extractors giving bad request error when query string is empty. ([#117](https://github.com/tokio-rs/axum/pull/117))
- Add `Path` extractor. ([#124](https://github.com/tokio-rs/axum/pull/124))
- Fixed the implementation of `IntoResponse` of `(HeaderMap, T)` and `(StatusCode, HeaderMap, T)` would ignore headers from `T` ([#137](https://github.com/tokio-rs/axum/pull/137))
- Deprecate `extract::UrlParams` and `extract::UrlParamsMap`. Use `extract::Path` instead ([#138](https://github.com/tokio-rs/axum/pull/138))

# 0.1.2 (01. August, 2021)

- Implement `Stream` for `WebSocket` ([#52](https://github.com/tokio-rs/axum/pull/52))
- Implement `Sink` for `WebSocket` ([#52](https://github.com/tokio-rs/axum/pull/52))
- Implement `Deref` most extractors ([#56](https://github.com/tokio-rs/axum/pull/56))
- Return `405 Method Not Allowed` for unsupported method for route ([#63](https://github.com/tokio-rs/axum/pull/63))
- Add extractor for remote connection info ([#55](https://github.com/tokio-rs/axum/pull/55))
- Improve error message of `MissingExtension` rejections ([#72](https://github.com/tokio-rs/axum/pull/72))
- Improve documentation for routing ([#71](https://github.com/tokio-rs/axum/pull/71))
- Clarify required response body type when routing to `tower::Service`s ([#69](https://github.com/tokio-rs/axum/pull/69))
- Add `axum::body::box_body` to converting an `http_body::Body` to `axum::body::BoxBody` ([#69](https://github.com/tokio-rs/axum/pull/69))
- Add `axum::sse` for Server-Sent Events ([#75](https://github.com/tokio-rs/axum/pull/75))
- Mention required dependencies in docs ([#77](https://github.com/tokio-rs/axum/pull/77))
- Fix WebSockets failing on Firefox ([#76](https://github.com/tokio-rs/axum/pull/76))

# 0.1.1 (30. July, 2021)

- Misc readme fixes.

# 0.1.0 (30. July, 2021)

- Initial release.
