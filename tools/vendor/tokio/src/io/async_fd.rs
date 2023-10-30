use crate::io::driver::{Handle, Interest, ReadyEvent, Registration};

use mio::unix::SourceFd;
use std::io;
use std::os::unix::io::{AsRawFd, RawFd};
use std::{task::Context, task::Poll};

/// Associates an IO object backed by a Unix file descriptor with the tokio
/// reactor, allowing for readiness to be polled. The file descriptor must be of
/// a type that can be used with the OS polling facilities (ie, `poll`, `epoll`,
/// `kqueue`, etc), such as a network socket or pipe, and the file descriptor
/// must have the nonblocking mode set to true.
///
/// Creating an AsyncFd registers the file descriptor with the current tokio
/// Reactor, allowing you to directly await the file descriptor being readable
/// or writable. Once registered, the file descriptor remains registered until
/// the AsyncFd is dropped.
///
/// The AsyncFd takes ownership of an arbitrary object to represent the IO
/// object. It is intended that this object will handle closing the file
/// descriptor when it is dropped, avoiding resource leaks and ensuring that the
/// AsyncFd can clean up the registration before closing the file descriptor.
/// The [`AsyncFd::into_inner`] function can be used to extract the inner object
/// to retake control from the tokio IO reactor.
///
/// The inner object is required to implement [`AsRawFd`]. This file descriptor
/// must not change while [`AsyncFd`] owns the inner object, i.e. the
/// [`AsRawFd::as_raw_fd`] method on the inner type must always return the same
/// file descriptor when called multiple times. Failure to uphold this results
/// in unspecified behavior in the IO driver, which may include breaking
/// notifications for other sockets/etc.
///
/// Polling for readiness is done by calling the async functions [`readable`]
/// and [`writable`]. These functions complete when the associated readiness
/// condition is observed. Any number of tasks can query the same `AsyncFd` in
/// parallel, on the same or different conditions.
///
/// On some platforms, the readiness detecting mechanism relies on
/// edge-triggered notifications. This means that the OS will only notify Tokio
/// when the file descriptor transitions from not-ready to ready. For this to
/// work you should first try to read or write and only poll for readiness
/// if that fails with an error of [`std::io::ErrorKind::WouldBlock`].
///
/// Tokio internally tracks when it has received a ready notification, and when
/// readiness checking functions like [`readable`] and [`writable`] are called,
/// if the readiness flag is set, these async functions will complete
/// immediately. This however does mean that it is critical to ensure that this
/// ready flag is cleared when (and only when) the file descriptor ceases to be
/// ready. The [`AsyncFdReadyGuard`] returned from readiness checking functions
/// serves this function; after calling a readiness-checking async function,
/// you must use this [`AsyncFdReadyGuard`] to signal to tokio whether the file
/// descriptor is no longer in a ready state.
///
/// ## Use with to a poll-based API
///
/// In some cases it may be desirable to use `AsyncFd` from APIs similar to
/// [`TcpStream::poll_read_ready`]. The [`AsyncFd::poll_read_ready`] and
/// [`AsyncFd::poll_write_ready`] functions are provided for this purpose.
/// Because these functions don't create a future to hold their state, they have
/// the limitation that only one task can wait on each direction (read or write)
/// at a time.
///
/// # Examples
///
/// This example shows how to turn [`std::net::TcpStream`] asynchronous using
/// `AsyncFd`.  It implements `read` as an async fn, and `AsyncWrite` as a trait
/// to show how to implement both approaches.
///
/// ```no_run
/// use futures::ready;
/// use std::io::{self, Read, Write};
/// use std::net::TcpStream;
/// use std::pin::Pin;
/// use std::task::{Context, Poll};
/// use tokio::io::AsyncWrite;
/// use tokio::io::unix::AsyncFd;
///
/// pub struct AsyncTcpStream {
///     inner: AsyncFd<TcpStream>,
/// }
///
/// impl AsyncTcpStream {
///     pub fn new(tcp: TcpStream) -> io::Result<Self> {
///         tcp.set_nonblocking(true)?;
///         Ok(Self {
///             inner: AsyncFd::new(tcp)?,
///         })
///     }
///
///     pub async fn read(&self, out: &mut [u8]) -> io::Result<usize> {
///         loop {
///             let mut guard = self.inner.readable().await?;
///
///             match guard.try_io(|inner| inner.get_ref().read(out)) {
///                 Ok(result) => return result,
///                 Err(_would_block) => continue,
///             }
///         }
///     }
/// }
///
/// impl AsyncWrite for AsyncTcpStream {
///     fn poll_write(
///         self: Pin<&mut Self>,
///         cx: &mut Context<'_>,
///         buf: &[u8]
///     ) -> Poll<io::Result<usize>> {
///         loop {
///             let mut guard = ready!(self.inner.poll_write_ready(cx))?;
///
///             match guard.try_io(|inner| inner.get_ref().write(buf)) {
///                 Ok(result) => return Poll::Ready(result),
///                 Err(_would_block) => continue,
///             }
///         }
///     }
///
///     fn poll_flush(
///         self: Pin<&mut Self>,
///         cx: &mut Context<'_>,
///     ) -> Poll<io::Result<()>> {
///         // tcp flush is a no-op
///         Poll::Ready(Ok(()))
///     }
///
///     fn poll_shutdown(
///         self: Pin<&mut Self>,
///         cx: &mut Context<'_>,
///     ) -> Poll<io::Result<()>> {
///         self.inner.get_ref().shutdown(std::net::Shutdown::Write)?;
///         Poll::Ready(Ok(()))
///     }
/// }
/// ```
///
/// [`readable`]: method@Self::readable
/// [`writable`]: method@Self::writable
/// [`AsyncFdReadyGuard`]: struct@self::AsyncFdReadyGuard
/// [`TcpStream::poll_read_ready`]: struct@crate::net::TcpStream
pub struct AsyncFd<T: AsRawFd> {
    registration: Registration,
    inner: Option<T>,
}

