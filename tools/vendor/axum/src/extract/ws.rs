//! Handle WebSocket connections.
//!
//! # Example
//!
//! ```
//! use axum::{
//!     extract::ws::{WebSocketUpgrade, WebSocket},
//!     routing::get,
//!     response::{IntoResponse, Response},
//!     Router,
//! };
//!
//! let app = Router::new().route("/ws", get(handler));
//!
//! async fn handler(ws: WebSocketUpgrade) -> Response {
//!     ws.on_upgrade(handle_socket)
//! }
//!
//! async fn handle_socket(mut socket: WebSocket) {
//!     while let Some(msg) = socket.recv().await {
//!         let msg = if let Ok(msg) = msg {
//!             msg
//!         } else {
//!             // client disconnected
//!             return;
//!         };
//!
//!         if socket.send(msg).await.is_err() {
//!             // client disconnected
//!             return;
//!         }
//!     }
//! }
//! # async {
//! # axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
//! # };
//! ```
//!
//! # Passing data and/or state to an `on_upgrade` callback
//!
//! ```
//! use axum::{
//!     extract::ws::{WebSocketUpgrade, WebSocket},
//!     response::Response,
//!     routing::get,
//!     Extension, Router,
//! };
//!
//! #[derive(Clone)]
//! struct State {
//!     // ...
//! }
//!
//! async fn handler(ws: WebSocketUpgrade, Extension(state): Extension<State>) -> Response {
//!     ws.on_upgrade(|socket| handle_socket(socket, state))
//! }
//!
//! async fn handle_socket(socket: WebSocket, state: State) {
//!     // ...
//! }
//!
//! let app = Router::new()
//!     .route("/ws", get(handler))
//!     .layer(Extension(State { /* ... */ }));
//! # async {
//! # axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
//! # };
//! ```
//!
//! # Read and write concurrently
//!
//! If you need to read and write concurrently from a [`WebSocket`] you can use
//! [`StreamExt::split`]:
//!
//! ```
//! use axum::{Error, extract::ws::{WebSocket, Message}};
//! use futures::{sink::SinkExt, stream::{StreamExt, SplitSink, SplitStream}};
//!
//! async fn handle_socket(mut socket: WebSocket) {
//!     let (mut sender, mut receiver) = socket.split();
//!
//!     tokio::spawn(write(sender));
//!     tokio::spawn(read(receiver));
//! }
//!
//! async fn read(receiver: SplitStream<WebSocket>) {
//!     // ...
//! }
//!
//! async fn write(sender: SplitSink<WebSocket, Message>) {
//!     // ...
//! }
//! ```
//!
//! [`StreamExt::split`]: https://docs.rs/futures/0.3.17/futures/stream/trait.StreamExt.html#method.split

