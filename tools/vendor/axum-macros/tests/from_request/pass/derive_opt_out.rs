use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
    response::{IntoResponse, Response},
};
use axum_macros::FromRequest;

#[derive(FromRequest)]
#[from_request(rejection_derive(!Display, !Error))]
struct Extractor {
    other: OtherExtractor,
}

struct OtherExtractor;

#[async_trait]
impl<B> FromRequest<B> for OtherExtractor
where
    B: Send + 'static,
{
    type Rejection = OtherExtractorRejection;

    async fn from_request(_req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        unimplemented!()
    }
}

#[derive(Debug)]
struct OtherExtractorRejection;

impl IntoResponse for OtherExtractorRejection {
    fn into_response(self) -> Response {
        unimplemented!()
    }
}

fn main() {}