/// Represents an IO-ready event detected on a particular file descriptor that
/// has not yet been acknowledged. This is a `must_use` structure to help ensure
/// that you do not forget to explicitly clear (or not clear) the event.
///
/// This type exposes an immutable reference to the underlying IO object.
#[must_use = "You must explicitly choose whether to clear the readiness state by calling a method on ReadyGuard"]
pub struct AsyncFdReadyGuard<'a, T: AsRawFd> {
    async_fd: &'a AsyncFd<T>,
    event: Option<ReadyEvent>,
}

/// Represents an IO-ready event detected on a particular file descriptor that
/// has not yet been acknowledged. This is a `must_use` structure to help ensure
/// that you do not forget to explicitly clear (or not clear) the event.
///
/// This type exposes a mutable reference to the underlying IO object.
#[must_use = "You must explicitly choose whether to clear the readiness state by calling a method on ReadyGuard"]
pub struct AsyncFdReadyMutGuard<'a, T: AsRawFd> {
    async_fd: &'a mut AsyncFd<T>,
    event: Option<ReadyEvent>,
}

const ALL_INTEREST: Interest = Interest::READABLE.add(Interest::WRITABLE);

impl<T: AsRawFd> AsyncFd<T> {
    /// Creates an AsyncFd backed by (and taking ownership of) an object
    /// implementing [`AsRawFd`]. The backing file descriptor is cached at the
    /// time of creation.
    ///
    /// This method must be called in the context of a tokio runtime.
    ///
    /// # Panics
    ///
    /// This function panics if there is no current reactor set, or if the `rt`
    /// feature flag is not enabled.
    #[inline]
    #[track_caller]
    pub fn new(inner: T) -> io::Result<Self>
    where
        T: AsRawFd,
    {
        Self::with_interest(inner, ALL_INTEREST)
    }

    /// Creates new instance as `new` with additional ability to customize interest,
    /// allowing to specify whether file descriptor will be polled for read, write or both.
    ///
    /// # Panics
    ///
    /// This function panics if there is no current reactor set, or if the `rt`
    /// feature flag is not enabled.
    #[inline]
    #[track_caller]
    pub fn with_interest(inner: T, interest: Interest) -> io::Result<Self>
    where
        T: AsRawFd,
    {
        Self::new_with_handle_and_interest(inner, Handle::current(), interest)
    }

    pub(crate) fn new_with_handle_and_interest(
        inner: T,
        handle: Handle,
        interest: Interest,
    ) -> io::Result<Self> {
        let fd = inner.as_raw_fd();

        let registration =
            Registration::new_with_interest_and_handle(&mut SourceFd(&fd), interest, handle)?;

        Ok(AsyncFd {
            registration,
            inner: Some(inner),
        })
    }

    /// Returns a shared reference to the backing object of this [`AsyncFd`].
    #[inline]
    pub fn get_ref(&self) -> &T {
        self.inner.as_ref().unwrap()
    }

