use axum_macros::debug_handler;
use std::future::{Ready, ready};

#[debug_handler]
fn handler() -> Ready<()> {
    ready(())
}

fn main() {}
