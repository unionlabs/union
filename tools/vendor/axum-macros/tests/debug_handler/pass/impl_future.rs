use axum_macros::debug_handler;
use std::future::Future;

#[debug_handler]
fn handler() -> impl Future<Output = ()> {
    async {}
}

fn main() {}
