//! Middleware that compresses response bodies.
//!
//! # Example
//!
//! Example showing how to respond with the compressed contents of a file.
//!
//! ```rust
//! use bytes::{Bytes, BytesMut};
//! use http::{Request, Response, header::ACCEPT_ENCODING};
//! use http_body::Body as _; // for Body::data
//! use hyper::Body;
//! use std::convert::Infallible;
//! use tokio::fs::{self, File};
//! use tokio_util::io::ReaderStream;
//! use tower::{Service, ServiceExt, ServiceBuilder, service_fn};
//! use tower_http::{compression::CompressionLayer, BoxError};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), BoxError> {
//! async fn handle(req: Request<Body>) -> Result<Response<Body>, Infallible> {
//!     // Open the file.
//!     let file = File::open("Cargo.toml").await.expect("file missing");
//!     // Convert the file into a `Stream`.
//!     let stream = ReaderStream::new(file);
//!     // Convert the `Stream` into a `Body`.
//!     let body = Body::wrap_stream(stream);
//!     // Create response.
//!     Ok(Response::new(body))
//! }
//!
//! let mut service = ServiceBuilder::new()
//!     // Compress responses based on the `Accept-Encoding` header.
//!     .layer(CompressionLayer::new())
//!     .service_fn(handle);
//!
//! // Call the service.
//! let request = Request::builder()
//!     .header(ACCEPT_ENCODING, "gzip")
//!     .body(Body::empty())?;
//!
//! let response = service
//!     .ready()
//!     .await?
//!     .call(request)
//!     .await?;
//!
//! assert_eq!(response.headers()["content-encoding"], "gzip");
//!
//! // Read the body
//! let mut body = response.into_body();
//! let mut bytes = BytesMut::new();
//! while let Some(chunk) = body.data().await {
//!     let chunk = chunk?;
//!     bytes.extend_from_slice(&chunk[..]);
//! }
//! let bytes: Bytes = bytes.freeze();
//!
//! // The compressed body should be smaller 🤞
//! let uncompressed_len = fs::read_to_string("Cargo.toml").await?.len();
//! assert!(bytes.len() < uncompressed_len);
//! #
//! # Ok(())
//! # }
//! ```
//!

pub mod predicate;

mod body;
mod future;
mod layer;
mod pin_project_cfg;
mod service;

#[doc(inline)]
pub use self::{
    body::CompressionBody,
    future::ResponseFuture,
    layer::CompressionLayer,
    predicate::{DefaultPredicate, Predicate},
    service::Compression,
};

#[cfg(test)]
mod tests {
    use super::*;
    use async_compression::tokio::write::{BrotliDecoder, BrotliEncoder};
    use bytes::BytesMut;
    use flate2::read::GzDecoder;
    use http_body::Body as _;
    use hyper::{Body, Error, Request, Response, Server};
    use std::sync::{Arc, RwLock};
    use std::{io::Read, net::SocketAddr};
    use tokio::io::AsyncWriteExt;
    use tower::{make::Shared, service_fn, Service, ServiceExt};

    // Compression filter allows every other request to be compressed
    #[derive(Clone)]
    struct Always;

    impl Predicate for Always {
        fn should_compress<B>(&self, _: &http::Response<B>) -> bool
        where
            B: http_body::Body,
        {
            true
        }
    }

    #[tokio::test]
    async fn works() {
        let svc = service_fn(handle);
        let mut svc = Compression::new(svc).compress_when(Always);

        // call the service
        let req = Request::builder()
            .header("accept-encoding", "gzip")
            .body(Body::empty())
            .unwrap();
        let res = svc.ready().await.unwrap().call(req).await.unwrap();

        // read the compressed body
        let mut body = res.into_body();
        let mut data = BytesMut::new();
        while let Some(chunk) = body.data().await {
            let chunk = chunk.unwrap();
            data.extend_from_slice(&chunk[..]);
        }
        let compressed_data = data.freeze().to_vec();

        // decompress the body
        // doing this with flate2 as that is much easier than async-compression and blocking during
        // tests is fine
        let mut decoder = GzDecoder::new(&compressed_data[..]);
        let mut decompressed = String::new();
        decoder.read_to_string(&mut decompressed).unwrap();

        assert_eq!(decompressed, "Hello, World!");
    }

