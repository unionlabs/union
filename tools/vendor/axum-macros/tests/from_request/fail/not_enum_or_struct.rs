use axum_macros::FromRequest;

#[derive(FromRequest)]
union Extractor {}

fn main() {}
