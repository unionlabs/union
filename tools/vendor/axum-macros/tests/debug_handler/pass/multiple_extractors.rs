use axum_macros::debug_handler;
use axum::http::{Method, Uri};

#[debug_handler]
async fn handler(_one: Method, _two: Uri, _three: String) {}

fn main() {}
