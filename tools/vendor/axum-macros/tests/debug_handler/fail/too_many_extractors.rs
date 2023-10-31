use axum_macros::debug_handler;
use axum::http::Uri;

#[debug_handler]
async fn handler(
    e1: Uri,
    e2: Uri,
    e3: Uri,
    e4: Uri,
    e5: Uri,
    e6: Uri,
    e7: Uri,
    e8: Uri,
    e9: Uri,
    e10: Uri,
    e11: Uri,
    e12: Uri,
    e13: Uri,
    e14: Uri,
    e15: Uri,
    e16: Uri,
    e17: Uri,
) {}

fn main() {}
