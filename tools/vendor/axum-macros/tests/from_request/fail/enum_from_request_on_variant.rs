use axum_macros::FromRequest;

#[derive(FromRequest, Clone)]
#[from_request(via(axum::Extension))]
enum Extractor {
    #[from_request(via(axum::Extension))]
    Foo,
}

fn main() {}
