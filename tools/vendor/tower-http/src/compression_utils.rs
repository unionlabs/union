//! Types used by compression and decompression middleware.

use crate::{content_encoding::SupportedEncodings, BoxError};
use bytes::{Bytes, BytesMut};
use futures_core::Stream;
use futures_util::ready;
use http::HeaderValue;
use http_body::Body;
use pin_project_lite::pin_project;
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::AsyncRead;
use tokio_util::io::{poll_read_buf, StreamReader};

#[derive(Debug, Clone, Copy)]
pub(crate) struct AcceptEncoding {
    pub(crate) gzip: bool,
    pub(crate) deflate: bool,
    pub(crate) br: bool,
}

impl AcceptEncoding {
    #[allow(dead_code)]
    pub(crate) fn to_header_value(self) -> Option<HeaderValue> {
        let accept = match (self.gzip(), self.deflate(), self.br()) {
            (true, true, true) => "gzip,deflate,br",
            (true, true, false) => "gzip,deflate",
            (true, false, true) => "gzip,br",
            (true, false, false) => "gzip",
            (false, true, true) => "deflate,br",
            (false, true, false) => "deflate",
            (false, false, true) => "br",
            (false, false, false) => return None,
        };
        Some(HeaderValue::from_static(accept))
    }

    #[allow(dead_code)]
    pub(crate) fn set_gzip(&mut self, enable: bool) {
        self.gzip = enable;
    }

    #[allow(dead_code)]
    pub(crate) fn set_deflate(&mut self, enable: bool) {
        self.deflate = enable;
    }

    #[allow(dead_code)]
    pub(crate) fn set_br(&mut self, enable: bool) {
        self.br = enable;
    }
}

impl SupportedEncodings for AcceptEncoding {
    #[allow(dead_code)]
    fn gzip(&self) -> bool {
        #[cfg(any(feature = "decompression-gzip", feature = "compression-gzip"))]
        {
            self.gzip
        }
        #[cfg(not(any(feature = "decompression-gzip", feature = "compression-gzip")))]
        {
            false
        }
    }

    #[allow(dead_code)]
    fn deflate(&self) -> bool {
        #[cfg(any(feature = "decompression-deflate", feature = "compression-deflate"))]
        {
            self.deflate
        }
        #[cfg(not(any(feature = "decompression-deflate", feature = "compression-deflate")))]
        {
            false
        }
    }

    #[allow(dead_code)]
    fn br(&self) -> bool {
        #[cfg(any(feature = "decompression-br", feature = "compression-br"))]
        {
            self.br
        }
        #[cfg(not(any(feature = "decompression-br", feature = "compression-br")))]
        {
            false
        }
    }
}

impl Default for AcceptEncoding {
    fn default() -> Self {
        AcceptEncoding {
            gzip: true,
            deflate: true,
            br: true,
        }
    }
}

/// A `Body` that has been converted into an `AsyncRead`.
pub(crate) type AsyncReadBody<B> =
    StreamReader<StreamErrorIntoIoError<BodyIntoStream<B>, <B as Body>::Error>, <B as Body>::Data>;

/// Trait for applying some decorator to an `AsyncRead`
pub(crate) trait DecorateAsyncRead {
    type Input: AsyncRead;
    type Output: AsyncRead;

    /// Apply the decorator
    fn apply(input: Self::Input) -> Self::Output;

    /// Get a pinned mutable reference to the original input.
    ///
    /// This is necessary to implement `Body::poll_trailers`.
    fn get_pin_mut(pinned: Pin<&mut Self::Output>) -> Pin<&mut Self::Input>;
}

pin_project! {
    /// `Body` that has been decorated by an `AsyncRead`
    pub(crate) struct WrapBody<M: DecorateAsyncRead> {
        #[pin]
        pub(crate) read: M::Output,
    }
}

impl<M: DecorateAsyncRead> WrapBody<M> {
    #[allow(dead_code)]
    pub(crate) fn new<B>(body: B) -> Self
    where
        B: Body,
        M: DecorateAsyncRead<Input = AsyncReadBody<B>>,
    {
        // convert `Body` into a `Stream`
        let stream = BodyIntoStream::new(body);

        // an adapter that converts the error type into `io::Error` while storing the actual error
        // `StreamReader` requires the error type is `io::Error`
        let stream = StreamErrorIntoIoError::<_, B::Error>::new(stream);

        // convert `Stream` into an `AsyncRead`
        let read = StreamReader::new(stream);

        // apply decorator to `AsyncRead` yieling another `AsyncRead`
        let read = M::apply(read);

        Self { read }
    }
}

