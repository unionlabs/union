use core::fmt;
use core::pin::Pin;
use futures_core::future::{FusedFuture, Future};
use futures_core::ready;
use futures_core::stream::Stream;
use futures_core::task::{Context, Poll};
use pin_project_lite::pin_project;

pin_project! {
    /// Future for the [`any`](super::StreamExt::any) method.
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct Any<St, Fut, F> {
        #[pin]
        stream: St,
        f: F,
        accum: Option<bool>,
        #[pin]
        future: Option<Fut>,
    }
}

impl<St, Fut, F> fmt::Debug for Any<St, Fut, F>
where
    St: fmt::Debug,
    Fut: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Any")
            .field("stream", &self.stream)
            .field("accum", &self.accum)
            .field("future", &self.future)
            .finish()
    }
}

impl<St, Fut, F> Any<St, Fut, F>
where
    St: Stream,
    F: FnMut(St::Item) -> Fut,
    Fut: Future<Output = bool>,
{
    pub(super) fn new(stream: St, f: F) -> Self {
        Self { stream, f, accum: Some(false), future: None }
    }
}

impl<St, Fut, F> FusedFuture for Any<St, Fut, F>
where
    St: Stream,
    F: FnMut(St::Item) -> Fut,
    Fut: Future<Output = bool>,
{
    fn is_terminated(&self) -> bool {
        self.accum.is_none() && self.future.is_none()
    }
}

impl<St, Fut, F> Future for Any<St, Fut, F>
where
    St: Stream,
    F: FnMut(St::Item) -> Fut,
    Fut: Future<Output = bool>,
{
    type Output = bool;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<bool> {
        let mut this = self.project();
        Poll::Ready(loop {
            if let Some(fut) = this.future.as_mut().as_pin_mut() {
                // we're currently processing a future to produce a new accum value
                let acc = this.accum.unwrap() || ready!(fut.poll(cx));
                if acc {
                    break true;
                } // early exit
                *this.accum = Some(acc);
                this.future.set(None);
            } else if this.accum.is_some() {
                // we're waiting on a new item from the stream
                match ready!(this.stream.as_mut().poll_next(cx)) {
                    Some(item) => {
                        this.future.set(Some((this.f)(item)));
                    }
                    None => {
                        break this.accum.take().unwrap();
                    }
                }
            } else {
                panic!("Any polled after completion")
            }
        })
    }
}
