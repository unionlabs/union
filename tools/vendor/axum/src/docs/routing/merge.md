Merge two routers into one.

This is useful for breaking apps into smaller pieces and combining them
into one.

```rust
use axum::{
    routing::get,
    Router,
};
#
# async fn users_list() {}
# async fn users_show() {}
# async fn teams_list() {}

// define some routes separately
let user_routes = Router::new()
    .route("/users", get(users_list))
    .route("/users/:id", get(users_show));

let team_routes = Router::new()
    .route("/teams", get(teams_list));

// combine them into one
let app = Router::new()
    .merge(user_routes)
    .merge(team_routes);

// could also do `user_routes.merge(team_routes)`

// Our app now accepts
// - GET /users
// - GET /users/:id
// - POST /teams
# async {
# hyper::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
# };
```

## Panics

- If two routers that each have a [fallback](Router::fallback) are merged. This
  is because `Router` only allows a single fallback.
