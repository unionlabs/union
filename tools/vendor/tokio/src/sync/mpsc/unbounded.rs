use crate::loom::sync::atomic::AtomicUsize;
use crate::sync::mpsc::chan;
use crate::sync::mpsc::error::{SendError, TryRecvError};

use std::fmt;
use std::task::{Context, Poll};

/// Send values to the associated `UnboundedReceiver`.
///
/// Instances are created by the
/// [`unbounded_channel`](unbounded_channel) function.
pub struct UnboundedSender<T> {
    chan: chan::Tx<T, Semaphore>,
}

impl<T> Clone for UnboundedSender<T> {
    fn clone(&self) -> Self {
        UnboundedSender {
            chan: self.chan.clone(),
        }
    }
}

impl<T> fmt::Debug for UnboundedSender<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("UnboundedSender")
            .field("chan", &self.chan)
            .finish()
    }
}

/// Receive values from the associated `UnboundedSender`.
///
/// Instances are created by the
/// [`unbounded_channel`](unbounded_channel) function.
///
/// This receiver can be turned into a `Stream` using [`UnboundedReceiverStream`].
///
/// [`UnboundedReceiverStream`]: https://docs.rs/tokio-stream/0.1/tokio_stream/wrappers/struct.UnboundedReceiverStream.html
pub struct UnboundedReceiver<T> {
    /// The channel receiver
    chan: chan::Rx<T, Semaphore>,
}

impl<T> fmt::Debug for UnboundedReceiver<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("UnboundedReceiver")
            .field("chan", &self.chan)
            .finish()
    }
}

/// Creates an unbounded mpsc channel for communicating between asynchronous
/// tasks without backpressure.
///
/// A `send` on this channel will always succeed as long as the receive half has
/// not been closed. If the receiver falls behind, messages will be arbitrarily
/// buffered.
///
/// **Note** that the amount of available system memory is an implicit bound to
/// the channel. Using an `unbounded` channel has the ability of causing the
/// process to run out of memory. In this case, the process will be aborted.
pub fn unbounded_channel<T>() -> (UnboundedSender<T>, UnboundedReceiver<T>) {
    let (tx, rx) = chan::channel(AtomicUsize::new(0));

    let tx = UnboundedSender::new(tx);
    let rx = UnboundedReceiver::new(rx);

    (tx, rx)
}

/// No capacity
type Semaphore = AtomicUsize;

impl<T> UnboundedReceiver<T> {
    pub(crate) fn new(chan: chan::Rx<T, Semaphore>) -> UnboundedReceiver<T> {
        UnboundedReceiver { chan }
    }

    /// Receives the next value for this receiver.
    ///
    /// This method returns `None` if the channel has been closed and there are
    /// no remaining messages in the channel's buffer. This indicates that no
    /// further values can ever be received from this `Receiver`. The channel is
    /// closed when all senders have been dropped, or when [`close`] is called.
    ///
    /// If there are no messages in the channel's buffer, but the channel has
    /// not yet been closed, this method will sleep until a message is sent or
    /// the channel is closed.
    ///
    /// # Cancel safety
    ///
    /// This method is cancel safe. If `recv` is used as the event in a
    /// [`tokio::select!`](crate::select) statement and some other branch
    /// completes first, it is guaranteed that no messages were received on this
    /// channel.
    ///
    /// [`close`]: Self::close
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::sync::mpsc;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let (tx, mut rx) = mpsc::unbounded_channel();
    ///
    ///     tokio::spawn(async move {
    ///         tx.send("hello").unwrap();
    ///     });
    ///
    ///     assert_eq!(Some("hello"), rx.recv().await);
    ///     assert_eq!(None, rx.recv().await);
    /// }
    /// ```
    ///
    /// Values are buffered:
    ///
    /// ```
    /// use tokio::sync::mpsc;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let (tx, mut rx) = mpsc::unbounded_channel();
    ///
    ///     tx.send("hello").unwrap();
    ///     tx.send("world").unwrap();
    ///
    ///     assert_eq!(Some("hello"), rx.recv().await);
    ///     assert_eq!(Some("world"), rx.recv().await);
    /// }
    /// ```
    pub async fn recv(&mut self) -> Option<T> {
        use crate::future::poll_fn;

        poll_fn(|cx| self.poll_recv(cx)).await
    }

