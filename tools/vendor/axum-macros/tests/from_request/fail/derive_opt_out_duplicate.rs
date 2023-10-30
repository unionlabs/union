use axum_macros::FromRequest;

#[derive(FromRequest)]
#[from_request(rejection_derive(!Error, !Error))]
struct Extractor {
    body: String,
}

fn main() {}
