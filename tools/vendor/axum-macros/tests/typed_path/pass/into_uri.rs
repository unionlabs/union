use axum_extra::routing::TypedPath;
use axum::http::Uri;
use serde::Deserialize;

#[derive(TypedPath, Deserialize)]
#[typed_path("/:id")]
struct Named {
    id: u32,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/:id")]
struct Unnamed(u32);

#[derive(TypedPath, Deserialize)]
#[typed_path("/")]
struct Unit;

fn main() {
    let _: Uri = Named { id: 1 }.to_uri();
    let _: Uri = Unnamed(1).to_uri();
    let _: Uri = Unit.to_uri();
}
