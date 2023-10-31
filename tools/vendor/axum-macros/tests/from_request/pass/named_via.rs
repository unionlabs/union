use axum::{
    body::Body,
    extract::{
        rejection::{ExtensionRejection, TypedHeaderRejection},
        Extension, FromRequest, TypedHeader,
    },
    headers::{self, UserAgent},
};
use axum_macros::FromRequest;
use std::convert::Infallible;

#[derive(FromRequest)]
struct Extractor {
    #[from_request(via(Extension))]
    state: State,
    #[from_request(via(TypedHeader))]
    user_agent: UserAgent,
    #[from_request(via(TypedHeader))]
    content_type: headers::ContentType,
    #[from_request(via(TypedHeader))]
    etag: Option<headers::ETag>,
    #[from_request(via(TypedHeader))]
    host: Result<headers::Host, TypedHeaderRejection>,
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
        ExtractorRejection::State(inner) => {
            let _: ExtensionRejection = inner;
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

#[derive(Clone)]
struct State;

fn main() {}
