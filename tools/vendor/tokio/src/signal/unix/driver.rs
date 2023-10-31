#![cfg_attr(not(feature = "rt"), allow(dead_code))]

//! Signal driver

use crate::io::driver::{Driver as IoDriver, Interest};
use crate::io::PollEvented;
use crate::park::Park;
use crate::signal::registry::globals;

use mio::net::UnixStream;
use std::io::{self, Read};
use std::ptr;
use std::sync::{Arc, Weak};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use std::time::Duration;

/// Responsible for registering wakeups when an OS signal is received, and
/// subsequently dispatching notifications to any signal listeners as appropriate.
///
/// Note: this driver relies on having an enabled IO driver in order to listen to
/// pipe write wakeups.
#[derive(Debug)]
pub(crate) struct Driver {
    /// Thread parker. The `Driver` park implementation delegates to this.
    park: IoDriver,

    /// A pipe for receiving wake events from the signal handler
    receiver: PollEvented<UnixStream>,

    /// Shared state
    inner: Arc<Inner>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Handle {
    inner: Weak<Inner>,
}

#[derive(Debug)]
pub(super) struct Inner(());

// ===== impl Driver =====

impl Driver {
    /// Creates a new signal `Driver` instance that delegates wakeups to `park`.
    pub(crate) fn new(park: IoDriver) -> io::Result<Self> {
        use std::mem::ManuallyDrop;
        use std::os::unix::io::{AsRawFd, FromRawFd};

        // NB: We give each driver a "fresh" receiver file descriptor to avoid
        // the issues described in alexcrichton/tokio-process#42.
        //
        // In the past we would reuse the actual receiver file descriptor and
        // swallow any errors around double registration of the same descriptor.
        // I'm not sure if the second (failed) registration simply doesn't end
        // up receiving wake up notifications, or there could be some race
        // condition when consuming readiness events, but having distinct
        // descriptors for distinct PollEvented instances appears to mitigate
        // this.
        //
        // Unfortunately we cannot just use a single global PollEvented instance
        // either, since we can't compare Handles or assume they will always
        // point to the exact same reactor.
        //
        // Mio 0.7 removed `try_clone()` as an API due to unexpected behavior
        // with registering dups with the same reactor. In this case, duping is
        // safe as each dup is registered with separate reactors **and** we
        // only expect at least one dup to receive the notification.

        // Manually drop as we don't actually own this instance of UnixStream.
        let receiver_fd = globals().receiver.as_raw_fd();

        // safety: there is nothing unsafe about this, but the `from_raw_fd` fn is marked as unsafe.
        let original =
            ManuallyDrop::new(unsafe { std::os::unix::net::UnixStream::from_raw_fd(receiver_fd) });
        let receiver = UnixStream::from_std(original.try_clone()?);
        let receiver = PollEvented::new_with_interest_and_handle(
            receiver,
            Interest::READABLE | Interest::WRITABLE,
            park.handle(),
        )?;

        Ok(Self {
            park,
            receiver,
            inner: Arc::new(Inner(())),
        })
    }

    /// Returns a handle to this event loop which can be sent across threads
    /// and can be used as a proxy to the event loop itself.
    pub(crate) fn handle(&self) -> Handle {
        Handle {
            inner: Arc::downgrade(&self.inner),
        }
    }

    fn process(&self) {
        // Check if the pipe is ready to read and therefore has "woken" us up
        //
        // To do so, we will `poll_read_ready` with a noop waker, since we don't
        // need to actually be notified when read ready...
        let waker = unsafe { Waker::from_raw(RawWaker::new(ptr::null(), &NOOP_WAKER_VTABLE)) };
        let mut cx = Context::from_waker(&waker);

        let ev = match self.receiver.registration().poll_read_ready(&mut cx) {
            Poll::Ready(Ok(ev)) => ev,
            Poll::Ready(Err(e)) => panic!("reactor gone: {}", e),
            Poll::Pending => return, // No wake has arrived, bail
        };

        // Drain the pipe completely so we can receive a new readiness event
        // if another signal has come in.
        let mut buf = [0; 128];
        loop {
            match (&*self.receiver).read(&mut buf) {
                Ok(0) => panic!("EOF on self-pipe"),
                Ok(_) => continue, // Keep reading
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => panic!("Bad read on self-pipe: {}", e),
            }
        }

        self.receiver.registration().clear_readiness(ev);

        // Broadcast any signals which were received
        globals().broadcast();
    }
}

const NOOP_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(noop_clone, noop, noop, noop);

unsafe fn noop_clone(_data: *const ()) -> RawWaker {
    RawWaker::new(ptr::null(), &NOOP_WAKER_VTABLE)
}

unsafe fn noop(_data: *const ()) {}

// ===== impl Park for Driver =====

impl Park for Driver {
    type Unpark = <IoDriver as Park>::Unpark;
    type Error = io::Error;

    fn unpark(&self) -> Self::Unpark {
        self.park.unpark()
    }

    fn park(&mut self) -> Result<(), Self::Error> {
        self.park.park()?;
        self.process();
        Ok(())
    }

    fn park_timeout(&mut self, duration: Duration) -> Result<(), Self::Error> {
        self.park.park_timeout(duration)?;
        self.process();
        Ok(())
    }

    fn shutdown(&mut self) {
        self.park.shutdown()
    }
}

// ===== impl Handle =====

impl Handle {
    pub(super) fn check_inner(&self) -> io::Result<()> {
        if self.inner.strong_count() > 0 {
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "signal driver gone"))
        }
    }
}

cfg_rt! {
    impl Handle {
        /// Returns a handle to the current driver
        ///
        /// # Panics
        ///
        /// This function panics if there is no current signal driver set.
        #[track_caller]
        pub(super) fn current() -> Self {
            crate::runtime::context::signal_handle().expect(
                "there is no signal driver running, must be called from the context of Tokio runtime",
            )
        }
    }
}

cfg_not_rt! {
    impl Handle {
        /// Returns a handle to the current driver
        ///
        /// # Panics
        ///
        /// This function panics if there is no current signal driver set.
        #[track_caller]
        pub(super) fn current() -> Self {
            panic!(
                "there is no signal driver running, must be called from the context of Tokio runtime or with\
                `rt` enabled.",
            )
        }
    }
}
