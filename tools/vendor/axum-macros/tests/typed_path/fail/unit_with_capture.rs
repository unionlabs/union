use axum_macros::TypedPath;
use serde::Deserialize;

#[derive(TypedPath, Deserialize)]
#[typed_path("/users/:id")]
struct MyPath;

fn main() {}
