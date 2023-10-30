use axum::extract::Path;
use axum_macros::debug_handler;

#[debug_handler]
async fn handler(_: Path<String>, _: Path<String>) {}

fn main() {}
