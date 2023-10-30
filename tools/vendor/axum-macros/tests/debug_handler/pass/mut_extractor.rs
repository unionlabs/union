use axum_macros::debug_handler;

#[debug_handler]
async fn handler(mut foo: String) -> String {
    foo += "bar";
    foo
}

fn main() {}
