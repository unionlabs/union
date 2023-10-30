use axum_macros::FromRequest;

#[derive(FromRequest)]
#[from_request(rejection_derive(!Debug))]
struct Extractor {
    body: String,
}

fn main() {}
