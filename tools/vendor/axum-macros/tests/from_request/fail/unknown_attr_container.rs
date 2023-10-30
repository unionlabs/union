use axum_macros::FromRequest;

#[derive(FromRequest)]
#[from_request(foo)]
struct Extractor;

fn main() {}
