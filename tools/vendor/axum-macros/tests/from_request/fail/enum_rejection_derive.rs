use axum_macros::FromRequest;

#[derive(FromRequest, Clone)]
#[from_request(rejection_derive(!Error))]
enum Extractor {}

fn main() {}
