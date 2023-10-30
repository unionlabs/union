//! An async multi-producer multi-consumer channel.
//!
//! There are two kinds of channels:
//!
//! 1. [Bounded][`bounded()`] channel with limited capacity.
//! 2. [Unbounded][`unbounded()`] channel with unlimited capacity.
//!
//! A channel has the [`Sender`] and [`Receiver`] side. Both sides are cloneable and can be shared
//! among multiple threads.
//!
//! When all [`Sender`]s or all [`Receiver`]s are dropped, the channel becomes closed. When a
//! channel is closed, no more messages can be sent, but remaining messages can still be received.
//!
//! The channel can also be closed manually by calling [`Sender::close()`] or
//! [`Receiver::close()`].
//!
//! # Examples
//!
//! ```
//! # futures_lite::future::block_on(async {
//! let (s, r) = async_channel::unbounded();
//!
//! assert_eq!(s.send("Hello").await, Ok(()));
//! assert_eq!(r.recv().await, Ok("Hello"));
//! # });
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]

use std::error;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::process;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::usize;

use concurrent_queue::{ConcurrentQueue, PopError, PushError};
use event_listener::{Event, EventListener};
use futures_core::stream::Stream;

struct Channel<T> {
    /// Inner message queue.
    queue: ConcurrentQueue<T>,

    /// Send operations waiting while the channel is full.
    send_ops: Event,

    /// Receive operations waiting while the channel is empty and not closed.
    recv_ops: Event,

    /// Stream operations while the channel is empty and not closed.
    stream_ops: Event,

    /// The number of currently active `Sender`s.
    sender_count: AtomicUsize,

    /// The number of currently active `Receivers`s.
    receiver_count: AtomicUsize,
}

impl<T> Channel<T> {
    /// Closes the channel and notifies all blocked operations.
    ///
    /// Returns `true` if this call has closed the channel and it was not closed already.
    fn close(&self) -> bool {
        if self.queue.close() {
            // Notify all send operations.
            self.send_ops.notify(usize::MAX);

            // Notify all receive and stream operations.
            self.recv_ops.notify(usize::MAX);
            self.stream_ops.notify(usize::MAX);

            true
        } else {
            false
        }
    }
}

/// Creates a bounded channel.
///
/// The created channel has space to hold at most `cap` messages at a time.
///
/// # Panics
///
/// Capacity must be a positive number. If `cap` is zero, this function will panic.
///
/// # Examples
///
/// ```
/// # futures_lite::future::block_on(async {
/// use async_channel::{bounded, TryRecvError, TrySendError};
///
/// let (s, r) = bounded(1);
///
/// assert_eq!(s.send(10).await, Ok(()));
/// assert_eq!(s.try_send(20), Err(TrySendError::Full(20)));
///
/// assert_eq!(r.recv().await, Ok(10));
/// assert_eq!(r.try_recv(), Err(TryRecvError::Empty));
/// # });
pub fn bounded<T>(cap: usize) -> (Sender<T>, Receiver<T>) {
    assert!(cap > 0, "capacity cannot be zero");

    let channel = Arc::new(Channel {
        queue: ConcurrentQueue::bounded(cap),
        send_ops: Event::new(),
        recv_ops: Event::new(),
        stream_ops: Event::new(),
        sender_count: AtomicUsize::new(1),
        receiver_count: AtomicUsize::new(1),
    });

    let s = Sender {
        channel: channel.clone(),
    };
    let r = Receiver {
        channel,
        listener: None,
    };
    (s, r)
}

/// Creates an unbounded channel.
///
/// The created channel can hold an unlimited number of messages.
///
/// # Examples
///
/// ```
/// # futures_lite::future::block_on(async {
/// use async_channel::{unbounded, TryRecvError};
///
/// let (s, r) = unbounded();
///
/// assert_eq!(s.send(10).await, Ok(()));
/// assert_eq!(s.send(20).await, Ok(()));
///
/// assert_eq!(r.recv().await, Ok(10));
/// assert_eq!(r.recv().await, Ok(20));
/// assert_eq!(r.try_recv(), Err(TryRecvError::Empty));
/// # });
pub fn unbounded<T>() -> (Sender<T>, Receiver<T>) {
    let channel = Arc::new(Channel {
        queue: ConcurrentQueue::unbounded(),
        send_ops: Event::new(),
        recv_ops: Event::new(),
        stream_ops: Event::new(),
        sender_count: AtomicUsize::new(1),
        receiver_count: AtomicUsize::new(1),
    });

    let s = Sender {
        channel: channel.clone(),
    };
    let r = Receiver {
        channel,
        listener: None,
    };
    (s, r)
}

