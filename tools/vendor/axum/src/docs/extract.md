Types and traits for extracting data from requests.

# Table of contents

- [Intro](#intro)
- [Common extractors](#common-extractors)
- [Applying multiple extractors](#applying-multiple-extractors)
- [Be careful when extracting `Request`](#be-careful-when-extracting-request)
- [Optional extractors](#optional-extractors)
- [Customizing extractor responses](#customizing-extractor-responses)
- [Accessing inner errors](#accessing-inner-errors)
- [Defining custom extractors](#defining-custom-extractors)
- [Accessing other extractors in `FromRequest` implementations](#accessing-other-extractors-in-fromrequest-implementations)
- [Request body limits](#request-body-limits)
- [Request body extractors](#request-body-extractors)
- [Running extractors from middleware](#running-extractors-from-middleware)

# Intro

A handler function is an async function that takes any number of
"extractors" as arguments. An extractor is a type that implements
[`FromRequest`](crate::extract::FromRequest).

For example, [`Json`] is an extractor that consumes the request body and
deserializes it as JSON into some target type:

```rust,no_run
use axum::{
    extract::Json,
    routing::post,
    handler::Handler,
    Router,
};
use serde::Deserialize;

#[derive(Deserialize)]
struct CreateUser {
    email: String,
    password: String,
}

async fn create_user(Json(payload): Json<CreateUser>) {
    // ...
}

let app = Router::new().route("/users", post(create_user));
# async {
# axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
# };
```

# Common extractors

Some commonly used extractors are:

```rust,no_run
use axum::{
    extract::{Json, TypedHeader, Path, Extension, Query},
    routing::post,
    headers::UserAgent,
    http::{Request, header::HeaderMap},
    body::{Bytes, Body},
    Router,
};
use serde_json::Value;
use std::collections::HashMap;

// `Path` gives you the path parameters and deserializes them. See its docs for
// more details
async fn path(Path(user_id): Path<u32>) {}

// `Query` gives you the query parameters and deserializes them.
async fn query(Query(params): Query<HashMap<String, String>>) {}

// `HeaderMap` gives you all the headers
async fn headers(headers: HeaderMap) {}

// `TypedHeader` can be used to extract a single header
// note this requires you've enabled axum's `headers` feature
async fn user_agent(TypedHeader(user_agent): TypedHeader<UserAgent>) {}

// `String` consumes the request body and ensures it is valid utf-8
async fn string(body: String) {}

// `Bytes` gives you the raw request body
async fn bytes(body: Bytes) {}

// We've already seen `Json` for parsing the request body as json
async fn json(Json(payload): Json<Value>) {}

// `Request` gives you the whole request for maximum control
async fn request(request: Request<Body>) {}

// `Extension` extracts data from "request extensions"
// This is commonly used to share state with handlers
async fn extension(Extension(state): Extension<State>) {}

#[derive(Clone)]
struct State { /* ... */ }

let app = Router::new()
    .route("/path/:user_id", post(path))
    .route("/query", post(query))
    .route("/user_agent", post(user_agent))
    .route("/headers", post(headers))
    .route("/string", post(string))
    .route("/bytes", post(bytes))
    .route("/json", post(json))
    .route("/request", post(request))
    .route("/extension", post(extension));
# async {
# axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
# };
```

# Applying multiple extractors

You can also apply multiple extractors:

```rust,no_run
use axum::{
    extract::{Path, Query},
    routing::get,
    Router,
};
use uuid::Uuid;
use serde::Deserialize;

let app = Router::new().route("/users/:id/things", get(get_user_things));

#[derive(Deserialize)]
struct Pagination {
    page: usize,
    per_page: usize,
}

impl Default for Pagination {
    fn default() -> Self {
        Self { page: 1, per_page: 30 }
    }
}

async fn get_user_things(
    Path(user_id): Path<Uuid>,
    pagination: Option<Query<Pagination>>,
) {
    let Query(pagination) = pagination.unwrap_or_default();

    // ...
}
# async {
# axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
# };
```

Extractors always run in the order of the function parameters that is from
left to right.

# Be careful when extracting `Request`

[`Request`] is itself an extractor:

```rust,no_run
use axum::{http::Request, body::Body};

async fn handler(request: Request<Body>) {
    // ...
}
```

However be careful when combining it with other extractors since it will consume
all extensions and the request body. Therefore it is recommended to always apply
the request extractor last:

```rust,no_run
use axum::{http::Request, Extension, body::Body};

// this will fail at runtime since `Request<Body>` will have consumed all the
// extensions so `Extension<State>` will be missing
async fn broken(
    request: Request<Body>,
    Extension(state): Extension<State>,
) {
    // ...
}

// this will work since we extract `Extension<State>` before `Request<Body>`
async fn works(
    Extension(state): Extension<State>,
    request: Request<Body>,
) {
    // ...
}

#[derive(Clone)]
struct State {};
```

# Extracting request bodies

Since request bodies are asynchronous streams they can only be extracted once:

```rust,no_run
use axum::{Json, http::Request, body::{Bytes, Body}};
use serde_json::Value;

// this will fail at runtime since `Json<Value>` and `Bytes` both attempt to extract
// the body
//
// the solution is to only extract the body once so remove either
// `body_json: Json<Value>` or `body_bytes: Bytes`
async fn broken(
    body_json: Json<Value>,
    body_bytes: Bytes,
) {
    // ...
}

// this doesn't work either for the same reason: `Bytes` and `Request<Body>`
// both extract the body
async fn also_broken(
    body_json: Json<Value>,
    request: Request<Body>,
) {
    // ...
}
```

Also keep this in mind if you extract or otherwise consume the body in
middleware. You either need to not extract the body in handlers or make sure
your middleware reinserts the body using [`RequestParts::body_mut`] so it's
available to handlers.

# Optional extractors

All extractors defined in axum will reject the request if it doesn't match.
If you wish to make an extractor optional you can wrap it in `Option`:

```rust,no_run
use axum::{
    extract::Json,
    routing::post,
    Router,
};
use serde_json::Value;

async fn create_user(payload: Option<Json<Value>>) {
    if let Some(payload) = payload {
        // We got a valid JSON payload
    } else {
        // Payload wasn't valid JSON
    }
}

let app = Router::new().route("/users", post(create_user));
# async {
# axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
# };
```

Wrapping extractors in `Result` makes them optional and gives you the reason
the extraction failed:

```rust,no_run
use axum::{
    extract::{Json, rejection::JsonRejection},
    routing::post,
    Router,
};
use serde_json::Value;

async fn create_user(payload: Result<Json<Value>, JsonRejection>) {
    match payload {
        Ok(payload) => {
            // We got a valid JSON payload
        }
        Err(JsonRejection::MissingJsonContentType(_)) => {
            // Request didn't have `Content-Type: application/json`
            // header
        }
        Err(JsonRejection::JsonDataError(_)) => {
            // Couldn't deserialize the body into the target type
        }
        Err(JsonRejection::JsonSyntaxError(_)) => {
            // Syntax error in the body
        }
        Err(JsonRejection::BytesRejection(_)) => {
            // Failed to extract the request body
        }
        Err(_) => {
            // `JsonRejection` is marked `#[non_exhaustive]` so match must
            // include a catch-all case.
        }
    }
}

let app = Router::new().route("/users", post(create_user));
# async {
# axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
# };
```

# Customizing extractor responses

If an extractor fails it will return a response with the error and your
handler will not be called. To customize the error response you have a two
options:

1. Use `Result<T, T::Rejection>` as your extractor like shown in ["Optional
   extractors"](#optional-extractors). This works well if you're only using
   the extractor in a single handler.
2. Create your own extractor that in its [`FromRequest`] implemention calls
   one of axum's built in extractors but returns a different response for
   rejections. See the [customize-extractor-error] example for more details.

# Accessing inner errors

axum's built-in extractors don't directly expose the inner error. This gives us
more flexibility and allows us to change internal implementations without
breaking the public API.

For example that means while [`Json`] is implemented using [`serde_json`] it
doesn't directly expose the [`serde_json::Error`] thats contained in
[`JsonRejection::JsonDataError`]. However it is still possible to access via
methods from [`std::error::Error`]:

```rust
use std::error::Error;
use axum::{
    extract::{Json, rejection::JsonRejection},
    response::IntoResponse,
    http::StatusCode,
};
use serde_json::{json, Value};

async fn handler(
    result: Result<Json<Value>, JsonRejection>,
) -> Result<Json<Value>, (StatusCode, String)> {
    match result {
        // if the client sent valid JSON then we're good
        Ok(Json(payload)) => Ok(Json(json!({ "payload": payload }))),

        Err(err) => match err {
            JsonRejection::JsonDataError(err) => {
                Err(serde_json_error_response(err))
            }
            JsonRejection::JsonSyntaxError(err) => {
                Err(serde_json_error_response(err))
            }
            // handle other rejections from the `Json` extractor
            JsonRejection::MissingJsonContentType(_) => Err((
                StatusCode::BAD_REQUEST,
                "Missing `Content-Type: application/json` header".to_string(),
            )),
            JsonRejection::BytesRejection(_) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to buffer request body".to_string(),
            )),
            // we must provide a catch-all case since `JsonRejection` is marked
            // `#[non_exhaustive]`
            _ => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unknown error".to_string(),
            )),
        },
    }
}

// attempt to extract the inner `serde_json::Error`, if that succeeds we can
// provide a more specific error
fn serde_json_error_response<E>(err: E) -> (StatusCode, String)
where
    E: Error + 'static,
{
    if let Some(serde_json_err) = find_error_source::<serde_json::Error>(&err) {
        (
            StatusCode::BAD_REQUEST,
            format!(
                "Invalid JSON at line {} column {}",
                serde_json_err.line(),
                serde_json_err.column()
            ),
        )
    } else {
        (StatusCode::BAD_REQUEST, "Unknown error".to_string())
    }
}

// attempt to downcast `err` into a `T` and if that fails recursively try and
// downcast `err`'s source
fn find_error_source<'a, T>(err: &'a (dyn Error + 'static)) -> Option<&'a T>
where
    T: Error + 'static,
{
    if let Some(err) = err.downcast_ref::<T>() {
        Some(err)
    } else if let Some(source) = err.source() {
        find_error_source(source)
    } else {
        None
    }
}
```

Note that while this approach works it might break in the future if axum changes
its implementation to use a different error type internally. Such changes might
happen without major breaking versions.

# Defining custom extractors

You can also define your own extractors by implementing [`FromRequest`]:

```rust,no_run
use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
    routing::get,
    Router,
};
use http::{StatusCode, header::{HeaderValue, USER_AGENT}};

struct ExtractUserAgent(HeaderValue);

#[async_trait]
impl<B> FromRequest<B> for ExtractUserAgent
where
    B: Send,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        if let Some(user_agent) = req.headers().get(USER_AGENT) {
            Ok(ExtractUserAgent(user_agent.clone()))
        } else {
            Err((StatusCode::BAD_REQUEST, "`User-Agent` header is missing"))
        }
    }
}

async fn handler(ExtractUserAgent(user_agent): ExtractUserAgent) {
    // ...
}

let app = Router::new().route("/foo", get(handler));
# async {
# axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
# };
```

# Accessing other extractors in [`FromRequest`] implementations

When defining custom extractors you often need to access another extractors
in your implementation.

```rust
use axum::{
    async_trait,
    extract::{Extension, FromRequest, RequestParts, TypedHeader},
    headers::{authorization::Bearer, Authorization},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};

#[derive(Clone)]
struct State {
    // ...
}

struct AuthenticatedUser {
    // ...
}

#[async_trait]
impl<B> FromRequest<B> for AuthenticatedUser
where
    B: Send,
{
    type Rejection = Response;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(token)) = 
            TypedHeader::<Authorization<Bearer>>::from_request(req)
                .await
                .map_err(|err| err.into_response())?;

        let Extension(state): Extension<State> = Extension::from_request(req)
            .await
            .map_err(|err| err.into_response())?;

        // actually perform the authorization...
        unimplemented!()
    }
}

async fn handler(user: AuthenticatedUser) {
    // ...
}

let state = State { /* ... */ };

let app = Router::new().route("/", get(handler)).layer(Extension(state));
# async {
# axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
# };
```

# Request body limits

For security reasons, [`Bytes`] will, by default, not accept bodies larger than
2MB. This also applies to extractors that uses [`Bytes`] internally such as
`String`, [`Json`], and [`Form`].

For more details, including how to disable this limit, see [`DefaultBodyLimit`].

# Request body extractors

Most of the time your request body type will be [`body::Body`] (a re-export
of [`hyper::Body`]), which is directly supported by all extractors.

However if you're applying a tower middleware that changes the request body type
you might have to apply a different body type to some extractors:

```rust
use std::{
    task::{Context, Poll},
    pin::Pin,
};
use tower_http::map_request_body::MapRequestBodyLayer;
use axum::{
    extract::{self, BodyStream},
    body::{Body, HttpBody},
    routing::get,
    http::{header::HeaderMap, Request},
    Router,
};

struct MyBody<B>(B);

impl<B> HttpBody for MyBody<B>
where
    B: HttpBody + Unpin,
{
    type Data = B::Data;
    type Error = B::Error;

    fn poll_data(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        Pin::new(&mut self.0).poll_data(cx)
    }

    fn poll_trailers(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        Pin::new(&mut self.0).poll_trailers(cx)
    }
}

let app = Router::new()
    .route(
        "/string",
        // `String` works directly with any body type
        get(|_: String| async {})
    )
    .route(
        "/body",
        // `extract::Body` defaults to `axum::body::Body`
        // but can be customized
        get(|_: extract::RawBody<MyBody<Body>>| async {})
    )
    .route(
        "/body-stream",
        // same for `extract::BodyStream`
        get(|_: extract::BodyStream| async {}),
    )
    .route(
        // and `Request<_>`
        "/request",
        get(|_: Request<MyBody<Body>>| async {})
    )
    // middleware that changes the request body type
    .layer(MapRequestBodyLayer::new(MyBody));
# async {
# axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
# };
```

# Running extractors from middleware

Extractors can also be run from middleware by making a [`RequestParts`] and
running your extractor:

```rust
use axum::{
    Router,
    middleware::{self, Next},
    extract::{RequestParts, TypedHeader},
    http::{Request, StatusCode},
    response::Response,
    headers::authorization::{Authorization, Bearer},
};

async fn auth_middleware<B>(
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode>
where
    B: Send,
{
    // running extractors requires a `RequestParts`
    let mut request_parts = RequestParts::new(request);

    // `TypedHeader<Authorization<Bearer>>` extracts the auth token but
    // `RequestParts::extract` works with anything that implements `FromRequest`
    let auth = request_parts.extract::<TypedHeader<Authorization<Bearer>>>()
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    if !token_is_valid(auth.token()) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // get the request back so we can run `next`
    //
    // `try_into_request` will fail if you have extracted the request body. We
    // know that `TypedHeader` never does that.
    //
    // see the `consume-body-in-extractor-or-middleware` example if you need to
    // extract the body
    let request = request_parts.try_into_request().expect("body extracted");

    Ok(next.run(request).await)
}

fn token_is_valid(token: &str) -> bool {
    // ...
    # false
}

let app = Router::new().layer(middleware::from_fn(auth_middleware));
# let _: Router = app;
```

[`body::Body`]: crate::body::Body
[`Bytes`]: crate::body::Bytes
[customize-extractor-error]: https://github.com/tokio-rs/axum/blob/main/examples/customize-extractor-error/src/main.rs
[`HeaderMap`]: https://docs.rs/http/latest/http/header/struct.HeaderMap.html
[`Request`]: https://docs.rs/http/latest/http/struct.Request.html
[`RequestParts::body_mut`]: crate::extract::RequestParts::body_mut