use self::rejection::*;
use super::{FromRequest, RequestParts};
use crate::{
    body::{self, Bytes},
    response::Response,
    Error,
};
use async_trait::async_trait;
use futures_util::{
    sink::{Sink, SinkExt},
    stream::{Stream, StreamExt},
};
use http::{
    header::{self, HeaderName, HeaderValue},
    Method, StatusCode,
};
use hyper::upgrade::{OnUpgrade, Upgraded};
use sha1::{Digest, Sha1};
use std::{
    borrow::Cow,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tokio_tungstenite::{
    tungstenite::{
        self as ts,
        protocol::{self, WebSocketConfig},
    },
    WebSocketStream,
};

/// Extractor for establishing WebSocket connections.
///
/// Note: This extractor requires the request method to be `GET` so it should
/// always be used with [`get`](crate::routing::get). Requests with other methods will be
/// rejected.
///
/// See the [module docs](self) for an example.
#[derive(Debug)]
#[cfg_attr(docsrs, doc(cfg(feature = "ws")))]
pub struct WebSocketUpgrade {
    config: WebSocketConfig,
    /// The chosen protocol sent in the `Sec-WebSocket-Protocol` header of the response.
    protocol: Option<HeaderValue>,
    sec_websocket_key: HeaderValue,
    on_upgrade: OnUpgrade,
    sec_websocket_protocol: Option<HeaderValue>,
}

impl WebSocketUpgrade {
    /// Set the size of the internal message send queue.
    pub fn max_send_queue(mut self, max: usize) -> Self {
        self.config.max_send_queue = Some(max);
        self
    }

    /// Set the maximum message size (defaults to 64 megabytes)
    pub fn max_message_size(mut self, max: usize) -> Self {
        self.config.max_message_size = Some(max);
        self
    }

    /// Set the maximum frame size (defaults to 16 megabytes)
    pub fn max_frame_size(mut self, max: usize) -> Self {
        self.config.max_frame_size = Some(max);
        self
    }

    /// Set the known protocols.
    ///
    /// If the protocol name specified by `Sec-WebSocket-Protocol` header
    /// to match any of them, the upgrade response will include `Sec-WebSocket-Protocol` header and
    /// return the protocol name.
    ///
    /// The protocols should be listed in decreasing order of preference: if the client offers
    /// multiple protocols that the server could support, the server will pick the first one in
    /// this list.
    ///
    /// # Examples
    ///
    /// ```
    /// use axum::{
    ///     extract::ws::{WebSocketUpgrade, WebSocket},
    ///     routing::get,
    ///     response::{IntoResponse, Response},
    ///     Router,
    /// };
    ///
    /// let app = Router::new().route("/ws", get(handler));
    ///
    /// async fn handler(ws: WebSocketUpgrade) -> Response {
    ///     ws.protocols(["graphql-ws", "graphql-transport-ws"])
    ///         .on_upgrade(|socket| async {
    ///             // ...
    ///         })
    /// }
    /// # async {
    /// # axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
    /// # };
    /// ```
    pub fn protocols<I>(mut self, protocols: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Cow<'static, str>>,
    {
        if let Some(req_protocols) = self
            .sec_websocket_protocol
            .as_ref()
            .and_then(|p| p.to_str().ok())
        {
            self.protocol = protocols
                .into_iter()
                // FIXME: This will often allocate a new `String` and so is less efficient than it
                // could be. But that can't be fixed without breaking changes to the public API.
                .map(Into::into)
                .find(|protocol| {
                    req_protocols
                        .split(',')
                        .any(|req_protocol| req_protocol.trim() == protocol)
                })
                .map(|protocol| match protocol {
                    Cow::Owned(s) => HeaderValue::from_str(&s).unwrap(),
                    Cow::Borrowed(s) => HeaderValue::from_static(s),
                });
        }

        self
    }

    /// Finalize upgrading the connection and call the provided callback with
    /// the stream.
    ///
    /// When using `WebSocketUpgrade`, the response produced by this method
    /// should be returned from the handler. See the [module docs](self) for an
    /// example.
    pub fn on_upgrade<F, Fut>(self, callback: F) -> Response
    where
        F: FnOnce(WebSocket) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let on_upgrade = self.on_upgrade;
        let config = self.config;

        let protocol = self.protocol.clone();

        tokio::spawn(async move {
            let upgraded = on_upgrade.await.expect("connection upgrade failed");
            let socket =
                WebSocketStream::from_raw_socket(upgraded, protocol::Role::Server, Some(config))
                    .await;
            let socket = WebSocket {
                inner: socket,
                protocol,
            };
            callback(socket).await;
        });

        #[allow(clippy::declare_interior_mutable_const)]
        const UPGRADE: HeaderValue = HeaderValue::from_static("upgrade");
        #[allow(clippy::declare_interior_mutable_const)]
        const WEBSOCKET: HeaderValue = HeaderValue::from_static("websocket");

        let mut builder = Response::builder()
            .status(StatusCode::SWITCHING_PROTOCOLS)
            .header(header::CONNECTION, UPGRADE)
            .header(header::UPGRADE, WEBSOCKET)
            .header(
                header::SEC_WEBSOCKET_ACCEPT,
                sign(self.sec_websocket_key.as_bytes()),
            );

        if let Some(protocol) = self.protocol {
            builder = builder.header(header::SEC_WEBSOCKET_PROTOCOL, protocol);
        }

        builder.body(body::boxed(body::Empty::new())).unwrap()
    }
}

#[async_trait]
impl<B> FromRequest<B> for WebSocketUpgrade
where
    B: Send,
{
    type Rejection = WebSocketUpgradeRejection;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        if req.method() != Method::GET {
            return Err(MethodNotGet.into());
        }

        if !header_contains(req, header::CONNECTION, "upgrade") {
            return Err(InvalidConnectionHeader.into());
        }

        if !header_eq(req, header::UPGRADE, "websocket") {
            return Err(InvalidUpgradeHeader.into());
        }

        if !header_eq(req, header::SEC_WEBSOCKET_VERSION, "13") {
            return Err(InvalidWebSocketVersionHeader.into());
        }

        let sec_websocket_key = req
            .headers_mut()
            .remove(header::SEC_WEBSOCKET_KEY)
            .ok_or(WebSocketKeyHeaderMissing)?;

        let on_upgrade = req
            .extensions_mut()
            .remove::<OnUpgrade>()
            .ok_or(ConnectionNotUpgradable)?;

        let sec_websocket_protocol = req.headers().get(header::SEC_WEBSOCKET_PROTOCOL).cloned();

        Ok(Self {
            config: Default::default(),
            protocol: None,
            sec_websocket_key,
            on_upgrade,
            sec_websocket_protocol,
        })
    }
}