/// The sending side of a channel.
///
/// Senders can be cloned and shared among threads. When all senders associated with a channel are
/// dropped, the channel becomes closed.
///
/// The channel can also be closed manually by calling [`Sender::close()`].
pub struct Sender<T> {
    /// Inner channel state.
    channel: Arc<Channel<T>>,
}

impl<T> Sender<T> {
    /// Attempts to send a message into the channel.
    ///
    /// If the channel is full or closed, this method returns an error.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_channel::{bounded, TrySendError};
    ///
    /// let (s, r) = bounded(1);
    ///
    /// assert_eq!(s.try_send(1), Ok(()));
    /// assert_eq!(s.try_send(2), Err(TrySendError::Full(2)));
    ///
    /// drop(r);
    /// assert_eq!(s.try_send(3), Err(TrySendError::Closed(3)));
    /// ```
    pub fn try_send(&self, msg: T) -> Result<(), TrySendError<T>> {
        match self.channel.queue.push(msg) {
            Ok(()) => {
                // Notify a single blocked receive operation. If the notified operation then
                // receives a message or gets canceled, it will notify another blocked receive
                // operation.
                self.channel.recv_ops.notify(1);

                // Notify all blocked streams.
                self.channel.stream_ops.notify(usize::MAX);

                Ok(())
            }
            Err(PushError::Full(msg)) => Err(TrySendError::Full(msg)),
            Err(PushError::Closed(msg)) => Err(TrySendError::Closed(msg)),
        }
    }

