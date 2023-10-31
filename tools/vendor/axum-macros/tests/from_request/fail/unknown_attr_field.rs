use axum_macros::FromRequest;

#[derive(FromRequest)]
struct Extractor(#[from_request(foo)] String);

fn main() {}