    /// Returns a mutable reference to the backing object of this [`AsyncFd`].
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        self.inner.as_mut().unwrap()
    }

    fn take_inner(&mut self) -> Option<T> {
        let fd = self.inner.as_ref().map(AsRawFd::as_raw_fd);

        if let Some(fd) = fd {
            let _ = self.registration.deregister(&mut SourceFd(&fd));
        }

        self.inner.take()
    }

    /// Deregisters this file descriptor and returns ownership of the backing
    /// object.
    pub fn into_inner(mut self) -> T {
        self.take_inner().unwrap()
    }

    /// Polls for read readiness.
    ///
    /// If the file descriptor is not currently ready for reading, this method
    /// will store a clone of the [`Waker`] from the provided [`Context`]. When the
    /// file descriptor becomes ready for reading, [`Waker::wake`] will be called.
    ///
    /// Note that on multiple calls to [`poll_read_ready`] or
    /// [`poll_read_ready_mut`], only the `Waker` from the `Context` passed to the
    /// most recent call is scheduled to receive a wakeup. (However,
    /// [`poll_write_ready`] retains a second, independent waker).
    ///
    /// This method is intended for cases where creating and pinning a future
    /// via [`readable`] is not feasible. Where possible, using [`readable`] is
    /// preferred, as this supports polling from multiple tasks at once.
    ///
    /// This method takes `&self`, so it is possible to call this method
    /// concurrently with other methods on this struct. This method only
    /// provides shared access to the inner IO resource when handling the
    /// [`AsyncFdReadyGuard`].
    ///
    /// [`poll_read_ready`]: method@Self::poll_read_ready
    /// [`poll_read_ready_mut`]: method@Self::poll_read_ready_mut
    /// [`poll_write_ready`]: method@Self::poll_write_ready
    /// [`readable`]: method@Self::readable
    /// [`Context`]: struct@std::task::Context
    /// [`Waker`]: struct@std::task::Waker
    /// [`Waker::wake`]: method@std::task::Waker::wake
    pub fn poll_read_ready<'a>(
        &'a self,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<AsyncFdReadyGuard<'a, T>>> {
        let event = ready!(self.registration.poll_read_ready(cx))?;

        Ok(AsyncFdReadyGuard {
            async_fd: self,
            event: Some(event),
        })
        .into()
    }

    /// Polls for read readiness.
    ///
    /// If the file descriptor is not currently ready for reading, this method
    /// will store a clone of the [`Waker`] from the provided [`Context`]. When the
    /// file descriptor becomes ready for reading, [`Waker::wake`] will be called.
    ///
    /// Note that on multiple calls to [`poll_read_ready`] or
    /// [`poll_read_ready_mut`], only the `Waker` from the `Context` passed to the
    /// most recent call is scheduled to receive a wakeup. (However,
    /// [`poll_write_ready`] retains a second, independent waker).
    ///
    /// This method is intended for cases where creating and pinning a future
    /// via [`readable`] is not feasible. Where possible, using [`readable`] is
    /// preferred, as this supports polling from multiple tasks at once.
    ///
    /// This method takes `&mut self`, so it is possible to access the inner IO
    /// resource mutably when handling the [`AsyncFdReadyMutGuard`].
    ///
    /// [`poll_read_ready`]: method@Self::poll_read_ready
    /// [`poll_read_ready_mut`]: method@Self::poll_read_ready_mut
    /// [`poll_write_ready`]: method@Self::poll_write_ready
    /// [`readable`]: method@Self::readable
    /// [`Context`]: struct@std::task::Context
    /// [`Waker`]: struct@std::task::Waker
    /// [`Waker::wake`]: method@std::task::Waker::wake
    pub fn poll_read_ready_mut<'a>(
        &'a mut self,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<AsyncFdReadyMutGuard<'a, T>>> {
        let event = ready!(self.registration.poll_read_ready(cx))?;

        Ok(AsyncFdReadyMutGuard {
            async_fd: self,
            event: Some(event),
        })
        .into()
    }

    /// Polls for write readiness.
    ///
    /// If the file descriptor is not currently ready for writing, this method
    /// will store a clone of the [`Waker`] from the provided [`Context`]. When the
    /// file descriptor becomes ready for writing, [`Waker::wake`] will be called.
    ///
    /// Note that on multiple calls to [`poll_write_ready`] or
    /// [`poll_write_ready_mut`], only the `Waker` from the `Context` passed to the
    /// most recent call is scheduled to receive a wakeup. (However,
    /// [`poll_read_ready`] retains a second, independent waker).
    ///
    /// This method is intended for cases where creating and pinning a future
    /// via [`writable`] is not feasible. Where possible, using [`writable`] is
    /// preferred, as this supports polling from multiple tasks at once.
    ///
    /// This method takes `&self`, so it is possible to call this method
    /// concurrently with other methods on this struct. This method only
    /// provides shared access to the inner IO resource when handling the
    /// [`AsyncFdReadyGuard`].
    ///
    /// [`poll_read_ready`]: method@Self::poll_read_ready
    /// [`poll_write_ready`]: method@Self::poll_write_ready
    /// [`poll_write_ready_mut`]: method@Self::poll_write_ready_mut
    /// [`writable`]: method@Self::readable
    /// [`Context`]: struct@std::task::Context
    /// [`Waker`]: struct@std::task::Waker
    /// [`Waker::wake`]: method@std::task::Waker::wake
    pub fn poll_write_ready<'a>(
        &'a self,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<AsyncFdReadyGuard<'a, T>>> {
        let event = ready!(self.registration.poll_write_ready(cx))?;

        Ok(AsyncFdReadyGuard {
            async_fd: self,
            event: Some(event),
        })
        .into()
    }

    /// Polls for write readiness.
    ///
    /// If the file descriptor is not currently ready for writing, this method
    /// will store a clone of the [`Waker`] from the provided [`Context`]. When the
    /// file descriptor becomes ready for writing, [`Waker::wake`] will be called.
    ///
    /// Note that on multiple calls to [`poll_write_ready`] or
    /// [`poll_write_ready_mut`], only the `Waker` from the `Context` passed to the
    /// most recent call is scheduled to receive a wakeup. (However,
    /// [`poll_read_ready`] retains a second, independent waker).
    ///
    /// This method is intended for cases where creating and pinning a future
    /// via [`writable`] is not feasible. Where possible, using [`writable`] is
    /// preferred, as this supports polling from multiple tasks at once.
    ///
    /// This method takes `&mut self`, so it is possible to access the inner IO
    /// resource mutably when handling the [`AsyncFdReadyMutGuard`].
    ///
    /// [`poll_read_ready`]: method@Self::poll_read_ready
    /// [`poll_write_ready`]: method@Self::poll_write_ready
    /// [`poll_write_ready_mut`]: method@Self::poll_write_ready_mut
    /// [`writable`]: method@Self::readable
    /// [`Context`]: struct@std::task::Context
    /// [`Waker`]: struct@std::task::Waker
    /// [`Waker::wake`]: method@std::task::Waker::wake
    pub fn poll_write_ready_mut<'a>(
        &'a mut self,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<AsyncFdReadyMutGuard<'a, T>>> {
        let event = ready!(self.registration.poll_write_ready(cx))?;

        Ok(AsyncFdReadyMutGuard {
            async_fd: self,
            event: Some(event),
        })
        .into()
    }

    async fn readiness(&self, interest: Interest) -> io::Result<AsyncFdReadyGuard<'_, T>> {
        let event = self.registration.readiness(interest).await?;

        Ok(AsyncFdReadyGuard {
            async_fd: self,
            event: Some(event),
        })
    }

    async fn readiness_mut(
        &mut self,
        interest: Interest,
    ) -> io::Result<AsyncFdReadyMutGuard<'_, T>> {
        let event = self.registration.readiness(interest).await?;

        Ok(AsyncFdReadyMutGuard {
            async_fd: self,
            event: Some(event),
        })
    }

    /// Waits for the file descriptor to become readable, returning a
    /// [`AsyncFdReadyGuard`] that must be dropped to resume read-readiness
    /// polling.
    ///
    /// This method takes `&self`, so it is possible to call this method
    /// concurrently with other methods on this struct. This method only
    /// provides shared access to the inner IO resource when handling the
    /// [`AsyncFdReadyGuard`].
    #[allow(clippy::needless_lifetimes)] // The lifetime improves rustdoc rendering.
    pub async fn readable<'a>(&'a self) -> io::Result<AsyncFdReadyGuard<'a, T>> {
        self.readiness(Interest::READABLE).await
    }

    /// Waits for the file descriptor to become readable, returning a
    /// [`AsyncFdReadyMutGuard`] that must be dropped to resume read-readiness
    /// polling.
    ///
    /// This method takes `&mut self`, so it is possible to access the inner IO
    /// resource mutably when handling the [`AsyncFdReadyMutGuard`].
    #[allow(clippy::needless_lifetimes)] // The lifetime improves rustdoc rendering.
    pub async fn readable_mut<'a>(&'a mut self) -> io::Result<AsyncFdReadyMutGuard<'a, T>> {
        self.readiness_mut(Interest::READABLE).await
    }

    /// Waits for the file descriptor to become writable, returning a
    /// [`AsyncFdReadyGuard`] that must be dropped to resume write-readiness
    /// polling.
    ///
    /// This method takes `&self`, so it is possible to call this method
    /// concurrently with other methods on this struct. This method only
    /// provides shared access to the inner IO resource when handling the
    /// [`AsyncFdReadyGuard`].
    #[allow(clippy::needless_lifetimes)] // The lifetime improves rustdoc rendering.
    pub async fn writable<'a>(&'a self) -> io::Result<AsyncFdReadyGuard<'a, T>> {
        self.readiness(Interest::WRITABLE).await
    }

    /// Waits for the file descriptor to become writable, returning a
    /// [`AsyncFdReadyMutGuard`] that must be dropped to resume write-readiness
    /// polling.
    ///
    /// This method takes `&mut self`, so it is possible to access the inner IO
    /// resource mutably when handling the [`AsyncFdReadyMutGuard`].
    #[allow(clippy::needless_lifetimes)] // The lifetime improves rustdoc rendering.
    pub async fn writable_mut<'a>(&'a mut self) -> io::Result<AsyncFdReadyMutGuard<'a, T>> {
        self.readiness_mut(Interest::WRITABLE).await
    }
}