    /// Sends a message into the channel.
    ///
    /// If the channel is full, this method waits until there is space for a message.
    ///
    /// If the channel is closed, this method returns an error.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures_lite::future::block_on(async {
    /// use async_channel::{unbounded, SendError};
    ///
    /// let (s, r) = unbounded();
    ///
    /// assert_eq!(s.send(1).await, Ok(()));
    /// drop(r);
    /// assert_eq!(s.send(2).await, Err(SendError(2)));
    /// # });
    /// ```
    pub fn send(&self, msg: T) -> Send<'_, T> {
        Send {
            sender: self,
            listener: None,
            msg: Some(msg),
        }
    }

    /// Closes the channel.
    ///
    /// Returns `true` if this call has closed the channel and it was not closed already.
    ///
    /// The remaining messages can still be received.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures_lite::future::block_on(async {
    /// use async_channel::{unbounded, RecvError};
    ///
    /// let (s, r) = unbounded();
    /// assert_eq!(s.send(1).await, Ok(()));
    /// assert!(s.close());
    ///
    /// assert_eq!(r.recv().await, Ok(1));
    /// assert_eq!(r.recv().await, Err(RecvError));
    /// # });
    /// ```
    pub fn close(&self) -> bool {
        self.channel.close()
    }

    /// Returns `true` if the channel is closed.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures_lite::future::block_on(async {
    /// use async_channel::{unbounded, RecvError};
    ///
    /// let (s, r) = unbounded::<()>();
    /// assert!(!s.is_closed());
    ///
    /// drop(r);
    /// assert!(s.is_closed());
    /// # });
    /// ```
    pub fn is_closed(&self) -> bool {
        self.channel.queue.is_closed()
    }

    /// Returns `true` if the channel is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures_lite::future::block_on(async {
    /// use async_channel::unbounded;
    ///
    /// let (s, r) = unbounded();
    ///
    /// assert!(s.is_empty());
    /// s.send(1).await;
    /// assert!(!s.is_empty());
    /// # });
    /// ```
    pub fn is_empty(&self) -> bool {
        self.channel.queue.is_empty()
    }

    /// Returns `true` if the channel is full.
    ///
    /// Unbounded channels are never full.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures_lite::future::block_on(async {
    /// use async_channel::bounded;
    ///
    /// let (s, r) = bounded(1);
    ///
    /// assert!(!s.is_full());
    /// s.send(1).await;
    /// assert!(s.is_full());
    /// # });
    /// ```
    pub fn is_full(&self) -> bool {
        self.channel.queue.is_full()
    }

    /// Returns the number of messages in the channel.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures_lite::future::block_on(async {
    /// use async_channel::unbounded;
    ///
    /// let (s, r) = unbounded();
    /// assert_eq!(s.len(), 0);
    ///
    /// s.send(1).await;
    /// s.send(2).await;
    /// assert_eq!(s.len(), 2);
    /// # });
    /// ```
    pub fn len(&self) -> usize {
        self.channel.queue.len()
    }

    /// Returns the channel capacity if it's bounded.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_channel::{bounded, unbounded};
    ///
    /// let (s, r) = bounded::<i32>(5);
    /// assert_eq!(s.capacity(), Some(5));
    ///
    /// let (s, r) = unbounded::<i32>();
    /// assert_eq!(s.capacity(), None);
    /// ```
    pub fn capacity(&self) -> Option<usize> {
        self.channel.queue.capacity()
    }

    /// Returns the number of receivers for the channel.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures_lite::future::block_on(async {
    /// use async_channel::unbounded;
    ///
    /// let (s, r) = unbounded::<()>();
    /// assert_eq!(s.receiver_count(), 1);
    ///
    /// let r2 = r.clone();
    /// assert_eq!(s.receiver_count(), 2);
    /// # });
    /// ```
    pub fn receiver_count(&self) -> usize {
        self.channel.receiver_count.load(Ordering::SeqCst)
    }

    /// Returns the number of senders for the channel.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures_lite::future::block_on(async {
    /// use async_channel::unbounded;
    ///
    /// let (s, r) = unbounded::<()>();
    /// assert_eq!(s.sender_count(), 1);
    ///
    /// let s2 = s.clone();
    /// assert_eq!(s.sender_count(), 2);
    /// # });
    /// ```
    pub fn sender_count(&self) -> usize {
        self.channel.sender_count.load(Ordering::SeqCst)
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        // Decrement the sender count and close the channel if it drops down to zero.
        if self.channel.sender_count.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.channel.close();
        }
    }
}

impl<T> fmt::Debug for Sender<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sender {{ .. }}")
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Sender<T> {
        let count = self.channel.sender_count.fetch_add(1, Ordering::Relaxed);

        // Make sure the count never overflows, even if lots of sender clones are leaked.
        if count > usize::MAX / 2 {
            process::abort();
        }

        Sender {
            channel: self.channel.clone(),
        }
    }
}

/// The receiving side of a channel.
///
/// Receivers can be cloned and shared among threads. When all receivers associated with a channel
/// are dropped, the channel becomes closed.
///
/// The channel can also be closed manually by calling [`Receiver::close()`].
///
/// Receivers implement the [`Stream`] trait.
pub struct Receiver<T> {
    /// Inner channel state.
    channel: Arc<Channel<T>>,

    /// Listens for a send or close event to unblock this stream.
    listener: Option<EventListener>,
}

