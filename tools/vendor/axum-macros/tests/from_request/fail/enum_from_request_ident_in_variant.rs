use axum_macros::FromRequest;

#[derive(FromRequest, Clone)]
#[from_request(via(axum::Extension))]
enum Extractor {
    Foo {
        #[from_request(via(axum::Extension))]
        foo: (),
    }
}

fn main() {}