impl<T: AsRawFd> AsRawFd for AsyncFd<T> {
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_ref().unwrap().as_raw_fd()
    }
}

impl<T: std::fmt::Debug + AsRawFd> std::fmt::Debug for AsyncFd<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncFd")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<T: AsRawFd> Drop for AsyncFd<T> {
    fn drop(&mut self) {
        let _ = self.take_inner();
    }
}

impl<'a, Inner: AsRawFd> AsyncFdReadyGuard<'a, Inner> {
    /// Indicates to tokio that the file descriptor is no longer ready. The
    /// internal readiness flag will be cleared, and tokio will wait for the
    /// next edge-triggered readiness notification from the OS.
    ///
    /// It is critical that this function not be called unless your code
    /// _actually observes_ that the file descriptor is _not_ ready. Do not call
    /// it simply because, for example, a read succeeded; it should be called
    /// when a read is observed to block.
    ///
    /// [`drop`]: method@std::mem::drop
    pub fn clear_ready(&mut self) {
        if let Some(event) = self.event.take() {
            self.async_fd.registration.clear_readiness(event);
        }
    }

    /// This method should be invoked when you intentionally want to keep the
    /// ready flag asserted.
    ///
    /// While this function is itself a no-op, it satisfies the `#[must_use]`
    /// constraint on the [`AsyncFdReadyGuard`] type.
    pub fn retain_ready(&mut self) {
        // no-op
    }

