use axum_macros::FromRequest;
use axum::extract::Extension;

#[derive(FromRequest)]
struct Extractor(#[from_request(via(Extension))] State);

#[derive(Clone)]
struct State;

fn assert_from_request()
where
    Extractor: axum::extract::FromRequest<axum::body::Body>,
{
}

fn main() {}
