use crate::body::{Bytes, HttpBody};
use crate::extract::{has_content_type, rejection::*, FromRequest, RequestParts};
use crate::BoxError;
use async_trait::async_trait;
use axum_core::response::{IntoResponse, Response};
use http::header::CONTENT_TYPE;
use http::{Method, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::ops::Deref;

/// URL encoded extractor and response.
///
/// # As extractor
///
/// If used as an extractor `Form` will deserialize `application/x-www-form-urlencoded` request
/// bodies into some target type via [`serde::Deserialize`].
///
/// ```rust
/// use axum::Form;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct SignUp {
///     username: String,
///     password: String,
/// }
///
/// async fn accept_form(Form(sign_up): Form<SignUp>) {
///     // ...
/// }
/// ```
///
/// Note that `Content-Type: multipart/form-data` requests are not supported. Use [`Multipart`]
/// instead.
///
/// # As response
///
/// ```rust
/// use axum::Form;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Payload {
///     value: String,
/// }
///
/// async fn handler() -> Form<Payload> {
///     Form(Payload { value: "foo".to_owned() })
/// }
/// ```
///
/// [`Multipart`]: crate::extract::Multipart
#[cfg_attr(docsrs, doc(cfg(feature = "form")))]
#[derive(Debug, Clone, Copy, Default)]
pub struct Form<T>(pub T);

#[async_trait]
impl<T, B> FromRequest<B> for Form<T>
where
    T: DeserializeOwned,
    B: HttpBody + Send,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = FormRejection;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        if req.method() == Method::GET {
            let query = req.uri().query().unwrap_or_default();
            let value = serde_urlencoded::from_str(query)
                .map_err(FailedToDeserializeQueryString::__private_new::<(), _>)?;
            Ok(Form(value))
        } else {
            if !has_content_type(req, &mime::APPLICATION_WWW_FORM_URLENCODED) {
                return Err(InvalidFormContentType.into());
            }

            let bytes = Bytes::from_request(req).await?;
            let value = serde_urlencoded::from_bytes(&bytes)
                .map_err(FailedToDeserializeQueryString::__private_new::<(), _>)?;

            Ok(Form(value))
        }
    }
}

impl<T> IntoResponse for Form<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        match serde_urlencoded::to_string(&self.0) {
            Ok(body) => (
                [(CONTENT_TYPE, mime::APPLICATION_WWW_FORM_URLENCODED.as_ref())],
                body,
            )
                .into_response(),
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
        }
    }
}

impl<T> Deref for Form<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::{Empty, Full};
    use crate::extract::RequestParts;
    use http::Request;
    use serde::{Deserialize, Serialize};
    use std::fmt::Debug;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Pagination {
        size: Option<u64>,
        page: Option<u64>,
    }

    async fn check_query<T: DeserializeOwned + PartialEq + Debug>(uri: impl AsRef<str>, value: T) {
        let mut req = RequestParts::new(
            Request::builder()
                .uri(uri.as_ref())
                .body(Empty::<Bytes>::new())
                .unwrap(),
        );
        assert_eq!(Form::<T>::from_request(&mut req).await.unwrap().0, value);
    }

    async fn check_body<T: Serialize + DeserializeOwned + PartialEq + Debug>(value: T) {
        let mut req = RequestParts::new(
            Request::builder()
                .uri("http://example.com/test")
                .method(Method::POST)
                .header(
                    http::header::CONTENT_TYPE,
                    mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
                )
                .body(Full::<Bytes>::new(
                    serde_urlencoded::to_string(&value).unwrap().into(),
                ))
                .unwrap(),
        );
        assert_eq!(Form::<T>::from_request(&mut req).await.unwrap().0, value);
    }

    #[tokio::test]
    async fn test_form_query() {
        check_query(
            "http://example.com/test",
            Pagination {
                size: None,
                page: None,
            },
        )
        .await;

        check_query(
            "http://example.com/test?size=10",
            Pagination {
                size: Some(10),
                page: None,
            },
        )
        .await;

        check_query(
            "http://example.com/test?size=10&page=20",
            Pagination {
                size: Some(10),
                page: Some(20),
            },
        )
        .await;
    }

    #[tokio::test]
    async fn test_form_body() {
        check_body(Pagination {
            size: None,
            page: None,
        })
        .await;

        check_body(Pagination {
            size: Some(10),
            page: None,
        })
        .await;

        check_body(Pagination {
            size: Some(10),
            page: Some(20),
        })
        .await;
    }

    #[tokio::test]
    async fn test_incorrect_content_type() {
        let mut req = RequestParts::new(
            Request::builder()
                .uri("http://example.com/test")
                .method(Method::POST)
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Full::<Bytes>::new(
                    serde_urlencoded::to_string(&Pagination {
                        size: Some(10),
                        page: None,
                    })
                    .unwrap()
                    .into(),
                ))
                .unwrap(),
        );
        assert!(matches!(
            Form::<Pagination>::from_request(&mut req)
                .await
                .unwrap_err(),
            FormRejection::InvalidFormContentType(InvalidFormContentType)
        ));
    }
}