    /// Tries to receive the next value for this receiver.
    ///
    /// This method returns the [`Empty`] error if the channel is currently
    /// empty, but there are still outstanding [senders] or [permits].
    ///
    /// This method returns the [`Disconnected`] error if the channel is
    /// currently empty, and there are no outstanding [senders] or [permits].
    ///
    /// Unlike the [`poll_recv`] method, this method will never return an
    /// [`Empty`] error spuriously.
    ///
    /// [`Empty`]: crate::sync::mpsc::error::TryRecvError::Empty
    /// [`Disconnected`]: crate::sync::mpsc::error::TryRecvError::Disconnected
    /// [`poll_recv`]: Self::poll_recv
    /// [senders]: crate::sync::mpsc::Sender
    /// [permits]: crate::sync::mpsc::Permit
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::sync::mpsc;
    /// use tokio::sync::mpsc::error::TryRecvError;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let (tx, mut rx) = mpsc::unbounded_channel();
    ///
    ///     tx.send("hello").unwrap();
    ///
    ///     assert_eq!(Ok("hello"), rx.try_recv());
    ///     assert_eq!(Err(TryRecvError::Empty), rx.try_recv());
    ///
    ///     tx.send("hello").unwrap();
    ///     // Drop the last sender, closing the channel.
    ///     drop(tx);
    ///
    ///     assert_eq!(Ok("hello"), rx.try_recv());
    ///     assert_eq!(Err(TryRecvError::Disconnected), rx.try_recv());
    /// }
    /// ```
    pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
        self.chan.try_recv()
    }

    /// Blocking receive to call outside of asynchronous contexts.
    ///
    /// # Panics
    ///
    /// This function panics if called within an asynchronous execution
    /// context.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::thread;
    /// use tokio::sync::mpsc;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let (tx, mut rx) = mpsc::unbounded_channel::<u8>();
    ///
    ///     let sync_code = thread::spawn(move || {
    ///         assert_eq!(Some(10), rx.blocking_recv());
    ///     });
    ///
    ///     let _ = tx.send(10);
    ///     sync_code.join().unwrap();
    /// }
    /// ```
    #[track_caller]
    #[cfg(feature = "sync")]
    pub fn blocking_recv(&mut self) -> Option<T> {
        crate::future::block_on(self.recv())
    }

    /// Closes the receiving half of a channel, without dropping it.
    ///
    /// This prevents any further messages from being sent on the channel while
    /// still enabling the receiver to drain messages that are buffered.
    ///
    /// To guarantee that no messages are dropped, after calling `close()`,
    /// `recv()` must be called until `None` is returned.
    pub fn close(&mut self) {
        self.chan.close();
    }

    /// Polls to receive the next message on this channel.
    ///
    /// This method returns:
    ///
    ///  * `Poll::Pending` if no messages are available but the channel is not
    ///    closed, or if a spurious failure happens.
    ///  * `Poll::Ready(Some(message))` if a message is available.
    ///  * `Poll::Ready(None)` if the channel has been closed and all messages
    ///    sent before it was closed have been received.
    ///
    /// When the method returns `Poll::Pending`, the `Waker` in the provided
    /// `Context` is scheduled to receive a wakeup when a message is sent on any
    /// receiver, or when the channel is closed.  Note that on multiple calls to
    /// `poll_recv`, only the `Waker` from the `Context` passed to the most
    /// recent call is scheduled to receive a wakeup.
    ///
    /// If this method returns `Poll::Pending` due to a spurious failure, then
    /// the `Waker` will be notified when the situation causing the spurious
    /// failure has been resolved. Note that receiving such a wakeup does not
    /// guarantee that the next call will succeed — it could fail with another
    /// spurious failure.
    pub fn poll_recv(&mut self, cx: &mut Context<'_>) -> Poll<Option<T>> {
        self.chan.recv(cx)
    }
}

