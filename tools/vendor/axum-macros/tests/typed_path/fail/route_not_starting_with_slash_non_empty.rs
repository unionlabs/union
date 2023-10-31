use axum_extra::routing::TypedPath;

#[derive(TypedPath)]
#[typed_path(":foo")]
struct MyPath;

fn main() {}