impl<T> Receiver<T> {
    /// Attempts to receive a message from the channel.
    ///
    /// If the channel is empty or closed, this method returns an error.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures_lite::future::block_on(async {
    /// use async_channel::{unbounded, TryRecvError};
    ///
    /// let (s, r) = unbounded();
    /// assert_eq!(s.send(1).await, Ok(()));
    ///
    /// assert_eq!(r.try_recv(), Ok(1));
    /// assert_eq!(r.try_recv(), Err(TryRecvError::Empty));
    ///
    /// drop(s);
    /// assert_eq!(r.try_recv(), Err(TryRecvError::Closed));
    /// # });
    /// ```
    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        match self.channel.queue.pop() {
            Ok(msg) => {
                // Notify a single blocked send operation. If the notified operation then sends a
                // message or gets canceled, it will notify another blocked send operation.
                self.channel.send_ops.notify(1);

                Ok(msg)
            }
            Err(PopError::Empty) => Err(TryRecvError::Empty),
            Err(PopError::Closed) => Err(TryRecvError::Closed),
        }
    }

    /// Receives a message from the channel.
    ///
    /// If the channel is empty, this method waits until there is a message.
    ///
    /// If the channel is closed, this method receives a message or returns an error if there are
    /// no more messages.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures_lite::future::block_on(async {
    /// use async_channel::{unbounded, RecvError};
    ///
    /// let (s, r) = unbounded();
    ///
    /// assert_eq!(s.send(1).await, Ok(()));
    /// drop(s);
    ///
    /// assert_eq!(r.recv().await, Ok(1));
    /// assert_eq!(r.recv().await, Err(RecvError));
    /// # });
    /// ```
    pub fn recv(&self) -> Recv<'_, T> {
        Recv {
            receiver: self,
            listener: None,
        }
    }

    /// Closes the channel.
    ///
    /// Returns `true` if this call has closed the channel and it was not closed already.
    ///
    /// The remaining messages can still be received.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures_lite::future::block_on(async {
    /// use async_channel::{unbounded, RecvError};
    ///
    /// let (s, r) = unbounded();
    /// assert_eq!(s.send(1).await, Ok(()));
    ///
    /// assert!(r.close());
    /// assert_eq!(r.recv().await, Ok(1));
    /// assert_eq!(r.recv().await, Err(RecvError));
    /// # });
    /// ```
    pub fn close(&self) -> bool {
        self.channel.close()
    }

    /// Returns `true` if the channel is closed.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures_lite::future::block_on(async {
    /// use async_channel::{unbounded, RecvError};
    ///
    /// let (s, r) = unbounded::<()>();
    /// assert!(!r.is_closed());
    ///
    /// drop(s);
    /// assert!(r.is_closed());
    /// # });
    /// ```
    pub fn is_closed(&self) -> bool {
        self.channel.queue.is_closed()
    }

    /// Returns `true` if the channel is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures_lite::future::block_on(async {
    /// use async_channel::unbounded;
    ///
    /// let (s, r) = unbounded();
    ///
    /// assert!(s.is_empty());
    /// s.send(1).await;
    /// assert!(!s.is_empty());
    /// # });
    /// ```
    pub fn is_empty(&self) -> bool {
        self.channel.queue.is_empty()
    }

    /// Returns `true` if the channel is full.
    ///
    /// Unbounded channels are never full.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures_lite::future::block_on(async {
    /// use async_channel::bounded;
    ///
    /// let (s, r) = bounded(1);
    ///
    /// assert!(!r.is_full());
    /// s.send(1).await;
    /// assert!(r.is_full());
    /// # });
    /// ```
    pub fn is_full(&self) -> bool {
        self.channel.queue.is_full()
    }

    /// Returns the number of messages in the channel.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures_lite::future::block_on(async {
    /// use async_channel::unbounded;
    ///
    /// let (s, r) = unbounded();
    /// assert_eq!(r.len(), 0);
    ///
    /// s.send(1).await;
    /// s.send(2).await;
    /// assert_eq!(r.len(), 2);
    /// # });
    /// ```
    pub fn len(&self) -> usize {
        self.channel.queue.len()
    }

    /// Returns the channel capacity if it's bounded.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_channel::{bounded, unbounded};
    ///
    /// let (s, r) = bounded::<i32>(5);
    /// assert_eq!(r.capacity(), Some(5));
    ///
    /// let (s, r) = unbounded::<i32>();
    /// assert_eq!(r.capacity(), None);
    /// ```
    pub fn capacity(&self) -> Option<usize> {
        self.channel.queue.capacity()
    }

    /// Returns the number of receivers for the channel.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures_lite::future::block_on(async {
    /// use async_channel::unbounded;
    ///
    /// let (s, r) = unbounded::<()>();
    /// assert_eq!(r.receiver_count(), 1);
    ///
    /// let r2 = r.clone();
    /// assert_eq!(r.receiver_count(), 2);
    /// # });
    /// ```
    pub fn receiver_count(&self) -> usize {
        self.channel.receiver_count.load(Ordering::SeqCst)
    }

    /// Returns the number of senders for the channel.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures_lite::future::block_on(async {
    /// use async_channel::unbounded;
    ///
    /// let (s, r) = unbounded::<()>();
    /// assert_eq!(r.sender_count(), 1);
    ///
    /// let s2 = s.clone();
    /// assert_eq!(r.sender_count(), 2);
    /// # });
    /// ```
    pub fn sender_count(&self) -> usize {
        self.channel.sender_count.load(Ordering::SeqCst)
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        // Decrement the receiver count and close the channel if it drops down to zero.
        if self.channel.receiver_count.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.channel.close();
        }
    }
}