    /// Performs the provided IO operation.
    ///
    /// If `f` returns a [`WouldBlock`] error, the readiness state associated
    /// with this file descriptor is cleared, and the method returns
    /// `Err(TryIoError::WouldBlock)`. You will typically need to poll the
    /// `AsyncFd` again when this happens.
    ///
    /// This method helps ensure that the readiness state of the underlying file
    /// descriptor remains in sync with the tokio-side readiness state, by
    /// clearing the tokio-side state only when a [`WouldBlock`] condition
    /// occurs. It is the responsibility of the caller to ensure that `f`
    /// returns [`WouldBlock`] only if the file descriptor that originated this
    /// `AsyncFdReadyGuard` no longer expresses the readiness state that was queried to
    /// create this `AsyncFdReadyGuard`.
    ///
    /// [`WouldBlock`]: std::io::ErrorKind::WouldBlock
    // Alias for old name in 0.x
    #[cfg_attr(docsrs, doc(alias = "with_io"))]
    pub fn try_io<R>(
        &mut self,
        f: impl FnOnce(&'a AsyncFd<Inner>) -> io::Result<R>,
    ) -> Result<io::Result<R>, TryIoError> {
        let result = f(self.async_fd);

        if let Err(e) = result.as_ref() {
            if e.kind() == io::ErrorKind::WouldBlock {
                self.clear_ready();
            }
        }

        match result {
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => Err(TryIoError(())),
            result => Ok(result),
        }
    }

    /// Returns a shared reference to the inner [`AsyncFd`].
    pub fn get_ref(&self) -> &'a AsyncFd<Inner> {
        self.async_fd
    }

