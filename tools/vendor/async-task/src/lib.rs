//! Task abstraction for building executors.
//!
//! To spawn a future onto an executor, we first need to allocate it on the heap and keep some
//! state attached to it. The state indicates whether the future is ready for polling, waiting to
//! be woken up, or completed. Such a stateful future is called a *task*.
//!
//! All executors have a queue that holds scheduled tasks:
//!
//! ```
//! let (sender, receiver) = flume::unbounded();
//! #
//! # // A future that will get spawned.
//! # let future = async { 1 + 2 };
//! #
//! # // A function that schedules the task when it gets woken up.
//! # let schedule = move |runnable| sender.send(runnable).unwrap();
//! #
//! # // Create a task.
//! # let (runnable, task) = async_task::spawn(future, schedule);
//! ```
//!
//! A task is created using either [`spawn()`], [`spawn_local()`], or [`spawn_unchecked()`] which
//! return a [`Runnable`] and a [`Task`]:
//!
//! ```
//! # let (sender, receiver) = flume::unbounded();
//! #
//! // A future that will be spawned.
//! let future = async { 1 + 2 };
//!
//! // A function that schedules the task when it gets woken up.
//! let schedule = move |runnable| sender.send(runnable).unwrap();
//!
//! // Construct a task.
//! let (runnable, task) = async_task::spawn(future, schedule);
//!
//! // Push the task into the queue by invoking its schedule function.
//! runnable.schedule();
//! ```
//!
//! The [`Runnable`] is used to poll the task's future, and the [`Task`] is used to await its
//! output.
//!
//! Finally, we need a loop that takes scheduled tasks from the queue and runs them:
//!
//! ```no_run
//! # let (sender, receiver) = flume::unbounded();
//! #
//! # // A future that will get spawned.
//! # let future = async { 1 + 2 };
//! #
//! # // A function that schedules the task when it gets woken up.
//! # let schedule = move |runnable| sender.send(runnable).unwrap();
//! #
//! # // Create a task.
//! # let (runnable, task) = async_task::spawn(future, schedule);
//! #
//! # // Push the task into the queue by invoking its schedule function.
//! # runnable.schedule();
//! #
//! for runnable in receiver {
//!     runnable.run();
//! }
//! ```
//!
//! Method [`run()`][`Runnable::run()`] polls the task's future once. Then, the [`Runnable`]
//! vanishes and only reappears when its [`Waker`][`core::task::Waker`] wakes the task, thus
//! scheduling it to be run again.

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]
#![doc(test(attr(deny(rust_2018_idioms, warnings))))]
#![doc(test(attr(allow(unused_extern_crates, unused_variables))))]

extern crate alloc;

/// We can't use `?` in const contexts yet, so this macro acts
/// as a workaround.
macro_rules! leap {
    ($x: expr) => {{
        match ($x) {
            Some(val) => val,
            None => return None,
        }
    }};
}

mod header;
mod raw;
mod runnable;
mod state;
mod task;
mod utils;

pub use crate::runnable::{spawn, spawn_unchecked, Runnable};
pub use crate::task::{FallibleTask, Task};

#[cfg(feature = "std")]
pub use crate::runnable::spawn_local;
