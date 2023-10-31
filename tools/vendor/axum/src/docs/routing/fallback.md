Add a fallback service to the router.

This service will be called if no routes matches the incoming request.

```rust
use axum::{
    Router,
    routing::get,
    handler::Handler,
    response::IntoResponse,
    http::{StatusCode, Uri},
};

let app = Router::new()
    .route("/foo", get(|| async { /* ... */ }))
    .fallback(fallback.into_service());

async fn fallback(uri: Uri) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, format!("No route for {}", uri))
}
# async {
# hyper::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
# };
```

Fallbacks only apply to routes that aren't matched by anything in the
router. If a handler is matched by a request but returns 404 the
fallback is not called.