    #[allow(dead_code)]
    async fn is_compatible_with_hyper() {
        let svc = service_fn(handle);
        let svc = Compression::new(svc);

        let make_service = Shared::new(svc);

        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
        let server = Server::bind(&addr).serve(make_service);
        server.await.unwrap();
    }

    #[tokio::test]
    async fn no_recompress() {
        const DATA: &str = "Hello, World! I'm already compressed with br!";

        let svc = service_fn(|_| async {
            let buf = {
                let mut buf = Vec::new();

                let mut enc = BrotliEncoder::new(&mut buf);
                enc.write_all(DATA.as_bytes()).await?;
                enc.flush().await?;
                buf
            };

            let resp = Response::builder()
                .header("content-encoding", "br")
                .body(Body::from(buf))
                .unwrap();
            Ok::<_, std::io::Error>(resp)
        });
        let mut svc = Compression::new(svc);

        // call the service
        //
        // note: the accept-encoding doesn't match the content-encoding above, so that
        // we're able to see if the compression layer triggered or not
        let req = Request::builder()
            .header("accept-encoding", "gzip")
            .body(Body::empty())
            .unwrap();
        let res = svc.ready().await.unwrap().call(req).await.unwrap();

        // check we didn't recompress
        assert_eq!(
            res.headers()
                .get("content-encoding")
                .and_then(|h| h.to_str().ok())
                .unwrap_or_default(),
            "br",
        );

        // read the compressed body
        let mut body = res.into_body();
        let mut data = BytesMut::new();
        while let Some(chunk) = body.data().await {
            let chunk = chunk.unwrap();
            data.extend_from_slice(&chunk[..]);
        }

        // decompress the body
        let data = {
            let mut output_buf = Vec::new();
            let mut decoder = BrotliDecoder::new(&mut output_buf);
            decoder
                .write_all(&data)
                .await
                .expect("couldn't brotli-decode");
            decoder.flush().await.expect("couldn't flush");
            output_buf
        };

        assert_eq!(data, DATA.as_bytes());
    }

    async fn handle(_req: Request<Body>) -> Result<Response<Body>, Error> {
        Ok(Response::new(Body::from("Hello, World!")))
    }

    #[tokio::test]
    async fn will_not_compress_if_filtered_out() {
        use predicate::Predicate;

        const DATA: &str = "Hello world uncompressed";

        let svc_fn = service_fn(|_| async {
            let resp = Response::builder()
                // .header("content-encoding", "br")
                .body(Body::from(DATA.as_bytes()))
                .unwrap();
            Ok::<_, std::io::Error>(resp)
        });

        // Compression filter allows every other request to be compressed
        #[derive(Default, Clone)]
        struct EveryOtherResponse(Arc<RwLock<u64>>);

        impl Predicate for EveryOtherResponse {
            fn should_compress<B>(&self, _: &http::Response<B>) -> bool
            where
                B: http_body::Body,
            {
                let mut guard = self.0.write().unwrap();
                let should_compress = *guard % 2 != 0;
                *guard += 1;
                dbg!(should_compress)
            }
        }

        let mut svc = Compression::new(svc_fn).compress_when(EveryOtherResponse::default());
        let req = Request::builder()
            .header("accept-encoding", "br")
            .body(Body::empty())
            .unwrap();
        let res = svc.ready().await.unwrap().call(req).await.unwrap();

        // read the uncompressed body
        let mut body = res.into_body();
        let mut data = BytesMut::new();
        while let Some(chunk) = body.data().await {
            let chunk = chunk.unwrap();
            data.extend_from_slice(&chunk[..]);
        }
        let still_uncompressed = String::from_utf8(data.to_vec()).unwrap();
        assert_eq!(DATA, &still_uncompressed);

        // Compression filter will compress the next body
        let req = Request::builder()
            .header("accept-encoding", "br")
            .body(Body::empty())
            .unwrap();
        let res = svc.ready().await.unwrap().call(req).await.unwrap();

        // read the compressed body
        let mut body = res.into_body();
        let mut data = BytesMut::new();
        while let Some(chunk) = body.data().await {
            let chunk = chunk.unwrap();
            data.extend_from_slice(&chunk[..]);
        }
        assert!(String::from_utf8(data.to_vec()).is_err());
    }
}
