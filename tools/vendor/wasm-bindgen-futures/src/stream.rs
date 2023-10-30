//! Converting JavaScript `AsyncIterator`s to Rust `Stream`s.
//!
//! Analogous to the promise to future convertion, this module allows the
//! turing objects implementing the async iterator protocol into `Stream`s
//! that produce values that can be awaited from.
//!

use crate::JsFuture;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::stream::Stream;
use js_sys::{AsyncIterator, IteratorNext};
use wasm_bindgen::{prelude::*, JsCast};

/// A `Stream` that yields values from an underlying `AsyncIterator`.
pub struct JsStream {
    iter: AsyncIterator,
    next: Option<JsFuture>,
    done: bool,
}

impl JsStream {
    fn next_future(&self) -> Result<JsFuture, JsValue> {
        self.iter.next().map(JsFuture::from)
    }
}

impl From<AsyncIterator> for JsStream {
    fn from(iter: AsyncIterator) -> Self {
        JsStream {
            iter,
            next: None,
            done: false,
        }
    }
}

impl Stream for JsStream {
    type Item = Result<JsValue, JsValue>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        if self.done {
            return Poll::Ready(None);
        }

        let future = match self.next.as_mut() {
            Some(val) => val,
            None => match self.next_future() {
                Ok(val) => {
                    self.next = Some(val);
                    self.next.as_mut().unwrap()
                }
                Err(e) => {
                    self.done = true;
                    return Poll::Ready(Some(Err(e)));
                }
            },
        };

        match Pin::new(future).poll(cx) {
            Poll::Ready(res) => match res {
                Ok(iter_next) => {
                    let next = iter_next.unchecked_into::<IteratorNext>();
                    if next.done() {
                        self.done = true;
                        Poll::Ready(None)
                    } else {
                        self.next.take();
                        Poll::Ready(Some(Ok(next.value())))
                    }
                }
                Err(e) => {
                    self.done = true;
                    Poll::Ready(Some(Err(e)))
                }
            },
            Poll::Pending => Poll::Pending,
        }
    }
}
