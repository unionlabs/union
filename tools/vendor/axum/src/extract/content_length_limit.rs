use super::{rejection::*, FromRequest, RequestParts};
use async_trait::async_trait;
use axum_core::response::IntoResponse;
use http::Method;
use std::ops::Deref;

/// Extractor that will reject requests with a body larger than some size.
///
/// `GET`, `HEAD`, and `OPTIONS` requests are rejected if they have a `Content-Length` header,
/// otherwise they're accepted without the body being checked.
///
/// # Example
///
/// ```rust,no_run
/// use axum::{
///     extract::ContentLengthLimit,
///     routing::post,
///     Router,
/// };
///
/// async fn handler(body: ContentLengthLimit<String, 1024>) {
///     // ...
/// }
///
/// let app = Router::new().route("/", post(handler));
/// # async {
/// # axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
/// # };
/// ```
///
/// This requires the request to have a `Content-Length` header.
///
/// If you want to limit the size of request bodies without requiring a `Content-Length` header,
/// consider using [`tower_http::limit::RequestBodyLimitLayer`].
#[derive(Debug, Clone)]
pub struct ContentLengthLimit<T, const N: u64>(pub T);

#[async_trait]
impl<T, B, const N: u64> FromRequest<B> for ContentLengthLimit<T, N>
where
    T: FromRequest<B>,
    T::Rejection: IntoResponse,
    B: Send,
{
    type Rejection = ContentLengthLimitRejection<T::Rejection>;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let content_length = req
            .headers()
            .get(http::header::CONTENT_LENGTH)
            .and_then(|value| value.to_str().ok()?.parse::<u64>().ok());

        match (content_length, req.method()) {
            (content_length, &(Method::GET | Method::HEAD | Method::OPTIONS)) => {
                if content_length.is_some() {
                    return Err(ContentLengthLimitRejection::ContentLengthNotAllowed(
                        ContentLengthNotAllowed,
                    ));
                } else if req
                    .headers()
                    .get(http::header::TRANSFER_ENCODING)
                    .map_or(false, |value| value.as_bytes() == b"chunked")
                {
                    return Err(ContentLengthLimitRejection::LengthRequired(LengthRequired));
                }
            }
            (Some(content_length), _) if content_length > N => {
                return Err(ContentLengthLimitRejection::PayloadTooLarge(
                    PayloadTooLarge,
                ));
            }
            (None, _) => {
                return Err(ContentLengthLimitRejection::LengthRequired(LengthRequired));
            }
            _ => {}
        }

        let value = T::from_request(req)
            .await
            .map_err(ContentLengthLimitRejection::Inner)?;

        Ok(Self(value))
    }
}

impl<T, const N: u64> Deref for ContentLengthLimit<T, N> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        body::Bytes,
        routing::{get, post},
        test_helpers::*,
        Router,
    };
    use http::StatusCode;
    use serde::Deserialize;

    #[tokio::test]
    async fn body_with_length_limit() {
        use std::iter::repeat;

        #[derive(Debug, Deserialize)]
        #[allow(dead_code)]
        struct Input {
            foo: String,
        }

        const LIMIT: u64 = 8;

        let app = Router::new().route(
            "/",
            post(|_body: ContentLengthLimit<Bytes, LIMIT>| async {}),
        );

        let client = TestClient::new(app);
        let res = client
            .post("/")
            .body(repeat(0_u8).take((LIMIT - 1) as usize).collect::<Vec<_>>())
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::OK);

        let res = client
            .post("/")
            .body(repeat(0_u8).take(LIMIT as usize).collect::<Vec<_>>())
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::OK);

        let res = client
            .post("/")
            .body(repeat(0_u8).take((LIMIT + 1) as usize).collect::<Vec<_>>())
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::PAYLOAD_TOO_LARGE);

        let res = client
            .post("/")
            .body(reqwest::Body::wrap_stream(futures_util::stream::iter(
                vec![Ok::<_, std::io::Error>(Bytes::new())],
            )))
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::LENGTH_REQUIRED);
    }

    #[tokio::test]
    async fn get_request_without_content_length_is_accepted() {
        let app = Router::new().route("/", get(|_body: ContentLengthLimit<Bytes, 1337>| async {}));

        let client = TestClient::new(app);

        let res = client.get("/").send().await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn get_request_with_content_length_is_rejected() {
        let app = Router::new().route("/", get(|_body: ContentLengthLimit<Bytes, 1337>| async {}));

        let client = TestClient::new(app);

        let res = client
            .get("/")
            .header("content-length", 3)
            .body("foo")
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn get_request_with_chunked_encoding_is_rejected() {
        let app = Router::new().route("/", get(|_body: ContentLengthLimit<Bytes, 1337>| async {}));

        let client = TestClient::new(app);

        let res = client
            .get("/")
            .header("transfer-encoding", "chunked")
            .body("3\r\nfoo\r\n0\r\n\r\n")
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::LENGTH_REQUIRED);
    }
}
