use axum_macros::TypedPath;

#[derive(TypedPath)]
#[typed_path("/users/:id")]
struct MyPath {
    id: u32,
}

fn main() {}
