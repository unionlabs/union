use axum_extra::routing::TypedPath;
use serde::Deserialize;

#[derive(TypedPath, Deserialize)]
#[typed_path("/:param")]
struct Named {
    param: String,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/:param")]
struct Unnamed(String);

fn main() {
    assert_eq!(
        format!(
            "{}",
            Named {
                param: "a b".to_string()
            }
        ),
        "/a%20b"
    );

    assert_eq!(
        format!(
            "{}",
            Unnamed("a b".to_string()),
        ),
        "/a%20b"
    );
}
