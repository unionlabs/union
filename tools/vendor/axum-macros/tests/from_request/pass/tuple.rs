use axum_macros::FromRequest;

#[derive(FromRequest)]
struct Extractor(axum::http::HeaderMap, String);

fn assert_from_request()
where
    Extractor: axum::extract::FromRequest<axum::body::Body>,
{
}

fn main() {}
