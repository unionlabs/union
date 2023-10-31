Nest a group of routes (or a [`Service`]) at some path.

This allows you to break your application into smaller pieces and compose
them together.

# Example

```rust
use axum::{
    routing::{get, post},
    Router,
};

let user_routes = Router::new().route("/:id", get(|| async {}));

let team_routes = Router::new().route("/", post(|| async {}));

let api_routes = Router::new()
    .nest("/users", user_routes)
    .nest("/teams", team_routes);

let app = Router::new().nest("/api", api_routes);

// Our app now accepts
// - GET /api/users/:id
// - POST /api/teams
# async {
# axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
# };
```

# How the URI changes

Note that nested routes will not see the original request URI but instead
have the matched prefix stripped. This is necessary for services like static
file serving to work. Use [`OriginalUri`] if you need the original request
URI.

# Captures from outer routes

Take care when using `nest` together with dynamic routes as nesting also
captures from the outer routes:

```rust
use axum::{
    extract::Path,
    routing::get,
    Router,
};
use std::collections::HashMap;

async fn users_get(Path(params): Path<HashMap<String, String>>) {
    // Both `version` and `id` were captured even though `users_api` only
    // explicitly captures `id`.
    let version = params.get("version");
    let id = params.get("id");
}

let users_api = Router::new().route("/users/:id", get(users_get));

let app = Router::new().nest("/:version/api", users_api);
# async {
# axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
# };
```

# Nesting services

`nest` also accepts any [`Service`]. This can for example be used with
[`tower_http::services::ServeDir`] to serve static files from a directory:

```rust
use axum::{
    Router,
    routing::get_service,
    http::StatusCode,
    error_handling::HandleErrorLayer,
};
use std::{io, convert::Infallible};
use tower_http::services::ServeDir;

// Serves files inside the `public` directory at `GET /public/*`
let serve_dir_service = get_service(ServeDir::new("public"))
    .handle_error(|error: io::Error| async move {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Unhandled internal error: {}", error),
        )
    });

let app = Router::new().nest("/public", serve_dir_service);
# async {
# axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
# };
```

# Differences to wildcard routes

Nested routes are similar to wildcard routes. The difference is that
wildcard routes still see the whole URI whereas nested routes will have
the prefix stripped:

```rust
use axum::{routing::get, http::Uri, Router};

let app = Router::new()
    .route("/foo/*rest", get(|uri: Uri| async {
        // `uri` will contain `/foo`
    }))
    .nest("/bar", get(|uri: Uri| async {
        // `uri` will _not_ contain `/bar`
    }));
# async {
# axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
# };
```

# Panics

- If the route overlaps with another route. See [`Router::route`]
for more details.
- If the route contains a wildcard (`*`).
- If `path` is empty.
- If the nested router has a [fallback](Router::fallback). This is because
  `Router` only allows a single fallback.

[`OriginalUri`]: crate::extract::OriginalUri