impl<B, M> Body for WrapBody<M>
where
    B: Body,
    B::Error: Into<BoxError>,
    M: DecorateAsyncRead<Input = AsyncReadBody<B>>,
{
    type Data = Bytes;
    type Error = BoxError;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let mut this = self.project();
        let mut buf = BytesMut::new();

        let read = match ready!(poll_read_buf(this.read.as_mut(), cx, &mut buf)) {
            Ok(read) => read,
            Err(err) => {
                let body_error: Option<B::Error> = M::get_pin_mut(this.read)
                    .get_pin_mut()
                    .project()
                    .error
                    .take();

                if let Some(body_error) = body_error {
                    return Poll::Ready(Some(Err(body_error.into())));
                } else if err.raw_os_error() == Some(SENTINEL_ERROR_CODE) {
                    // SENTINEL_ERROR_CODE only gets used when storing an underlying body error
                    unreachable!()
                } else {
                    return Poll::Ready(Some(Err(err.into())));
                }
            }
        };

        if read == 0 {
            Poll::Ready(None)
        } else {
            Poll::Ready(Some(Ok(buf.freeze())))
        }
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<http::HeaderMap>, Self::Error>> {
        let this = self.project();
        let body = M::get_pin_mut(this.read)
            .get_pin_mut()
            .get_pin_mut()
            .get_pin_mut();
        body.poll_trailers(cx).map_err(Into::into)
    }
}

pin_project! {
    // When https://github.com/hyperium/http-body/pull/36 is merged we can remove this
    pub(crate) struct BodyIntoStream<B> {
        #[pin]
        body: B,
    }
}

#[allow(dead_code)]
impl<B> BodyIntoStream<B> {
    pub(crate) fn new(body: B) -> Self {
        Self { body }
    }

    /// Get a reference to the inner body
    pub(crate) fn get_ref(&self) -> &B {
        &self.body
    }

    /// Get a mutable reference to the inner body
    pub(crate) fn get_mut(&mut self) -> &mut B {
        &mut self.body
    }

    /// Get a pinned mutable reference to the inner body
    pub(crate) fn get_pin_mut(self: Pin<&mut Self>) -> Pin<&mut B> {
        self.project().body
    }

    /// Consume `self`, returning the inner body
    pub(crate) fn into_inner(self) -> B {
        self.body
    }
}

impl<B> Stream for BodyIntoStream<B>
where
    B: Body,
{
    type Item = Result<B::Data, B::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().body.poll_data(cx)
    }
}

pin_project! {
    pub(crate) struct StreamErrorIntoIoError<S, E> {
        #[pin]
        inner: S,
        error: Option<E>,
    }
}

impl<S, E> StreamErrorIntoIoError<S, E> {
    pub(crate) fn new(inner: S) -> Self {
        Self { inner, error: None }
    }

    /// Get a reference to the inner body
    pub(crate) fn get_ref(&self) -> &S {
        &self.inner
    }

    /// Get a mutable reference to the inner inner
    pub(crate) fn get_mut(&mut self) -> &mut S {
        &mut self.inner
    }

    /// Get a pinned mutable reference to the inner inner
    pub(crate) fn get_pin_mut(self: Pin<&mut Self>) -> Pin<&mut S> {
        self.project().inner
    }

    /// Consume `self`, returning the inner inner
    pub(crate) fn into_inner(self) -> S {
        self.inner
    }
}

impl<S, T, E> Stream for StreamErrorIntoIoError<S, E>
where
    S: Stream<Item = Result<T, E>>,
{
    type Item = Result<T, io::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match ready!(this.inner.poll_next(cx)) {
            None => Poll::Ready(None),
            Some(Ok(value)) => Poll::Ready(Some(Ok(value))),
            Some(Err(err)) => {
                *this.error = Some(err);
                Poll::Ready(Some(Err(io::Error::from_raw_os_error(SENTINEL_ERROR_CODE))))
            }
        }
    }
}

pub(crate) const SENTINEL_ERROR_CODE: i32 = -837459418;
