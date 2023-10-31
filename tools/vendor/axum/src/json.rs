use crate::{
    body::{Bytes, HttpBody},
    extract::{rejection::*, FromRequest, RequestParts},
    BoxError,
};
use async_trait::async_trait;
use axum_core::response::{IntoResponse, Response};
use bytes::{BufMut, BytesMut};
use http::{
    header::{self, HeaderValue},
    StatusCode,
};
use serde::{de::DeserializeOwned, Serialize};
use std::ops::{Deref, DerefMut};

/// JSON Extractor / Response.
///
/// When used as an extractor, it can deserialize request bodies into some type that
/// implements [`serde::Deserialize`]. The request will be rejected (and a [`JsonRejection`] will
/// be returned) if:
///
/// - The request doesn't have a `Content-Type: application/json` (or similar) header.
/// - The body doesn't contain syntactically valid JSON.
/// - The body contains syntactically valid JSON but it couldn't be deserialized into the target
/// type.
/// - Buffering the request body fails.
///
/// See [`JsonRejection`] for more details.
///
/// # Extractor example
///
/// ```rust,no_run
/// use axum::{
///     extract,
///     routing::post,
///     Router,
/// };
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct CreateUser {
///     email: String,
///     password: String,
/// }
///
/// async fn create_user(extract::Json(payload): extract::Json<CreateUser>) {
///     // payload is a `CreateUser`
/// }
///
/// let app = Router::new().route("/users", post(create_user));
/// # async {
/// # axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
/// # };
/// ```
///
/// When used as a response, it can serialize any type that implements [`serde::Serialize`] to
/// `JSON`, and will automatically set `Content-Type: application/json` header.
///
/// # Response example
///
/// ```
/// use axum::{
///     extract::Path,
///     routing::get,
///     Router,
///     Json,
/// };
/// use serde::Serialize;
/// use uuid::Uuid;
///
/// #[derive(Serialize)]
/// struct User {
///     id: Uuid,
///     username: String,
/// }
///
/// async fn get_user(Path(user_id) : Path<Uuid>) -> Json<User> {
///     let user = find_user(user_id).await;
///     Json(user)
/// }
///
/// async fn find_user(user_id: Uuid) -> User {
///     // ...
///     # unimplemented!()
/// }
///
/// let app = Router::new().route("/users/:id", get(get_user));
/// # async {
/// # axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
/// # };
/// ```
#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
pub struct Json<T>(pub T);

#[async_trait]
impl<T, B> FromRequest<B> for Json<T>
where
    T: DeserializeOwned,
    B: HttpBody + Send,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = JsonRejection;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        if json_content_type(req) {
            let bytes = Bytes::from_request(req).await?;

            let value = match serde_json::from_slice(&bytes) {
                Ok(value) => value,
                Err(err) => {
                    let rejection = match err.classify() {
                        serde_json::error::Category::Data => JsonDataError::from_err(err).into(),
                        serde_json::error::Category::Syntax | serde_json::error::Category::Eof => {
                            JsonSyntaxError::from_err(err).into()
                        }
                        serde_json::error::Category::Io => {
                            if cfg!(debug_assertions) {
                                // we don't use `serde_json::from_reader` and instead always buffer
                                // bodies first, so we shouldn't encounter any IO errors
                                unreachable!()
                            } else {
                                JsonSyntaxError::from_err(err).into()
                            }
                        }
                    };
                    return Err(rejection);
                }
            };

            Ok(Json(value))
        } else {
            Err(MissingJsonContentType.into())
        }
    }
}

fn json_content_type<B>(req: &RequestParts<B>) -> bool {
    let content_type = if let Some(content_type) = req.headers().get(header::CONTENT_TYPE) {
        content_type
    } else {
        return false;
    };

    let content_type = if let Ok(content_type) = content_type.to_str() {
        content_type
    } else {
        return false;
    };

    let mime = if let Ok(mime) = content_type.parse::<mime::Mime>() {
        mime
    } else {
        return false;
    };

    let is_json_content_type = mime.type_() == "application"
        && (mime.subtype() == "json" || mime.suffix().map_or(false, |name| name == "json"));

    is_json_content_type
}

impl<T> Deref for Json<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Json<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for Json<T> {
    fn from(inner: T) -> Self {
        Self(inner)
    }
}

impl<T> IntoResponse for Json<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let mut buf = BytesMut::new().writer();
        match serde_json::to_writer(&mut buf, &self.0) {
            Ok(()) => (
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
                )],
                buf.into_inner().freeze(),
            )
                .into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
                )],
                err.to_string(),
            )
                .into_response(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{routing::post, test_helpers::*, Router};
    use serde::Deserialize;
    use serde_json::{json, Value};

    #[tokio::test]
    async fn deserialize_body() {
        #[derive(Debug, Deserialize)]
        struct Input {
            foo: String,
        }

        let app = Router::new().route("/", post(|input: Json<Input>| async { input.0.foo }));

        let client = TestClient::new(app);
        let res = client.post("/").json(&json!({ "foo": "bar" })).send().await;
        let body = res.text().await;

        assert_eq!(body, "bar");
    }

    #[tokio::test]
    async fn consume_body_to_json_requires_json_content_type() {
        #[derive(Debug, Deserialize)]
        struct Input {
            foo: String,
        }

        let app = Router::new().route("/", post(|input: Json<Input>| async { input.0.foo }));

        let client = TestClient::new(app);
        let res = client.post("/").body(r#"{ "foo": "bar" }"#).send().await;

        let status = res.status();
        dbg!(res.text().await);

        assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn json_content_types() {
        async fn valid_json_content_type(content_type: &str) -> bool {
            println!("testing {:?}", content_type);

            let app = Router::new().route("/", post(|Json(_): Json<Value>| async {}));

            let res = TestClient::new(app)
                .post("/")
                .header("content-type", content_type)
                .body("{}")
                .send()
                .await;

            res.status() == StatusCode::OK
        }

        assert!(valid_json_content_type("application/json").await);
        assert!(valid_json_content_type("application/json; charset=utf-8").await);
        assert!(valid_json_content_type("application/json;charset=utf-8").await);
        assert!(valid_json_content_type("application/cloudevents+json").await);
        assert!(!valid_json_content_type("text/json").await);
    }

    #[tokio::test]
    async fn invalid_json_syntax() {
        let app = Router::new().route("/", post(|_: Json<serde_json::Value>| async {}));

        let client = TestClient::new(app);
        let res = client
            .post("/")
            .body("{")
            .header("content-type", "application/json")
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }
}
