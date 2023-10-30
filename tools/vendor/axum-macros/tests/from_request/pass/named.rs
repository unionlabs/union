use axum::{
    body::Body,
    extract::{FromRequest, TypedHeader, rejection::{TypedHeaderRejection, StringRejection}},
    headers::{self, UserAgent},
};
use axum_macros::FromRequest;
use std::convert::Infallible;

#[derive(FromRequest)]
struct Extractor {
    uri: axum::http::Uri,
    user_agent: TypedHeader<UserAgent>,
    content_type: TypedHeader<headers::ContentType>,
    etag: Option<TypedHeader<headers::ETag>>,
    host: Result<TypedHeader<headers::Host>, TypedHeaderRejection>,
    body: String,
}

fn assert_from_request()
where
    Extractor: FromRequest<Body, Rejection = ExtractorRejection>,
{
}

fn assert_rejection(rejection: ExtractorRejection)
where
    ExtractorRejection: std::fmt::Debug + std::fmt::Display + std::error::Error,
{
    match rejection {
        ExtractorRejection::Uri(inner) => {
            let _: Infallible = inner;
        }
        ExtractorRejection::Body(inner) => {
            let _: StringRejection = inner;
        }
        ExtractorRejection::UserAgent(inner) => {
            let _: TypedHeaderRejection = inner;
        }
        ExtractorRejection::ContentType(inner) => {
            let _: TypedHeaderRejection = inner;
        }
        ExtractorRejection::Etag(inner) => {
            let _: Infallible = inner;
        }
        ExtractorRejection::Host(inner) => {
            let _: Infallible = inner;
        }
    }
}

fn main() {}
