Add another route to the router.

`path` is a string of path segments separated by `/`. Each segment
can be either static, a capture, or a wildcard.

`service` is the [`Service`] that should receive the request if the path matches
`path`. `service` will commonly be a handler wrapped in a method router like
[`get`](crate::routing::get). See [`handler`](crate::handler) for more details
on handlers.

# Static paths

Examples:

- `/`
- `/foo`
- `/users/123`

If the incoming request matches the path exactly the corresponding service will
be called.

# Captures

Paths can contain segments like `/:key` which matches any single segment and
will store the value captured at `key`.

Examples:

- `/:key`
- `/users/:id`
- `/users/:id/tweets`

Captures can be extracted using [`Path`](crate::extract::Path). See its
documentation for more details.

It is not possible to create segments that only match some types like numbers or
regular expression. You must handle that manually in your handlers.

[`MatchedPath`](crate::extract::MatchedPath) can be used to extract the matched
path rather than the actual path.

# Wildcards

Paths can end in `/*key` which matches all segments and will store the segments
captured at `key`.

Examples:

- `/*key`
- `/assets/*path`
- `/:id/:repo/*tree`

Wildcard captures can also be extracted using [`Path`](crate::extract::Path).

# Accepting multiple methods

To accept multiple methods for the same route you must add all handlers at the
same time:

```rust
use axum::{Router, routing::{get, delete}, extract::Path};

let app = Router::new().route(
    "/",
    get(get_root).post(post_root).delete(delete_root),
);

async fn get_root() {}

async fn post_root() {}

async fn delete_root() {}
# async {
# axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
# };
```

# More examples

```rust
use axum::{Router, routing::{get, delete}, extract::Path};

let app = Router::new()
    .route("/", get(root))
    .route("/users", get(list_users).post(create_user))
    .route("/users/:id", get(show_user))
    .route("/api/:version/users/:id/action", delete(do_users_action))
    .route("/assets/*path", get(serve_asset));

async fn root() {}

async fn list_users() {}

async fn create_user() {}

async fn show_user(Path(id): Path<u64>) {}

async fn do_users_action(Path((version, id)): Path<(String, u64)>) {}

async fn serve_asset(Path(path): Path<String>) {}
# async {
# axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
# };
```

# Routing to any [`Service`]

axum also supports routing to general [`Service`]s:

```rust,no_run
use axum::{
    Router,
    body::Body,
    routing::{any_service, get_service},
    http::{Request, StatusCode},
    error_handling::HandleErrorLayer,
};
use tower_http::services::ServeFile;
use http::Response;
use std::{convert::Infallible, io};
use tower::service_fn;

let app = Router::new()
    .route(
        // Any request to `/` goes to a service
        "/",
        // Services whose response body is not `axum::body::BoxBody`
        // can be wrapped in `axum::routing::any_service` (or one of the other routing filters)
        // to have the response body mapped
        any_service(service_fn(|_: Request<Body>| async {
            let res = Response::new(Body::from("Hi from `GET /`"));
            Ok::<_, Infallible>(res)
        }))
    )
    .route(
        "/foo",
        // This service's response body is `axum::body::BoxBody` so
        // it can be routed to directly.
        service_fn(|req: Request<Body>| async move {
            let body = Body::from(format!("Hi from `{} /foo`", req.method()));
            let body = axum::body::boxed(body);
            let res = Response::new(body);
            Ok::<_, Infallible>(res)
        })
    )
    .route(
        // GET `/static/Cargo.toml` goes to a service from tower-http
        "/static/Cargo.toml",
        get_service(ServeFile::new("Cargo.toml"))
            // though we must handle any potential errors
            .handle_error(|error: io::Error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {}", error),
                )
            })
    );
# async {
# axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
# };
```

Routing to arbitrary services in this way has complications for backpressure
([`Service::poll_ready`]). See the [Routing to services and backpressure] module
for more details.

[Routing to services and backpressure]: middleware/index.html#routing-to-servicesmiddleware-and-backpressure

# Panics

Panics if the route overlaps with another route:

```rust,should_panic
use axum::{routing::get, Router};

let app = Router::new()
    .route("/", get(|| async {}))
    .route("/", get(|| async {}));
# async {
# axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
# };
```

The static route `/foo` and the dynamic route `/:key` are not considered to
overlap and `/foo` will take precedence.

Take care when using [`Router::nest`] as it behaves like a wildcard route.
Therefore this setup panics:

```rust,should_panic
use axum::{routing::get, Router};

let app = Router::new()
    // this is similar to `/api/*`
    .nest("/api", get(|| async {}))
    // which overlaps with this route
    .route("/api/users", get(|| async {}));
# async {
# axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
# };
```

Also panics if `path` is empty.

## Nesting

`route` cannot be used to nest `Router`s. Instead use [`Router::nest`].

Attempting to will result in a panic:

```rust,should_panic
use axum::{routing::get, Router};

let app = Router::new().route(
    "/",
    Router::new().route("/foo", get(|| async {})),
);
# async {
# axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
# };
```
