use axum_macros::FromRequest;
use axum::extract::Extension;

#[derive(FromRequest)]
struct Extractor(#[from_request(via(Extension), via(Extension))] State);

#[derive(Clone)]
struct State;

fn main() {}
