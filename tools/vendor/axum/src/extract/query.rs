use super::{rejection::*, FromRequest, RequestParts};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use std::ops::Deref;

/// Extractor that deserializes query strings into some type.
///
/// `T` is expected to implement [`serde::Deserialize`].
///
/// # Example
///
/// ```rust,no_run
/// use axum::{
///     extract::Query,
///     routing::get,
///     Router,
/// };
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Pagination {
///     page: usize,
///     per_page: usize,
/// }
///
/// // This will parse query strings like `?page=2&per_page=30` into `Pagination`
/// // structs.
/// async fn list_things(pagination: Query<Pagination>) {
///     let pagination: Pagination = pagination.0;
///
///     // ...
/// }
///
/// let app = Router::new().route("/list_things", get(list_things));
/// # async {
/// # axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
/// # };
/// ```
///
/// If the query string cannot be parsed it will reject the request with a `422
/// Unprocessable Entity` response.
///
/// For handling values being empty vs missing see the (query-params-with-empty-strings)[example]
/// example.
///
/// [example]: https://github.com/tokio-rs/axum/blob/main/examples/query-params-with-empty-strings/src/main.rs
#[cfg_attr(docsrs, doc(cfg(feature = "query")))]
#[derive(Debug, Clone, Copy, Default)]
pub struct Query<T>(pub T);

#[async_trait]
impl<T, B> FromRequest<B> for Query<T>
where
    T: DeserializeOwned,
    B: Send,
{
    type Rejection = QueryRejection;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let query = req.uri().query().unwrap_or_default();
        let value = serde_urlencoded::from_str(query)
            .map_err(FailedToDeserializeQueryString::__private_new::<(), _>)?;
        Ok(Query(value))
    }
}

impl<T> Deref for Query<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extract::RequestParts;
    use http::Request;
    use serde::Deserialize;
    use std::fmt::Debug;

    async fn check<T: DeserializeOwned + PartialEq + Debug>(uri: impl AsRef<str>, value: T) {
        let mut req = RequestParts::new(Request::builder().uri(uri.as_ref()).body(()).unwrap());
        assert_eq!(Query::<T>::from_request(&mut req).await.unwrap().0, value);
    }

    #[tokio::test]
    async fn test_query() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct Pagination {
            size: Option<u64>,
            page: Option<u64>,
        }

        check(
            "http://example.com/test",
            Pagination {
                size: None,
                page: None,
            },
        )
        .await;

        check(
            "http://example.com/test?size=10",
            Pagination {
                size: Some(10),
                page: None,
            },
        )
        .await;

        check(
            "http://example.com/test?size=10&page=20",
            Pagination {
                size: Some(10),
                page: Some(20),
            },
        )
        .await;
    }
}
