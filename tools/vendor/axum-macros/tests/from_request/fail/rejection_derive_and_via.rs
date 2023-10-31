use axum_macros::FromRequest;
use axum::extract::Extension;

#[derive(FromRequest, Clone)]
#[from_request(rejection_derive(!Error), via(Extension))]
struct Extractor {
    config: String,
}

fn main() {}
