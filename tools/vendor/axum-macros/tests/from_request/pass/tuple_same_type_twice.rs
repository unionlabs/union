use axum::extract::{Query, rejection::*};
use axum_macros::FromRequest;
use serde::Deserialize;

#[derive(FromRequest)]
struct Extractor(
    Query<Payload>,
    axum::extract::Json<Payload>,
);

#[derive(Deserialize)]
struct Payload {}

fn assert_from_request()
where
    Extractor: axum::extract::FromRequest<axum::body::Body>,
{
}

fn assert_rejection(rejection: ExtractorRejection)
where
    ExtractorRejection: std::fmt::Debug + std::fmt::Display + std::error::Error,
{
    match rejection {
        ExtractorRejection::QueryPayload(inner) => {
            let _: QueryRejection = inner;
        }
        ExtractorRejection::JsonPayload(inner) => {
            let _: JsonRejection = inner;
        }
    }
}

fn main() {}
