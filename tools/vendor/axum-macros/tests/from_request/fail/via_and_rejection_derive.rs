use axum_macros::FromRequest;
use axum::extract::Extension;

#[derive(FromRequest, Clone)]
#[from_request(via(Extension), rejection_derive(!Error))]
struct Extractor {
    config: String,
}

fn main() {}
