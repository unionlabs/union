use crate::stream::Fuse;
use alloc::vec::Vec;
use core::mem;
use core::pin::Pin;
use futures_core::stream::{FusedStream, Stream};
use futures_core::task::{Context, Poll};
#[cfg(feature = "sink")]
use futures_sink::Sink;
use pin_project_lite::pin_project;

pin_project! {
    /// Stream for the [`ready_chunks`](super::StreamExt::ready_chunks) method.
    #[derive(Debug)]
    #[must_use = "streams do nothing unless polled"]
    pub struct ReadyChunks<St: Stream> {
        #[pin]
        stream: Fuse<St>,
        items: Vec<St::Item>,
        cap: usize, // https://github.com/rust-lang/futures-rs/issues/1475
    }
}

impl<St: Stream> ReadyChunks<St>
where
    St: Stream,
{
    pub(super) fn new(stream: St, capacity: usize) -> Self {
        assert!(capacity > 0);

        Self {
            stream: super::Fuse::new(stream),
            items: Vec::with_capacity(capacity),
            cap: capacity,
        }
    }

    delegate_access_inner!(stream, St, (.));
}

impl<St: Stream> Stream for ReadyChunks<St> {
    type Item = Vec<St::Item>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            match this.stream.as_mut().poll_next(cx) {
                // Flush all collected data if underlying stream doesn't contain
                // more ready values
                Poll::Pending => {
                    return if this.items.is_empty() {
                        Poll::Pending
                    } else {
                        Poll::Ready(Some(mem::replace(this.items, Vec::with_capacity(*this.cap))))
                    }
                }

                // Push the ready item into the buffer and check whether it is full.
                // If so, replace our buffer with a new and empty one and return
                // the full one.
                Poll::Ready(Some(item)) => {
                    this.items.push(item);
                    if this.items.len() >= *this.cap {
                        return Poll::Ready(Some(mem::replace(
                            this.items,
                            Vec::with_capacity(*this.cap),
                        )));
                    }
                }

                // Since the underlying stream ran out of values, return what we
                // have buffered, if we have anything.
                Poll::Ready(None) => {
                    let last = if this.items.is_empty() {
                        None
                    } else {
                        let full_buf = mem::replace(this.items, Vec::new());
                        Some(full_buf)
                    };

                    return Poll::Ready(last);
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let chunk_len = if self.items.is_empty() { 0 } else { 1 };
        let (lower, upper) = self.stream.size_hint();
        let lower = lower.saturating_add(chunk_len);
        let upper = match upper {
            Some(x) => x.checked_add(chunk_len),
            None => None,
        };
        (lower, upper)
    }
}

impl<St: FusedStream> FusedStream for ReadyChunks<St> {
    fn is_terminated(&self) -> bool {
        self.stream.is_terminated() && self.items.is_empty()
    }
}

// Forwarding impl of Sink from the underlying stream
#[cfg(feature = "sink")]
impl<S, Item> Sink<Item> for ReadyChunks<S>
where
    S: Stream + Sink<Item>,
{
    type Error = S::Error;

    delegate_sink!(stream, Item);
}
