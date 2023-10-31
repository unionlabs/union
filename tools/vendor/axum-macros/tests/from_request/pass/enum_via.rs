use axum::{body::Body, routing::get, Extension, Router};
use axum_macros::FromRequest;

#[derive(FromRequest, Clone)]
#[from_request(via(Extension))]
enum Extractor {}

async fn foo(_: Extractor) {}

fn main() {
    Router::<Body>::new().route("/", get(foo));
}
