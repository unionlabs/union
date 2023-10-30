//! Split a single value implementing `AsyncRead + AsyncWrite` into separate
//! `AsyncRead` and `AsyncWrite` handles.
//!
//! To restore this read/write object from its `split::ReadHalf` and
//! `split::WriteHalf` use `unsplit`.

use crate::io::{AsyncRead, AsyncWrite, ReadBuf};

use std::cell::UnsafeCell;
use std::fmt;
use std::io;
use std::pin::Pin;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Release};
use std::sync::Arc;
use std::task::{Context, Poll};

cfg_io_util! {
    /// The readable half of a value returned from [`split`](split()).
    pub struct ReadHalf<T> {
        inner: Arc<Inner<T>>,
    }

    /// The writable half of a value returned from [`split`](split()).
    pub struct WriteHalf<T> {
        inner: Arc<Inner<T>>,
    }

    /// Splits a single value implementing `AsyncRead + AsyncWrite` into separate
    /// `AsyncRead` and `AsyncWrite` handles.
    ///
    /// To restore this read/write object from its `ReadHalf` and
    /// `WriteHalf` use [`unsplit`](ReadHalf::unsplit()).
    pub fn split<T>(stream: T) -> (ReadHalf<T>, WriteHalf<T>)
    where
        T: AsyncRead + AsyncWrite,
    {
        let inner = Arc::new(Inner {
            locked: AtomicBool::new(false),
            stream: UnsafeCell::new(stream),
        });

        let rd = ReadHalf {
            inner: inner.clone(),
        };

        let wr = WriteHalf { inner };

        (rd, wr)
    }
}

struct Inner<T> {
    locked: AtomicBool,
    stream: UnsafeCell<T>,
}

struct Guard<'a, T> {
    inner: &'a Inner<T>,
}

impl<T> ReadHalf<T> {
    /// Checks if this `ReadHalf` and some `WriteHalf` were split from the same
    /// stream.
    pub fn is_pair_of(&self, other: &WriteHalf<T>) -> bool {
        other.is_pair_of(self)
    }

    /// Reunites with a previously split `WriteHalf`.
    ///
    /// # Panics
    ///
    /// If this `ReadHalf` and the given `WriteHalf` do not originate from the
    /// same `split` operation this method will panic.
    /// This can be checked ahead of time by comparing the stream ID
    /// of the two halves.
    #[track_caller]
    pub fn unsplit(self, wr: WriteHalf<T>) -> T {
        if self.is_pair_of(&wr) {
            drop(wr);

            let inner = Arc::try_unwrap(self.inner)
                .ok()
                .expect("`Arc::try_unwrap` failed");

            inner.stream.into_inner()
        } else {
            panic!("Unrelated `split::Write` passed to `split::Read::unsplit`.")
        }
    }
}

impl<T> WriteHalf<T> {
    /// Checks if this `WriteHalf` and some `ReadHalf` were split from the same
    /// stream.
    pub fn is_pair_of(&self, other: &ReadHalf<T>) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl<T: AsyncRead> AsyncRead for ReadHalf<T> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let mut inner = ready!(self.inner.poll_lock(cx));
        inner.stream_pin().poll_read(cx, buf)
    }
}

impl<T: AsyncWrite> AsyncWrite for WriteHalf<T> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        let mut inner = ready!(self.inner.poll_lock(cx));
        inner.stream_pin().poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        let mut inner = ready!(self.inner.poll_lock(cx));
        inner.stream_pin().poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        let mut inner = ready!(self.inner.poll_lock(cx));
        inner.stream_pin().poll_shutdown(cx)
    }
}

impl<T> Inner<T> {
    fn poll_lock(&self, cx: &mut Context<'_>) -> Poll<Guard<'_, T>> {
        if self
            .locked
            .compare_exchange(false, true, Acquire, Acquire)
            .is_ok()
        {
            Poll::Ready(Guard { inner: self })
        } else {
            // Spin... but investigate a better strategy

            std::thread::yield_now();
            cx.waker().wake_by_ref();

            Poll::Pending
        }
    }
}

impl<T> Guard<'_, T> {
    fn stream_pin(&mut self) -> Pin<&mut T> {
        // safety: the stream is pinned in `Arc` and the `Guard` ensures mutual
        // exclusion.
        unsafe { Pin::new_unchecked(&mut *self.inner.stream.get()) }
    }
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        self.inner.locked.store(false, Release);
    }
}

unsafe impl<T: Send> Send for ReadHalf<T> {}
unsafe impl<T: Send> Send for WriteHalf<T> {}
unsafe impl<T: Sync> Sync for ReadHalf<T> {}
unsafe impl<T: Sync> Sync for WriteHalf<T> {}

impl<T: fmt::Debug> fmt::Debug for ReadHalf<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("split::ReadHalf").finish()
    }
}

impl<T: fmt::Debug> fmt::Debug for WriteHalf<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("split::WriteHalf").finish()
    }
}
