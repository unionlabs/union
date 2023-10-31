use axum::{body::BoxBody, http::Request};
use axum_macros::debug_handler;

#[debug_handler(body = BoxBody)]
async fn handler(_: Request<BoxBody>) {}

#[debug_handler(body = axum::body::BoxBody,)]
async fn handler_with_trailing_comma_and_type_path(_: Request<axum::body::BoxBody>) {}

fn main() {}
