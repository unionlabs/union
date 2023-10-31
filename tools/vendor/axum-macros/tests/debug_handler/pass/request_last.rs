use axum::{body::Body, extract::Extension, http::Request};
use axum_macros::debug_handler;

#[debug_handler]
async fn handler(_: Extension<String>, _: Request<Body>) {}

fn main() {}
