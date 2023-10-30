//! Asynchronous green-threads.
//!
//! ## What are Tasks?
//!
//! A _task_ is a light weight, non-blocking unit of execution. A task is similar
//! to an OS thread, but rather than being managed by the OS scheduler, they are
//! managed by the [Tokio runtime][rt]. Another name for this general pattern is
//! [green threads]. If you are familiar with [Go's goroutines], [Kotlin's
//! coroutines], or [Erlang's processes], you can think of Tokio's tasks as
//! something similar.
//!
//! Key points about tasks include:
//!
//! * Tasks are **light weight**. Because tasks are scheduled by the Tokio
//!   runtime rather than the operating system, creating new tasks or switching
//!   between tasks does not require a context switch and has fairly low
//!   overhead. Creating, running, and destroying large numbers of tasks is
//!   quite cheap, especially compared to OS threads.
//!
//! * Tasks are scheduled **cooperatively**. Most operating systems implement
//!   _preemptive multitasking_. This is a scheduling technique where the
//!   operating system allows each thread to run for a period of time, and then
//!   _preempts_ it, temporarily pausing that thread and switching to another.
//!   Tasks, on the other hand, implement _cooperative multitasking_. In
//!   cooperative multitasking, a task is allowed to run until it _yields_,
//!   indicating to the Tokio runtime's scheduler that it cannot currently
//!   continue executing. When a task yields, the Tokio runtime switches to
//!   executing the next task.
//!
//! * Tasks are **non-blocking**. Typically, when an OS thread performs I/O or
//!   must synchronize with another thread, it _blocks_, allowing the OS to
//!   schedule another thread. When a task cannot continue executing, it must
//!   yield instead, allowing the Tokio runtime to schedule another task. Tasks
//!   should generally not perform system calls or other operations that could
//!   block a thread, as this would prevent other tasks running on the same
//!   thread from executing as well. Instead, this module provides APIs for
//!   running blocking operations in an asynchronous context.
//!
//! [rt]: crate::runtime
//! [green threads]: https://en.wikipedia.org/wiki/Green_threads
//! [Go's goroutines]: https://tour.golang.org/concurrency/1
//! [Kotlin's coroutines]: https://kotlinlang.org/docs/reference/coroutines-overview.html
//! [Erlang's processes]: http://erlang.org/doc/getting_started/conc_prog.html#processes
//!
//! ## Working with Tasks
//!
//! This module provides the following APIs for working with tasks:
//!
//! ### Spawning
//!
//! Perhaps the most important function in this module is [`task::spawn`]. This
//! function can be thought of as an async equivalent to the standard library's
//! [`thread::spawn`][`std::thread::spawn`]. It takes an `async` block or other
//! [future], and creates a new task to run that work concurrently:
//!
//! ```
//! use tokio::task;
//!
//! # async fn doc() {
//! task::spawn(async {
//!     // perform some work here...
//! });
//! # }
//! ```
//!
//! Like [`std::thread::spawn`], `task::spawn` returns a [`JoinHandle`] struct.
//! A `JoinHandle` is itself a future which may be used to await the output of
//! the spawned task. For example:
//!
//! ```
//! use tokio::task;
//!
//! # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let join = task::spawn(async {
//!     // ...
//!     "hello world!"
//! });
//!
//! // ...
//!
//! // Await the result of the spawned task.
//! let result = join.await?;
//! assert_eq!(result, "hello world!");
//! # Ok(())
//! # }
//! ```
//!
//! Again, like `std::thread`'s [`JoinHandle` type][thread_join], if the spawned
//! task panics, awaiting its `JoinHandle` will return a [`JoinError`]. For
//! example:
//!
//! ```
//! use tokio::task;
//!
//! # #[tokio::main] async fn main() {
//! let join = task::spawn(async {
//!     panic!("something bad happened!")
//! });
//!
//! // The returned result indicates that the task failed.
//! assert!(join.await.is_err());
//! # }
//! ```
//!
//! `spawn`, `JoinHandle`, and `JoinError` are present when the "rt"
//! feature flag is enabled.
//!
//! [`task::spawn`]: crate::task::spawn()
//! [future]: std::future::Future
//! [`std::thread::spawn`]: std::thread::spawn
//! [`JoinHandle`]: crate::task::JoinHandle
//! [thread_join]: std::thread::JoinHandle
//! [`JoinError`]: crate::task::JoinError
//!
//! ### Blocking and Yielding
//!
//! As we discussed above, code running in asynchronous tasks should not perform
//! operations that can block. A blocking operation performed in a task running
//! on a thread that is also running other tasks would block the entire thread,
//! preventing other tasks from running.
//!
//! Instead, Tokio provides two APIs for running blocking operations in an
//! asynchronous context: [`task::spawn_blocking`] and [`task::block_in_place`].
//!
//! Be aware that if you call a non-async method from async code, that non-async
//! method is still inside the asynchronous context, so you should also avoid
//! blocking operations there. This includes destructors of objects destroyed in
//! async code.
//!
//! #### spawn_blocking
//!
//! The `task::spawn_blocking` function is similar to the `task::spawn` function
//! discussed in the previous section, but rather than spawning an
//! _non-blocking_ future on the Tokio runtime, it instead spawns a
//! _blocking_ function on a dedicated thread pool for blocking tasks. For
//! example:
//!
//! ```
//! use tokio::task;
//!
//! # async fn docs() {
//! task::spawn_blocking(|| {
//!     // do some compute-heavy work or call synchronous code
//! });
//! # }
//! ```
//!
//! Just like `task::spawn`, `task::spawn_blocking` returns a `JoinHandle`
//! which we can use to await the result of the blocking operation:
//!
//! ```rust
//! # use tokio::task;
//! # async fn docs() -> Result<(), Box<dyn std::error::Error>>{
//! let join = task::spawn_blocking(|| {
//!     // do some compute-heavy work or call synchronous code
//!     "blocking completed"
//! });
//!
//! let result = join.await?;
//! assert_eq!(result, "blocking completed");
//! # Ok(())
//! # }
//! ```
//!
//! #### block_in_place
//!
//! When using the [multi-threaded runtime][rt-multi-thread], the [`task::block_in_place`]
//! function is also available. Like `task::spawn_blocking`, this function
//! allows running a blocking operation from an asynchronous context. Unlike
//! `spawn_blocking`, however, `block_in_place` works by transitioning the
//! _current_ worker thread to a blocking thread, moving other tasks running on
//! that thread to another worker thread. This can improve performance by avoiding
//! context switches.
//!
//! For example:
//!
//! ```
//! use tokio::task;
//!
//! # async fn docs() {
//! let result = task::block_in_place(|| {
//!     // do some compute-heavy work or call synchronous code
//!     "blocking completed"
//! });
//!
//! assert_eq!(result, "blocking completed");
//! # }
//! ```
//!
//! #### yield_now
//!
//! In addition, this module provides a [`task::yield_now`] async function
//! that is analogous to the standard library's [`thread::yield_now`]. Calling
//! and `await`ing this function will cause the current task to yield to the
//! Tokio runtime's scheduler, allowing other tasks to be
//! scheduled. Eventually, the yielding task will be polled again, allowing it
//! to execute. For example:
//!
//! ```rust
//! use tokio::task;
//!
//! # #[tokio::main] async fn main() {
//! async {
//!     task::spawn(async {
//!         // ...
//!         println!("spawned task done!")
//!     });
//!
//!     // Yield, allowing the newly-spawned task to execute first.
//!     task::yield_now().await;
//!     println!("main task done!");
//! }
//! # .await;
//! # }
//! ```
//!
//! ### Cooperative scheduling
//!
//! A single call to [`poll`] on a top-level task may potentially do a lot of
//! work before it returns `Poll::Pending`. If a task runs for a long period of
//! time without yielding back to the executor, it can starve other tasks
//! waiting on that executor to execute them, or drive underlying resources.
//! Since Rust does not have a runtime, it is difficult to forcibly preempt a
//! long-running task. Instead, this module provides an opt-in mechanism for
//! futures to collaborate with the executor to avoid starvation.
//!
//! Consider a future like this one:
//!
//! ```
//! # use tokio_stream::{Stream, StreamExt};
//! async fn drop_all<I: Stream + Unpin>(mut input: I) {
//!     while let Some(_) = input.next().await {}
//! }
//! ```
//!
//! It may look harmless, but consider what happens under heavy load if the
//! input stream is _always_ ready. If we spawn `drop_all`, the task will never
//! yield, and will starve other tasks and resources on the same executor.
//!
//! To account for this, Tokio has explicit yield points in a number of library
//! functions, which force tasks to return to the executor periodically.
//!
//!
//! #### unconstrained
//!
//! If necessary, [`task::unconstrained`] lets you opt out a future of Tokio's cooperative
//! scheduling. When a future is wrapped with `unconstrained`, it will never be forced to yield to
//! Tokio. For example:
//!
//! ```
//! # #[tokio::main]
//! # async fn main() {
//! use tokio::{task, sync::mpsc};
//!
//! let fut = async {
//!     let (tx, mut rx) = mpsc::unbounded_channel();
//!
//!     for i in 0..1000 {
//!         let _ = tx.send(());
//!         // This will always be ready. If coop was in effect, this code would be forced to yield
//!         // periodically. However, if left unconstrained, then this code will never yield.
//!         rx.recv().await;
//!     }
//! };
//!
//! task::unconstrained(fut).await;
//! # }
//! ```
//!
//! [`task::spawn_blocking`]: crate::task::spawn_blocking
//! [`task::block_in_place`]: crate::task::block_in_place
//! [rt-multi-thread]: ../runtime/index.html#threaded-scheduler
//! [`task::yield_now`]: crate::task::yield_now()
//! [`thread::yield_now`]: std::thread::yield_now
//! [`task::unconstrained`]: crate::task::unconstrained()
//! [`poll`]: method@std::future::Future::poll

cfg_rt! {
    pub use crate::runtime::task::{JoinError, JoinHandle};

    mod blocking;
    pub use blocking::spawn_blocking;

    mod spawn;
    pub use spawn::spawn;

    cfg_rt_multi_thread! {
        pub use blocking::block_in_place;
    }

    mod yield_now;
    pub use yield_now::yield_now;

    cfg_unstable! {
        mod consume_budget;
        pub use consume_budget::consume_budget;
    }

    mod local;
    pub use local::{spawn_local, LocalSet};

    mod task_local;
    pub use task_local::LocalKey;

    mod unconstrained;
    pub use unconstrained::{unconstrained, Unconstrained};

    cfg_unstable! {
        pub mod join_set;
        #[doc(inline)]
        pub use join_set::JoinSet;
        pub use crate::runtime::task::{Id, AbortHandle};
    }

    cfg_trace! {
        mod builder;
        pub use builder::Builder;
    }

    /// Task-related futures.
    pub mod futures {
        pub use super::task_local::TaskLocalFuture;
    }
}
