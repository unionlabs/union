use axum_macros::debug_handler;

#[debug_handler(foo)]
async fn handler() {}

fn main() {}