impl<T> UnboundedSender<T> {
    pub(crate) fn new(chan: chan::Tx<T, Semaphore>) -> UnboundedSender<T> {
        UnboundedSender { chan }
    }

    /// Attempts to send a message on this `UnboundedSender` without blocking.
    ///
    /// This method is not marked async because sending a message to an unbounded channel
    /// never requires any form of waiting. Because of this, the `send` method can be
    /// used in both synchronous and asynchronous code without problems.
    ///
    /// If the receive half of the channel is closed, either due to [`close`]
    /// being called or the [`UnboundedReceiver`] having been dropped, this
    /// function returns an error. The error includes the value passed to `send`.
    ///
    /// [`close`]: UnboundedReceiver::close
    /// [`UnboundedReceiver`]: UnboundedReceiver
    pub fn send(&self, message: T) -> Result<(), SendError<T>> {
        if !self.inc_num_messages() {
            return Err(SendError(message));
        }

        self.chan.send(message);
        Ok(())
    }

    fn inc_num_messages(&self) -> bool {
        use std::process;
        use std::sync::atomic::Ordering::{AcqRel, Acquire};

        let mut curr = self.chan.semaphore().load(Acquire);

        loop {
            if curr & 1 == 1 {
                return false;
            }

            if curr == usize::MAX ^ 1 {
                // Overflowed the ref count. There is no safe way to recover, so
                // abort the process. In practice, this should never happen.
                process::abort()
            }

            match self
                .chan
                .semaphore()
                .compare_exchange(curr, curr + 2, AcqRel, Acquire)
            {
                Ok(_) => return true,
                Err(actual) => {
                    curr = actual;
                }
            }
        }
    }

    /// Completes when the receiver has dropped.
    ///
    /// This allows the producers to get notified when interest in the produced
    /// values is canceled and immediately stop doing work.
    ///
    /// # Cancel safety
    ///
    /// This method is cancel safe. Once the channel is closed, it stays closed
    /// forever and all future calls to `closed` will return immediately.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::sync::mpsc;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let (tx1, rx) = mpsc::unbounded_channel::<()>();
    ///     let tx2 = tx1.clone();
    ///     let tx3 = tx1.clone();
    ///     let tx4 = tx1.clone();
    ///     let tx5 = tx1.clone();
    ///     tokio::spawn(async move {
    ///         drop(rx);
    ///     });
    ///
    ///     futures::join!(
    ///         tx1.closed(),
    ///         tx2.closed(),
    ///         tx3.closed(),
    ///         tx4.closed(),
    ///         tx5.closed()
    ///     );
    ////     println!("Receiver dropped");
    /// }
    /// ```
    pub async fn closed(&self) {
        self.chan.closed().await
    }

    /// Checks if the channel has been closed. This happens when the
    /// [`UnboundedReceiver`] is dropped, or when the
    /// [`UnboundedReceiver::close`] method is called.
    ///
    /// [`UnboundedReceiver`]: crate::sync::mpsc::UnboundedReceiver
    /// [`UnboundedReceiver::close`]: crate::sync::mpsc::UnboundedReceiver::close
    ///
    /// ```
    /// let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<()>();
    /// assert!(!tx.is_closed());
    ///
    /// let tx2 = tx.clone();
    /// assert!(!tx2.is_closed());
    ///
    /// drop(rx);
    /// assert!(tx.is_closed());
    /// assert!(tx2.is_closed());
    /// ```
    pub fn is_closed(&self) -> bool {
        self.chan.is_closed()
    }

    /// Returns `true` if senders belong to the same channel.
    ///
    /// # Examples
    ///
    /// ```
    /// let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<()>();
    /// let  tx2 = tx.clone();
    /// assert!(tx.same_channel(&tx2));
    ///
    /// let (tx3, rx3) = tokio::sync::mpsc::unbounded_channel::<()>();
    /// assert!(!tx3.same_channel(&tx2));
    /// ```
    pub fn same_channel(&self, other: &Self) -> bool {
        self.chan.same_channel(&other.chan)
    }
}
