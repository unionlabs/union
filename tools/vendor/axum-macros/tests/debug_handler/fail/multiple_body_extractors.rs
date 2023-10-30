use axum_macros::debug_handler;
use axum::body::Bytes;

#[debug_handler]
async fn handler(_: String, _: Bytes) {}

fn main() {}
