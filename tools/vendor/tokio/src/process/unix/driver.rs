#![cfg_attr(not(feature = "rt"), allow(dead_code))]

//! Process driver.

use crate::park::Park;
use crate::process::unix::GlobalOrphanQueue;
use crate::signal::unix::driver::{Driver as SignalDriver, Handle as SignalHandle};

use std::io;
use std::time::Duration;

/// Responsible for cleaning up orphaned child processes on Unix platforms.
#[derive(Debug)]
pub(crate) struct Driver {
    park: SignalDriver,
    signal_handle: SignalHandle,
}

// ===== impl Driver =====

impl Driver {
    /// Creates a new signal `Driver` instance that delegates wakeups to `park`.
    pub(crate) fn new(park: SignalDriver) -> Self {
        let signal_handle = park.handle();

        Self {
            park,
            signal_handle,
        }
    }
}

// ===== impl Park for Driver =====

impl Park for Driver {
    type Unpark = <SignalDriver as Park>::Unpark;
    type Error = io::Error;

    fn unpark(&self) -> Self::Unpark {
        self.park.unpark()
    }

    fn park(&mut self) -> Result<(), Self::Error> {
        self.park.park()?;
        GlobalOrphanQueue::reap_orphans(&self.signal_handle);
        Ok(())
    }

    fn park_timeout(&mut self, duration: Duration) -> Result<(), Self::Error> {
        self.park.park_timeout(duration)?;
        GlobalOrphanQueue::reap_orphans(&self.signal_handle);
        Ok(())
    }

    fn shutdown(&mut self) {
        self.park.shutdown()
    }
}
