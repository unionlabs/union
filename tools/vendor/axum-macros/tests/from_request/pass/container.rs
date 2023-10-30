use axum::{
    body::Body,
    extract::{rejection::JsonRejection, FromRequest, Json},
};
use axum_macros::FromRequest;
use serde::Deserialize;

#[derive(Deserialize, FromRequest)]
#[from_request(via(Json))]
struct Extractor {
    one: i32,
    two: String,
    three: bool,
}

fn assert_from_request()
where
    Extractor: FromRequest<Body, Rejection = JsonRejection>,
{
}

fn main() {}