impl<T> fmt::Debug for Receiver<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Receiver {{ .. }}")
    }
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Receiver<T> {
        let count = self.channel.receiver_count.fetch_add(1, Ordering::Relaxed);

        // Make sure the count never overflows, even if lots of receiver clones are leaked.
        if count > usize::MAX / 2 {
            process::abort();
        }

        Receiver {
            channel: self.channel.clone(),
            listener: None,
        }
    }
}

impl<T> Stream for Receiver<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            // If this stream is listening for events, first wait for a notification.
            if let Some(listener) = self.listener.as_mut() {
                futures_core::ready!(Pin::new(listener).poll(cx));
                self.listener = None;
            }

            loop {
                // Attempt to receive a message.
                match self.try_recv() {
                    Ok(msg) => {
                        // The stream is not blocked on an event - drop the listener.
                        self.listener = None;
                        return Poll::Ready(Some(msg));
                    }
                    Err(TryRecvError::Closed) => {
                        // The stream is not blocked on an event - drop the listener.
                        self.listener = None;
                        return Poll::Ready(None);
                    }
                    Err(TryRecvError::Empty) => {}
                }

                // Receiving failed - now start listening for notifications or wait for one.
                match self.listener.as_mut() {
                    None => {
                        // Create a listener and try sending the message again.
                        self.listener = Some(self.channel.stream_ops.listen());
                    }
                    Some(_) => {
                        // Go back to the outer loop to poll the listener.
                        break;
                    }
                }
            }
        }
    }
}

impl<T> futures_core::stream::FusedStream for Receiver<T> {
    fn is_terminated(&self) -> bool {
        self.channel.queue.is_closed() && self.channel.queue.is_empty()
    }
}

/// An error returned from [`Sender::send()`].
///
/// Received because the channel is closed.
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct SendError<T>(pub T);

impl<T> SendError<T> {
    /// Unwraps the message that couldn't be sent.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> error::Error for SendError<T> {}

impl<T> fmt::Debug for SendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SendError(..)")
    }
}

impl<T> fmt::Display for SendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sending into a closed channel")
    }
}

/// An error returned from [`Sender::try_send()`].
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TrySendError<T> {
    /// The channel is full but not closed.
    Full(T),

    /// The channel is closed.
    Closed(T),
}

impl<T> TrySendError<T> {
    /// Unwraps the message that couldn't be sent.
    pub fn into_inner(self) -> T {
        match self {
            TrySendError::Full(t) => t,
            TrySendError::Closed(t) => t,
        }
    }

    /// Returns `true` if the channel is full but not closed.
    pub fn is_full(&self) -> bool {
        match self {
            TrySendError::Full(_) => true,
            TrySendError::Closed(_) => false,
        }
    }

    /// Returns `true` if the channel is closed.
    pub fn is_closed(&self) -> bool {
        match self {
            TrySendError::Full(_) => false,
            TrySendError::Closed(_) => true,
        }
    }
}

impl<T> error::Error for TrySendError<T> {}

impl<T> fmt::Debug for TrySendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TrySendError::Full(..) => write!(f, "Full(..)"),
            TrySendError::Closed(..) => write!(f, "Closed(..)"),
        }
    }
}

impl<T> fmt::Display for TrySendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TrySendError::Full(..) => write!(f, "sending into a full channel"),
            TrySendError::Closed(..) => write!(f, "sending into a closed channel"),
        }
    }
}

/// An error returned from [`Receiver::recv()`].
///
/// Received because the channel is empty and closed.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct RecvError;

