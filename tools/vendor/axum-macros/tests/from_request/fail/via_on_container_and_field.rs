use axum_macros::FromRequest;
use axum::extract::Extension;

#[derive(FromRequest)]
#[from_request(via(Extension))]
struct Extractor(#[from_request(via(Extension))] State);

#[derive(Clone)]
struct State;

fn main() {}