fn header_eq<B>(req: &RequestParts<B>, key: HeaderName, value: &'static str) -> bool {
    if let Some(header) = req.headers().get(&key) {
        header.as_bytes().eq_ignore_ascii_case(value.as_bytes())
    } else {
        false
    }
}

fn header_contains<B>(req: &RequestParts<B>, key: HeaderName, value: &'static str) -> bool {
    let header = if let Some(header) = req.headers().get(&key) {
        header
    } else {
        return false;
    };

    if let Ok(header) = std::str::from_utf8(header.as_bytes()) {
        header.to_ascii_lowercase().contains(value)
    } else {
        false
    }
}

/// A stream of WebSocket messages.
#[derive(Debug)]
pub struct WebSocket {
    inner: WebSocketStream<Upgraded>,
    protocol: Option<HeaderValue>,
}

impl WebSocket {
    /// Receive another message.
    ///
    /// Returns `None` if the stream has closed.
    pub async fn recv(&mut self) -> Option<Result<Message, Error>> {
        self.next().await
    }

    /// Send a message.
    pub async fn send(&mut self, msg: Message) -> Result<(), Error> {
        self.inner
            .send(msg.into_tungstenite())
            .await
            .map_err(Error::new)
    }

    /// Gracefully close this WebSocket.
    pub async fn close(mut self) -> Result<(), Error> {
        self.inner.close(None).await.map_err(Error::new)
    }

    /// Return the selected WebSocket subprotocol, if one has been chosen.
    pub fn protocol(&self) -> Option<&HeaderValue> {
        self.protocol.as_ref()
    }
}

impl Stream for WebSocket {
    type Item = Result<Message, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match futures_util::ready!(self.inner.poll_next_unpin(cx)) {
                Some(Ok(msg)) => {
                    if let Some(msg) = Message::from_tungstenite(msg) {
                        return Poll::Ready(Some(Ok(msg)));
                    }
                }
                Some(Err(err)) => return Poll::Ready(Some(Err(Error::new(err)))),
                None => return Poll::Ready(None),
            }
        }
    }
}

impl Sink<Message> for WebSocket {
    type Error = Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.inner).poll_ready(cx).map_err(Error::new)
    }

    fn start_send(mut self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        Pin::new(&mut self.inner)
            .start_send(item.into_tungstenite())
            .map_err(Error::new)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.inner).poll_flush(cx).map_err(Error::new)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.inner).poll_close(cx).map_err(Error::new)
    }
}

/// Status code used to indicate why an endpoint is closing the WebSocket connection.
pub type CloseCode = u16;

/// A struct representing the close command.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CloseFrame<'t> {
    /// The reason as a code.
    pub code: CloseCode,
    /// The reason as text string.
    pub reason: Cow<'t, str>,
}