impl error::Error for RecvError {}

impl fmt::Display for RecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "receiving from an empty and closed channel")
    }
}

/// An error returned from [`Receiver::try_recv()`].
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum TryRecvError {
    /// The channel is empty but not closed.
    Empty,

    /// The channel is empty and closed.
    Closed,
}

impl TryRecvError {
    /// Returns `true` if the channel is empty but not closed.
    pub fn is_empty(&self) -> bool {
        match self {
            TryRecvError::Empty => true,
            TryRecvError::Closed => false,
        }
    }

    /// Returns `true` if the channel is empty and closed.
    pub fn is_closed(&self) -> bool {
        match self {
            TryRecvError::Empty => false,
            TryRecvError::Closed => true,
        }
    }
}

impl error::Error for TryRecvError {}

impl fmt::Display for TryRecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TryRecvError::Empty => write!(f, "receiving from an empty channel"),
            TryRecvError::Closed => write!(f, "receiving from an empty and closed channel"),
        }
    }
}

/// A future returned by [`Sender::send()`].
#[derive(Debug)]
#[must_use = "futures do nothing unless .awaited"]
pub struct Send<'a, T> {
    sender: &'a Sender<T>,
    listener: Option<EventListener>,
    msg: Option<T>,
}

impl<'a, T> Unpin for Send<'a, T> {}

impl<'a, T> Future for Send<'a, T> {
    type Output = Result<(), SendError<T>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = Pin::new(self);

        loop {
            let msg = this.msg.take().unwrap();
            // Attempt to send a message.
            match this.sender.try_send(msg) {
                Ok(()) => {
                    // If the capacity is larger than 1, notify another blocked send operation.
                    match this.sender.channel.queue.capacity() {
                        Some(1) => {}
                        Some(_) | None => this.sender.channel.send_ops.notify(1),
                    }
                    return Poll::Ready(Ok(()));
                }
                Err(TrySendError::Closed(msg)) => return Poll::Ready(Err(SendError(msg))),
                Err(TrySendError::Full(m)) => this.msg = Some(m),
            }

            // Sending failed - now start listening for notifications or wait for one.
            match &mut this.listener {
                None => {
                    // Start listening and then try receiving again.
                    this.listener = Some(this.sender.channel.send_ops.listen());
                }
                Some(l) => {
                    // Wait for a notification.
                    match Pin::new(l).poll(cx) {
                        Poll::Ready(_) => {
                            this.listener = None;
                            continue;
                        }

                        Poll::Pending => return Poll::Pending,
                    }
                }
            }
        }
    }
}

/// A future returned by [`Receiver::recv()`].
#[derive(Debug)]
#[must_use = "futures do nothing unless .awaited"]
pub struct Recv<'a, T> {
    receiver: &'a Receiver<T>,
    listener: Option<EventListener>,
}

impl<'a, T> Unpin for Recv<'a, T> {}

impl<'a, T> Future for Recv<'a, T> {
    type Output = Result<T, RecvError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = Pin::new(self);

        loop {
            // Attempt to receive a message.
            match this.receiver.try_recv() {
                Ok(msg) => {
                    // If the capacity is larger than 1, notify another blocked receive operation.
                    // There is no need to notify stream operations because all of them get
                    // notified every time a message is sent into the channel.
                    match this.receiver.channel.queue.capacity() {
                        Some(1) => {}
                        Some(_) | None => this.receiver.channel.recv_ops.notify(1),
                    }
                    return Poll::Ready(Ok(msg));
                }
                Err(TryRecvError::Closed) => return Poll::Ready(Err(RecvError)),
                Err(TryRecvError::Empty) => {}
            }

            // Receiving failed - now start listening for notifications or wait for one.
            match &mut this.listener {
                None => {
                    // Start listening and then try receiving again.
                    this.listener = Some(this.receiver.channel.recv_ops.listen());
                }
                Some(l) => {
                    // Wait for a notification.
                    match Pin::new(l).poll(cx) {
                        Poll::Ready(_) => {
                            this.listener = None;
                            continue;
                        }

                        Poll::Pending => return Poll::Pending,
                    }
                }
            }
        }
    }
}
