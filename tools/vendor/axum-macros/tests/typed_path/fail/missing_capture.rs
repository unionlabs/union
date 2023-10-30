use axum_macros::TypedPath;
use serde::Deserialize;

#[derive(TypedPath, Deserialize)]
#[typed_path("/users")]
struct MyPath {
    id: u32,
}

fn main() {}
