use axum_macros::FromRequest;

#[derive(FromRequest, Clone)]
enum Extractor {}

fn main() {}
