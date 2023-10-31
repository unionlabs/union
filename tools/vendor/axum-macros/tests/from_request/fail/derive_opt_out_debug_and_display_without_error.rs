use axum_macros::FromRequest;

#[derive(FromRequest)]
#[from_request(rejection_derive(!Debug, !Display))]
struct Extractor {
    body: String,
}

fn main() {}
