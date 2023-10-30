use axum_macros::debug_handler;

#[debug_handler]
async fn handler() -> bool {
    false
}

fn main() {}