/// A WebSocket message.
//
// This code comes from https://github.com/snapview/tungstenite-rs/blob/master/src/protocol/message.rs and is under following license:
// Copyright (c) 2017 Alexey Galakhov
// Copyright (c) 2016 Jason Housley
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Message {
    /// A text WebSocket message
    Text(String),
    /// A binary WebSocket message
    Binary(Vec<u8>),
    /// A ping message with the specified payload
    ///
    /// The payload here must have a length less than 125 bytes.
    ///
    /// Ping messages will be automatically responded to by the server, so you do not have to worry
    /// about dealing with them yourself.
    Ping(Vec<u8>),
    /// A pong message with the specified payload
    ///
    /// The payload here must have a length less than 125 bytes.
    ///
    /// Pong messages will be automatically sent to the client if a ping message is received, so
    /// you do not have to worry about constructing them yourself unless you want to implement a
    /// [unidirectional heartbeat](https://tools.ietf.org/html/rfc6455#section-5.5.3).
    Pong(Vec<u8>),
    /// A close message with the optional close frame.
    Close(Option<CloseFrame<'static>>),
}

impl Message {
    fn into_tungstenite(self) -> ts::Message {
        match self {
            Self::Text(text) => ts::Message::Text(text),
            Self::Binary(binary) => ts::Message::Binary(binary),
            Self::Ping(ping) => ts::Message::Ping(ping),
            Self::Pong(pong) => ts::Message::Pong(pong),
            Self::Close(Some(close)) => ts::Message::Close(Some(ts::protocol::CloseFrame {
                code: ts::protocol::frame::coding::CloseCode::from(close.code),
                reason: close.reason,
            })),
            Self::Close(None) => ts::Message::Close(None),
        }
    }

    fn from_tungstenite(message: ts::Message) -> Option<Self> {
        match message {
            ts::Message::Text(text) => Some(Self::Text(text)),
            ts::Message::Binary(binary) => Some(Self::Binary(binary)),
            ts::Message::Ping(ping) => Some(Self::Ping(ping)),
            ts::Message::Pong(pong) => Some(Self::Pong(pong)),
            ts::Message::Close(Some(close)) => Some(Self::Close(Some(CloseFrame {
                code: close.code.into(),
                reason: close.reason,
            }))),
            ts::Message::Close(None) => Some(Self::Close(None)),
            // we can ignore `Frame` frames as recommended by the tungstenite maintainers
            // https://github.com/snapview/tungstenite-rs/issues/268
            ts::Message::Frame(_) => None,
        }
    }

    /// Consume the WebSocket and return it as binary data.
    pub fn into_data(self) -> Vec<u8> {
        match self {
            Self::Text(string) => string.into_bytes(),
            Self::Binary(data) | Self::Ping(data) | Self::Pong(data) => data,
            Self::Close(None) => Vec::new(),
            Self::Close(Some(frame)) => frame.reason.into_owned().into_bytes(),
        }
    }

    /// Attempt to consume the WebSocket message and convert it to a String.
    pub fn into_text(self) -> Result<String, Error> {
        match self {
            Self::Text(string) => Ok(string),
            Self::Binary(data) | Self::Ping(data) | Self::Pong(data) => Ok(String::from_utf8(data)
                .map_err(|err| err.utf8_error())
                .map_err(Error::new)?),
            Self::Close(None) => Ok(String::new()),
            Self::Close(Some(frame)) => Ok(frame.reason.into_owned()),
        }
    }

    /// Attempt to get a &str from the WebSocket message,
    /// this will try to convert binary data to utf8.
    pub fn to_text(&self) -> Result<&str, Error> {
        match *self {
            Self::Text(ref string) => Ok(string),
            Self::Binary(ref data) | Self::Ping(ref data) | Self::Pong(ref data) => {
                Ok(std::str::from_utf8(data).map_err(Error::new)?)
            }
            Self::Close(None) => Ok(""),
            Self::Close(Some(ref frame)) => Ok(&frame.reason),
        }
    }
}

impl From<Message> for Vec<u8> {
    fn from(msg: Message) -> Self {
        msg.into_data()
    }
}

fn sign(key: &[u8]) -> HeaderValue {
    let mut sha1 = Sha1::default();
    sha1.update(key);
    sha1.update(&b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11"[..]);
    let b64 = Bytes::from(base64::encode(&sha1.finalize()));
    HeaderValue::from_maybe_shared(b64).expect("base64 is a valid value")
}

pub mod rejection {
    //! WebSocket specific rejections.

    define_rejection! {
        #[status = METHOD_NOT_ALLOWED]
        #[body = "Request method must be `GET`"]
        /// Rejection type for [`WebSocketUpgrade`](super::WebSocketUpgrade).
        pub struct MethodNotGet;
    }

    define_rejection! {
        #[status = BAD_REQUEST]
        #[body = "Connection header did not include 'upgrade'"]
        /// Rejection type for [`WebSocketUpgrade`](super::WebSocketUpgrade).
        pub struct InvalidConnectionHeader;
    }

    define_rejection! {
        #[status = BAD_REQUEST]
        #[body = "`Upgrade` header did not include 'websocket'"]
        /// Rejection type for [`WebSocketUpgrade`](super::WebSocketUpgrade).
        pub struct InvalidUpgradeHeader;
    }

    define_rejection! {
        #[status = BAD_REQUEST]
        #[body = "`Sec-WebSocket-Version` header did not include '13'"]
        /// Rejection type for [`WebSocketUpgrade`](super::WebSocketUpgrade).
        pub struct InvalidWebSocketVersionHeader;
    }

    define_rejection! {
        #[status = BAD_REQUEST]
        #[body = "`Sec-WebSocket-Key` header missing"]
        /// Rejection type for [`WebSocketUpgrade`](super::WebSocketUpgrade).
        pub struct WebSocketKeyHeaderMissing;
    }

    define_rejection! {
        #[status = UPGRADE_REQUIRED]
        #[body = "WebSocket request couldn't be upgraded since no upgrade state was present"]
        /// Rejection type for [`WebSocketUpgrade`](super::WebSocketUpgrade).
        ///
        /// This rejection is returned if the connection cannot be upgraded for example if the
        /// request is HTTP/1.0.
        ///
        /// See [MDN] for more details about connection upgrades.
        ///
        /// [MDN]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Upgrade
        pub struct ConnectionNotUpgradable;
    }

    composite_rejection! {
        /// Rejection used for [`WebSocketUpgrade`](super::WebSocketUpgrade).
        ///
        /// Contains one variant for each way the [`WebSocketUpgrade`](super::WebSocketUpgrade)
        /// extractor can fail.
        pub enum WebSocketUpgradeRejection {
            MethodNotGet,
            InvalidConnectionHeader,
            InvalidUpgradeHeader,
            InvalidWebSocketVersionHeader,
            WebSocketKeyHeaderMissing,
            ConnectionNotUpgradable,
        }
    }
}

pub mod close_code {
    //! Constants for [`CloseCode`]s.
    //!
    //! [`CloseCode`]: super::CloseCode

    /// Indicates a normal closure, meaning that the purpose for which the connection was
    /// established has been fulfilled.
    pub const NORMAL: u16 = 1000;

    /// Indicates that an endpoint is "going away", such as a server going down or a browser having
    /// navigated away from a page.
    pub const AWAY: u16 = 1001;

    /// Indicates that an endpoint is terminating the connection due to a protocol error.
    pub const PROTOCOL: u16 = 1002;

    /// Indicates that an endpoint is terminating the connection because it has received a type of
    /// data it cannot accept (e.g., an endpoint that understands only text data MAY send this if
    /// it receives a binary message).
    pub const UNSUPPORTED: u16 = 1003;

    /// Indicates that no status code was included in a closing frame.
    pub const STATUS: u16 = 1005;

    /// Indicates an abnormal closure.
    pub const ABNORMAL: u16 = 1006;

    /// Indicates that an endpoint is terminating the connection because it has received data
    /// within a message that was not consistent with the type of the message (e.g., non-UTF-8
    /// RFC3629 data within a text message).
    pub const INVALID: u16 = 1007;

    /// Indicates that an endpoint is terminating the connection because it has received a message
    /// that violates its policy. This is a generic status code that can be returned when there is
    /// no other more suitable status code (e.g., `UNSUPPORTED` or `SIZE`) or if there is a need to
    /// hide specific details about the policy.
    pub const POLICY: u16 = 1008;

    /// Indicates that an endpoint is terminating the connection because it has received a message
    /// that is too big for it to process.
    pub const SIZE: u16 = 1009;

    /// Indicates that an endpoint (client) is terminating the connection because it has expected
    /// the server to negotiate one or more extension, but the server didn't return them in the
    /// response message of the WebSocket handshake. The list of extensions that are needed should
    /// be given as the reason for closing. Note that this status code is not used by the server,
    /// because it can fail the WebSocket handshake instead.
    pub const EXTENSION: u16 = 1010;

    /// Indicates that a server is terminating the connection because it encountered an unexpected
    /// condition that prevented it from fulfilling the request.
    pub const ERROR: u16 = 1011;

    /// Indicates that the server is restarting.
    pub const RESTART: u16 = 1012;

    /// Indicates that the server is overloaded and the client should either connect to a different
    /// IP (when multiple targets exist), or reconnect to the same IP when a user has performed an
    /// action.
    pub const AGAIN: u16 = 1013;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{body::Body, routing::get};
    use http::{Request, Version};
    use tower::ServiceExt;

    #[tokio::test]
    async fn rejects_http_1_0_requests() {
        let svc = get(|ws: Result<WebSocketUpgrade, WebSocketUpgradeRejection>| {
            let rejection = ws.unwrap_err();
            assert!(matches!(
                rejection,
                WebSocketUpgradeRejection::ConnectionNotUpgradable(_)
            ));
            std::future::ready(())
        });

        let req = Request::builder()
            .version(Version::HTTP_10)
            .method(Method::GET)
            .header("upgrade", "websocket")
            .header("connection", "Upgrade")
            .header("sec-websocket-key", "6D69KGBOr4Re+Nj6zx9aQA==")
            .header("sec-websocket-version", "13")
            .body(Body::empty())
            .unwrap();

        let res = svc.oneshot(req).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
    }
}