    /// Returns a shared reference to the backing object of the inner [`AsyncFd`].
    pub fn get_inner(&self) -> &'a Inner {
        self.get_ref().get_ref()
    }
}

impl<'a, Inner: AsRawFd> AsyncFdReadyMutGuard<'a, Inner> {
    /// Indicates to tokio that the file descriptor is no longer ready. The
    /// internal readiness flag will be cleared, and tokio will wait for the
    /// next edge-triggered readiness notification from the OS.
    ///
    /// It is critical that this function not be called unless your code
    /// _actually observes_ that the file descriptor is _not_ ready. Do not call
    /// it simply because, for example, a read succeeded; it should be called
    /// when a read is observed to block.
    ///
    /// [`drop`]: method@std::mem::drop
    pub fn clear_ready(&mut self) {
        if let Some(event) = self.event.take() {
            self.async_fd.registration.clear_readiness(event);
        }
    }

    /// This method should be invoked when you intentionally want to keep the
    /// ready flag asserted.
    ///
    /// While this function is itself a no-op, it satisfies the `#[must_use]`
    /// constraint on the [`AsyncFdReadyGuard`] type.
    pub fn retain_ready(&mut self) {
        // no-op
    }

    /// Performs the provided IO operation.
    ///
    /// If `f` returns a [`WouldBlock`] error, the readiness state associated
    /// with this file descriptor is cleared, and the method returns
    /// `Err(TryIoError::WouldBlock)`. You will typically need to poll the
    /// `AsyncFd` again when this happens.
    ///
    /// This method helps ensure that the readiness state of the underlying file
    /// descriptor remains in sync with the tokio-side readiness state, by
    /// clearing the tokio-side state only when a [`WouldBlock`] condition
    /// occurs. It is the responsibility of the caller to ensure that `f`
    /// returns [`WouldBlock`] only if the file descriptor that originated this
    /// `AsyncFdReadyGuard` no longer expresses the readiness state that was queried to
    /// create this `AsyncFdReadyGuard`.
    ///
    /// [`WouldBlock`]: std::io::ErrorKind::WouldBlock
    pub fn try_io<R>(
        &mut self,
        f: impl FnOnce(&mut AsyncFd<Inner>) -> io::Result<R>,
    ) -> Result<io::Result<R>, TryIoError> {
        let result = f(self.async_fd);

        if let Err(e) = result.as_ref() {
            if e.kind() == io::ErrorKind::WouldBlock {
                self.clear_ready();
            }
        }

        match result {
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => Err(TryIoError(())),
            result => Ok(result),
        }
    }

    /// Returns a shared reference to the inner [`AsyncFd`].
    pub fn get_ref(&self) -> &AsyncFd<Inner> {
        self.async_fd
    }

    /// Returns a mutable reference to the inner [`AsyncFd`].
    pub fn get_mut(&mut self) -> &mut AsyncFd<Inner> {
        self.async_fd
    }

    /// Returns a shared reference to the backing object of the inner [`AsyncFd`].
    pub fn get_inner(&self) -> &Inner {
        self.get_ref().get_ref()
    }

    /// Returns a mutable reference to the backing object of the inner [`AsyncFd`].
    pub fn get_inner_mut(&mut self) -> &mut Inner {
        self.get_mut().get_mut()
    }
}

impl<'a, T: std::fmt::Debug + AsRawFd> std::fmt::Debug for AsyncFdReadyGuard<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReadyGuard")
            .field("async_fd", &self.async_fd)
            .finish()
    }
}

impl<'a, T: std::fmt::Debug + AsRawFd> std::fmt::Debug for AsyncFdReadyMutGuard<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MutReadyGuard")
            .field("async_fd", &self.async_fd)
            .finish()
    }
}

/// The error type returned by [`try_io`].
///
/// This error indicates that the IO resource returned a [`WouldBlock`] error.
///
/// [`WouldBlock`]: std::io::ErrorKind::WouldBlock
/// [`try_io`]: method@AsyncFdReadyGuard::try_io
#[derive(Debug)]
pub struct TryIoError(());
