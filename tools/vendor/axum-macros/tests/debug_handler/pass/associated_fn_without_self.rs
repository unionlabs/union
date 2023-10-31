use axum_macros::debug_handler;

struct A;

impl A {
    #[debug_handler]
    async fn handler() {}
}

fn main() {}
