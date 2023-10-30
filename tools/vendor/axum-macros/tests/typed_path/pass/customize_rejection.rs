use axum::{
    extract::rejection::PathRejection,
    response::{IntoResponse, Response},
};
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;

#[derive(TypedPath, Deserialize)]
#[typed_path("/:foo", rejection(MyRejection))]
struct MyPathNamed {
    foo: String,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/", rejection(MyRejection))]
struct MyPathUnit;

#[derive(TypedPath, Deserialize)]
#[typed_path("/:foo", rejection(MyRejection))]
struct MyPathUnnamed(String);

struct MyRejection;

impl IntoResponse for MyRejection {
    fn into_response(self) -> Response {
        ().into_response()
    }
}

impl From<PathRejection> for MyRejection {
    fn from(_: PathRejection) -> Self {
        Self
    }
}

impl Default for MyRejection {
    fn default() -> Self {
        Self
    }
}

fn main() {
    axum::Router::<axum::body::Body>::new()
        .typed_get(|_: Result<MyPathNamed, MyRejection>| async {})
        .typed_post(|_: Result<MyPathUnnamed, MyRejection>| async {})
        .typed_put(|_: Result<MyPathUnit, MyRejection>| async {});
}
