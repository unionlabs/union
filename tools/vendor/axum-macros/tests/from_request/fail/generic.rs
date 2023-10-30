use axum_macros::FromRequest;

#[derive(FromRequest)]
struct Extractor<T>(Option<T>);

fn main() {}
